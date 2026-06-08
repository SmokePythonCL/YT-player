use std::fs;
use std::path::Path;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

pub fn init_logger(app_data_dir: &Path) -> Result<(), anyhow::Error> {
    if !app_data_dir.exists() {
        fs::create_dir_all(app_data_dir)?;
    }

    let log_path = app_data_dir.join("app.log");
    let file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(log_path)?;

    let file_layer = fmt::layer()
        .with_writer(std::sync::Mutex::new(file))
        .with_ansi(false);

    let stdout_layer = fmt::layer()
        .with_ansi(true);

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(file_layer)
        .with(stdout_layer)
        .init();

    Ok(())
}
