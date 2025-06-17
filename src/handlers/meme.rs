use axum::{
    extract::{State, Path, Query},
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use serde::Serialize;
use serde::Deserialize;

use crate::services::meme::MemeService;
use crate::utils::error::AppError;

#[derive(Deserialize)]
pub struct RandomMemeQuery {
    redirect: Option<bool>,
}

#[derive(Serialize)]
pub struct MemeListItem {
    pub id: u32,
    pub mime_type: String,
    pub filename: String,
    pub size_bytes: u64,
}

pub async fn random_meme(
    State(state): State<Arc<RwLock<MemeService>>>,
    Query(query): Query<RandomMemeQuery>,
) -> impl IntoResponse {
    let state = state.read().await;
    
    match state.get_random().await {
        Ok((meme, content)) => {
            // 如果设置了 redirect 参数，则重定向到 get 端点
            if query.redirect.unwrap_or(false) {
                let mut headers = HeaderMap::new();
                headers.insert(
                    header::LOCATION,
                    format!("/memes/get/{}", meme.id).parse().unwrap()
                );
                return (StatusCode::FOUND, headers, Vec::new());
            }

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

pub async fn list_memes(
    State(state): State<Arc<RwLock<MemeService>>>,
) -> Json<Vec<MemeListItem>> {
    let service = state.read().await;
    let memes = service.get_all_memes();
    
    let mut meme_list: Vec<MemeListItem> = memes.into_iter()
        .map(|(id, meme)| MemeListItem {
            id: *id,
            mime_type: meme.mime_type.clone(),
            filename: meme.filename.clone(),
            size_bytes: meme.size_bytes,
        })
        .collect();
    
    // 按 id 排序
    meme_list.sort_by_key(|meme| meme.id);
    
    Json(meme_list)
}

pub async fn get_meme_by_id(
    State(state): State<Arc<RwLock<MemeService>>>,
    Path(id): Path<u32>,
) -> impl IntoResponse {
    let state = state.read().await;
    
    match state.get_by_id(id).await {
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
        Err(AppError::NotFound(msg)) => {
            info!("获取表情包失败: {}", msg);
            (StatusCode::NOT_FOUND, HeaderMap::new(), Vec::new())
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