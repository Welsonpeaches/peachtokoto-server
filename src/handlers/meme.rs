use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

use crate::services::meme::MemeService;

pub async fn random_meme(
    State(state): State<Arc<RwLock<MemeService>>>,
) -> impl IntoResponse {
    let state = state.read().await;
    
    match state.get_random().await {
        Ok((meme, content)) => {
            let mut resp_headers = HeaderMap::new();
            resp_headers.insert(header::CONTENT_TYPE, meme.mime_type.parse().unwrap());
            
            // 记录访问信息
            info!(
                "返回表情包ID: {}, 类型: {}",
                meme.id,
                meme.mime_type
            );

            (StatusCode::OK, resp_headers, content)
        }
        Err(_) => {
            info!("获取表情包失败");
            (StatusCode::INTERNAL_SERVER_ERROR, HeaderMap::new(), Vec::new())
        }
    }
}

pub async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}