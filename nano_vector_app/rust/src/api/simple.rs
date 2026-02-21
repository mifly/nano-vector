use crate::db::DatabaseManager;
use crate::embeddings::TextEmbeddingModel;
use anyhow::Result;

pub struct SearchResult {
    pub id: i32,
    pub text: String,
    pub score: f32,
}

#[flutter_rust_bridge::frb(opaque)]
pub struct AppCore {
    db: tokio::sync::Mutex<DatabaseManager>,
    model: TextEmbeddingModel,
}

impl AppCore {
    pub async fn new(db_path: String, vector_db_path: String) -> Result<AppCore> {
        let db = DatabaseManager::new(&db_path, &vector_db_path).await?;
        let model = TextEmbeddingModel::new().await?;
        Ok(AppCore {
            db: tokio::sync::Mutex::new(db),
            model,
        })
    }

    pub async fn index_text(&self, text: String) -> Result<()> {
        let mut db = self.db.lock().await;
        
        let doc_id = db.insert_document(&text)?;
        let chunks = self.model.chunk_text(&text, 200)?;
        
        let mut chunk_ids = Vec::new();
        let mut chunk_texts = Vec::new();
        
        for (i, chunk_text) in chunks.iter().enumerate() {
            let chunk_id = db.insert_chunk(doc_id, chunk_text, i)?;
            chunk_ids.push(chunk_id as i32);
            chunk_texts.push(chunk_text.clone());
        }

        let embeddings = self.model.embed_texts(
            &chunk_texts.iter().map(|s| s.as_str()).collect::<Vec<&str>>()
        )?;

        db.insert_vectors(chunk_ids, chunk_texts, embeddings).await?;
        Ok(())
    }

    pub async fn search_text(&self, query: String, limit: usize) -> Result<Vec<SearchResult>> {
        let query_embedding = self.model.embed_query(&query)?;
        
        let mut db = self.db.lock().await;
        let results = db.search_vectors(query_embedding, limit).await?;
        
        Ok(results.into_iter().map(|(id, text, score)| SearchResult { id, text, score }).collect())
    }
}

#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    flutter_rust_bridge::setup_default_user_utils();
}
