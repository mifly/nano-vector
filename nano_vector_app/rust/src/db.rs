use anyhow::{Context, Result};
use rusqlite::params;
use std::sync::Arc;

pub struct DatabaseManager {
    pub sqlite_conn: rusqlite::Connection,
}

impl DatabaseManager {
    pub async fn new(db_path: &str, _vector_db_path: &str) -> Result<Self> {
        unsafe {
            rusqlite::ffi::sqlite3_auto_extension(Some(std::mem::transmute(
                sqlite_vec::sqlite3_vec_init as *const (),
            )));
        }

        let sqlite_conn = rusqlite::Connection::open(db_path)
            .context("Failed to open SQLite database")?;

        // Initialize SQLite schema
        sqlite_conn.execute(
            "CREATE TABLE IF NOT EXISTS documents (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        sqlite_conn.execute(
            "CREATE TABLE IF NOT EXISTS chunks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                doc_id INTEGER NOT NULL,
                chunk_text TEXT NOT NULL,
                chunk_index INTEGER NOT NULL,
                FOREIGN KEY(doc_id) REFERENCES documents(id)
            )",
            [],
        )?;

        // Initialize vec0 extension table
        // bge-small-zh-v1.5 has 512 embedding dimensions
        sqlite_conn.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS vector_chunks USING vec0(
                id INTEGER PRIMARY KEY,
                vector float[512]
            )",
            [],
        )?;

        Ok(Self { sqlite_conn })
    }

    pub fn insert_document(&self, content: &str) -> Result<i64> {
        self.sqlite_conn.execute(
            "INSERT INTO documents (content) VALUES (?1)",
            params![content],
        )?;
        Ok(self.sqlite_conn.last_insert_rowid())
    }

    pub fn insert_chunk(&self, doc_id: i64, chunk_text: &str, chunk_index: usize) -> Result<i64> {
        self.sqlite_conn.execute(
            "INSERT INTO chunks (doc_id, chunk_text, chunk_index) VALUES (?1, ?2, ?3)",
            params![doc_id, chunk_text, chunk_index as i64],
        )?;
        Ok(self.sqlite_conn.last_insert_rowid())
    }

    pub async fn insert_vectors(&mut self, ids: Vec<i32>, _texts: Vec<String>, embeddings: Vec<Vec<f32>>) -> Result<()> {
        if ids.is_empty() {
            return Ok(());
        }

        for (id, embed) in ids.into_iter().zip(embeddings.into_iter()) {
            let vector_bytes: &[u8] = unsafe {
                std::slice::from_raw_parts(
                    embed.as_ptr() as *const u8,
                    embed.len() * std::mem::size_of::<f32>(),
                )
            };
            self.sqlite_conn.execute(
                "INSERT INTO vector_chunks (id, vector) VALUES (?1, ?2)",
                params![id as i64, vector_bytes],
            )?;
        }

        Ok(())
    }

    pub async fn search_vectors(&mut self, query_embedding: Vec<f32>, limit: usize) -> Result<Vec<(i32, String, f32)>> {
        let query_bytes: &[u8] = unsafe {
            std::slice::from_raw_parts(
                query_embedding.as_ptr() as *const u8,
                query_embedding.len() * std::mem::size_of::<f32>(),
            )
        };
        
        // Use JOIN to get the text, and match the vector
        let mut stmt = self.sqlite_conn.prepare(
            "SELECT v.id, c.chunk_text, distance
             FROM vector_chunks v
             JOIN chunks c ON v.id = c.id
             WHERE vector MATCH ?1 AND k = ?2
             ORDER BY distance"
        )?;

        let rows = stmt.query_map(params![query_bytes, limit as i64], |row| {
            let id: i32 = row.get(0)?;
            let text: String = row.get(1)?;
            let distance: f32 = row.get(2)?;
            Ok((id, text, distance))
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }

        Ok(results)
    }
}
