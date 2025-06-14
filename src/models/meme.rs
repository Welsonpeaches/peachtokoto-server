use std::path::PathBuf;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meme {
    pub id: u32,
    pub path: PathBuf,
    pub mime_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemeResponse {
    pub id: u32,
    pub mime_type: String,
}