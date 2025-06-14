use axum::{
    routing::get,
    Router,
};
use std::net::SocketAddr;
use tower_http::{
    trace::TraceLayer,
    cors::{CorsLayer, Any},
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use crate::utils::error::AppError;

mod config;
mod handlers;
mod models;
mod services;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 加载配置文件
    let config = config::Config::load_from_file("config.yml")?;
    tracing::info!("Configuration loaded successfully");

    // 初始化 MemeService
    let state = services::meme::MemeService::new(
        &config.storage.memes_dir,
        config.cache.max_size,
        config.cache.ttl_secs,
    ).await?;

    // 配置 CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 构建应用路由
    let app = Router::new()
        .route("/memes/random", get(handlers::meme::random_meme))
        .route("/memes/health", get(handlers::meme::health_check))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state);

    // 设置服务器地址
    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port)
        .parse()
        .map_err(|e| AppError::Internal(format!("Invalid address: {}", e)))?;
    tracing::info!("服务器启动在 {}", addr);

    // 启动服务器
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}