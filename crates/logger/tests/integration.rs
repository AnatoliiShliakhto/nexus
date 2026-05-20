#![allow(unused_crate_dependencies)]

use nx_logger::Logger;
use std::fs;
use tempfile::tempdir;
use tracing::{info, info_span};

#[test]
fn test_file_logging_and_field_redaction() {
    let dir = tempdir().unwrap();
    let log_dir = dir.path().join("logs");

    let _logger_res =
        Logger::builder().name("auth-service").path(&log_dir).json(true).console(false).init();

    let span = info_span!("user_login", user_id = 42, ip = "127.0.0.1");
    let enter = span.enter();

    info!(
        password = "super_secret_password",
        action = "login_attempt",
        success = true,
        "User attempted to login"
    );

    drop(enter);

    let mut log_content = String::new();
    for entry in fs::read_dir(&log_dir).unwrap() {
        let entry = entry.unwrap();
        if entry.path().extension().and_then(|e| e.to_str()) == Some("log") {
            log_content = fs::read_to_string(entry.path()).unwrap();
            break;
        }
    }

    assert!(!log_content.is_empty(), "Log file was not created or is empty");

    let lines: Vec<serde_json::Value> = log_content
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| serde_json::from_str(line).expect(&format!("Invalid JSON line: {line}")))
        .collect();

    let parsed_log = lines.last().expect("No logs found");

    assert_eq!(parsed_log["message"], "User attempted to login");
    assert_eq!(parsed_log["action"], "login_attempt");
    assert_eq!(parsed_log["success"], true);

    assert!(parsed_log.get("password").is_none(), "CRITICAL: Password leaked into logs!");

    if let Some(spans) = parsed_log.get("spans") {
        let spans_array = spans.as_array().unwrap();
        assert!(spans_array.contains(&serde_json::Value::String("user_login".to_owned())));

        assert_eq!(parsed_log["user_id"], 42);
        assert_eq!(parsed_log["ip"], "127.0.0.1");
    }
}
