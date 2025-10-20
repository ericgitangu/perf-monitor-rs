// Service-specific collectors for databases and infrastructure

#[cfg(feature = "mysql")]
pub mod mysql;

#[cfg(feature = "postgresql")]
pub mod postgresql;

#[cfg(feature = "redis-db")]
pub mod redis;

#[cfg(feature = "mysql")]
pub use mysql::{MySQLCollector, MySQLInstanceConfig, MySQLMetrics};

#[cfg(feature = "postgresql")]
pub use postgresql::{PostgreSQLCollector, PostgreSQLInstanceConfig, PostgreSQLMetrics};

#[cfg(feature = "redis-db")]
pub use redis::{RedisCollector, RedisInstanceConfig, RedisMetrics};
