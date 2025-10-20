use redis::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SidekiqConfig {
    pub redis_url: String,
    pub namespace: Option<String>,
    pub queues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidekiqQueueStats {
    pub name: String,
    pub size: u64,
    pub latency: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidekiqStats {
    pub processed: u64,
    pub failed: u64,
    pub busy: u64,
    pub enqueued: u64,
    pub scheduled_size: u64,
    pub retry_size: u64,
    pub dead_size: u64,
    pub workers_size: u64,
    pub queues: Vec<SidekiqQueueStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidekiqMetrics {
    pub stats: SidekiqStats,
    pub available: bool,
    pub error: Option<String>,
}

pub struct SidekiqCollector {
    client: Client,
    namespace: String,
    queues: Vec<String>,
}

impl SidekiqCollector {
    pub fn new(config: SidekiqConfig) -> crate::Result<Self> {
        let client = Client::open(config.redis_url.as_str()).map_err(|e| {
            crate::error::Error::CollectorError(format!("Sidekiq Redis client error: {}", e))
        })?;

        let namespace = config.namespace.unwrap_or_else(|| String::new());

        Ok(Self {
            client,
            namespace,
            queues: config.queues,
        })
    }

    fn get_key(&self, key: &str) -> String {
        if self.namespace.is_empty() {
            key.to_string()
        } else {
            format!("{}:{}", self.namespace, key)
        }
    }

    async fn get_stat(&self, conn: &mut redis::aio::MultiplexedConnection, key: &str) -> crate::Result<u64> {
        let full_key = self.get_key(key);
        let value: Option<String> = redis::cmd("GET")
            .arg(&full_key)
            .query_async(conn)
            .await
            .map_err(|e| {
                crate::error::Error::CollectorError(format!("Sidekiq stat read error: {}", e))
            })?;

        Ok(value.and_then(|v| v.parse::<u64>().ok()).unwrap_or(0))
    }

    async fn get_queue_size(&self, conn: &mut redis::aio::MultiplexedConnection, queue: &str) -> crate::Result<u64> {
        let queue_key = self.get_key(&format!("queue:{}", queue));
        let size: u64 = redis::cmd("LLEN")
            .arg(&queue_key)
            .query_async(conn)
            .await
            .map_err(|e| {
                crate::error::Error::CollectorError(format!("Sidekiq queue size error: {}", e))
            })?;

        Ok(size)
    }

    async fn get_queue_latency(&self, conn: &mut redis::aio::MultiplexedConnection, queue: &str) -> crate::Result<f64> {
        let queue_key = self.get_key(&format!("queue:{}", queue));

        // Get the first job in the queue (oldest)
        let job_json: Option<String> = redis::cmd("LINDEX")
            .arg(&queue_key)
            .arg(0)
            .query_async(conn)
            .await
            .map_err(|e| {
                crate::error::Error::CollectorError(format!("Sidekiq latency read error: {}", e))
            })?;

        if let Some(json) = job_json {
            // Parse the job JSON to get enqueued_at timestamp
            if let Ok(job) = serde_json::from_str::<serde_json::Value>(&json) {
                if let Some(enqueued_at) = job.get("enqueued_at").and_then(|v| v.as_f64()) {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs_f64();
                    return Ok(now - enqueued_at);
                }
            }
        }

        Ok(0.0)
    }

    async fn get_workers_count(&self, conn: &mut redis::aio::MultiplexedConnection) -> crate::Result<u64> {
        let workers_key = self.get_key("workers");
        let count: u64 = redis::cmd("SCARD")
            .arg(&workers_key)
            .query_async(conn)
            .await
            .map_err(|e| {
                crate::error::Error::CollectorError(format!("Sidekiq workers count error: {}", e))
            })?;

        Ok(count)
    }

    async fn get_scheduled_size(&self, conn: &mut redis::aio::MultiplexedConnection) -> crate::Result<u64> {
        let scheduled_key = self.get_key("schedule");
        let size: u64 = redis::cmd("ZCARD")
            .arg(&scheduled_key)
            .query_async(conn)
            .await
            .map_err(|e| {
                crate::error::Error::CollectorError(format!("Sidekiq scheduled size error: {}", e))
            })?;

        Ok(size)
    }

    async fn get_retry_size(&self, conn: &mut redis::aio::MultiplexedConnection) -> crate::Result<u64> {
        let retry_key = self.get_key("retry");
        let size: u64 = redis::cmd("ZCARD")
            .arg(&retry_key)
            .query_async(conn)
            .await
            .map_err(|e| {
                crate::error::Error::CollectorError(format!("Sidekiq retry size error: {}", e))
            })?;

        Ok(size)
    }

    async fn get_dead_size(&self, conn: &mut redis::aio::MultiplexedConnection) -> crate::Result<u64> {
        let dead_key = self.get_key("dead");
        let size: u64 = redis::cmd("ZCARD")
            .arg(&dead_key)
            .query_async(conn)
            .await
            .map_err(|e| {
                crate::error::Error::CollectorError(format!("Sidekiq dead size error: {}", e))
            })?;

        Ok(size)
    }

    pub async fn collect_async(&mut self) -> crate::Result<SidekiqMetrics> {
        match self.try_collect().await {
            Ok(stats) => Ok(SidekiqMetrics {
                stats,
                available: true,
                error: None,
            }),
            Err(e) => Ok(SidekiqMetrics {
                stats: SidekiqStats {
                    processed: 0,
                    failed: 0,
                    busy: 0,
                    enqueued: 0,
                    scheduled_size: 0,
                    retry_size: 0,
                    dead_size: 0,
                    workers_size: 0,
                    queues: Vec::new(),
                },
                available: false,
                error: Some(e.to_string()),
            }),
        }
    }

    async fn try_collect(&mut self) -> crate::Result<SidekiqStats> {
        let mut conn = self.client.get_multiplexed_async_connection().await.map_err(|e| {
            crate::error::Error::CollectorError(format!("Sidekiq Redis connection error: {}", e))
        })?;

        // Get basic stats
        let processed = self.get_stat(&mut conn, "stat:processed").await?;
        let failed = self.get_stat(&mut conn, "stat:failed").await?;

        // Get queue sizes
        let mut total_enqueued = 0u64;
        let mut queue_stats = Vec::new();

        for queue_name in &self.queues {
            let size = self.get_queue_size(&mut conn, queue_name).await?;
            let latency = if size > 0 {
                self.get_queue_latency(&mut conn, queue_name).await?
            } else {
                0.0
            };

            total_enqueued += size;

            queue_stats.push(SidekiqQueueStats {
                name: queue_name.clone(),
                size,
                latency,
            });
        }

        // Get workers count (busy workers)
        let workers_size = self.get_workers_count(&mut conn).await?;
        let busy = workers_size; // All workers in the set are busy

        // Get scheduled, retry, and dead sizes
        let scheduled_size = self.get_scheduled_size(&mut conn).await?;
        let retry_size = self.get_retry_size(&mut conn).await?;
        let dead_size = self.get_dead_size(&mut conn).await?;

        Ok(SidekiqStats {
            processed,
            failed,
            busy,
            enqueued: total_enqueued,
            scheduled_size,
            retry_size,
            dead_size,
            workers_size,
            queues: queue_stats,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sidekiq_config_serialization() {
        let config = SidekiqConfig {
            redis_url: "redis://localhost:6379/0".to_string(),
            namespace: Some("sidekiq".to_string()),
            queues: vec![
                "default".to_string(),
                "ug_mtn".to_string(),
                "mtn_open_api_debit".to_string(),
            ],
        };

        let serialized = serde_json::to_string(&config).unwrap();
        assert!(serialized.contains("redis://localhost"));
        assert!(serialized.contains("ug_mtn"));
    }

    #[test]
    fn test_sidekiq_queue_stats() {
        let stats = SidekiqQueueStats {
            name: "ug_mtn".to_string(),
            size: 150,
            latency: 2.5,
        };

        assert_eq!(stats.name, "ug_mtn");
        assert_eq!(stats.size, 150);
        assert_eq!(stats.latency, 2.5);
    }

    #[test]
    fn test_sidekiq_stats_aggregation() {
        let stats = SidekiqStats {
            processed: 10000,
            failed: 50,
            busy: 5,
            enqueued: 200,
            scheduled_size: 30,
            retry_size: 10,
            dead_size: 5,
            workers_size: 5,
            queues: vec![
                SidekiqQueueStats {
                    name: "default".to_string(),
                    size: 50,
                    latency: 1.2,
                },
                SidekiqQueueStats {
                    name: "ug_mtn".to_string(),
                    size: 150,
                    latency: 2.5,
                },
            ],
        };

        assert_eq!(stats.processed, 10000);
        assert_eq!(stats.failed, 50);
        assert_eq!(stats.queues.len(), 2);
        assert_eq!(stats.enqueued, 200);
    }

    #[test]
    fn test_namespace_key_generation() {
        let config = SidekiqConfig {
            redis_url: "redis://localhost:6379/0".to_string(),
            namespace: Some("sidekiq".to_string()),
            queues: vec!["default".to_string()],
        };

        let collector = SidekiqCollector::new(config).unwrap();
        assert_eq!(collector.get_key("stat:processed"), "sidekiq:stat:processed");
    }

    #[test]
    fn test_no_namespace_key_generation() {
        let config = SidekiqConfig {
            redis_url: "redis://localhost:6379/0".to_string(),
            namespace: None,
            queues: vec!["default".to_string()],
        };

        let collector = SidekiqCollector::new(config).unwrap();
        assert_eq!(collector.get_key("stat:processed"), "stat:processed");
    }
}
