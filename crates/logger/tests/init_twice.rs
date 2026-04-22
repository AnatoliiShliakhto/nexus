#![allow(unused_crate_dependencies)]

use nx_logger::error::LoggerError;
use nx_logger::{LevelFilter, Logger};

#[test]
fn init_twice_returns_subscriber_error() {
    let _logger = Logger::builder()
        .name("integration-init-twice")
        .level(LevelFilter::INFO)
        .init()
        .expect("first init should succeed");

    let err = Logger::builder()
        .name("integration-init-twice-second")
        .level(LevelFilter::INFO)
        .init()
        .expect_err("second init should fail");

    assert!(
        matches!(err, LoggerError::Subscriber { .. }),
        "expected subscriber error for second init"
    );
}
