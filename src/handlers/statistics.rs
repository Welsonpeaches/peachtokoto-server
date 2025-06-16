use std::sync::Arc;
use axum::{
    extract::State,
    Json,
};
use tokio::sync::RwLock;
use sysinfo::{System, Cpu, CpuRefreshKind, RefreshKind};
use crate::services::meme::MemeService;

#[derive(serde::Serialize)]
pub struct Statistics {
    total_requests: u64,
    requests_last_minute: u64,
    system_uptime_seconds: u64,
    service_uptime_seconds: u64,
    cpu_usage: f32,
    memory_usage: f32,
    total_memes: usize,
}

pub async fn get_statistics(
    State(state): State<Arc<RwLock<MemeService>>>,
) -> Json<Statistics> {
    let mut sys = System::new_with_specifics(RefreshKind::everything());
    sys.refresh_all();

    // 获取CPU使用率
    let cpu_usage = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() / sys.cpus().len() as f32;
    
    // 获取内存使用率
    let total_memory = sys.total_memory();
    let used_memory = sys.used_memory();
    let memory_usage = (used_memory as f32 / total_memory as f32) * 100.0;

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
    
    Json(Statistics {
        total_requests: service.get_request_count(),
        requests_last_minute: service.get_requests_last_minute(),
        system_uptime_seconds,
        service_uptime_seconds: service_uptime,
        cpu_usage,
        memory_usage,
        total_memes: service.get_total_memes(),
    })
} 