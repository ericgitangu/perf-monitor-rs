use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CeleryConfig {
    pub broker_url: String,
    pub broker_type: CeleryBrokerType,
    pub queues: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CeleryBrokerType {
    Redis,
    RabbitMQ,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CeleryQueueStats {
    pub name: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CeleryWorkerStats {
    pub hostname: String,
    pub active_tasks: u64,
    pub processed_tasks: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CeleryStats {
    pub active_tasks: u64,
    pub scheduled_tasks: u64,
    pub workers: Vec<CeleryWorkerStats>,
    pub queues: Vec<CeleryQueueStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CeleryMetrics {
    pub stats: CeleryStats,
    pub available: bool,
    pub error: Option<String>,
}

pub struct CeleryCollector {
    _broker_url: String,
    _broker_type: CeleryBrokerType,
    queues: Vec<String>,
}

impl CeleryCollector {
    pub fn new(config: CeleryConfig) -> crate::Result<Self> {
        Ok(Self {
            _broker_url: config.broker_url,
            _broker_type: config.broker_type,
            queues: config.queues,
        })
    }

    pub async fn collect_async(&mut self) -> crate::Result<CeleryMetrics> {
        match self.try_collect().await {
            Ok(stats) => Ok(CeleryMetrics {
                stats,
                available: true,
                error: None,
            }),
            Err(e) => Ok(CeleryMetrics {
                stats: CeleryStats {
                    active_tasks: 0,
                    scheduled_tasks: 0,
                    workers: Vec::new(),
                    queues: Vec::new(),
                },
                available: false,
                error: Some(e.to_string()),
            }),
        }
    }

    async fn try_collect(&self) -> crate::Result<CeleryStats> {
        // For now, return mock data
        // In a real implementation, this would query the Celery broker
        // using either Redis commands or RabbitMQ Management API

        let queues = self.queues.iter().map(|queue_name| {
            CeleryQueueStats {
                name: queue_name.clone(),
                size: 0,
            }
        }).collect();

        Ok(CeleryStats {
            active_tasks: 0,
            scheduled_tasks: 0,
            workers: Vec::new(),
            queues,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_celery_config_serialization() {
        let config = CeleryConfig {
            broker_url: "redis://localhost:6379/0".to_string(),
            broker_type: CeleryBrokerType::Redis,
            queues: vec!["celery".to_string(), "main-queue".to_string()],
        };

        let serialized = serde_json::to_string(&config).unwrap();
        assert!(serialized.contains("redis://localhost"));
        assert!(serialized.contains("celery"));
    }

    #[test]
    fn test_celery_queue_stats() {
        let stats = CeleryQueueStats {
            name: "celery".to_string(),
            size: 50,
        };

        assert_eq!(stats.name, "celery");
        assert_eq!(stats.size, 50);
    }

    #[test]
    fn test_celery_worker_stats() {
        let worker = CeleryWorkerStats {
            hostname: "worker1@hostname".to_string(),
            active_tasks: 3,
            processed_tasks: 1000,
        };

        assert_eq!(worker.hostname, "worker1@hostname");
        assert_eq!(worker.active_tasks, 3);
    }

    #[test]
    fn test_celery_stats_aggregation() {
        let stats = CeleryStats {
            active_tasks: 5,
            scheduled_tasks: 10,
            workers: vec![
                CeleryWorkerStats {
                    hostname: "worker1".to_string(),
                    active_tasks: 3,
                    processed_tasks: 500,
                },
                CeleryWorkerStats {
                    hostname: "worker2".to_string(),
                    active_tasks: 2,
                    processed_tasks: 750,
                },
            ],
            queues: vec![
                CeleryQueueStats {
                    name: "celery".to_string(),
                    size: 15,
                },
            ],
        };

        assert_eq!(stats.active_tasks, 5);
        assert_eq!(stats.workers.len(), 2);
        assert_eq!(stats.queues.len(), 1);
    }
}
