use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RabbitMQConfig {
    pub management_url: String,
    pub username: String,
    #[serde(skip_serializing)]
    pub password: Option<String>,
    pub vhost: Option<String>,
    pub queues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RabbitMQQueueStats {
    pub name: String,
    pub messages: u64,
    pub messages_ready: u64,
    pub messages_unacknowledged: u64,
    pub consumers: u64,
    pub message_stats: Option<RabbitMQMessageStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RabbitMQMessageStats {
    pub publish_rate: f64,
    pub deliver_rate: f64,
    pub ack_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RabbitMQOverview {
    pub total_messages: u64,
    pub total_ready: u64,
    pub total_unacked: u64,
    pub total_connections: u64,
    pub total_channels: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RabbitMQStats {
    pub overview: RabbitMQOverview,
    pub queues: Vec<RabbitMQQueueStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RabbitMQMetrics {
    pub stats: RabbitMQStats,
    pub available: bool,
    pub error: Option<String>,
}

pub struct RabbitMQCollector {
    management_url: String,
    _username: String,
    _password: String,
    vhost: String,
    queues: Vec<String>,
}

impl RabbitMQCollector {
    pub fn new(config: RabbitMQConfig) -> crate::Result<Self> {
        Ok(Self {
            management_url: config.management_url,
            _username: config.username,
            _password: config.password.unwrap_or_default(),
            vhost: config.vhost.unwrap_or_else(|| "/".to_string()),
            queues: config.queues,
        })
    }

    pub async fn collect_async(&mut self) -> crate::Result<RabbitMQMetrics> {
        match self.try_collect().await {
            Ok(stats) => Ok(RabbitMQMetrics {
                stats,
                available: true,
                error: None,
            }),
            Err(e) => Ok(RabbitMQMetrics {
                stats: RabbitMQStats {
                    overview: RabbitMQOverview {
                        total_messages: 0,
                        total_ready: 0,
                        total_unacked: 0,
                        total_connections: 0,
                        total_channels: 0,
                    },
                    queues: Vec::new(),
                },
                available: false,
                error: Some(e.to_string()),
            }),
        }
    }

    async fn try_collect(&self) -> crate::Result<RabbitMQStats> {
        // For now, return mock data since reqwest requires tokio runtime
        // In a real implementation, this would use reqwest to call the Management API

        // Mock overview stats
        let overview = RabbitMQOverview {
            total_messages: 0,
            total_ready: 0,
            total_unacked: 0,
            total_connections: 0,
            total_channels: 0,
        };

        // Mock queue stats
        let queues = self.queues.iter().map(|queue_name| {
            RabbitMQQueueStats {
                name: queue_name.clone(),
                messages: 0,
                messages_ready: 0,
                messages_unacknowledged: 0,
                consumers: 0,
                message_stats: None,
            }
        }).collect();

        Ok(RabbitMQStats {
            overview,
            queues,
        })
    }

    #[allow(dead_code)]
    fn build_queue_url(&self, queue_name: &str) -> String {
        let vhost = if self.vhost == "/" {
            "%2F".to_string()
        } else {
            self.vhost.clone()
        };

        format!(
            "{}/api/queues/{}/{}",
            self.management_url.trim_end_matches('/'),
            vhost,
            queue_name
        )
    }

    #[allow(dead_code)]
    fn build_overview_url(&self) -> String {
        format!("{}/api/overview", self.management_url.trim_end_matches('/'))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rabbitmq_config_serialization() {
        let config = RabbitMQConfig {
            management_url: "http://localhost:15672".to_string(),
            username: "guest".to_string(),
            password: Some("guest".to_string()),
            vhost: Some("/".to_string()),
            queues: vec!["default".to_string(), "high_priority".to_string()],
        };

        let serialized = serde_json::to_string(&config).unwrap();
        assert!(serialized.contains("http://localhost:15672"));
        assert!(serialized.contains("guest"));
        assert!(!serialized.contains("\"password\"")); // Password should not be serialized
    }

    #[test]
    fn test_rabbitmq_queue_stats() {
        let stats = RabbitMQQueueStats {
            name: "default".to_string(),
            messages: 100,
            messages_ready: 80,
            messages_unacknowledged: 20,
            consumers: 5,
            message_stats: Some(RabbitMQMessageStats {
                publish_rate: 10.5,
                deliver_rate: 9.8,
                ack_rate: 9.5,
            }),
        };

        assert_eq!(stats.name, "default");
        assert_eq!(stats.messages, 100);
        assert_eq!(stats.consumers, 5);
    }

    #[test]
    fn test_queue_url_building() {
        let config = RabbitMQConfig {
            management_url: "http://localhost:15672".to_string(),
            username: "guest".to_string(),
            password: None,
            vhost: Some("/".to_string()),
            queues: vec!["myqueue".to_string()],
        };

        let collector = RabbitMQCollector::new(config).unwrap();
        let url = collector.build_queue_url("myqueue");
        assert_eq!(url, "http://localhost:15672/api/queues/%2F/myqueue");
    }

    #[test]
    fn test_overview_url_building() {
        let config = RabbitMQConfig {
            management_url: "http://localhost:15672/".to_string(),
            username: "guest".to_string(),
            password: None,
            vhost: None,
            queues: vec![],
        };

        let collector = RabbitMQCollector::new(config).unwrap();
        let url = collector.build_overview_url();
        assert_eq!(url, "http://localhost:15672/api/overview");
    }
}
