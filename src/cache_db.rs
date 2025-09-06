use super::datatype::*;
use chrono::Timelike;
use rusqlite::{Connection, params};
use serde_json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CacheError {
    #[error("Serde Error: {0}")]
    Serde(serde_json::Error),

    #[error("Sqlite Error: {0}")]
    Sqlite(rusqlite::Error),
}

impl From<serde_json::Error> for CacheError {
    fn from(err: serde_json::Error) -> CacheError {
        CacheError::Serde(err)
    }
}

impl From<rusqlite::Error> for CacheError {
    fn from(err: rusqlite::Error) -> CacheError {
        CacheError::Sqlite(err)
    }
}

pub type CacheResult<T> = Result<T, CacheError>;

pub struct Cache {
    conn: Connection,
}

impl Cache {
    pub fn new(db_path: &str) -> CacheResult<Self> {
        let conn = Connection::open(db_path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS metadata (
            id TEXT PRIMARY KEY,
            serialized_data TEXT NOT NULL,
            name TEXT NOT NULL,
            b_finished BOOL NOT NULL,
            episode INTEGER,
            time_at_episode INTEGER,
            season INTERGER,
            logged_time INTEGER,
            note TEXT)",
            [],
        )?;
        Ok(Cache { conn })
    }

    pub fn query_cache(&self, hash_value: u64) -> Option<Metadata> {
        if let Ok(mut stmt) = self
            .conn
            .prepare("SELECT serialized_data FROM metadata WHERE id = ?1")
        {
            return stmt
                .query_row(params![hash_value.to_string()], |row| {
                    let serialized_data: String = row.get(0)?;
                    let metadata: Metadata =
                        serde_json::from_str(&serialized_data).map_err(|e| {
                            rusqlite::Error::FromSqlConversionFailure(
                                0,
                                rusqlite::types::Type::Text,
                                Box::new(e),
                            )
                        })?;
                    Ok(metadata)
                })
                .ok();
        }
        None
    }

    pub fn add_or_update_cache(&self, metadata: &Metadata) -> CacheResult<()> {
        let mut stmt = self.conn.prepare(
            "INSERT INTO metadata (
            id,
            serialized_data,
            name,
            b_finished,
            episode,
            time_at_episode,
            season,
            logged_time,
            note) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        )?;
        let serialized_data = serde_json::to_string(&metadata)?;
        stmt.execute(params![
            metadata.id.to_string(),
            serialized_data,
            metadata.name,
            metadata.b_finished,
            metadata.episode,
            metadata
                .time_at_episode
                .map(|t| t.num_seconds_from_midnight()),
            metadata.season,
            metadata.logged_time.map(|t| t.and_utc().timestamp()),
            metadata.note
        ])?;
        Ok(())
    }
}
