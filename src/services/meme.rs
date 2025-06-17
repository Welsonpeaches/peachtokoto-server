use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
    time::{Duration, SystemTime, Instant},
    path::PathBuf,
};
use tokio::sync::{RwLock, broadcast};
use crate::utils::error::{Result, AppError};
use crate::models::meme::Meme;
use tracing::{info, error};
use notify::{RecursiveMode, Watcher};
use std::sync::atomic::{AtomicU64, Ordering};
use parking_lot::Mutex;

const REQUEST_HISTORY_WINDOW: Duration = Duration::from_secs(60 * 15); // 扩展到15分钟
const ONE_MINUTE: Duration = Duration::from_secs(60);
const FIVE_MINUTES: Duration = Duration::from_secs(60 * 5);
const FIFTEEN_MINUTES: Duration = Duration::from_secs(60 * 15);

#[derive(Debug)]
pub struct MemeService {
    memes: HashMap<u32, Meme>,
    total_count: u32,
    content_cache: moka::future::Cache<u32, Vec<u8>>,
    memes_dir: PathBuf,
    reload_tx: broadcast::Sender<()>,
    _watcher: notify::RecommendedWatcher,
    request_count: AtomicU64,
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
    start_time: SystemTime,
    request_timestamps: Mutex<VecDeque<Instant>>,
    last_updated: Mutex<SystemTime>,
}

impl MemeService {
    pub async fn new(memes_dir: &str, max_size: u64, ttl_secs: u64) -> Result<Arc<RwLock<Self>>> {
        let memes_dir = PathBuf::from(memes_dir);
        let (reload_tx, _) = broadcast::channel(1);
        
        // 创建文件监控
        let reload_tx_clone = reload_tx.clone();
        let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
            match res {
                Ok(event) => {
                    // 只输出变更的文件路径
                    for path in event.paths {
                        info!("检测到文件变更: {}", path.display());
                    }
                    if let Err(e) = reload_tx_clone.send(()) {
                        error!("发送重载信号失败: {}", e);
                    }
                }
                Err(e) => error!("监控文件出错: {}", e),
            }
        })?;

        // 开始监控目录
        watcher.watch(&memes_dir, RecursiveMode::Recursive)?;
        info!("开始监控目录: {:?}", memes_dir);

        // 初始化缓存
        let content_cache = moka::future::Cache::builder()
            .max_capacity(max_size)
            .time_to_live(Duration::from_secs(ttl_secs))
            .build();

        // 创建服务实例
        let service = Arc::new(RwLock::new(Self {
            memes: HashMap::new(),
            total_count: 0,
            content_cache,
            memes_dir: memes_dir.clone(),
            reload_tx,
            _watcher: watcher,
            request_count: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            start_time: SystemTime::now(),
            request_timestamps: Mutex::new(VecDeque::with_capacity(1000)),
            last_updated: Mutex::new(SystemTime::now()),
        }));

        // 初始加载表情包
        service.write().await.reload_memes().await?;

        // 启动重载监听器
        Self::start_reload_listener(Arc::clone(&service));

        Ok(service)
    }

    async fn reload_memes(&mut self) -> Result<()> {
        let mut memes = HashMap::new();
        let mut count = 0;

        let mut entries = tokio::fs::read_dir(&self.memes_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            if entry.file_type().await?.is_file() {
                let path = entry.path();
                let mime_type = mime_guess::from_path(&path)
                    .first_or_octet_stream()
                    .to_string();

                let filename = path.file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                let size_bytes = tokio::fs::metadata(&path)
                    .await
                    .map(|metadata| metadata.len())
                    .unwrap_or(0);

                let meme = Meme {
                    id: count,
                    path,
                    mime_type,
                    filename,
                    size_bytes,
                };
                
                memes.insert(count, meme);
                count += 1;
            }
        }

        if count == 0 {
            return Err(AppError::Internal("No memes found".to_string()));
        }

        // 更新服务状态
        self.memes = memes;
        self.total_count = count;
        self.content_cache.invalidate_all();
        *self.last_updated.lock() = SystemTime::now();

        info!("重新加载了 {} 个表情包", count);
        Ok(())
    }

    fn start_reload_listener(service: Arc<RwLock<Self>>) {
        tokio::spawn(async move {
            loop {
                let mut rx = {
                    let service = service.read().await;
                    service.reload_tx.subscribe()
                };

                // 等待重载信号
                while let Ok(()) = rx.recv().await {
                    info!("正在重新加载表情包...");
                    if let Err(e) = service.write().await.reload_memes().await {
                        error!("重新加载表情包失败: {}", e);
                    }
                }

                // 如果 channel 关闭，等待一段时间后重试
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });
    }

    pub async fn get_random(&self) -> Result<(&Meme, Vec<u8>)> {
        // 增加请求计数并记录时间戳
        self.request_count.fetch_add(1, Ordering::Relaxed);
        self.record_request();
        
        let meme_id = fastrand::u32(..self.total_count);
        let meme = self.memes.get(&meme_id)
            .ok_or_else(|| AppError::NotFound("Meme not found".to_string()))?;

        // 尝试从缓存获取
        if let Some(content) = self.content_cache.get(&meme_id).await {
            tracing::debug!("Cache hit for meme {}", meme_id);
            self.cache_hits.fetch_add(1, Ordering::Relaxed);
            return Ok((meme, content));
        }

        // 如果缓存未命中，从文件读取
        tracing::debug!("Cache miss for meme {}, reading from disk", meme_id);
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
        let content = tokio::fs::read(&meme.path).await?;
        self.content_cache.insert(meme_id, content.clone()).await;
        
        Ok((meme, content))
    }

    pub fn get_request_count(&self) -> u64 {
        self.request_count.load(Ordering::Relaxed)
    }

    pub fn get_total_memes(&self) -> usize {
        self.memes.len()
    }

    pub fn get_start_time(&self) -> SystemTime {
        self.start_time
    }

    fn record_request(&self) {
        let mut timestamps = self.request_timestamps.lock();
        let now = Instant::now();
        
        // 移除超过一分钟的时间戳
        while timestamps.front()
            .map(|&t| now.duration_since(t) > REQUEST_HISTORY_WINDOW)
            .unwrap_or(false) 
        {
            timestamps.pop_front();
        }
        
        timestamps.push_back(now);
    }

    pub fn get_requests_in_window(&self, window: Duration) -> u64 {
        let now = Instant::now();
        let mut timestamps = self.request_timestamps.lock();
        
        // 清理超过窗口时间的记录
        while let Some(timestamp) = timestamps.front() {
            if now.duration_since(*timestamp) > REQUEST_HISTORY_WINDOW {
                timestamps.pop_front();
            } else {
                break;
            }
        }
        
        // 计算指定窗口内的请求数
        timestamps.iter()
            .filter(|&timestamp| now.duration_since(*timestamp) <= window)
            .count() as u64
    }

    pub fn get_requests_last_minute(&self) -> u64 {
        self.get_requests_in_window(ONE_MINUTE)
    }

    pub fn get_requests_last_5_minutes(&self) -> u64 {
        self.get_requests_in_window(FIVE_MINUTES)
    }

    pub fn get_requests_last_15_minutes(&self) -> u64 {
        self.get_requests_in_window(FIFTEEN_MINUTES)
    }

    pub fn get_last_updated(&self) -> SystemTime {
        *self.last_updated.lock()
    }

    pub fn get_cache_stats(&self) -> (u64, u64) {
        let hits = self.cache_hits.load(Ordering::Relaxed);
        let misses = self.cache_misses.load(Ordering::Relaxed);
        (hits, misses)
    }

    pub fn get_all_memes(&self) -> Vec<(&u32, &Meme)> {
        self.memes.iter().collect()
    }
}