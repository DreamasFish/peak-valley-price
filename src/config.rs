use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::error::{AppError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub region: RegionConfig,
    pub api: ApiConfig,
    pub cache: CacheConfig,
    pub pricing: PricingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionConfig {
    pub province: String,
    pub city: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub base_url: String,
    pub timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub ttl: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingConfig {
    pub default: DefaultPrice,
    pub schedule: ScheduleConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultPrice {
    pub peak_price: f64,
    pub flat_price: f64,
    pub valley_price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleConfig {
    pub peak_hours: Vec<Vec<u8>>,
    pub valley_hours: Vec<Vec<u8>>,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path();

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)
                .map_err(|e| AppError::Config(format!("读取配置文件失败: {}", e)))?;
            let config: AppConfig = toml::from_str(&content)
                .map_err(|e| AppError::Config(format!("解析配置文件失败: {}", e)))?;
            Ok(config)
        } else {
            Ok(Self::default_config())
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path();
        let content = toml::to_string_pretty(self)
            .map_err(|e| AppError::Config(format!("序列化配置失败: {}", e)))?;

        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| AppError::Config(format!("创建配置目录失败: {}", e)))?;
        }

        std::fs::write(&config_path, content)
            .map_err(|e| AppError::Config(format!("写入配置文件失败: {}", e)))?;
        Ok(())
    }

    fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("peak-valley-price")
            .join("config.toml")
    }

    pub fn default_config() -> Self {
        Self {
            region: RegionConfig {
                province: "jiangsu".to_string(),
                city: "wuxi".to_string(),
            },
            api: ApiConfig {
                base_url: "https://www.95598.cn".to_string(),
                timeout: 10,
            },
            cache: CacheConfig { ttl: 3600 },
            pricing: PricingConfig {
                default: DefaultPrice {
                    peak_price: 0.56,
                    flat_price: 0.56,
                    valley_price: 0.35,
                },
                schedule: ScheduleConfig {
                    peak_hours: vec![vec![8, 11], vec![17, 21]],
                    valley_hours: vec![vec![22, 6]],
                },
            },
        }
    }
}
