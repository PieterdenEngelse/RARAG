//! Generic parameter storage using JSON blobs.
//!
//! This module provides a simple key-value store where each config type
//! is stored as a JSON blob. This allows for flexible, type-safe config
//! storage without requiring database schema changes for new config types.

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParamStoreError {
    #[error("database error: {0}")]
    Database(String),
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error("deserialization error: {0}")]
    Deserialization(String),
    #[error("config not found: {0}")]
    NotFound(String),
    #[error("validation error: {0}")]
    Validation(String),
}

pub type Result<T> = std::result::Result<T, ParamStoreError>;

/// Initialize the app_config table if it doesn't exist.
pub fn init_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS app_config (
            config_type TEXT PRIMARY KEY,
            config_json TEXT NOT NULL,
            updated_at TEXT DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )
    .map_err(|e| ParamStoreError::Database(e.to_string()))?;
    Ok(())
}

/// Load a config by type, deserializing from JSON.
/// Returns None if the config doesn't exist.
pub fn load<T: DeserializeOwned>(conn: &Connection, config_type: &str) -> Result<Option<T>> {
    let json: Option<String> = conn
        .query_row(
            "SELECT config_json FROM app_config WHERE config_type = ?1",
            [config_type],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| ParamStoreError::Database(e.to_string()))?;

    match json {
        Some(json_str) => {
            let config: T = serde_json::from_str(&json_str)
                .map_err(|e| ParamStoreError::Deserialization(e.to_string()))?;
            Ok(Some(config))
        }
        None => Ok(None),
    }
}

/// Load a config by type, returning default if not found.
pub fn load_or_default<T: DeserializeOwned + Default>(
    conn: &Connection,
    config_type: &str,
) -> Result<T> {
    load(conn, config_type).map(|opt| opt.unwrap_or_default())
}

/// Save a config by type, serializing to JSON.
/// Uses INSERT OR REPLACE to handle both new and existing configs.
pub fn save<T: Serialize>(conn: &Connection, config_type: &str, config: &T) -> Result<()> {
    let json_str =
        serde_json::to_string(config).map_err(|e| ParamStoreError::Serialization(e.to_string()))?;

    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO app_config (config_type, config_json, updated_at)
         VALUES (?1, ?2, ?3)
         ON CONFLICT(config_type) DO UPDATE SET
            config_json = excluded.config_json,
            updated_at = excluded.updated_at",
        params![config_type, json_str, now],
    )
    .map_err(|e| ParamStoreError::Database(e.to_string()))?;

    Ok(())
}

/// Delete a config by type.
pub fn delete(conn: &Connection, config_type: &str) -> Result<bool> {
    let rows_affected = conn
        .execute(
            "DELETE FROM app_config WHERE config_type = ?1",
            [config_type],
        )
        .map_err(|e| ParamStoreError::Database(e.to_string()))?;

    Ok(rows_affected > 0)
}

/// List all config types stored in the database.
pub fn list_config_types(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn
        .prepare("SELECT config_type FROM app_config ORDER BY config_type")
        .map_err(|e| ParamStoreError::Database(e.to_string()))?;

    let types = stmt
        .query_map([], |row| row.get(0))
        .map_err(|e| ParamStoreError::Database(e.to_string()))?
        .collect::<std::result::Result<Vec<String>, _>>()
        .map_err(|e| ParamStoreError::Database(e.to_string()))?;

    Ok(types)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
    struct TestConfig {
        name: String,
        value: i32,
        enabled: bool,
    }

    fn setup_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_table(&conn).unwrap();
        conn
    }

    #[test]
    fn test_save_and_load() {
        let conn = setup_conn();
        let config = TestConfig {
            name: "test".to_string(),
            value: 42,
            enabled: true,
        };

        save(&conn, "test_config", &config).unwrap();
        let loaded: Option<TestConfig> = load(&conn, "test_config").unwrap();

        assert_eq!(loaded, Some(config));
    }

    #[test]
    fn test_load_nonexistent() {
        let conn = setup_conn();
        let loaded: Option<TestConfig> = load(&conn, "nonexistent").unwrap();
        assert_eq!(loaded, None);
    }

    #[test]
    fn test_load_or_default() {
        let conn = setup_conn();
        let loaded: TestConfig = load_or_default(&conn, "nonexistent").unwrap();
        assert_eq!(loaded, TestConfig::default());
    }

    #[test]
    fn test_update_existing() {
        let conn = setup_conn();
        let config1 = TestConfig {
            name: "first".to_string(),
            value: 1,
            enabled: false,
        };
        let config2 = TestConfig {
            name: "second".to_string(),
            value: 2,
            enabled: true,
        };

        save(&conn, "test_config", &config1).unwrap();
        save(&conn, "test_config", &config2).unwrap();

        let loaded: Option<TestConfig> = load(&conn, "test_config").unwrap();
        assert_eq!(loaded, Some(config2));
    }

    #[test]
    fn test_delete() {
        let conn = setup_conn();
        let config = TestConfig::default();

        save(&conn, "test_config", &config).unwrap();
        assert!(delete(&conn, "test_config").unwrap());

        let loaded: Option<TestConfig> = load(&conn, "test_config").unwrap();
        assert_eq!(loaded, None);
    }

    #[test]
    fn test_list_config_types() {
        let conn = setup_conn();

        save(&conn, "config_a", &TestConfig::default()).unwrap();
        save(&conn, "config_b", &TestConfig::default()).unwrap();
        save(&conn, "config_c", &TestConfig::default()).unwrap();

        let types = list_config_types(&conn).unwrap();
        assert_eq!(types, vec!["config_a", "config_b", "config_c"]);
    }
}
