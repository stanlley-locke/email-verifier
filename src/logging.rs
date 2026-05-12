use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};
use std::path::Path;

pub fn setup_logging(log_file: Option<&str>, quiet: bool, verbose: bool) -> anyhow::Result<()> {
    let level = if verbose {
        "debug"
    } else if quiet {
        "warn"
    } else {
        "info"
    };

    let env_filter = EnvFilter::builder()
        .with_default_directive(level.parse().unwrap())
        .from_env_lossy();

    // Collect all layers as boxed trait objects so the types unify.
    let mut layers: Vec<Box<dyn Layer<tracing_subscriber::Registry> + Send + Sync>> = Vec::new();

    if !quiet {
        let console_layer = tracing_subscriber::fmt::layer()
            .with_target(false)
            .with_thread_ids(false)
            .with_file(false)
            .with_line_number(false)
            .with_timer(tracing_subscriber::fmt::time::time())
            .compact()
            .boxed();
        layers.push(console_layer);
    }

    if let Some(log_path) = log_file {
        if let Some(parent) = Path::new(log_path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = std::fs::File::create(log_path)?;
        let file_layer = tracing_subscriber::fmt::layer()
            .with_writer(file)
            .with_ansi(false)
            .with_target(true)
            .with_thread_ids(true)
            .with_file(true)
            .with_line_number(true)
            .with_timer(tracing_subscriber::fmt::time::SystemTime)
            .json()
            .boxed();
        layers.push(file_layer);
    }

    tracing_subscriber::registry()
        .with(layers)
        .with(env_filter)
        .init();

    Ok(())
}


#[macro_export]
macro_rules! log_progress {
    ($logger:expr, $current:expr, $total:expr, $rate:expr, $corrections:expr) => {
        tracing::info!(
            current = $current,
            total = $total,
            rate = format!("{:.1}", $rate),
            corrections = $corrections,
            "Progress update"
        );
    };
}