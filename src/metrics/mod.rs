use prometheus::{Counter, Histogram, Gauge, Registry, Encoder, TextEncoder, Opts, HistogramOpts};
use lazy_static::lazy_static;
use std::time::Instant;

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
}

pub fn init_metrics() {
    REGISTRY.register(Box::new(REQUEST_COUNTER.clone())).unwrap();
    REGISTRY.register(Box::new(RESPONSE_TIME.clone())).unwrap();
    REGISTRY.register(Box::new(CACHE_HIT_RATE.clone())).unwrap();
    REGISTRY.register(Box::new(CACHE_SIZE.clone())).unwrap();
    REGISTRY.register(Box::new(ACTIVE_CONNECTIONS.clone())).unwrap();
    REGISTRY.register(Box::new(IMAGE_PROCESSING_TIME.clone())).unwrap();
}

pub fn get_metrics() -> String {
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