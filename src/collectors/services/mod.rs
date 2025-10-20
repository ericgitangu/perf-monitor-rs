// Service-specific collectors for databases and infrastructure

#[cfg(feature = "mysql")]
pub mod mysql;

#[cfg(feature = "postgresql")]
pub mod postgresql;

#[cfg(feature = "redis-db")]
pub mod redis;

pub mod sidekiq;
pub mod rabbitmq;
pub mod celery;

#[cfg(feature = "mysql")]
pub use mysql::{MySQLCollector, MySQLInstanceConfig, MySQLMetrics};

#[cfg(feature = "postgresql")]
pub use postgresql::{PostgreSQLCollector, PostgreSQLInstanceConfig, PostgreSQLMetrics};

#[cfg(feature = "redis-db")]
pub use redis::{RedisCollector, RedisInstanceConfig, RedisMetrics};

pub use sidekiq::{SidekiqCollector, SidekiqConfig, SidekiqMetrics};
pub use rabbitmq::{RabbitMQCollector, RabbitMQConfig, RabbitMQMetrics};
pub use celery::{CeleryCollector, CeleryConfig, CeleryMetrics};
