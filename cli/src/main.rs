#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]


use clap::Parser;
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::task;

use std::sync::Arc;

use email_verifier_core::dedup::DeduplicationManager;
use email_verifier_core::file_io::{detect_file_format, read_csv_file, read_excel_file, write_csv_output};
use email_verifier_core::logging;
use email_verifier_core::email_parser;
use email_verifier_core::output::OutputWriter;
use email_verifier_core::progress::ProgressReporter;
use email_verifier_core::types::CellKey;
use email_verifier_core::verifier::EmailVerifier;

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
    
    let config = email_verifier_core::runner::VerificationConfig {
        input_path: args.input,
        output_dir: args.output,
        sheet_name: args.sheet,
        column_pattern: args.column_pattern,
        no_smtp: args.no_smtp,
        timeout: args.timeout,
        retries: args.retries,
        workers: args.workers,
        auto_size_columns: true,
        quiet: args.quiet,
        color_theme: None,
    };
    
    let (min, comp, _, _) = email_verifier_core::runner::run_verification(config, None).await?;
    
    if !args.quiet {
        println!("\nProcessing complete.");
        println!("Minimal output: {}", min);
        println!("Comprehensive output: {}", comp);
    }
    
    Ok(())
}