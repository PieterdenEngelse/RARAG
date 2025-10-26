// src/memory/persistence.rs
// Persistence layer: save/load vector store metadata to disk

use crate::memory::VectorStore;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::{debug, info};

/// Serializable snapshot of vector store for persistence (metadata only)
#[derive(Debug, Serialize, Deserialize)]
pub struct VectorStoreSnapshot {
    pub record_count: usize,
    pub version: u32,
    pub timestamp: i64,
}

impl VectorStoreSnapshot {
    /// Create snapshot from vector store
    pub async fn from_store(store: &VectorStore) -> Result<Self, PersistenceError> {
        let stats = store.stats().await;
        let timestamp = chrono::Utc::now().timestamp();

        Ok(Self {
            record_count: stats.total_records,
            version: 1,
            timestamp,
        })
    }
}

/// Persistence error types
#[derive(Debug)]
pub enum PersistenceError {
    SerializationError(String),
    IoError(String),
    NotFound(String),
}

impl std::fmt::Display for PersistenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            Self::IoError(msg) => write!(f, "IO error: {}", msg),
            Self::NotFound(msg) => write!(f, "Not found: {}", msg),
        }
    }
}

impl std::error::Error for PersistenceError {}

impl From<serde_json::Error> for PersistenceError {
    fn from(err: serde_json::Error) -> Self {
        PersistenceError::SerializationError(err.to_string())
    }
}

impl From<std::io::Error> for PersistenceError {
    fn from(err: std::io::Error) -> Self {
        PersistenceError::IoError(err.to_string())
    }
}

impl From<crate::memory::vector_store::VectorStoreError> for PersistenceError {
    fn from(err: crate::memory::vector_store::VectorStoreError) -> Self {
        PersistenceError::SerializationError(err.to_string())
    }
}

/// Save vector store metadata to JSON file
pub async fn save_vector_store<P: AsRef<Path>>(
    store: &VectorStore,
    path: P,
) -> Result<(), PersistenceError> {
    let path = path.as_ref();
    debug!(path = ?path, "Saving vector store metadata");

    let snapshot = VectorStoreSnapshot::from_store(store).await?;
    let json = serde_json::to_string_pretty(&snapshot)?;

    std::fs::write(path, json)?;
    info!(path = ?path, records = snapshot.record_count, "Vector store metadata saved");

    Ok(())
}

/// Load vector store metadata from JSON file
pub async fn load_vector_store<P: AsRef<Path>>(
    path: P,
) -> Result<VectorStore, PersistenceError> {
    let path = path.as_ref();
    debug!(path = ?path, "Loading vector store metadata");

    if !path.exists() {
        return Err(PersistenceError::NotFound(format!(
            "Vector store file not found: {:?}",
            path
        )));
    }

    let json = std::fs::read_to_string(path)?;
    let snapshot: VectorStoreSnapshot = serde_json::from_str(&json)?;

    let store = VectorStore::with_defaults()?;

    info!(path = ?path, records = snapshot.record_count, "Vector store metadata loaded");
    Ok(store)
}

/// Backup vector store metadata (creates timestamped copy)
pub async fn backup_vector_store<P: AsRef<Path>>(
    store: &VectorStore,
    backup_dir: P,
) -> Result<PathBuf, PersistenceError> {
    let backup_dir = backup_dir.as_ref();
    std::fs::create_dir_all(backup_dir)?;

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let filename = format!("vector_store_backup_{}.json", timestamp);
    let backup_path = backup_dir.join(filename);

    save_vector_store(store, &backup_path).await?;
    info!(path = ?backup_path, "Vector store metadata backed up");

    Ok(backup_path)
}