// src/installer/logger.rs - v0.1.0
// Installation process logging with levels and formatted output
//
// INSTALLER IMPACT NOTES:
// - Logs all installation steps for debugging
// - Outputs to both console and optional log file
// - Critical for troubleshooting installer failures
// - Affects user feedback during installation

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

pub struct InstallLogger {
    level: LogLevel,
    log_file: Option<Mutex<std::fs::File>>,
}

impl InstallLogger {
    /// Create a new logger
    pub fn new(level: LogLevel, log_path: Option<PathBuf>) -> Result<Self, String> {
        let log_file = if let Some(path) = log_path {
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
                .map_err(|e| format!("Failed to open log file: {}", e))?;
            Some(Mutex::new(file))
        } else {
            None
        };

        Ok(InstallLogger { level, log_file })
    }

    /// Log with specified level
    pub fn log(&self, level: LogLevel, message: &str) {
        if level as u8 >= self.level as u8 {
            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
            let level_str = match level {
                LogLevel::Debug => "[DEBUG]",
                LogLevel::Info => "[INFO]",
                LogLevel::Warn => "[WARN]",
                LogLevel::Error => "[ERROR]",
            };

            let formatted = format!("{} {} {}", timestamp, level_str, message);

            // Console output
            match level {
                LogLevel::Error => eprintln!("{}", formatted),
                _ => println!("{}", formatted),
            }

            // File output
            if let Some(ref mutex_file) = self.log_file {
                if let Ok(mut file) = mutex_file.lock() {
                    let _ = writeln!(file, "{}", formatted);
                }
            }
        }
    }

    pub fn debug(&self, msg: &str) {
        self.log(LogLevel::Debug, msg);
    }
    pub fn info(&self, msg: &str) {
        self.log(LogLevel::Info, msg);
    }
    pub fn warn(&self, msg: &str) {
        self.log(LogLevel::Warn, msg);
    }
    pub fn error(&self, msg: &str) {
        self.log(LogLevel::Error, msg);
    }
}

impl Default for InstallLogger {
    fn default() -> Self {
        InstallLogger {
            level: LogLevel::Info,
            log_file: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger_creation() {
        let logger = InstallLogger::new(LogLevel::Info, None);
        assert!(logger.is_ok());
    }

    #[test]
    fn test_logger_default() {
        let logger = InstallLogger::default();
        assert_eq!(logger.level, LogLevel::Info);
    }
}
