// src/installer/errors.rs
// Phase 13.1.0a: Installer Error Handling
// Version: 13.1.0

use std::io;
use thiserror::Error;
use serde_json;

/// Result type for installer operations
pub type InstallerResult<T> = Result<T, InstallerError>;

/// Comprehensive error types for the installer
#[derive(Debug, Error)]
pub enum InstallerError {
    // System & Platform
    #[error("Unsupported platform: {0}")]
    UnsupportedPlatform(String),

    #[error("Insufficient permissions - {0}")]
    PermissionDenied(String),

    // Dependencies
    #[error("Missing required dependency: {dep} - {instruction}")]
    MissingDependency { dep: String, instruction: String },

    #[error("Invalid version for {name}: required {required}, found {found}")]
    InvalidVersion {
        name: String,
        required: String,
        found: String,
    },

    #[error("Dependency check failed: {0}")]
    DependencyCheckFailed(String),

    // Filesystem
    #[error("Failed to create directory {path}: {reason}")]
    DirectoryCreationFailed { path: String, reason: String },

    #[error("Failed to read file {path}: {reason}")]
    FileReadError { path: String, reason: String },

    #[error("Failed to write file {path}: {reason}")]
    FileWriteError { path: String, reason: String },

    #[error("Path does not exist: {0}")]
    PathNotFound(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    // Configuration
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Invalid configuration value: {key} = {value}")]
    InvalidConfigValue { key: String, value: String },

    #[error("Configuration file not found: {0}")]
    ConfigNotFound(String),

    // Build
    #[error("Backend build failed: {0}")]
    BackendBuildFailed(String),

    #[error("Frontend build failed: {0}")]
    FrontendBuildFailed(String),

    #[error("Build step {step} failed: {reason}")]
    BuildStepFailed { step: String, reason: String },

    // Database
    #[error("Database initialization failed: {0}")]
    DatabaseInitFailed(String),

    #[error("Database migration failed: {0}")]
    DatabaseMigrationFailed(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    // Service
    #[error("Service registration failed: {service} - {reason}")]
    ServiceRegistrationFailed { service: String, reason: String },

    #[error("Service not found")]
    ServiceNotFound(String),

    #[error("Service operation failed: {0}")]
    ServiceOperationFailed(String),

    // Network
    #[error("Network connectivity check failed: {0}")]
    NetworkError(String),

    #[error("Port {port} is already in use")]
    PortInUse { port: u16 },

    // Execution
    #[error("Command execution failed: {cmd} - {reason}")]
    CommandFailed { cmd: String, reason: String },

    #[error("Command '{cmd}' not found - {hint}")]
    CommandNotFound { cmd: String, hint: String },

    // General
    #[error("Installation error: {0}")]
    Other(String),

    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}

impl InstallerError {
    /// Get recovery hint for this error
    pub fn recovery_hint(&self) -> String {
        match self {
            InstallerError::MissingDependency { instruction, .. } => instruction.clone(),
            InstallerError::PermissionDenied(msg) => {
                format!("Try running with sudo: sudo {}", msg)
            }
            InstallerError::PortInUse { port } => {
                format!("Port {} is in use. Kill the process or use a different port.", port)
            }
            InstallerError::CommandNotFound { hint, .. } => hint.clone(),
            _ => "Check the logs for more details.".to_string(),
        }
    }

    /// Check if this is a recoverable error
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            InstallerError::PortInUse { .. }
                | InstallerError::DirectoryCreationFailed { .. }
                | InstallerError::NetworkError(_)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_recovery_hint() {
        let err = InstallerError::MissingDependency {
            dep: "Rust".to_string(),
            instruction: "Install from https://rustup.rs".to_string(),
        };
        assert!(!err.recovery_hint().is_empty());
    }

    #[test]
    fn test_recoverable_error() {
        let err = InstallerError::PortInUse { port: 3010 };
        assert!(err.is_recoverable());
    }

    #[test]
    fn test_error_display() {
        let err = InstallerError::Other("test error".to_string());
        assert_eq!(err.to_string(), "Installation error: test error");
    }
}