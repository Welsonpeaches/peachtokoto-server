use std::sync::Arc;
use axum::{
    extract::State,
    Json,
};
use tokio::sync::RwLock;
use utoipa::ToSchema;
use crate::services::meme::MemeService;
use crate::metrics::{
    SERVICE_UPTIME_SECONDS, TOTAL_MEMES, LAST_UPDATED_TIMESTAMP,
    CACHE_HITS, CACHE_MISSES, CACHE_HIT_RATE
};
use time::OffsetDateTime;

#[derive(serde::Serialize, ToSchema)]
pub struct Statistics {
    #[schema(example = 1000)]
    total_requests: u64,
    #[schema(example = 10)]
    requests_last_minute: u64,
    #[schema(example = 50)]
    requests_last_5min: u64,
    #[schema(example = 150)]
    requests_last_15min: u64,
    #[schema(example = 86400)]
    system_uptime_seconds: u64,
    #[schema(example = 3600)]
    service_uptime_seconds: u64,
    #[schema(example = 100)]
    total_memes: usize,
    #[schema(example = "2024-01-01T00:00:00Z")]
    last_updated: String,
    #[schema(example = 800)]
    cache_hits: u64,
    #[schema(example = 200)]
    cache_misses: u64,
    #[schema(example = 80.0)]
    cache_hit_rate: f64,
}

/// 获取服务器统计信息
#[utoipa::path(
    get,
    path = "/statistics",
    tag = "statistics",
    responses(
        (status = 200, description = "成功返回统计信息", body = Statistics)
    )
)]
pub async fn get_statistics(
    State(state): State<Arc<RwLock<MemeService>>>,
) -> Json<Statistics> {
    // 获取系统启动时间
    let system_uptime_seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let service = state.read().await;
    
    // 获取服务运行时间
    let service_uptime = service.get_start_time()
        .elapsed()
        .unwrap_or_default()
        .as_secs();

    // 获取缓存统计信息
    let (cache_hits, cache_misses) = service.get_cache_stats();
    let total_cache_requests = cache_hits + cache_misses;
    let cache_hit_rate = if total_cache_requests > 0 {
        (cache_hits as f64 / total_cache_requests as f64) * 100.0
    } else {
        0.0
    };

    // 格式化最后更新时间为ISO 8601格式
    let last_updated_timestamp = service.get_last_updated()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    
    let last_updated = service.get_last_updated()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| {
            let datetime = OffsetDateTime::from_unix_timestamp(d.as_secs() as i64)
                .unwrap_or(OffsetDateTime::now_utc());
            datetime.format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_else(|_| "Unknown".to_string())
        })
        .unwrap_or_else(|_| "Unknown".to_string());
    
    // 更新 Prometheus 指标
    SERVICE_UPTIME_SECONDS.set(service_uptime as f64);
    TOTAL_MEMES.set(service.get_total_memes() as f64);
    LAST_UPDATED_TIMESTAMP.set(last_updated_timestamp as f64);
    CACHE_HITS.reset();
    CACHE_HITS.inc_by(cache_hits as f64);
    CACHE_MISSES.reset();
    CACHE_MISSES.inc_by(cache_misses as f64);
    CACHE_HIT_RATE.set(cache_hit_rate / 100.0); // 转换为 0-1 范围
    
    Json(Statistics {
        total_requests: service.get_request_count(),
        requests_last_minute: service.get_requests_last_minute(),
        requests_last_5min: service.get_requests_last_5_minutes(),
        requests_last_15min: service.get_requests_last_15_minutes(),
        system_uptime_seconds,
        service_uptime_seconds: service_uptime,
        total_memes: service.get_total_memes(),
        last_updated,
        cache_hits,
        cache_misses,
        cache_hit_rate,
    })
}