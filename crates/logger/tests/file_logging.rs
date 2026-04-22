#![allow(unused_crate_dependencies)]

use nx_logger::{LevelFilter, Logger};
use std::fs;
use std::time::Duration;
use tempfile::tempdir;

#[test]
fn file_logging_creates_log_file() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_dir = tempdir()?;
    let log_dir = tmp_dir.path().join("logs");

    let logger = Logger::builder()
        .name("integration-file-logging")
        .path(&log_dir)
        .level(LevelFilter::INFO)
        .init()?;

    tracing::info!("hello from integration test");

    std::thread::sleep(Duration::from_millis(30));
    drop(logger);

    let entries = fs::read_dir(&log_dir)?;
    let log_file = entries
        .flatten()
        .map(|entry| entry.path())
        .find(|path| path.extension().and_then(|ext| ext.to_str()) == Some("log"))
        .expect("log file should be created");

    let metadata = fs::metadata(&log_file)?;
    assert!(metadata.len() > 0, "log file should not be empty");

    Ok(())
}
