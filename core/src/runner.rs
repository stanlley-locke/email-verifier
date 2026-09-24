use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::task;
use anyhow::Result;

use crate::dedup::DeduplicationManager;
use crate::file_io::{detect_file_format, read_csv_file, read_excel_file, write_csv_output};
use crate::email_parser;
use crate::output::OutputWriter;
use crate::progress::ProgressReporter;
use crate::types::CellKey;
use crate::verifier::EmailVerifier;

#[derive(Clone, Debug)]
pub struct VerificationConfig {
    pub input_path: PathBuf,
    pub output_dir: Option<PathBuf>,
    pub sheet_name: Option<String>,
    pub column_pattern: String,
    pub no_smtp: bool,
    pub timeout: u64,
    pub retries: u32,
    pub workers: usize,
    pub auto_size_columns: bool,
    pub quiet: bool,
    pub color_theme: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct ProgressEvent {
    pub completed: usize,
    pub total: usize,
    pub deliverable: usize,
    pub catch_all: usize,
    pub undeliverable: usize,
    pub speed_emails_per_sec: f64,
    pub status: String,
}

pub async fn run_verification(
    config: VerificationConfig,
    progress_tx: Option<tokio::sync::mpsc::Sender<ProgressEvent>>,
) -> Result<(String, String, usize, usize)> {
    let input_path = config.input_path.clone();
    
    // Determine output paths
    let output_base = config.output_dir.clone().unwrap_or_else(|| {
        let stem = input_path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("verified");
        input_path.parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .join(format!("verified_{}", stem))
    });
    
    let ext_str = input_path.extension().and_then(|e| e.to_str()).map(|e| format!(".{}", e)).unwrap_or_default();
    let output_minimal = format!("{}_minimal{}", output_base.display(), ext_str);
    let output_comprehensive = format!("{}_comprehensive{}", output_base.display(), ext_str);
    
    // Detect file format and read
    let format = detect_file_format(input_path.to_str().unwrap())?;
    
    let (rows, headers, _sheet_name) = match format {
        "excel" => read_excel_file(
            input_path.to_str().unwrap(),
            config.sheet_name.as_deref(),
        )?,
        "csv" => read_csv_file(input_path.to_str().unwrap())?,
        _ => anyhow::bail!("Unsupported format"),
    };
    
    let email_columns = email_parser::identify_email_columns(&headers, &config.column_pattern)?;
    if email_columns.is_empty() {
        anyhow::bail!("No email columns found with pattern '{}'", config.column_pattern);
    }
    
    let total_cells = email_parser::count_non_empty_email_cells(&rows, &email_columns);
    
    let mut status_columns = Vec::new();
    let mut offset = 0;
    let mut new_headers = headers.clone();
    for (email_col, email_header) in &email_columns {
        let status_col = email_col + offset + 1;
        status_columns.push((*email_col, status_col));
        new_headers.insert(status_col - 1, format!("{}_status", email_header));
        offset += 1;
    }
    
    let verifier = EmailVerifier::new(
        config.no_smtp,
        Duration::from_secs(config.timeout),
        config.retries,
    );
    let dedup = DeduplicationManager::new();
    let mut progress = ProgressReporter::new(total_cells, config.quiet);
    let semaphore = Arc::new(Semaphore::new(config.workers));
    
    let mut tasks = Vec::new();
    for (row_idx, row) in rows.iter().enumerate() {
        for (email_col, status_col) in &status_columns {
            if *email_col <= row.len() {
                let cell_value = &row[email_col - 1];
                let emails = email_parser::extract_emails_from_cell(cell_value);
                for (normalized, was_corrected, correction) in emails {
                    tasks.push((
                        row_idx + 2,
                        *email_col,
                        *status_col,
                        normalized,
                        cell_value.clone(),
                        email_columns.iter().find(|(c, _)| c == email_col).map(|(_, h)| h.clone()).unwrap_or_default(),
                        was_corrected,
                        correction,
                    ));
                }
            }
        }
    }
    
    let results = Arc::new(tokio::sync::Mutex::new(
        std::collections::HashMap::<CellKey, crate::types::VerificationResult>::new(),
    ));

    let total_tasks = tasks.len();
    let (tx, mut rx) = tokio::sync::mpsc::channel::<(String, String, crate::types::VerificationResult, CellKey)>(total_tasks.max(1));

    for task in tasks {
        let (row, _email_col, status_col, email, original, col_name, _was_corrected, _correction) = task;
        let semaphore = semaphore.clone();
        let verifier = verifier.clone();
        let dedup = dedup.clone();
        let tx = tx.clone();

        task::spawn(async move {
            let _permit = semaphore.acquire().await;
            let result = if let Some(cached) = dedup.get_or_queue(&email).await {
                cached
            } else {
                let r = verifier.verify_email(email.clone(), original).await;
                dedup.store_result(email.clone(), r.clone()).await;
                r
            };
            let _ = tx.send((email, col_name, result, (row, status_col))).await;
        });
    }
    drop(tx);

    let start_time = std::time::Instant::now();
    let mut completed_tasks = 0;

    while let Some((_, col_name, result, cell_key)) = rx.recv().await {
        {
            let mut res = results.lock().await;
            res.insert(cell_key, result.clone());
        }
        progress.update(&result, &col_name);
        completed_tasks += 1;

        if let Some(ref p_tx) = progress_tx {
            let stats = progress.get_stats();
            let elapsed = start_time.elapsed().as_secs_f64();
            let rate = if elapsed > 0.0 { completed_tasks as f64 / elapsed } else { 0.0 };
            
            let deliverable = *stats.status_counts.get("Deliverable").unwrap_or(&0);
            let catch_all = *stats.status_counts.get("Catch-All").unwrap_or(&0);
            let undeliverable = *stats.status_counts.get("Undeliverable").unwrap_or(&0);

            let _ = p_tx.send(ProgressEvent {
                completed: completed_tasks,
                total: total_tasks,
                deliverable,
                catch_all,
                undeliverable,
                speed_emails_per_sec: rate,
                status: "Processing".to_string()
            }).await;
        }
    }

    progress.finalize();
    
    // Write outputs
    {
        let guard = results.lock().await;
        OutputWriter::write_minimal_output(
            &output_minimal,
            &rows,
            &headers,
            &status_columns,
            &*guard,
        )?;
    }
    
    let out_format = detect_file_format(&output_comprehensive)?;
    {
        let guard = results.lock().await;
        match out_format {
            "excel" => {
                OutputWriter::write_comprehensive_output(
                    &output_comprehensive,
                    &rows,
                    &new_headers,
                    &status_columns,
                    &*guard,
                    config.auto_size_columns,
                    config.color_theme.clone(),
                )?;
            }
            "csv" => {
                write_csv_output(
                    &output_comprehensive,
                    &rows,
                    &new_headers,
                    &status_columns,
                    &*guard,
                )?;
            }
            _ => {}
        }
    }
    
    let mut stats = progress.get_stats();
    stats.dedup_stats = dedup.get_stats().await;
    let summary_file = format!("{}.summary.json", output_comprehensive);
    let summary_json = serde_json::to_string_pretty(&stats)?;
    std::fs::write(&summary_file, summary_json)?;
    
    let total_count = stats.total_processed;
    let deliverable_count = *stats.status_counts.get("Deliverable").unwrap_or(&0);
    
    Ok((output_minimal, output_comprehensive, total_count, deliverable_count))
}
