// ag/src/db/schema_init.rs v13.1.2
use crate::path_manager::PathManager;
use rusqlite::{Connection, Result as SqlResult};
use tracing::info;

pub struct SchemaInitializer;

impl SchemaInitializer {
    pub fn init(db_conn: &Connection) -> SqlResult<()> {
        info!("Initializing database schema v13.1.2");
        let schema_sql = include_str!("../db/schema.sql");
        db_conn.execute_batch(schema_sql)?;
        info!("Database schema initialized");
        Ok(())
    }

    pub fn create_fresh_db(path_manager: &PathManager) -> SqlResult<Connection> {
        let db_path = path_manager.db_path("documents");
        info!("Creating database at: {}", db_path.display());
        let conn = Connection::open(&db_path)?;
        Self::init(&conn)?;
        Ok(conn)
    }

    pub fn migrate(db_conn: &Connection, target_version: &str) -> SqlResult<()> {
        info!("Migrating database to version {}", target_version);
        Self::init(db_conn)?;
        Ok(())
    }
}
