use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Collection error: {0}")]
    Collection(String),

    #[error("UI error: {0}")]
    Ui(String),

    #[error("Export error: {0}")]
    Export(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Figment error: {0}")]
    Figment(#[from] figment::Error),

    #[cfg(feature = "mysql")]
    #[error("MySQL error: {0}")]
    MySQL(#[from] mysql_async::Error),

    #[cfg(feature = "postgresql")]
    #[error("PostgreSQL error: {0}")]
    PostgreSQL(#[from] tokio_postgres::Error),

    #[cfg(feature = "redis-db")]
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Collector error: {0}")]
    CollectorError(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
