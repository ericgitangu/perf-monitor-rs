use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;
use tokio_postgres::{Client, Config, NoTls};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PostgreSQLInstanceConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    #[serde(skip_serializing)]
    pub password: Option<String>,
    pub database: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostgreSQLInstanceMetrics {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub connections: i64,
    pub max_connections: i64,
    pub active_connections: i64,
    pub idle_connections: i64,
    pub cache_hit_ratio: f64,
    pub transactions_per_second: f64,
    pub commits: i64,
    pub rollbacks: i64,
    pub locks: i64,
    pub database_size_bytes: i64,
    pub version: String,
    pub uptime_seconds: i64,
    pub replication_lag_bytes: Option<i64>,
    pub available: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostgreSQLMetrics {
    pub instances: Vec<PostgreSQLInstanceMetrics>,
    pub total_connections: i64,
    pub total_active_connections: i64,
    pub total_transactions_per_second: f64,
}

pub struct PostgreSQLCollector {
    previous_stats: HashMap<String, (i64, Instant)>, // (commits + rollbacks, time)
}

impl PostgreSQLCollector {
    pub fn new(_instances: Vec<PostgreSQLInstanceConfig>) -> crate::Result<Self> {
        Ok(Self {
            previous_stats: HashMap::new(),
        })
    }

    async fn connect(&self, config: &PostgreSQLInstanceConfig) -> crate::Result<Client> {
        let mut pg_config = Config::new();
        pg_config
            .host(&config.host)
            .port(config.port)
            .user(&config.user)
            .dbname(&config.database);

        if let Some(password) = &config.password {
            pg_config.password(password);
        }

        let (client, connection) = pg_config.connect(NoTls).await.map_err(|e| {
            crate::error::Error::CollectorError(format!("PostgreSQL connection error: {}", e))
        })?;

        // Spawn the connection task
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                tracing::error!("PostgreSQL connection error: {}", e);
            }
        });

        Ok(client)
    }

    async fn collect_instance_metrics(
        &mut self,
        name: &str,
        config: &PostgreSQLInstanceConfig,
    ) -> PostgreSQLInstanceMetrics {
        match self.try_collect_instance_metrics(name, config).await {
            Ok(metrics) => metrics,
            Err(e) => PostgreSQLInstanceMetrics {
                name: name.to_string(),
                host: config.host.clone(),
                port: config.port,
                database: config.database.clone(),
                connections: 0,
                max_connections: 0,
                active_connections: 0,
                idle_connections: 0,
                cache_hit_ratio: 0.0,
                transactions_per_second: 0.0,
                commits: 0,
                rollbacks: 0,
                locks: 0,
                database_size_bytes: 0,
                version: String::from("unknown"),
                uptime_seconds: 0,
                replication_lag_bytes: None,
                available: false,
                error: Some(e.to_string()),
            },
        }
    }

    async fn try_collect_instance_metrics(
        &mut self,
        name: &str,
        config: &PostgreSQLInstanceConfig,
    ) -> crate::Result<PostgreSQLInstanceMetrics> {
        let client = self.connect(config).await?;

        // Get version
        let version_row = client.query_one("SELECT version()", &[]).await?;
        let version: String = version_row.get(0);

        // Get max connections
        let max_conn_row = client.query_one("SHOW max_connections", &[]).await?;
        let max_connections: String = max_conn_row.get(0);
        let max_connections = max_connections.parse::<i64>().unwrap_or(100);

        // Get connection stats
        let conn_stats = client
            .query_one(
                "SELECT count(*) as total,
                        count(*) FILTER (WHERE state = 'active') as active,
                        count(*) FILTER (WHERE state = 'idle') as idle
                 FROM pg_stat_activity
                 WHERE pid != pg_backend_pid()",
                &[],
            )
            .await?;

        let connections: i64 = conn_stats.get(0);
        let active_connections: i64 = conn_stats.get(1);
        let idle_connections: i64 = conn_stats.get(2);

        // Get cache hit ratio
        let cache_hit_row = client
            .query_one(
                "SELECT
                    sum(heap_blks_hit) as hits,
                    sum(heap_blks_read) as reads
                 FROM pg_statio_user_tables",
                &[],
            )
            .await?;

        let hits: Option<i64> = cache_hit_row.get(0);
        let reads: Option<i64> = cache_hit_row.get(1);

        let cache_hit_ratio = match (hits, reads) {
            (Some(h), Some(r)) if h + r > 0 => (h as f64 / (h + r) as f64) * 100.0,
            _ => 0.0,
        };

        // Get transaction stats
        let tx_stats = client
            .query_one(
                "SELECT
                    xact_commit,
                    xact_rollback
                 FROM pg_stat_database
                 WHERE datname = current_database()",
                &[],
            )
            .await?;

        let commits: i64 = tx_stats.get(0);
        let rollbacks: i64 = tx_stats.get(1);

        // Calculate transactions per second
        let total_transactions = commits + rollbacks;
        let now = Instant::now();
        let tps = if let Some((prev_tx, prev_time)) = self.previous_stats.get(name) {
            let elapsed = now.duration_since(*prev_time).as_secs_f64();
            if elapsed > 0.0 {
                (total_transactions.saturating_sub(*prev_tx)) as f64 / elapsed
            } else {
                0.0
            }
        } else {
            0.0
        };
        self.previous_stats.insert(name.to_string(), (total_transactions, now));

        // Get locks
        let locks_row = client
            .query_one("SELECT count(*) FROM pg_locks", &[])
            .await?;
        let locks: i64 = locks_row.get(0);

        // Get database size
        let size_row = client
            .query_one(
                "SELECT pg_database_size(current_database())",
                &[],
            )
            .await?;
        let database_size_bytes: i64 = size_row.get(0);

        // Get uptime
        let uptime_row = client
            .query_one(
                "SELECT EXTRACT(EPOCH FROM (now() - pg_postmaster_start_time()))::bigint",
                &[],
            )
            .await?;
        let uptime_seconds: i64 = uptime_row.get(0);

        // Try to get replication lag (if replica)
        let replication_lag_bytes = self.get_replication_lag(&client).await.ok();

        Ok(PostgreSQLInstanceMetrics {
            name: name.to_string(),
            host: config.host.clone(),
            port: config.port,
            database: config.database.clone(),
            connections,
            max_connections,
            active_connections,
            idle_connections,
            cache_hit_ratio,
            transactions_per_second: tps,
            commits,
            rollbacks,
            locks,
            database_size_bytes,
            version,
            uptime_seconds,
            replication_lag_bytes,
            available: true,
            error: None,
        })
    }

    async fn get_replication_lag(&self, client: &Client) -> crate::Result<i64> {
        let row = client
            .query_one(
                "SELECT
                    CASE
                        WHEN pg_last_wal_receive_lsn() = pg_last_wal_replay_lsn() THEN 0
                        ELSE EXTRACT(EPOCH FROM now() - pg_last_xact_replay_timestamp())::bigint
                    END as lag_seconds",
                &[],
            )
            .await?;

        let lag: i64 = row.get(0);
        Ok(lag)
    }

    pub async fn collect_async(
        &mut self,
        instances: &[PostgreSQLInstanceConfig],
    ) -> crate::Result<PostgreSQLMetrics> {
        let mut instance_metrics = Vec::new();

        for instance_config in instances {
            let metrics = self
                .collect_instance_metrics(&instance_config.name, instance_config)
                .await;
            instance_metrics.push(metrics);
        }

        let total_connections: i64 = instance_metrics
            .iter()
            .filter(|m| m.available)
            .map(|m| m.connections)
            .sum();

        let total_active_connections: i64 = instance_metrics
            .iter()
            .filter(|m| m.available)
            .map(|m| m.active_connections)
            .sum();

        let total_transactions_per_second: f64 = instance_metrics
            .iter()
            .filter(|m| m.available)
            .map(|m| m.transactions_per_second)
            .sum();

        Ok(PostgreSQLMetrics {
            instances: instance_metrics,
            total_connections,
            total_active_connections,
            total_transactions_per_second,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_postgresql_instance_config_serialization() {
        let config = PostgreSQLInstanceConfig {
            host: "localhost".to_string(),
            port: 5432,
            user: "monitor".to_string(),
            password: Some("secret".to_string()),
            database: "postgres".to_string(),
            name: "test".to_string(),
        };

        let serialized = serde_json::to_string(&config).unwrap();
        assert!(serialized.contains("localhost"));
        assert!(serialized.contains("5432"));
        assert!(!serialized.contains("secret")); // Password should not be serialized
    }

    #[test]
    fn test_postgresql_metrics_aggregation() {
        let metrics = PostgreSQLMetrics {
            instances: vec![
                PostgreSQLInstanceMetrics {
                    name: "db1".to_string(),
                    host: "localhost".to_string(),
                    port: 5432,
                    database: "app1".to_string(),
                    connections: 20,
                    max_connections: 100,
                    active_connections: 5,
                    idle_connections: 15,
                    cache_hit_ratio: 99.5,
                    transactions_per_second: 50.0,
                    commits: 1000,
                    rollbacks: 10,
                    locks: 25,
                    database_size_bytes: 1073741824,
                    version: "PostgreSQL 12.1".to_string(),
                    uptime_seconds: 86400,
                    replication_lag_bytes: None,
                    available: true,
                    error: None,
                },
            ],
            total_connections: 20,
            total_active_connections: 5,
            total_transactions_per_second: 50.0,
        };

        assert_eq!(metrics.instances.len(), 1);
        assert_eq!(metrics.total_connections, 20);
        assert_eq!(metrics.total_active_connections, 5);
    }
}
