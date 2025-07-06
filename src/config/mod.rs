use crate::utils::error::{AppError, Result};
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
pub struct LoggingConfig {
    pub directory: String,
    pub file_prefix: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    pub server: ServerConfig,
    pub storage: StorageConfig,
    pub cache: CacheConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            directory: "logs".to_string(),
            file_prefix: "jiangtokoto".to_string(),
        }
    }
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
            logging: LoggingConfig::default(),
        }
    }
}

impl Config {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Arc<Self>> {
        let path = path.as_ref();

        // 如果配置文件不存在，创建默认配置
        // 如果配置文件不存在
        if !path.exists() {
            // 检查示例配置文件是否存在
            let example_path = path.with_extension("yml.example");
            
            if example_path.exists() {
                tracing::info!("从示例配置创建新的配置文件");
                fs::copy(&example_path, path)
                    .map_err(|e| AppError::Internal(format!("复制示例配置文件失败: {}", e)))?;
            } else {
                // 如果示例配置不存在，创建默认配置
                tracing::info!("配置文件不存在，创建默认配置");
                let config = Config::default();
                let config_str = serde_yaml::to_string(&config)
                    .map_err(|e| AppError::Internal(format!("序列化默认配置失败: {}", e)))?;

                // 确保目录存在
                if let Some(parent) = path.parent() {
                    if !parent.exists() {
                        fs::create_dir_all(parent)
                            .map_err(|e| AppError::Internal(format!("创建配置目录失败: {}", e)))?;
                    }
                }

                fs::write(path, config_str)
                    .map_err(|e| AppError::Internal(format!("写入默认配置文件失败: {}", e)))?;

                tracing::info!("默认配置文件已创建: {:?}", path);
            }
        }

        // 读取现有配置
        let config_str = fs::read_to_string(path)
            .map_err(|e| AppError::Internal(format!("Failed to read config file: {}", e)))?;

        let config: Config = serde_yaml::from_str(&config_str)
            .map_err(|e| AppError::Internal(format!("Failed to parse config file: {}", e)))?;

        // 验证配置
        config.validate()?;

        // 确保表情包目录存在
        if !Path::new(&config.storage.memes_dir).exists() {
            fs::create_dir_all(&config.storage.memes_dir)
                .map_err(|e| AppError::Internal(format!("Failed to create memes directory: {}", e)))?;
            tracing::info!("表情包目录已创建: {}", config.storage.memes_dir);
        }

        Ok(Arc::new(config))
    }

    pub fn validate(&self) -> Result<()> {
        if self.cache.max_size == 0 {
            return Err(AppError::Internal("Cache max_size must be greater than 0".to_string()));
        }
        
        if self.cache.ttl_secs == 0 {
            return Err(AppError::Internal("Cache TTL must be greater than 0".to_string()));
        }
        
        if self.server.port == 0 {
            return Err(AppError::Internal("Server port must be greater than 0".to_string()));
        }
        
        if self.server.host.is_empty() {
            return Err(AppError::Internal("Server host cannot be empty".to_string()));
        }
        
        if self.storage.memes_dir.is_empty() {
            return Err(AppError::Internal("Memes directory path cannot be empty".to_string()));
        }
        
        Ok(())
    }
}

