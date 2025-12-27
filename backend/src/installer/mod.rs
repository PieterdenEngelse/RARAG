// src/installer/mod.rs
// Phase 13.1.1: Installer Framework - Core Module
// Version: 13.1.1

// Existing Phase 13.1.0 modules
pub mod checks;
pub mod config;
pub mod detector;
pub mod errors;
pub mod health;
pub mod logger;
pub mod platform;
pub mod wizard;

// NEW Phase 13.1.1 modules
pub mod ci_cd_builder;
pub mod env_validator;
pub mod platform_installers;
pub mod uninstaller;

// Re-exports
pub use ci_cd_builder::InstallerBuilder;
pub use env_validator::{PostInstallValidator, PreInstallValidator};
pub use errors::{InstallerError, InstallerResult};
pub use logger::InstallLogger;
pub use platform::Platform;
pub use platform_installers::PlatformPaths;
pub use uninstaller::{UninstallReport, Uninstaller};
