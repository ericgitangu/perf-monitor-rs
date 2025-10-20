use redis::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RedisInstanceConfig {
    pub host: String,
    pub port: u16,
    #[serde(skip_serializing)]
    pub password: Option<String>,
    pub db: u8,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisKeyspaceStats {
    pub db_index: u8,
    pub keys: u64,
    pub expires: u64,
    pub avg_ttl: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisInstanceMetrics {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub version: String,
    pub uptime_seconds: u64,
    pub connected_clients: u64,
    pub blocked_clients: u64,
    pub used_memory_bytes: u64,
    pub used_memory_rss_bytes: u64,
    pub used_memory_peak_bytes: u64,
    pub maxmemory_bytes: u64,
    pub memory_fragmentation_ratio: f64,
    pub ops_per_second: f64,
    pub instantaneous_ops_per_sec: u64,
    pub hit_rate: f64,
    pub keyspace_hits: u64,
    pub keyspace_misses: u64,
    pub evicted_keys: u64,
    pub expired_keys: u64,
    pub total_commands_processed: u64,
    pub keyspace: Vec<RedisKeyspaceStats>,
    pub replication_role: String,
    pub connected_slaves: u64,
    pub available: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisMetrics {
    pub instances: Vec<RedisInstanceMetrics>,
    pub total_connected_clients: u64,
    pub total_used_memory_bytes: u64,
    pub total_ops_per_second: f64,
}

pub struct RedisCollector {
    clients: HashMap<String, Client>,
    previous_stats: HashMap<String, (u64, Instant)>, // (total_commands, time)
}

impl RedisCollector {
    pub fn new(instances: Vec<RedisInstanceConfig>) -> crate::Result<Self> {
        let mut clients = HashMap::new();

        for instance in instances {
            let connection_string = if let Some(password) = &instance.password {
                format!(
                    "redis://:{}@{}:{}/{}",
                    password, instance.host, instance.port, instance.db
                )
            } else {
                format!("redis://{}:{}/{}", instance.host, instance.port, instance.db)
            };

            let client = Client::open(connection_string).map_err(|e| {
                crate::error::Error::CollectorError(format!("Redis client error: {}", e))
            })?;

            clients.insert(instance.name.clone(), client);
        }

        Ok(Self {
            clients,
            previous_stats: HashMap::new(),
        })
    }

    async fn collect_instance_metrics(
        &mut self,
        name: &str,
        client: &Client,
        config: &RedisInstanceConfig,
    ) -> RedisInstanceMetrics {
        match self.try_collect_instance_metrics(name, client, config).await {
            Ok(metrics) => metrics,
            Err(e) => RedisInstanceMetrics {
                name: name.to_string(),
                host: config.host.clone(),
                port: config.port,
                version: String::from("unknown"),
                uptime_seconds: 0,
                connected_clients: 0,
                blocked_clients: 0,
                used_memory_bytes: 0,
                used_memory_rss_bytes: 0,
                used_memory_peak_bytes: 0,
                maxmemory_bytes: 0,
                memory_fragmentation_ratio: 0.0,
                ops_per_second: 0.0,
                instantaneous_ops_per_sec: 0,
                hit_rate: 0.0,
                keyspace_hits: 0,
                keyspace_misses: 0,
                evicted_keys: 0,
                expired_keys: 0,
                total_commands_processed: 0,
                keyspace: Vec::new(),
                replication_role: String::from("unknown"),
                connected_slaves: 0,
                available: false,
                error: Some(e.to_string()),
            },
        }
    }

    async fn try_collect_instance_metrics(
        &mut self,
        name: &str,
        client: &Client,
        config: &RedisInstanceConfig,
    ) -> crate::Result<RedisInstanceMetrics> {
        let mut conn = client.get_multiplexed_async_connection().await.map_err(|e| {
            crate::error::Error::CollectorError(format!("Redis connection error: {}", e))
        })?;

        // Get INFO output
        let info: String = redis::cmd("INFO")
            .query_async(&mut conn)
            .await
            .map_err(|e| {
                crate::error::Error::CollectorError(format!("Redis INFO error: {}", e))
            })?;

        let info_map = self.parse_info(&info);

        // Parse Server section
        let version = info_map
            .get("redis_version")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());

        let uptime_seconds = info_map
            .get("uptime_in_seconds")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);

        // Parse Clients section
        let connected_clients = info_map
            .get("connected_clients")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);

        let blocked_clients = info_map
            .get("blocked_clients")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);

        // Parse Memory section
        let used_memory_bytes = info_map
            .get("used_memory")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);

        let used_memory_rss_bytes = info_map
            .get("used_memory_rss")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);

        let used_memory_peak_bytes = info_map
            .get("used_memory_peak")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);

        let maxmemory_bytes = info_map
            .get("maxmemory")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);

        let memory_fragmentation_ratio = info_map
            .get("mem_fragmentation_ratio")
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(0.0);

        // Parse Stats section
        let total_commands_processed = info_map
            .get("total_commands_processed")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);

        let instantaneous_ops_per_sec = info_map
            .get("instantaneous_ops_per_sec")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);

        let keyspace_hits = info_map
            .get("keyspace_hits")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);

        let keyspace_misses = info_map
            .get("keyspace_misses")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);

        let evicted_keys = info_map
            .get("evicted_keys")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);

        let expired_keys = info_map
            .get("expired_keys")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);

        // Calculate hit rate
        let total_hits_misses = keyspace_hits + keyspace_misses;
        let hit_rate = if total_hits_misses > 0 {
            (keyspace_hits as f64 / total_hits_misses as f64) * 100.0
        } else {
            0.0
        };

        // Calculate ops per second
        let now = Instant::now();
        let ops_per_second = if let Some((prev_commands, prev_time)) = self.previous_stats.get(name)
        {
            let elapsed = now.duration_since(*prev_time).as_secs_f64();
            if elapsed > 0.0 {
                (total_commands_processed.saturating_sub(*prev_commands)) as f64 / elapsed
            } else {
                0.0
            }
        } else {
            0.0
        };
        self.previous_stats
            .insert(name.to_string(), (total_commands_processed, now));

        // Parse Replication section
        let replication_role = info_map
            .get("role")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());

        let connected_slaves = info_map
            .get("connected_slaves")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);

        // Parse Keyspace section
        let keyspace = self.parse_keyspace(&info_map);

        Ok(RedisInstanceMetrics {
            name: name.to_string(),
            host: config.host.clone(),
            port: config.port,
            version,
            uptime_seconds,
            connected_clients,
            blocked_clients,
            used_memory_bytes,
            used_memory_rss_bytes,
            used_memory_peak_bytes,
            maxmemory_bytes,
            memory_fragmentation_ratio,
            ops_per_second,
            instantaneous_ops_per_sec,
            hit_rate,
            keyspace_hits,
            keyspace_misses,
            evicted_keys,
            expired_keys,
            total_commands_processed,
            keyspace,
            replication_role,
            connected_slaves,
            available: true,
            error: None,
        })
    }

    fn parse_info(&self, info: &str) -> HashMap<String, String> {
        let mut map = HashMap::new();

        for line in info.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once(':') {
                map.insert(key.trim().to_string(), value.trim().to_string());
            }
        }

        map
    }

    fn parse_keyspace(&self, info_map: &HashMap<String, String>) -> Vec<RedisKeyspaceStats> {
        let mut keyspace = Vec::new();

        for i in 0..16 {
            let db_key = format!("db{}", i);
            if let Some(db_info) = info_map.get(&db_key) {
                // Parse "keys=X,expires=Y,avg_ttl=Z"
                let mut keys = 0;
                let mut expires = 0;
                let mut avg_ttl = 0;

                for part in db_info.split(',') {
                    if let Some((k, v)) = part.split_once('=') {
                        match k.trim() {
                            "keys" => keys = v.parse::<u64>().unwrap_or(0),
                            "expires" => expires = v.parse::<u64>().unwrap_or(0),
                            "avg_ttl" => avg_ttl = v.parse::<u64>().unwrap_or(0),
                            _ => {}
                        }
                    }
                }

                keyspace.push(RedisKeyspaceStats {
                    db_index: i,
                    keys,
                    expires,
                    avg_ttl,
                });
            }
        }

        keyspace
    }

    pub async fn collect_async(
        &mut self,
        instances: &[RedisInstanceConfig],
    ) -> crate::Result<RedisMetrics> {
        let mut instance_metrics = Vec::new();

        for instance_config in instances {
            // Clone the client Arc to avoid borrow checker issues
            let client_opt = self.clients.get(&instance_config.name).cloned();
            if let Some(client) = client_opt {
                let metrics = self
                    .collect_instance_metrics(&instance_config.name, &client, instance_config)
                    .await;
                instance_metrics.push(metrics);
            }
        }

        let total_connected_clients: u64 = instance_metrics
            .iter()
            .filter(|m| m.available)
            .map(|m| m.connected_clients)
            .sum();

        let total_used_memory_bytes: u64 = instance_metrics
            .iter()
            .filter(|m| m.available)
            .map(|m| m.used_memory_bytes)
            .sum();

        let total_ops_per_second: f64 = instance_metrics
            .iter()
            .filter(|m| m.available)
            .map(|m| m.ops_per_second)
            .sum();

        Ok(RedisMetrics {
            instances: instance_metrics,
            total_connected_clients,
            total_used_memory_bytes,
            total_ops_per_second,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redis_instance_config_serialization() {
        let config = RedisInstanceConfig {
            host: "localhost".to_string(),
            port: 6379,
            password: Some("secret".to_string()),
            db: 0,
            name: "test".to_string(),
        };

        let serialized = serde_json::to_string(&config).unwrap();
        assert!(serialized.contains("localhost"));
        assert!(serialized.contains("6379"));
        assert!(!serialized.contains("secret")); // Password should not be serialized
    }

    #[test]
    fn test_redis_info_parsing() {
        let collector = RedisCollector {
            clients: HashMap::new(),
            previous_stats: HashMap::new(),
        };

        let info = r#"
# Server
redis_version:6.2.6
uptime_in_seconds:86400

# Clients
connected_clients:10
blocked_clients:2

# Memory
used_memory:1048576
maxmemory:10485760
"#;

        let parsed = collector.parse_info(info);
        assert_eq!(parsed.get("redis_version"), Some(&"6.2.6".to_string()));
        assert_eq!(parsed.get("uptime_in_seconds"), Some(&"86400".to_string()));
        assert_eq!(parsed.get("connected_clients"), Some(&"10".to_string()));
    }

    #[test]
    fn test_redis_keyspace_parsing() {
        let collector = RedisCollector {
            clients: HashMap::new(),
            previous_stats: HashMap::new(),
        };

        let mut info_map = HashMap::new();
        info_map.insert("db0".to_string(), "keys=1000,expires=100,avg_ttl=3600".to_string());
        info_map.insert("db1".to_string(), "keys=500,expires=50,avg_ttl=7200".to_string());

        let keyspace = collector.parse_keyspace(&info_map);
        assert_eq!(keyspace.len(), 2);
        assert_eq!(keyspace[0].keys, 1000);
        assert_eq!(keyspace[0].expires, 100);
        assert_eq!(keyspace[1].keys, 500);
    }
}
