use calamine::{open_workbook_auto, Reader, Sheets};
use csv::ReaderBuilder;
use std::fs::File;
use std::io::BufReader;
use anyhow::Result;

pub fn detect_file_format(filepath: &str) -> Result<&'static str> {
    let ext = std::path::Path::new(filepath)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .ok_or_else(|| anyhow::anyhow!("Could not determine file extension"))?;
    
    match ext.as_str() {
        "xlsx" | "xlsm" => Ok("excel"),
        "csv" => Ok("csv"),
        _ => Err(anyhow::anyhow!("Unsupported file format: .{}", ext)),
    }
}

pub fn read_excel_file(
    filepath: &str,
    sheet_name: Option<&str>,
) -> Result<(Vec<Vec<String>>, Vec<String>, String)> {
    let mut workbook: Sheets<BufReader<std::fs::File>> = open_workbook_auto(filepath)
        .map_err(|e| anyhow::anyhow!("Failed to open Excel file: {}", e))?;
    
    let sheet_names = workbook.sheet_names();
    let sheet = if let Some(name) = sheet_name {
        if !sheet_names.contains(&name.to_string()) {
            return Err(anyhow::anyhow!(
                "Sheet '{}' not found. Available: {:?}",
                name,
                sheet_names
            ));
        }
        name.to_string()
    } else {
        sheet_names.first()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No sheets found in workbook"))?
    };
    
    let mut rows = Vec::new();
    if let Ok(range) = workbook.worksheet_range(&sheet) {
        for row in range.rows() {
            let row_data: Vec<String> = row
                .iter()
                .map(|cell| match cell {
                    calamine::DataType::String(s) => s.clone(),
                    calamine::DataType::Int(i) => i.to_string(),
                    calamine::DataType::Float(f) => f.to_string(),
                    calamine::DataType::Bool(b) => b.to_string(),
                    calamine::DataType::DateTime(d) => d.to_string(),
                    calamine::DataType::Error(e) => format!("#ERROR: {}", e),
                    calamine::DataType::Empty => String::new(),
                    _ => String::new(),
                })
                .collect();
            rows.push(row_data);
        }
    }
    
    let headers = rows.first().cloned().unwrap_or_default();
    let data_rows = rows.into_iter().skip(1).collect();
    
    Ok((data_rows, headers, sheet))
}

pub fn read_csv_file(filepath: &str) -> Result<(Vec<Vec<String>>, Vec<String>, String)> {
    let file = File::open(filepath)?;
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(BufReader::new(file));
    
    let headers = reader.headers()?.iter().map(|s| s.to_string()).collect();
    
    let mut rows = Vec::new();
    for result in reader.records() {
        let record = result?;
        rows.push(record.iter().map(|s| s.to_string()).collect());
    }
    
    Ok((rows, headers, "Sheet1".to_string()))
}

pub fn write_csv_output(
    filepath: &str,
    rows: &[Vec<String>],
    headers: &[String],
    status_columns: &[(usize, usize)],
    results: &std::collections::HashMap<(usize, usize), crate::types::VerificationResult>,
) -> Result<()> {
    let mut output_headers = headers.to_vec();
    
    // Insert status column headers (in reverse order to maintain indices)
    for (email_col, status_col) in status_columns.iter().rev() {
        let email_header = headers.get(*email_col - 1)
            .cloned()
            .unwrap_or_else(|| format!("Email{}", email_col));
        output_headers.insert(*status_col - 1, format!("{}_status", email_header));
    }
    
    let file = File::create(filepath)?;
    let mut writer = csv::Writer::from_writer(file);
    
    writer.write_record(&output_headers)?;
    
    for (row_idx, row_data) in rows.iter().enumerate() {
        let mut output_row = row_data.clone();
        
        // Insert status values (in reverse order to maintain indices)
        for (email_col, status_col) in status_columns.iter().rev() {
            let key = (row_idx + 2, *status_col);
            if let Some(result) = results.get(&key) {
                output_row.insert(*status_col - 1, result.status.as_str().to_string());
            }
        }
        
        writer.write_record(&output_row)?;
    }
    
    writer.flush()?;
    Ok(())
}