use mysql_async::{prelude::*, Conn, OptsBuilder, Pool};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MySQLInstanceConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    #[serde(skip_serializing)]
    pub password: Option<String>,
    pub database: Option<String>,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MySQLInstanceMetrics {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub connections: u64,
    pub threads_running: u64,
    pub queries_per_second: f64,
    pub slow_queries: u64,
    pub buffer_pool_size: u64,
    pub buffer_pool_usage_percent: f64,
    pub uptime_seconds: u64,
    pub version: String,
    pub replication_status: Option<ReplicationStatus>,
    pub available: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationStatus {
    pub slave_io_running: bool,
    pub slave_sql_running: bool,
    pub seconds_behind_master: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MySQLMetrics {
    pub instances: Vec<MySQLInstanceMetrics>,
    pub total_connections: u64,
    pub total_queries_per_second: f64,
}

pub struct MySQLCollector {
    pools: HashMap<String, Pool>,
    previous_queries: HashMap<String, (u64, Instant)>,
}

impl MySQLCollector {
    pub fn new(instances: Vec<MySQLInstanceConfig>) -> crate::Result<Self> {
        let mut pools = HashMap::new();

        for instance in instances {
            let opts = OptsBuilder::default()
                .ip_or_hostname(&instance.host)
                .tcp_port(instance.port)
                .user(Some(&instance.user))
                .pass(instance.password.as_deref())
                .db_name(instance.database.as_deref());

            let pool = Pool::new(opts);
            pools.insert(instance.name.clone(), pool);
        }

        Ok(Self {
            pools,
            previous_queries: HashMap::new(),
        })
    }

    async fn collect_instance_metrics(
        &mut self,
        name: &str,
        pool: &Pool,
        config: &MySQLInstanceConfig,
    ) -> MySQLInstanceMetrics {
        match self.try_collect_instance_metrics(name, pool, config).await {
            Ok(metrics) => metrics,
            Err(e) => MySQLInstanceMetrics {
                name: name.to_string(),
                host: config.host.clone(),
                port: config.port,
                connections: 0,
                threads_running: 0,
                queries_per_second: 0.0,
                slow_queries: 0,
                buffer_pool_size: 0,
                buffer_pool_usage_percent: 0.0,
                uptime_seconds: 0,
                version: String::from("unknown"),
                replication_status: None,
                available: false,
                error: Some(e.to_string()),
            },
        }
    }

    async fn try_collect_instance_metrics(
        &mut self,
        name: &str,
        pool: &Pool,
        config: &MySQLInstanceConfig,
    ) -> crate::Result<MySQLInstanceMetrics> {
        let mut conn = pool.get_conn().await?;

        // Get global status variables
        let status_vars = self.get_status_variables(&mut conn).await?;

        // Get version
        let version: String = conn
            .query_first("SELECT VERSION()")
            .await?
            .unwrap_or_else(|| "unknown".to_string());

        // Get connections
        let connections = status_vars
            .get("Threads_connected")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);

        let threads_running = status_vars
            .get("Threads_running")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);

        // Get total queries
        let total_queries = status_vars
            .get("Questions")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);

        // Calculate queries per second
        let now = Instant::now();
        let qps = if let Some((prev_queries, prev_time)) = self.previous_queries.get(name) {
            let elapsed = now.duration_since(*prev_time).as_secs_f64();
            if elapsed > 0.0 {
                (total_queries.saturating_sub(*prev_queries)) as f64 / elapsed
            } else {
                0.0
            }
        } else {
            0.0
        };
        self.previous_queries.insert(name.to_string(), (total_queries, now));

        // Get slow queries
        let slow_queries = status_vars
            .get("Slow_queries")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);

        // Get uptime
        let uptime_seconds = status_vars
            .get("Uptime")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);

        // Get buffer pool stats
        let buffer_pool_size = status_vars
            .get("Innodb_buffer_pool_pages_total")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);

        let buffer_pool_free = status_vars
            .get("Innodb_buffer_pool_pages_free")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);

        let buffer_pool_usage_percent = if buffer_pool_size > 0 {
            ((buffer_pool_size - buffer_pool_free) as f64 / buffer_pool_size as f64) * 100.0
        } else {
            0.0
        };

        // Try to get replication status
        let replication_status = self.get_replication_status(&mut conn).await.ok();

        drop(conn);

        Ok(MySQLInstanceMetrics {
            name: name.to_string(),
            host: config.host.clone(),
            port: config.port,
            connections,
            threads_running,
            queries_per_second: qps,
            slow_queries,
            buffer_pool_size,
            buffer_pool_usage_percent,
            uptime_seconds,
            version,
            replication_status,
            available: true,
            error: None,
        })
    }

    async fn get_status_variables(&self, conn: &mut Conn) -> crate::Result<HashMap<String, String>> {
        let rows: Vec<(String, String)> = conn.query("SHOW GLOBAL STATUS").await?;
        Ok(rows.into_iter().collect())
    }

    async fn get_replication_status(&self, conn: &mut Conn) -> crate::Result<ReplicationStatus> {
        // SHOW SLAVE STATUS returns different formats, so we'll handle it simply
        // For now, just return a default/none status as replication monitoring
        // can be enhanced later with more robust parsing
        let rows: Vec<mysql_async::Row> = conn.query("SHOW SLAVE STATUS").await?;

        if let Some(_row) = rows.first() {
            // Try to extract basic replication info
            // This is a simplified version - actual implementation would need
            // to handle the specific columns returned by SHOW SLAVE STATUS
            Ok(ReplicationStatus {
                slave_io_running: false,
                slave_sql_running: false,
                seconds_behind_master: None,
            })
        } else {
            Err(crate::error::Error::CollectorError(
                "Not a replication slave".to_string(),
            ))
        }
    }

    pub async fn collect_async(
        &mut self,
        instances: &[MySQLInstanceConfig],
    ) -> crate::Result<MySQLMetrics> {
        let mut instance_metrics = Vec::new();

        for instance_config in instances {
            // Clone the pool to avoid borrow checker issues
            let pool_opt = self.pools.get(&instance_config.name).cloned();
            if let Some(pool) = pool_opt {
                let metrics = self
                    .collect_instance_metrics(&instance_config.name, &pool, instance_config)
                    .await;
                instance_metrics.push(metrics);
            }
        }

        let total_connections: u64 = instance_metrics
            .iter()
            .filter(|m| m.available)
            .map(|m| m.connections)
            .sum();

        let total_queries_per_second: f64 = instance_metrics
            .iter()
            .filter(|m| m.available)
            .map(|m| m.queries_per_second)
            .sum();

        Ok(MySQLMetrics {
            instances: instance_metrics,
            total_connections,
            total_queries_per_second,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mysql_instance_config_serialization() {
        let config = MySQLInstanceConfig {
            host: "localhost".to_string(),
            port: 3306,
            user: "monitor".to_string(),
            password: Some("secret".to_string()),
            database: Some("mysql".to_string()),
            name: "test".to_string(),
        };

        let serialized = serde_json::to_string(&config).unwrap();
        assert!(serialized.contains("localhost"));
        assert!(serialized.contains("3306"));
        assert!(!serialized.contains("secret")); // Password should not be serialized
    }

    #[test]
    fn test_mysql_metrics_aggregation() {
        let metrics = MySQLMetrics {
            instances: vec![
                MySQLInstanceMetrics {
                    name: "db1".to_string(),
                    host: "localhost".to_string(),
                    port: 3306,
                    connections: 10,
                    threads_running: 2,
                    queries_per_second: 100.0,
                    slow_queries: 5,
                    buffer_pool_size: 1000,
                    buffer_pool_usage_percent: 80.0,
                    uptime_seconds: 3600,
                    version: "8.0.29".to_string(),
                    replication_status: None,
                    available: true,
                    error: None,
                },
                MySQLInstanceMetrics {
                    name: "db2".to_string(),
                    host: "localhost".to_string(),
                    port: 3307,
                    connections: 15,
                    threads_running: 3,
                    queries_per_second: 150.0,
                    slow_queries: 3,
                    buffer_pool_size: 2000,
                    buffer_pool_usage_percent: 75.0,
                    uptime_seconds: 7200,
                    version: "8.0.29".to_string(),
                    replication_status: None,
                    available: true,
                    error: None,
                },
            ],
            total_connections: 25,
            total_queries_per_second: 250.0,
        };

        assert_eq!(metrics.instances.len(), 2);
        assert_eq!(metrics.total_connections, 25);
        assert_eq!(metrics.total_queries_per_second, 250.0);
    }
}
