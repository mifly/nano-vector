use anyhow::{Context, Result};
use candle_core::{Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config, DTYPE};
use hf_hub::{api::sync::Api, Repo, RepoType};
use tokenizers::Tokenizer;

pub struct TextEmbeddingModel {
    model: BertModel,
    tokenizer: Tokenizer,
    device: Device,
}

impl TextEmbeddingModel {
    pub async fn new() -> Result<Self> {
        let device = Device::Cpu;

        println!("Loading bge-small-zh-v1.5 model from Hugging Face Hub (this might take a while on first run)...");
        let api = hf_hub::api::tokio::Api::new().context("Failed to create Hugging Face API")?;
        let repo = api.repo(Repo::with_revision(
            "BAAI/bge-small-zh-v1.5".to_string(),
            RepoType::Model,
            "main".to_string(),
        ));

        let config_filename = repo.get("config.json").await.context("Failed to get config.json")?;
        let tokenizer_filename = repo.get("tokenizer.json").await.context("Failed to get tokenizer.json")?;
        let weights_filename = repo.get("model.safetensors").await.context("Failed to get model.safetensors")?;

        println!("Initializing Tokenizer...");
        let tokenizer = Tokenizer::from_file(tokenizer_filename)
            .map_err(|e| anyhow::anyhow!("Failed to parse tokenizer: {}", e))?;

        println!("Loading Config...");
        let config = std::fs::read_to_string(config_filename)?;
        let mut config: Config = serde_json::from_str(&config)?;
        // Use a fast approx gelu if desired, but default is fine

        println!("Loading Model Weights...");
        let vb = unsafe { VarBuilder::from_mmaped_safetensors(&[weights_filename], DTYPE, &device)? };
        let model = BertModel::load(vb, &config)?;

        Ok(Self {
            model,
            tokenizer,
            device,
        })
    }

    /// Split text into chunks of at most max_tokens
    pub fn chunk_text(&self, text: &str, max_tokens: usize) -> Result<Vec<String>> {
        let encoding = self.tokenizer.encode(text, true)
            .map_err(|e| anyhow::anyhow!("Tokenization failed: {}", e))?;
        
        let ids = encoding.get_ids();
        let mut chunks = Vec::new();

        for chunk_ids in ids.chunks(max_tokens) {
            let chunk_str = self.tokenizer.decode(chunk_ids, true)
                .map_err(|e| anyhow::anyhow!("Decoding failed: {}", e))?;
            if !chunk_str.trim().is_empty() {
                chunks.push(chunk_str);
            }
        }
        
        Ok(chunks)
    }

    /// Generate embeddings for a list of texts (e.g. chunks)
    pub fn embed_texts(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        let mut embeddings = Vec::new();
        
        for text in texts {
            // Encode the text
            let encoding = self.tokenizer.encode(*text, true)
                .map_err(|e| anyhow::anyhow!("Tokenization error: {}", e))?;
            
            let ids = encoding.get_ids().to_vec();
            let len = ids.len();
            
            // Create a tensor for the input IDs, shape [1, seq_len]
            let token_ids = Tensor::new(ids.as_slice(), &self.device)?
                .unsqueeze(0)?;
            
            // Create attention mask (all 1s)
            let token_type_ids = Tensor::zeros(token_ids.shape(), candle_core::DType::U32, &self.device)?;

            // Run through BERT
            let output = self.model.forward(&token_ids, &token_type_ids, None)?;
            
            // bge models usually use the CLS token (first token) or mean pooling.
            // BAAI/bge usually uses CLS pooling. 
            // output shape: [1, seq_len, hidden_size]
            // We slice the first token across the sequence dimension (dim=1)
            let cls_embedding = output.narrow(1, 0, 1)?.squeeze(1)?; // [1, hidden_size]
            
            // Normalize the embeddings (L2 norm) as bge recommends
            let sq = cls_embedding.sqr()?.sum_keepdim(1)?;
            let norm = sq.sqrt()?;
            let normalized_embedding = cls_embedding.broadcast_div(&norm)?;
            
            // Convert to Vec<f32>
            let embedding_vec: Vec<f32> = normalized_embedding.squeeze(0)?.to_vec1()?;
            embeddings.push(embedding_vec);
        }
        
        Ok(embeddings)
    }

    /// Generate an embedding for a search query.
    /// bge models typically require a prefix for retrieval tasks.
    pub fn embed_query(&self, query: &str) -> Result<Vec<f32>> {
        let prefixed_query = format!("为这个句子生成表示以用于检索相关文章：{}", query);
        let mut res = self.embed_texts(&[&prefixed_query])?;
        Ok(res.pop().unwrap())
    }
}
