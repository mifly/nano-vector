use anyhow::{Context, Result};
use arrow_array::{
    builder::{Float32Builder, FixedSizeListBuilder, Int32Builder, StringBuilder},
    cast::AsArray,
    RecordBatch,
    RecordBatchIterator,
};
use arrow_schema::{DataType, Field, Schema};
use futures::StreamExt;
use lancedb::connection::Connection;
use lancedb::table::Table;
use lancedb::query::{ExecutableQuery, QueryBase};
use rusqlite::params;
use std::sync::Arc;

pub struct DatabaseManager {
    pub sqlite_conn: rusqlite::Connection,
    pub lance_db: Connection,
    pub lance_table: Option<Table>,
}

impl DatabaseManager {
    pub async fn new(db_path: &str, vector_db_path: &str) -> Result<Self> {
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

        // Initialize LanceDB connection
        let lance_db = lancedb::connect(vector_db_path).execute().await
            .context("Failed to connect to LanceDB")?;

        let table_names = lance_db.table_names().execute().await?;
        let table = if table_names.contains(&"vector_chunks".to_string()) {
            Some(lance_db.open_table("vector_chunks").execute().await?)
        } else {
            None
        };

        Ok(Self {
            sqlite_conn,
            lance_db,
            lance_table: table,
        })
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

    pub async fn insert_vectors(&mut self, ids: Vec<i32>, texts: Vec<String>, embeddings: Vec<Vec<f32>>) -> Result<()> {
        if ids.is_empty() {
            return Ok(());
        }

        // Define LanceDB schema
        let dim = embeddings[0].len() as i32;
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new("text", DataType::Utf8, false),
            Field::new(
                "vector",
                DataType::FixedSizeList(
                    Arc::new(Field::new("item", DataType::Float32, true)),
                    dim,
                ),
                false,
            ),
        ]));

        // Build Arrow arrays
        let mut id_builder = Int32Builder::new();
        let mut text_builder = StringBuilder::new();
        
        let vector_values_builder = Float32Builder::new();
        let mut vector_builder = FixedSizeListBuilder::new(vector_values_builder, dim);

        for (i, ((id, text), embed)) in ids.into_iter().zip(texts.into_iter()).zip(embeddings.into_iter()).enumerate() {
            id_builder.append_value(id);
            text_builder.append_value(text);
            
            for &val in &embed {
                vector_builder.values().append_value(val);
            }
            vector_builder.append(true);
        }

        let id_array = Arc::new(id_builder.finish());
        let text_array = Arc::new(text_builder.finish());
        let vector_array = Arc::new(vector_builder.finish());

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![id_array, text_array, vector_array],
        )?;
        
        let batches = RecordBatchIterator::new(vec![Ok(batch)], schema.clone());

        if let Some(table) = &self.lance_table {
            table.add(batches).execute().await?;
        } else {
            let table = self.lance_db
                .create_table("vector_chunks", batches)
                .execute()
                .await?;
            self.lance_table = Some(table);
        }

        Ok(())
    }

    pub async fn search_vectors(&mut self, query_embedding: Vec<f32>, limit: usize) -> Result<Vec<(i32, String, f32)>> {
        let table = self.lance_table.as_ref()
            .context("Vector table not initialized (empty db)")?;
            
        let mut search = table.query().nearest_to(query_embedding)?.limit(limit).execute().await?;

        let mut results = Vec::new();

        while let Some(batch_res) = search.next().await {
            let batch: RecordBatch = batch_res?;
            let id_array = batch.column_by_name("id").unwrap().as_primitive::<arrow_array::types::Int32Type>();
            let text_array = batch.column_by_name("text").unwrap().as_string::<i32>();
            
            // LanceDB also returns an implicit _distance column in search results
            let distance_array = batch.column_by_name("_distance").unwrap().as_primitive::<arrow_array::types::Float32Type>();
            
            for i in 0..batch.num_rows() {
                results.push((
                    id_array.value(i),
                    text_array.value(i).to_string(),
                    distance_array.value(i),
                ));
            }
        }
        
        Ok(results)
    }
}
