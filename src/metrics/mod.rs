use prometheus::{Counter, Histogram, Gauge, Registry, Encoder, TextEncoder, Opts, HistogramOpts};
use lazy_static::lazy_static;
use std::time::{Instant, SystemTime};
use std::sync::OnceLock;

// 全局服务启动时间
static SERVICE_START_TIME: OnceLock<SystemTime> = OnceLock::new();

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();
    
    pub static ref REQUEST_COUNTER: Counter = Counter::with_opts(
        Opts::new("meme_requests_total", "Total number of meme requests")
    ).unwrap();
    
    pub static ref RESPONSE_TIME: Histogram = Histogram::with_opts(
        HistogramOpts::new("meme_response_duration_seconds", "Response time for meme requests")
    ).unwrap();
    
    pub static ref CACHE_HIT_RATE: Gauge = Gauge::with_opts(
        Opts::new("meme_cache_hit_rate", "Cache hit rate")
    ).unwrap();
    
    pub static ref CACHE_SIZE: Gauge = Gauge::with_opts(
        Opts::new("meme_cache_size", "Current cache size")
    ).unwrap();
    
    pub static ref ACTIVE_CONNECTIONS: Gauge = Gauge::with_opts(
        Opts::new("meme_active_connections", "Number of active connections")
    ).unwrap();
    
    pub static ref IMAGE_PROCESSING_TIME: Histogram = Histogram::with_opts(
        HistogramOpts::new("meme_image_processing_duration_seconds", "Time spent processing images")
    ).unwrap();
    
    // 新增的统计指标
    pub static ref SERVICE_UPTIME_SECONDS: Gauge = Gauge::with_opts(
        Opts::new("service_uptime_seconds", "Service uptime in seconds")
    ).unwrap();
    
    pub static ref TOTAL_MEMES: Gauge = Gauge::with_opts(
        Opts::new("total_memes", "Total number of memes available")
    ).unwrap();
    
    pub static ref LAST_UPDATED_TIMESTAMP: Gauge = Gauge::with_opts(
        Opts::new("last_updated_timestamp", "Last updated timestamp (Unix timestamp)")
    ).unwrap();
    
    pub static ref CACHE_HITS: Counter = Counter::with_opts(
        Opts::new("cache_hits_total", "Total number of cache hits")
    ).unwrap();
    
    pub static ref CACHE_MISSES: Counter = Counter::with_opts(
        Opts::new("cache_misses_total", "Total number of cache misses")
    ).unwrap();
}

pub fn init_metrics() {
    REGISTRY.register(Box::new(REQUEST_COUNTER.clone())).unwrap();
    REGISTRY.register(Box::new(RESPONSE_TIME.clone())).unwrap();
    REGISTRY.register(Box::new(CACHE_HIT_RATE.clone())).unwrap();
    REGISTRY.register(Box::new(CACHE_SIZE.clone())).unwrap();
    REGISTRY.register(Box::new(ACTIVE_CONNECTIONS.clone())).unwrap();
    REGISTRY.register(Box::new(IMAGE_PROCESSING_TIME.clone())).unwrap();
    
    // 注册新增的指标
    REGISTRY.register(Box::new(SERVICE_UPTIME_SECONDS.clone())).unwrap();
    REGISTRY.register(Box::new(TOTAL_MEMES.clone())).unwrap();
    REGISTRY.register(Box::new(LAST_UPDATED_TIMESTAMP.clone())).unwrap();
    REGISTRY.register(Box::new(CACHE_HITS.clone())).unwrap();
    REGISTRY.register(Box::new(CACHE_MISSES.clone())).unwrap();
}

/// 设置服务启动时间
pub fn set_service_start_time(start_time: SystemTime) {
    SERVICE_START_TIME.set(start_time).ok();
}

pub fn get_metrics() -> String {
    // 按需更新服务运行时间
    if let Some(start_time) = SERVICE_START_TIME.get() {
        if let Ok(uptime) = start_time.elapsed() {
            SERVICE_UPTIME_SECONDS.set(uptime.as_secs() as f64);
        }
    }
    
    let encoder = TextEncoder::new();
    let metric_families = REGISTRY.gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}

pub struct Timer {
    start: Instant,
    histogram: &'static Histogram,
}

impl Timer {
    pub fn new(histogram: &'static Histogram) -> Self {
        Self {
            start: Instant::now(),
            histogram,
        }
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        self.histogram.observe(duration.as_secs_f64());
    }
}

#[macro_export]
macro_rules! time_operation {
    ($histogram:expr, $operation:expr) => {{
        let _timer = crate::metrics::Timer::new($histogram);
        $operation
    }};
}