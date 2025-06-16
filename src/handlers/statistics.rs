use std::sync::Arc;
use axum::{
    extract::State,
    Json,
};
use tokio::sync::RwLock;
use crate::services::meme::MemeService;
use time::OffsetDateTime;

#[derive(serde::Serialize)]
pub struct Statistics {
    total_requests: u64,
    requests_last_minute: u64,
    requests_last_5min: u64,
    requests_last_15min: u64,
    system_uptime_seconds: u64,
    service_uptime_seconds: u64,
    total_memes: usize,
    last_updated: String,
}

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

    // 格式化最后更新时间为ISO 8601格式
    let last_updated = service.get_last_updated()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| {
            let datetime = OffsetDateTime::from_unix_timestamp(d.as_secs() as i64)
                .unwrap_or(OffsetDateTime::now_utc());
            datetime.format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_else(|_| "Unknown".to_string())
        })
        .unwrap_or_else(|_| "Unknown".to_string());
    
    Json(Statistics {
        total_requests: service.get_request_count(),
        requests_last_minute: service.get_requests_last_minute(),
        requests_last_5min: service.get_requests_last_5_minutes(),
        requests_last_15min: service.get_requests_last_15_minutes(),
        system_uptime_seconds,
        service_uptime_seconds: service_uptime,
        total_memes: service.get_total_memes(),
        last_updated,
    })
} 