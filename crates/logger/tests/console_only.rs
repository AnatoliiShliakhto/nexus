#![allow(unused_crate_dependencies)]

use nx_logger::{LevelFilter, Logger};

#[test]
fn init_console_only_is_successful() {
    let logger = Logger::builder()
        .name("integration-console-only")
        .console(true)
        .level(LevelFilter::INFO)
        .init();

    assert!(logger.is_ok(), "Console-only logger should initialize successfully");
}

#[test]
fn init_fails_when_all_layers_disabled() {
    let logger = Logger::builder().name("integration-none").console(false).init();

    assert!(logger.is_err(), "Logger should fail if no layers (console/file) are enabled");
}
