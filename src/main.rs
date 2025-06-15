use axum::{
    routing::get,
    Router,
    extract::ConnectInfo,
};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tower_http::{
    trace::{TraceLayer, OnResponse},
    cors::{CorsLayer, Any},
};
use tracing::{Level, info, Span};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use crate::utils::error::AppError;

#[derive(Clone)]
struct CustomOnResponse;

impl<B> OnResponse<B> for CustomOnResponse {
    fn on_response(self, response: &axum::response::Response<B>, latency: Duration, span: &Span) {
        let status = response.status();
        info!(parent: span,
            status = %status,
            latency = ?latency,
            "响应完成"
        );
    }
}

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
    let config_clone = Arc::new(config.clone());
    let app = Router::new()
        .route("/memes/random", get(handlers::meme::random_meme))
        .route("/memes/health", get(handlers::meme::health_check))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(move |request: &axum::http::Request<_>| {
                    let remote_addr = if config_clone.server.proxy.enabled {
                        request
                            .headers()
                            .get(&config_clone.server.proxy.ip_header)
                            .and_then(|h| h.to_str().ok())
                            .map(|s| s.split(',').next().unwrap_or(s).trim().to_string())
                            .unwrap_or_else(|| "unknown".to_string())
                    } else {
                        request
                            .extensions()
                            .get::<ConnectInfo<SocketAddr>>()
                            .map(|ci| ci.0.ip().to_string())
                            .unwrap_or_else(|| "unknown".to_string())
                    };

                    tracing::span!(
                        Level::INFO,
                        "请求",
                        method = %request.method(),
                        uri = %request.uri(),
                        ip = %remote_addr,
                    )
                })
                .on_response(CustomOnResponse)
        )
        .layer(cors)
        .with_state(state);

    // 设置服务器地址
    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port)
        .parse()
        .map_err(|e| AppError::Internal(format!("Invalid address: {}", e)))?;
    tracing::info!("服务器启动在 {}", addr);

    // 启动服务器
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("服务器启动在 {}", addr);
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>()
    ).await?;

    Ok(())
}