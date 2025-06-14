use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::RwLock;
use crate::utils::error::{Result, AppError};
use crate::models::meme::Meme;

#[derive(Debug)]
pub struct MemeService {
    memes: HashMap<u32, Meme>,
    total_count: u32,
    content_cache: moka::future::Cache<u32, Vec<u8>>,
}

impl MemeService {
    pub async fn new(memes_dir: &str, max_size: u64, ttl_secs: u64) -> Result<Arc<RwLock<Self>>> {
        let mut memes = HashMap::new();
        let mut count = 0;

        let mut entries = tokio::fs::read_dir(memes_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            if entry.file_type().await?.is_file() {
                let path = entry.path();
                let mime_type = mime_guess::from_path(&path)
                    .first_or_octet_stream()
                    .to_string();

                let meme = Meme {
                    id: count,
                    path,
                    mime_type,
                };
                
                memes.insert(count, meme);
                count += 1;
            }
        }

        if count == 0 {
            return Err(AppError::Internal("No memes found".to_string()));
        }

        // 初始化缓存
        let content_cache = moka::future::Cache::builder()
            .max_capacity(max_size)
            .time_to_live(Duration::from_secs(ttl_secs))
            .build();

        Ok(Arc::new(RwLock::new(Self {
            memes,
            total_count: count,
            content_cache,
        })))
    }

    pub async fn get_random(&self) -> Result<(&Meme, Vec<u8>)> {
        let meme_id = fastrand::u32(..self.total_count);
        let meme = self.memes.get(&meme_id)
            .ok_or_else(|| AppError::NotFound("Meme not found".to_string()))?;

        // 尝试从缓存获取
        if let Some(content) = self.content_cache.get(&meme_id).await {
            tracing::debug!("Cache hit for meme {}", meme_id);
            return Ok((meme, content));
        }

        // 如果缓存未命中，从文件读取
        tracing::debug!("Cache miss for meme {}, reading from disk", meme_id);
        let content = tokio::fs::read(&meme.path).await?;
        self.content_cache.insert(meme_id, content.clone()).await;
        
        Ok((meme, content))
    }
}