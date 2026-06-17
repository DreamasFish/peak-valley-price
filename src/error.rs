use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("配置错误: {0}")]
    Config(String),

    #[error("API 请求失败: {0}")]
    Api(String),

    #[error("数据解析错误: {0}")]
    Parse(String),

    #[error("未找到地区: {0}")]
    RegionNotFound(String),

    #[error("缓存错误: {0}")]
    Cache(String),

    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, AppError>;
