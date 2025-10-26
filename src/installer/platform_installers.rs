
// src/installer/platform_installers.rs
// Version: 13.1.1 - SIMPLIFIED for your AG system

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::installer::errors::InstallerResult;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Platform {
    Linux,
    MacOS,
    Windows,
}

impl Platform {
    pub fn current() -> Self {
        #[cfg(target_os = "linux")]
        return Platform::Linux;
        #[cfg(target_os = "macos")]
        return Platform::MacOS;
        #[cfg(target_os = "windows")]
        return Platform::Windows;
    }

    pub fn as_str(&self) -> &str {
        match self {
            Platform::Linux => "Linux",
            Platform::MacOS => "macOS",
            Platform::Windows => "Windows",
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlatformPaths {
    pub install_root: PathBuf,
    pub bin_dir: PathBuf,
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub log_dir: PathBuf,
}

impl PlatformPaths {
    pub fn for_platform(_platform: Platform, prefix: Option<PathBuf>) -> Self {
        let root = prefix.unwrap_or_else(|| PathBuf::from("/opt/ag"));
        Self {
            install_root: root.clone(),
            bin_dir: root.join("bin"),
            config_dir: root.join("config"),
            data_dir: root.join("data"),
            cache_dir: root.join(".cache"),
            log_dir: root.join("logs"),
        }
    }

    pub fn create_all(&self) -> InstallerResult<()> {
        std::fs::create_dir_all(&self.bin_dir).ok();
        std::fs::create_dir_all(&self.config_dir).ok();
        std::fs::create_dir_all(&self.data_dir).ok();
        std::fs::create_dir_all(&self.cache_dir).ok();
        std::fs::create_dir_all(&self.log_dir).ok();
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub binary_path: PathBuf,
}

impl ServiceConfig {
    pub fn new(_platform: Platform, binary_path: PathBuf) -> Self {
        Self { binary_path }
    }

    pub fn generate_install_script(&self) -> InstallerResult<String> {
        Ok(format!("# Service script for {:?}", self.binary_path))
    }
}

#[derive(Debug)]
pub struct EnvironmentSetup;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_current() {
        let platform = Platform::current();
        assert!(!platform.as_str().is_empty());
    }

    #[test]
    fn test_platform_paths() {
        let paths = PlatformPaths::for_platform(Platform::Linux, Some(PathBuf::from("/test")));
        assert_eq!(paths.install_root, PathBuf::from("/test"));
    }
}
