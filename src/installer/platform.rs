// src/installer/platform.rs - v0.1.0
// Platform detection and OS-specific installation path resolution
// 
// INSTALLER IMPACT NOTES:
// - Needed for cross-platform installation (Linux, macOS, Windows)
// - Affects directory structure creation in install.sh
// - Determines dependency package availability per OS
// - Version pinning required for stable installs

use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Linux,
    MacOS,
    Windows,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct PlatformInfo {
    pub os: Platform,
    pub arch: String,
    pub home_dir: PathBuf,
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
}

impl Platform {
    /// Detect current platform
    pub fn detect() -> Self {
        match env::consts::OS {
            "linux" => Platform::Linux,
            "macos" => Platform::MacOS,
            "windows" => Platform::Windows,
            _ => Platform::Unknown,
        }
    }

    /// Get platform-specific shell
    pub fn shell(&self) -> &'static str {
        match self {
            Platform::Linux | Platform::MacOS => "/bin/bash",
            Platform::Windows => "powershell.exe",
            Platform::Unknown => "/bin/sh",
        }
    }

    /// Get package manager for platform
    pub fn package_manager(&self) -> &'static str {
        match self {
            Platform::Linux => "apt-get", // Debian-based; consider extending
            Platform::MacOS => "brew",
            Platform::Windows => "choco",
            Platform::Unknown => "unknown",
        }
    }
}

impl PlatformInfo {
    /// Initialize platform info with current environment
    pub fn new() -> Result<Self, String> {
        let os = Platform::detect();
        let arch = env::consts::ARCH.to_string();
        
        let home_dir = dirs::home_dir()
            .ok_or_else(|| "Could not determine home directory".to_string())?;
        
        let config_dir = match os {
            Platform::Linux => home_dir.join(".config/fro"),
            Platform::MacOS => home_dir.join("Library/Application Support/fro"),
            Platform::Windows => home_dir.join("AppData/Local/fro"),
            Platform::Unknown => home_dir.join(".fro"),
        };
        
        let data_dir = match os {
            Platform::Linux => home_dir.join(".local/share/fro"),
            Platform::MacOS => home_dir.join("Library/Application Support/fro/data"),
            Platform::Windows => home_dir.join("AppData/Local/fro/data"),
            Platform::Unknown => home_dir.join(".fro/data"),
        };
        
        Ok(PlatformInfo {
            os,
            arch,
            home_dir,
            config_dir,
            data_dir,
        })
    }

    /// Create required directories for installation
    pub fn ensure_directories(&self) -> Result<(), String> {
        std::fs::create_dir_all(&self.config_dir)
            .map_err(|e| format!("Failed to create config dir: {}", e))?;
        std::fs::create_dir_all(&self.data_dir)
            .map_err(|e| format!("Failed to create data dir: {}", e))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detection() {
        let platform = Platform::detect();
        assert_ne!(platform, Platform::Unknown);
    }

    #[test]
    fn test_platform_info_creation() {
        let info = PlatformInfo::new();
        assert!(info.is_ok());
    }
}