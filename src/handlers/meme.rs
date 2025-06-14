use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{
    services::meme::MemeService,
    models::meme::MemeResponse,
};

pub async fn random_meme(
    State(state): State<Arc<RwLock<MemeService>>>,
) -> impl IntoResponse {
    let state = state.read().await;
    
    match state.get_random().await {
        Ok((meme, content)) => {
            let mut headers = HeaderMap::new();
            headers.insert(header::CONTENT_TYPE, meme.mime_type.parse().unwrap());
            
            let response = MemeResponse {
                id: meme.id,
                mime_type: meme.mime_type.clone(),
            };
            
            tracing::debug!("Serving meme: {:?}", response);
            (StatusCode::OK, headers, content).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to get random meme: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, HeaderMap::new(), e.to_string().into_bytes()).into_response()
        }
    }
}

pub async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}