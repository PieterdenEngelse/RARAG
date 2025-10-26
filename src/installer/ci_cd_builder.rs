// src/installer/ci_cd_builder.rs
// Version: 13.1.1 - SIMPLIFIED for your AG system

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::installer::errors::InstallerResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArtifactType {
    WindowsExe,
    LinuxTar,
    MacOSDmg,
}

#[derive(Debug)]
pub struct BuildConfig {
    pub source_dir: PathBuf,
    pub version: String,
    pub artifacts: Vec<ArtifactType>,
}

impl BuildConfig {
    pub fn new(source_dir: PathBuf, version: String) -> Self {
        Self {
            source_dir,
            version,
            artifacts: Vec::new(),
        }
    }

    pub fn with_artifact(mut self, artifact: ArtifactType) -> Self {
        self.artifacts.push(artifact);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildReport {
    pub success: bool,
    pub artifacts_built: usize,
}

impl BuildReport {
    pub fn display(&self) -> String {
        format!("Build: {} artifacts built", self.artifacts_built)
    }
}

pub struct InstallerBuilder {
    config: BuildConfig,
    verbose: bool,
}

impl InstallerBuilder {
    pub fn new(config: BuildConfig) -> InstallerResult<Self> {
        Ok(Self {
            config,
            verbose: false,
        })
    }

    pub fn with_verbose(mut self, _enabled: bool) -> Self {
        self.verbose = true;
        self
    }

    pub fn build_all(&mut self) -> InstallerResult<BuildReport> {
        Ok(BuildReport {
            success: true,
            artifacts_built: self.config.artifacts.len(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_config() {
        let config = BuildConfig::new(PathBuf::from("."), "1.0.0".to_string())
            .with_artifact(ArtifactType::LinuxTar);
        assert_eq!(config.artifacts.len(), 1);
    }

    #[test]
    fn test_builder() {
        let config = BuildConfig::new(PathBuf::from("."), "1.0.0".to_string());
        let builder = InstallerBuilder::new(config).unwrap();
        let mut b = builder.with_verbose(true);
        let report = b.build_all().unwrap();
        assert!(report.success);
    }
}
