use crate::utils::error::{AppError, Result};
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path, sync::Arc};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProxyConfig {
    pub enabled: bool,
    pub ip_header: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    #[serde(default)]
    pub proxy: ProxyConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StorageConfig {
    pub memes_dir: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CacheConfig {
    pub max_size: u64,
    pub ttl_secs: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    pub server: ServerConfig,
    pub storage: StorageConfig,
    pub cache: CacheConfig,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            ip_header: "x-forwarded-for".to_string(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 3001,
                proxy: ProxyConfig::default(),
            },
            storage: StorageConfig {
                memes_dir: "assets/jiangtokoto-images/images".to_string(),
            },
            cache: CacheConfig {
                max_size: 100,
                ttl_secs: 300,
            },
        }
    }
}

impl Config {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Arc<Self>> {
        let path = path.as_ref();

        // 如果配置文件不存在，创建默认配置
        if !path.exists() {
            tracing::info!("配置文件不存在，创建默认配置");
            let config = Config::default();
            let config_str = serde_yaml::to_string(&config)
                .map_err(|e| AppError::Internal(format!("Failed to serialize default config: {}", e)))?;

            // 确保目录存在
            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)
                        .map_err(|e| AppError::Internal(format!("Failed to create config directory: {}", e)))?;
                }
            }

            fs::write(path, config_str)
                .map_err(|e| AppError::Internal(format!("Failed to write default config file: {}", e)))?;

            tracing::info!("默认配置文件已创建: {:?}", path);

            // 创建 memes 目录
            if !Path::new(&config.storage.memes_dir).exists() {
                fs::create_dir_all(&config.storage.memes_dir)
                    .map_err(|e| AppError::Internal(format!("Failed to create memes directory: {}", e)))?;
                tracing::info!("表情包目录已创建: {}", config.storage.memes_dir);
            }

            return Ok(Arc::new(config));
        }

        // 读取现有配置
        let config_str = fs::read_to_string(path)
            .map_err(|e| AppError::Internal(format!("Failed to read config file: {}", e)))?;

        let config: Config = serde_yaml::from_str(&config_str)
            .map_err(|e| AppError::Internal(format!("Failed to parse config file: {}", e)))?;

        // 确保表情包目录存在
        if !Path::new(&config.storage.memes_dir).exists() {
            fs::create_dir_all(&config.storage.memes_dir)
                .map_err(|e| AppError::Internal(format!("Failed to create memes directory: {}", e)))?;
            tracing::info!("表情包目录已创建: {}", config.storage.memes_dir);
        }

        Ok(Arc::new(config))
    }
}

pub fn load_config() -> std::io::Result<Arc<Config>> {
    dotenv().ok();

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .unwrap_or(3000);

    let memes_dir = std::env::var("MEMES_DIR")
        .unwrap_or_else(|_| "assets/memes".to_string());

    Ok(Arc::new(Config {
        server: ServerConfig {
            host: "0.0.0.0".to_string(),
            port,
            proxy: ProxyConfig::default(),
        },
        storage: StorageConfig { memes_dir },
        cache: CacheConfig {
            max_size: 1024,
            ttl_secs: 3600,
        },
    }))
}