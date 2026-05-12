#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

extern crate anyhow;
extern crate lazy_static;

mod config;
mod dedup;
mod dns_resolver;
mod email_parser;
mod file_io;
mod logging;
mod output;
mod progress;
mod smtp_verifier;
mod typo_correction;
mod types;
mod verifier;

use clap::Parser;
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::task;

use std::sync::Arc;

use crate::dedup::DeduplicationManager;
use crate::file_io::{detect_file_format, read_csv_file, read_excel_file, write_csv_output};
use crate::output::OutputWriter;
use crate::progress::ProgressReporter;
use crate::types::CellKey;
use crate::verifier::EmailVerifier;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(after_help = r#"
Examples:
  email-verifier input.xlsx
  email-verifier data.csv -o verified -s "Batch 3"
  email-verifier input.xlsx --no-smtp --timeout 15 --retries 5
  email-verifier input.csv --column-pattern ".*mail.*" --quiet --log verify.log
"#)]
struct Args {
    /// Input file path (.xlsx, .xlsm, or .csv)
    input: PathBuf,
    
    /// Output file base name (auto-appends _minimal and _comprehensive)
    #[arg(short, long)]
    output: Option<PathBuf>,
    
    /// Sheet name for Excel input (default: active sheet)
    #[arg(short, long)]
    sheet: Option<String>,
    
    /// Regex pattern to identify email columns
    #[arg(long, default_value = r"(?i).*email.*")]
    column_pattern: String,
    
    /// Skip SMTP verification (syntax + DNS only)
    #[arg(long)]
    no_smtp: bool,
    
    /// SMTP timeout in seconds
    #[arg(long, default_value_t = 10)]
    timeout: u64,
    
    /// Max retry attempts
    #[arg(long, default_value_t = 3)]
    retries: u32,
    
    /// Concurrent worker threads
    #[arg(long, default_value_t = 20)]
    workers: usize,
    
    /// Disable auto-sizing of Excel columns
    #[arg(long)]
    no_auto_size: bool,
    
    /// Suppress non-critical console output
    #[arg(short, long)]
    quiet: bool,
    
    /// Path to log file for detailed output
    #[arg(long)]
    log: Option<PathBuf>,
    
    /// Enable verbose/debug logging
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    
    // Setup logging
    logging::setup_logging(
        args.log.as_deref().and_then(|p| p.to_str()),
        args.quiet,
        args.verbose,
    )?;
    
    // Validate input file
    if !args.input.exists() {
        anyhow::bail!("Input file not found: {:?}", args.input);
    }
    
    // Determine output paths
    let output_base = args.output.clone().unwrap_or_else(|| {
        let stem = args.input.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("verified");
        args.input.parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .join(format!("verified_{}", stem))
    });
    
    let output_minimal = format!(
        "{}_minimal{}",
        output_base.display(),
        args.input.extension()
            .and_then(|e| e.to_str())
            .map(|e| format!(".{}", e))
            .unwrap_or_default()
    );
    
    let output_comprehensive = format!(
        "{}_comprehensive{}",
        output_base.display(),
        args.input.extension()
            .and_then(|e| e.to_str())
            .map(|e| format!(".{}", e))
            .unwrap_or_default()
    );
    
    // Detect file format and read
    let format = detect_file_format(args.input.to_str().unwrap())?;
    tracing::info!("Reading {} file: {:?}", format, args.input);
    
    let (rows, headers, sheet_name) = match format {
        "excel" => read_excel_file(
            args.input.to_str().unwrap(),
            args.sheet.as_deref(),
        )?,
        "csv" => read_csv_file(args.input.to_str().unwrap())?,
        _ => anyhow::bail!("Unsupported format"),
    };
    
    tracing::info!("Loaded {} rows from sheet '{}'", rows.len(), sheet_name);
    
    // Identify email columns
    let email_columns = email_parser::identify_email_columns(
        &headers,
        &args.column_pattern,
    )?;
    
    if email_columns.is_empty() {
        anyhow::bail!("No email columns found with pattern '{}'", args.column_pattern);
    }
    
    tracing::info!(
        "Found {} email columns: {:?}",
        email_columns.len(),
        email_columns.iter().map(|(_, h)| h).collect::<Vec<_>>()
    );
    
    // Count non-empty cells for accurate progress
    let total_cells = email_parser::count_non_empty_email_cells(&rows, &email_columns);
    tracing::info!("Total non-empty email cells to verify: {}", total_cells);
    
    // Prepare status columns
    let mut status_columns = Vec::new();
    let mut offset = 0;
    let mut new_headers = headers.clone();
    
    for (email_col, email_header) in &email_columns {
        let status_col = email_col + offset + 1;
        status_columns.push((*email_col, status_col));
        new_headers.insert(status_col - 1, format!("{}_status", email_header));
        offset += 1;
    }
    
    // Initialize components
    let verifier = EmailVerifier::new(
        args.no_smtp,
        Duration::from_secs(args.timeout),
        args.retries,
    );
    
    let dedup = DeduplicationManager::new();
    let mut progress = ProgressReporter::new(total_cells, args.quiet);
    let semaphore = Arc::new(Semaphore::new(args.workers));
    
    // Build verification tasks
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
                        email_columns.iter()
                            .find(|(c, _)| c == email_col)
                            .map(|(_, h)| h.clone())
                            .unwrap_or_default(),
                        was_corrected,
                        correction,
                    ));
                }
            }
        }
    }
    
    tracing::info!("Queued {} email verifications", tasks.len());

    // Execute verifications concurrently with real-time progress updates.
    //
    // Architecture:
    //   - Each task is spawned immediately (semaphore acquired INSIDE the task)
    //   - Completed results are sent through an mpsc channel
    //   - The main loop drains the channel and updates the progress bar live
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
            // Acquire permit inside the task — all tasks are spawned immediately
            let _permit = semaphore.acquire().await;

            // Check dedup cache
            let result = if let Some(cached) = dedup.get_or_queue(&email).await {
                cached
            } else {
                // First time seeing this email — run full verification
                let r = verifier.verify_email(email.clone(), original).await;
                dedup.store_result(email.clone(), r.clone()).await;
                r
            };

            let _ = tx.send((email, col_name, result, (row, status_col))).await;
        });
    }

    // Drop the original sender so the channel closes when all tasks finish
    drop(tx);

    // Drain results channel — update progress bar and results map in real time
    while let Some((_, col_name, result, cell_key)) = rx.recv().await {
        {
            let mut res = results.lock().await;
            res.insert(cell_key, result.clone());
        }
        progress.update(&result, &col_name);
    }


    progress.finalize();
    
    // Write outputs
    tracing::info!("Writing minimal output: {}", output_minimal);
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
    
    tracing::info!("Writing comprehensive output: {}", output_comprehensive);
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
                    !args.no_auto_size,
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
    
    // Generate JSON summary
    let mut stats = progress.get_stats();
    stats.dedup_stats = dedup.get_stats().await;
    
    let summary_file = format!("{}.summary.json", output_comprehensive);
    let summary_json = serde_json::to_string_pretty(&stats)?;
    std::fs::write(&summary_file, summary_json)?;
    tracing::info!("Summary saved to: {}", summary_file);
    
    if !args.quiet {
        println!("\nProcessing complete.");
        println!("Minimal output: {}", output_minimal);
        println!("Comprehensive output: {}", output_comprehensive);
        println!("Summary JSON: {}", summary_file);
    }
    
    Ok(())
}