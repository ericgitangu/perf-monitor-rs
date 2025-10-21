// Service-specific collectors for databases and infrastructure

#[cfg(feature = "mysql")]
pub mod mysql;

#[cfg(feature = "postgresql")]
pub mod postgresql;

#[cfg(feature = "redis-db")]
pub mod redis;

#[cfg(feature = "mongodb-db")]
pub mod mongodb;

#[cfg(feature = "mysql")]
pub mod sphinx;

pub mod sidekiq;
pub mod rabbitmq;
pub mod celery;
pub mod puma;

#[cfg(feature = "mysql")]
pub use mysql::{MySQLCollector, MySQLInstanceConfig, MySQLMetrics};

#[cfg(feature = "postgresql")]
pub use postgresql::{PostgreSQLCollector, PostgreSQLInstanceConfig, PostgreSQLMetrics};

#[cfg(feature = "redis-db")]
pub use redis::{RedisCollector, RedisInstanceConfig, RedisMetrics};

#[cfg(feature = "mongodb-db")]
pub use mongodb::{MongoDBCollector, MongoDBInstanceConfig, MongoDBMetrics};

#[cfg(feature = "mysql")]
pub use sphinx::{SphinxCollector, SphinxInstanceConfig, SphinxMetrics};

pub use sidekiq::{SidekiqCollector, SidekiqConfig, SidekiqMetrics};
pub use rabbitmq::{RabbitMQCollector, RabbitMQConfig, RabbitMQMetrics};
pub use celery::{CeleryCollector, CeleryConfig, CeleryMetrics};
pub use puma::{PumaCollector, PumaInstanceConfig, PumaMetrics};
