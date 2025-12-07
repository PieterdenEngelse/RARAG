//! Frontend logging for Dioxus
//!
//! Logs to browser console with structured format

use serde_json::json;
use std::sync::Once;

static INIT: Once = Once::new();

pub struct Logger;

impl Logger {
    /// Initialize logging (call once at app startup)
    pub fn init() {
        INIT.call_once(|| {
            Self::log_info("Frontend monitoring initialized");
        });
    }

    /// Log an info message
    pub fn log_info(msg: &str) {
        Self::log_with_level("INFO", msg);
    }

    /// Log a warning message
    pub fn warn(msg: &str) {
        Self::log_with_level("WARN", msg);
    }

    /// Log an error message
    pub fn error(msg: &str) {
        Self::log_with_level("ERROR", msg);
        Self::report_error_to_backend(msg);
    }

    /// Log a debug message
    pub fn debug(msg: &str) {
        Self::log_with_level("DEBUG", msg);
    }

    /// Generic log method
    pub fn log(msg: &str) {
        Self::log_info(msg);
    }

    /// Log with level and timestamp
    fn log_with_level(level: &str, msg: &str) {
        let timestamp = Self::timestamp();
        let log_entry = json!({
            "timestamp": timestamp,
            "level": level,
            "message": msg,
        });

        let log_line = format!("[{}] {} - {}", timestamp, level, msg);

        // Log to browser console
        match level {
            "ERROR" => {
                web_sys::console::error_1(&log_line.into());
            }
            "WARN" => {
                web_sys::console::warn_1(&log_line.into());
            }
            "DEBUG" => {
                web_sys::console::debug_1(&log_line.into());
            }
            _ => {
                web_sys::console::log_1(&log_line.into());
            }
        }

        // Store in local log buffer
        Self::store_log_entry(log_entry);
    }

    /// Report error to backend
    fn report_error_to_backend(msg: &str) {
        // This will be implemented in Phase 15 when we add backend integration
        let _ = msg;
    }

    /// Store log entry in memory (up to 100 entries)
    fn store_log_entry(entry: serde_json::Value) {
        // Store in thread-local storage or global state
        // For now, this is a placeholder
        let _ = entry;
    }

    /// Get current timestamp
    fn timestamp() -> String {
        chrono::Local::now().format("%H:%M:%S%.3f").to_string()
    }

    /// Export all logs as JSON
    pub fn export_logs() -> String {
        json!([]).to_string()
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
    use super::*;

    #[test]
    fn test_logger_init() {
        Logger::init();
        Logger::log("Test message");
        Logger::warn("Test warning");
        Logger::error("Test error");
    }
}
