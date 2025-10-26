// src/installer/mod.rs
// Phase 13.1.1: Installer Framework - Core Module
// Version: 13.1.1

// Existing Phase 13.1.0 modules
pub mod platform;
pub mod logger;
pub mod errors;
pub mod config;
pub mod detector;
pub mod checks;
pub mod wizard;
pub mod health;

// NEW Phase 13.1.1 modules
pub mod uninstaller;
pub mod platform_installers;
pub mod env_validator;
pub mod ci_cd_builder;

// Re-exports
pub use errors::{InstallerError, InstallerResult};
pub use platform::Platform;
pub use logger::InstallLogger;
pub use uninstaller::{Uninstaller, UninstallReport};
pub use platform_installers::PlatformPaths;
pub use env_validator::{PreInstallValidator, PostInstallValidator};
pub use ci_cd_builder::InstallerBuilder;