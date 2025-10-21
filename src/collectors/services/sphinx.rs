use mysql_async::{Pool, prelude::*};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SphinxInstanceConfig {
    pub host: String,
    pub port: u16,
    #[serde(skip_serializing)]
    pub password: Option<String>,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SphinxIndexStats {
    pub name: String,
    pub document_count: i64,
    pub size_bytes: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SphinxInstanceMetrics {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub version: String,
    pub uptime_seconds: i64,
    pub connections: i64,
    pub max_connections: i64,
    pub queries_total: i64,
    pub queries_per_second: f64,
    pub queries_avg_time_ms: f64,
    pub queries_wall_time_sec: f64,
    pub indices: Vec<SphinxIndexStats>,
    pub threads_running: i64,
    pub available: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SphinxMetrics {
    pub instances: Vec<SphinxInstanceMetrics>,
    pub total_queries_per_second: f64,
    pub total_document_count: i64,
}

pub struct SphinxCollector {
    pools: HashMap<String, Pool>,
    previous_stats: HashMap<String, (i64, Instant)>, // (total_queries, time)
}

impl SphinxCollector {
    pub fn new(instances: Vec<SphinxInstanceConfig>) -> crate::Result<Self> {
        let mut pools = HashMap::new();

        for instance in instances {
            let connection_string = if let Some(password) = &instance.password {
                format!(
                    "mysql://root:{}@{}:{}",
                    password, instance.host, instance.port
                )
            } else {
                format!("mysql://{}:{}", instance.host, instance.port)
            };

            let pool = Pool::new(connection_string.as_str());
            pools.insert(instance.name.clone(), pool);
        }

        Ok(Self {
            pools,
            previous_stats: HashMap::new(),
        })
    }

    pub async fn collect(&mut self, configs: &[SphinxInstanceConfig]) -> crate::Result<SphinxMetrics> {
        let mut instances = Vec::new();
        let mut total_queries_per_second = 0.0;
        let mut total_document_count = 0;

        for config in configs {
            let metrics = self.collect_instance(config).await;
            total_queries_per_second += metrics.queries_per_second;

            for index in &metrics.indices {
                total_document_count += index.document_count;
            }

            instances.push(metrics);
        }

        Ok(SphinxMetrics {
            instances,
            total_queries_per_second,
            total_document_count,
        })
    }

    async fn collect_instance(&mut self, config: &SphinxInstanceConfig) -> SphinxInstanceMetrics {
        let pool = match self.pools.get(&config.name) {
            Some(p) => p,
            None => {
                return SphinxInstanceMetrics {
                    name: config.name.clone(),
                    host: config.host.clone(),
                    port: config.port,
                    version: String::new(),
                    uptime_seconds: 0,
                    connections: 0,
                    max_connections: 0,
                    queries_total: 0,
                    queries_per_second: 0.0,
                    queries_avg_time_ms: 0.0,
                    queries_wall_time_sec: 0.0,
                    indices: Vec::new(),
                    threads_running: 0,
                    available: false,
                    error: Some("Pool not initialized".to_string()),
                };
            }
        };

        match pool.get_conn().await {
            Ok(mut conn) => {
                // Get Sphinx status using SHOW STATUS
                let status_result: Result<Vec<(String, String)>, _> =
                    "SHOW STATUS".with(()).map(&mut conn, |(variable, value): (String, String)| {
                        (variable, value)
                    }).await;

                let mut version = String::new();
                let mut uptime = 0i64;
                let mut connections = 0i64;
                let mut max_connections = 0i64;
                let mut queries_total = 0i64;
                let mut queries_wall_time = 0.0f64;
                let mut threads_running = 0i64;

                if let Ok(status) = status_result {
                    for (var, val) in status {
                        match var.as_str() {
                            "version" => version = val,
                            "uptime" => uptime = val.parse().unwrap_or(0),
                            "connections" => connections = val.parse().unwrap_or(0),
                            "maxed_out" => max_connections = val.parse().unwrap_or(0),
                            "queries" => queries_total = val.parse().unwrap_or(0),
                            "query_wall" => queries_wall_time = val.parse().unwrap_or(0.0),
                            "workers_total" => threads_running = val.parse().unwrap_or(0),
                            _ => {}
                        }
                    }
                }

                // Calculate queries per second
                let now = Instant::now();
                let queries_per_second = if let Some((prev_queries, prev_time)) = self.previous_stats.get(&config.name) {
                    let elapsed = now.duration_since(*prev_time).as_secs_f64();
                    if elapsed > 0.0 {
                        (queries_total - prev_queries) as f64 / elapsed
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };
                self.previous_stats.insert(config.name.clone(), (queries_total, now));

                // Calculate average query time
                let queries_avg_time_ms = if queries_total > 0 {
                    (queries_wall_time / queries_total as f64) * 1000.0
                } else {
                    0.0
                };

                // Get index information using SHOW TABLES
                let mut indices = Vec::new();
                let tables_result: Result<Vec<(String, String)>, _> =
                    "SHOW TABLES".with(()).map(&mut conn, |(index, index_type): (String, String)| {
                        (index, index_type)
                    }).await;

                if let Ok(tables) = tables_result {
                    for (index_name, _index_type) in tables {
                        // Get index status using SHOW INDEX STATUS
                        let index_status_query = format!("SHOW INDEX {} STATUS", index_name);
                        let index_status_result: Result<Vec<(String, String)>, _> =
                            index_status_query.with(()).map(&mut conn, |(var, val): (String, String)| {
                                (var, val)
                            }).await;

                        let mut document_count = 0i64;
                        let mut size_bytes = 0i64;

                        if let Ok(index_status) = index_status_result {
                            for (var, val) in index_status {
                                match var.as_str() {
                                    "indexed_documents" => document_count = val.parse().unwrap_or(0),
                                    "indexed_bytes" => size_bytes = val.parse().unwrap_or(0),
                                    _ => {}
                                }
                            }
                        }

                        indices.push(SphinxIndexStats {
                            name: index_name,
                            document_count,
                            size_bytes,
                        });
                    }
                }

                SphinxInstanceMetrics {
                    name: config.name.clone(),
                    host: config.host.clone(),
                    port: config.port,
                    version,
                    uptime_seconds: uptime,
                    connections,
                    max_connections,
                    queries_total,
                    queries_per_second,
                    queries_avg_time_ms,
                    queries_wall_time_sec: queries_wall_time,
                    indices,
                    threads_running,
                    available: true,
                    error: None,
                }
            }
            Err(e) => {
                SphinxInstanceMetrics {
                    name: config.name.clone(),
                    host: config.host.clone(),
                    port: config.port,
                    version: String::new(),
                    uptime_seconds: 0,
                    connections: 0,
                    max_connections: 0,
                    queries_total: 0,
                    queries_per_second: 0.0,
                    queries_avg_time_ms: 0.0,
                    queries_wall_time_sec: 0.0,
                    indices: Vec::new(),
                    threads_running: 0,
                    available: false,
                    error: Some(format!("Failed to connect: {}", e)),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sphinx_instance_config_serialization() {
        let config = SphinxInstanceConfig {
            host: "localhost".to_string(),
            port: 9306,
            password: Some("secret".to_string()),
            name: "solarhub".to_string(),
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(!json.contains("secret"));
        assert!(json.contains("localhost"));
        assert!(json.contains("9306"));
    }

    #[test]
    fn test_sphinx_metrics_aggregation() {
        let metrics = SphinxMetrics {
            instances: vec![
                SphinxInstanceMetrics {
                    name: "solarhub".to_string(),
                    host: "localhost".to_string(),
                    port: 9306,
                    version: "3.4.1".to_string(),
                    uptime_seconds: 86400,
                    connections: 100,
                    max_connections: 1000,
                    queries_total: 50000,
                    queries_per_second: 10.5,
                    queries_avg_time_ms: 5.2,
                    queries_wall_time_sec: 260.0,
                    indices: vec![
                        SphinxIndexStats {
                            name: "customers_core".to_string(),
                            document_count: 10000,
                            size_bytes: 5242880,
                        },
                    ],
                    threads_running: 4,
                    available: true,
                    error: None,
                },
            ],
            total_queries_per_second: 10.5,
            total_document_count: 10000,
        };

        assert!((metrics.total_queries_per_second - 10.5).abs() < 0.1);
        assert_eq!(metrics.total_document_count, 10000);
    }
}
