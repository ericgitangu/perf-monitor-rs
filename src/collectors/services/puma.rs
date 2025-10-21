use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PumaInstanceConfig {
    pub url: String,
    #[serde(skip_serializing)]
    pub token: Option<String>,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PumaWorkerStats {
    pub index: usize,
    pub pid: i64,
    pub phase: String,
    pub booted: bool,
    pub last_checkin: String,
    pub last_status: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PumaThreadStats {
    pub pool_capacity: i64,
    pub max_threads: i64,
    pub running: i64,
    pub backlog: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PumaInstanceMetrics {
    pub name: String,
    pub url: String,
    pub phase: String,
    pub workers: i64,
    pub booted_workers: i64,
    pub old_workers: i64,
    pub running_threads: i64,
    pub max_threads: i64,
    pub pool_capacity: i64,
    pub backlog: i64,
    pub requests_count: i64,
    pub worker_details: Vec<PumaWorkerStats>,
    pub available: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PumaMetrics {
    pub instances: Vec<PumaInstanceMetrics>,
    pub total_workers: i64,
    pub total_threads: i64,
    pub total_backlog: i64,
}

pub struct PumaCollector {
    client: Client,
}

impl PumaCollector {
    pub fn new() -> crate::Result<Self> {
        Ok(Self {
            client: Client::new(),
        })
    }

    pub async fn collect(&self, configs: &[PumaInstanceConfig]) -> crate::Result<PumaMetrics> {
        let mut instances = Vec::new();
        let mut total_workers = 0;
        let mut total_threads = 0;
        let mut total_backlog = 0;

        for config in configs {
            let metrics = self.collect_instance(config).await;
            total_workers += metrics.workers;
            total_threads += metrics.running_threads;
            total_backlog += metrics.backlog;
            instances.push(metrics);
        }

        Ok(PumaMetrics {
            instances,
            total_workers,
            total_threads,
            total_backlog,
        })
    }

    async fn collect_instance(&self, config: &PumaInstanceConfig) -> PumaInstanceMetrics {
        let stats_url = if config.url.ends_with("/stats") {
            config.url.clone()
        } else {
            format!("{}/stats", config.url)
        };

        let mut request = self.client.get(&stats_url);

        if let Some(token) = &config.token {
            request = request.query(&[("token", token)]);
        }

        match request.send().await {
            Ok(response) => {
                match response.json::<Value>().await {
                    Ok(data) => {
                        let _phase = data.get("phase")
                            .and_then(|v| v.as_i64())
                            .unwrap_or(0);
                        let phase_str = data.get("phase")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string();

                        let mut workers = 0i64;
                        let mut booted_workers = 0i64;
                        let old_workers = 0i64;
                        let mut running_threads = 0i64;
                        let mut max_threads = 0i64;
                        let mut pool_capacity = 0i64;
                        let mut backlog = 0i64;
                        let mut requests_count = 0i64;
                        let mut worker_details = Vec::new();

                        // Parse worker_status if in clustered mode
                        if let Some(worker_status) = data.get("worker_status") {
                            if let Some(workers_array) = worker_status.as_array() {
                                workers = workers_array.len() as i64;

                                for worker in workers_array {
                                    if let Some(worker_obj) = worker.as_object() {
                                        let booted = worker_obj.get("booted")
                                            .and_then(|v| v.as_bool())
                                            .unwrap_or(false);

                                        if booted {
                                            booted_workers += 1;
                                        }

                                        let last_status = worker_obj.get("last_status").cloned().unwrap_or(Value::Null);

                                        // Parse thread stats from last_status
                                        if let Some(status_obj) = last_status.as_object() {
                                            if let Some(backlog_val) = status_obj.get("backlog").and_then(|v| v.as_i64()) {
                                                backlog += backlog_val;
                                            }
                                            if let Some(running_val) = status_obj.get("running").and_then(|v| v.as_i64()) {
                                                running_threads += running_val;
                                            }
                                            if let Some(max_val) = status_obj.get("max_threads").and_then(|v| v.as_i64()) {
                                                max_threads = max_val.max(max_threads);
                                            }
                                            if let Some(pool_val) = status_obj.get("pool_capacity").and_then(|v| v.as_i64()) {
                                                pool_capacity = pool_val.max(pool_capacity);
                                            }
                                            if let Some(requests_val) = status_obj.get("requests_count").and_then(|v| v.as_i64()) {
                                                requests_count += requests_val;
                                            }
                                        }

                                        worker_details.push(PumaWorkerStats {
                                            index: worker_obj.get("index").and_then(|v| v.as_i64()).unwrap_or(0) as usize,
                                            pid: worker_obj.get("pid").and_then(|v| v.as_i64()).unwrap_or(0),
                                            phase: worker_obj.get("phase").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                            booted,
                                            last_checkin: worker_obj.get("last_checkin").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                            last_status,
                                        });
                                    }
                                }
                            }
                        } else {
                            // Single mode (no workers)
                            workers = 1;
                            booted_workers = 1;

                            backlog = data.get("backlog").and_then(|v| v.as_i64()).unwrap_or(0);
                            running_threads = data.get("running").and_then(|v| v.as_i64()).unwrap_or(0);
                            max_threads = data.get("max_threads").and_then(|v| v.as_i64()).unwrap_or(0);
                            pool_capacity = data.get("pool_capacity").and_then(|v| v.as_i64()).unwrap_or(0);
                            requests_count = data.get("requests_count").and_then(|v| v.as_i64()).unwrap_or(0);
                        }

                        PumaInstanceMetrics {
                            name: config.name.clone(),
                            url: config.url.clone(),
                            phase: phase_str,
                            workers,
                            booted_workers,
                            old_workers,
                            running_threads,
                            max_threads,
                            pool_capacity,
                            backlog,
                            requests_count,
                            worker_details,
                            available: true,
                            error: None,
                        }
                    }
                    Err(e) => {
                        PumaInstanceMetrics {
                            name: config.name.clone(),
                            url: config.url.clone(),
                            phase: String::new(),
                            workers: 0,
                            booted_workers: 0,
                            old_workers: 0,
                            running_threads: 0,
                            max_threads: 0,
                            pool_capacity: 0,
                            backlog: 0,
                            requests_count: 0,
                            worker_details: Vec::new(),
                            available: false,
                            error: Some(format!("Failed to parse response: {}", e)),
                        }
                    }
                }
            }
            Err(e) => {
                PumaInstanceMetrics {
                    name: config.name.clone(),
                    url: config.url.clone(),
                    phase: String::new(),
                    workers: 0,
                    booted_workers: 0,
                    old_workers: 0,
                    running_threads: 0,
                    max_threads: 0,
                    pool_capacity: 0,
                    backlog: 0,
                    requests_count: 0,
                    worker_details: Vec::new(),
                    available: false,
                    error: Some(format!("Failed to connect: {}", e)),
                }
            }
        }
    }
}

impl Default for PumaCollector {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_puma_instance_config_serialization() {
        let config = PumaInstanceConfig {
            url: "http://localhost:9292/stats".to_string(),
            token: Some("secret_token".to_string()),
            name: "solarhub".to_string(),
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(!json.contains("secret_token")); // Token should not be serialized
        assert!(json.contains("localhost"));
        assert!(json.contains("9292"));
    }

    #[test]
    fn test_puma_metrics_aggregation() {
        let metrics = PumaMetrics {
            instances: vec![
                PumaInstanceMetrics {
                    name: "solarhub".to_string(),
                    url: "http://localhost:9292/stats".to_string(),
                    phase: "0".to_string(),
                    workers: 4,
                    booted_workers: 4,
                    old_workers: 0,
                    running_threads: 20,
                    max_threads: 5,
                    pool_capacity: 5,
                    backlog: 0,
                    requests_count: 1000,
                    worker_details: Vec::new(),
                    available: true,
                    error: None,
                },
            ],
            total_workers: 4,
            total_threads: 20,
            total_backlog: 0,
        };

        assert_eq!(metrics.total_workers, 4);
        assert_eq!(metrics.total_threads, 20);
        assert_eq!(metrics.total_backlog, 0);
    }
}
