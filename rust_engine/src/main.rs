mod db;
mod embeddings;

use anyhow::Result;
use clap::{Parser, Subcommand};
use db::DatabaseManager;
use embeddings::TextEmbeddingModel;
use std::path::Path;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Index a given text document
    Index {
        #[arg(short, long)]
        text: String,
    },
    /// Search using a natural language query
    Search {
        #[arg(short, long)]
        query: String,
        #[arg(short, long, default_value_t = 3)]
        limit: usize,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Ensure data directory exists
    std::fs::create_dir_all("data")?;

    println!("Initializing Database...");
    let mut db_manager = DatabaseManager::new("data/app.db", "data/lancedb").await?;

    match &cli.command {
        Commands::Index { text } => {
            println!("Initializing Embedding Model...");
            let model = TextEmbeddingModel::new()?;

            println!("Indexing document (length: {} chars)...", text.len());
            let doc_id = db_manager.insert_document(text)?;

            // Chunk the text into 200 tokens max
            let chunks = model.chunk_text(text, 200)?;
            println!("Split text into {} chunks.", chunks.len());

            let mut chunk_ids = Vec::new();
            let mut chunk_texts = Vec::new();
            
            for (i, chunk_text) in chunks.iter().enumerate() {
                let chunk_id = db_manager.insert_chunk(doc_id, chunk_text, i)?;
                chunk_ids.push(chunk_id as i32);
                chunk_texts.push(chunk_text.clone());
            }

            // Generate embeddings
            println!("Generating embeddings...");
            let embeddings = model.embed_texts(
                &chunk_texts.iter().map(|s| s.as_str()).collect::<Vec<&str>>()
            )?;

            println!("Saving to Vector DB...");
            db_manager.insert_vectors(chunk_ids, chunk_texts, embeddings).await?;

            println!("Indexing complete!");
        }
        Commands::Search { query, limit } => {
            println!("Initializing Embedding Model...");
            let model = TextEmbeddingModel::new()?;

            println!("Encoding query...");
            let query_embedding = model.embed_query(query)?;

            println!("Searching Vector DB...");
            let results = db_manager.search_vectors(query_embedding, *limit).await?;

            if results.is_empty() {
                println!("No results found.");
            } else {
                println!("Top {} Results:", limit);
                for (i, (id, text, distance)) in results.iter().enumerate() {
                    println!("\n[{}] ID: {} | Distance: {:.4}", i + 1, id, distance);
                    println!("Text: {}", text);
                }
            }
        }
    }

    Ok(())
}
