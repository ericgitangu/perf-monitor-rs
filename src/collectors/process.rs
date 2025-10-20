use super::MetricCollector;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use sysinfo::{ProcessRefreshKind, RefreshKind, System};

/// Service detection patterns
const SERVICE_PATTERNS: &[(&str, &str)] = &[
    ("mysql", "mysqld"),
    ("postgres", "postgres"),
    ("redis", "redis-server"),
    ("mongodb", "mongod"),
    ("rabbitmq", "beam.*rabbitmq"),
    ("sidekiq", "sidekiq"),
    ("celery", "celery"),
    ("puma", "puma"),
    ("nginx", "nginx"),
    ("elasticsearch", "elasticsearch"),
    ("sphinx", "searchd"),
    ("node", "node"),
    ("ruby", "ruby"),
    ("python", "python"),
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cmd: Vec<String>,
    pub cpu_usage: f32,
    pub memory: u64,  // bytes
    pub memory_percent: f32,
    pub status: String,
    pub service_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessMetrics {
    pub processes: Vec<ProcessInfo>,
    pub total_processes: usize,
    pub running_processes: usize,
    pub services: HashMap<String, Vec<ProcessInfo>>,
}

pub struct ProcessCollector {
    system: System,
    filter_pattern: Option<String>,
}

impl ProcessCollector {
    pub fn new() -> Self {
        let mut system = System::new_with_specifics(
            RefreshKind::new().with_processes(ProcessRefreshKind::everything()),
        );

        system.refresh_processes();

        Self {
            system,
            filter_pattern: None,
        }
    }

    pub fn with_filter(mut self, pattern: String) -> Self {
        self.filter_pattern = Some(pattern);
        self
    }

    fn detect_service_type(name: &str, cmd: &[String]) -> Option<String> {
        let full_cmd = cmd.join(" ");

        for (service_name, pattern) in SERVICE_PATTERNS {
            if name.contains(pattern) || full_cmd.contains(pattern) {
                return Some(service_name.to_string());
            }
        }

        None
    }

    fn format_process_status(status: &str) -> String {
        match status {
            "Run" => "running".to_string(),
            "Sleep" => "sleeping".to_string(),
            "Stop" => "stopped".to_string(),
            "Zombie" => "zombie".to_string(),
            _ => status.to_lowercase(),
        }
    }
}

impl Default for ProcessCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricCollector for ProcessCollector {
    type Metrics = ProcessMetrics;

    fn name(&self) -> &str {
        "process"
    }

    fn collect(&mut self) -> crate::Result<Self::Metrics> {
        self.system.refresh_processes();

        let total_memory = self.system.total_memory();
        let mut processes = Vec::new();
        let mut services: HashMap<String, Vec<ProcessInfo>> = HashMap::new();
        let mut running_count = 0;

        for (pid, process) in self.system.processes() {
            let name = process.name().to_string();

            // Apply filter if set
            if let Some(ref pattern) = self.filter_pattern {
                if !name.contains(pattern) {
                    continue;
                }
            }

            let cmd: Vec<String> = process.cmd().iter().map(|s| s.to_string()).collect();
            let service_type = Self::detect_service_type(&name, &cmd);

            let memory = process.memory();
            let memory_percent = if total_memory > 0 {
                (memory as f64 / total_memory as f64 * 100.0) as f32
            } else {
                0.0
            };

            let status_str = format!("{:?}", process.status());
            let status = Self::format_process_status(&status_str);

            if status == "running" {
                running_count += 1;
            }

            let process_info = ProcessInfo {
                pid: pid.as_u32(),
                name: name.clone(),
                cmd,
                cpu_usage: process.cpu_usage(),
                memory,
                memory_percent,
                status,
                service_type: service_type.clone(),
            };

            // Group by service type
            if let Some(svc_type) = &service_type {
                services
                    .entry(svc_type.clone())
                    .or_default()
                    .push(process_info.clone());
            }

            processes.push(process_info);
        }

        // Sort processes by CPU usage (descending)
        processes.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap());

        Ok(ProcessMetrics {
            total_processes: processes.len(),
            running_processes: running_count,
            processes,
            services,
        })
    }

    fn interval(&self) -> Duration {
        Duration::from_secs(2)
    }

    fn refresh(&mut self) -> crate::Result<()> {
        self.system.refresh_processes();
        Ok(())
    }
}

impl ProcessMetrics {
    /// Get top N processes by CPU usage
    pub fn top_cpu(&self, n: usize) -> &[ProcessInfo] {
        let end = n.min(self.processes.len());
        &self.processes[..end]
    }

    /// Get top N processes by memory usage
    pub fn top_memory(&self, n: usize) -> Vec<&ProcessInfo> {
        let mut sorted: Vec<&ProcessInfo> = self.processes.iter().collect();
        sorted.sort_by(|a, b| b.memory.cmp(&a.memory));
        sorted.into_iter().take(n).collect()
    }

    /// Get all processes of a specific service type
    pub fn get_service_processes(&self, service_name: &str) -> Option<&Vec<ProcessInfo>> {
        self.services.get(service_name)
    }

    /// Get aggregate stats for a service
    pub fn service_stats(&self, service_name: &str) -> Option<ServiceStats> {
        self.services.get(service_name).map(|procs| {
            let total_cpu: f32 = procs.iter().map(|p| p.cpu_usage).sum();
            let total_memory: u64 = procs.iter().map(|p| p.memory).sum();
            let count = procs.len();

            ServiceStats {
                service_name: service_name.to_string(),
                process_count: count,
                total_cpu_usage: total_cpu,
                total_memory,
                avg_cpu_usage: if count > 0 {
                    total_cpu / count as f32
                } else {
                    0.0
                },
                avg_memory: if count > 0 {
                    total_memory / count as u64
                } else {
                    0
                },
            }
        })
    }

    /// List all detected services
    pub fn list_services(&self) -> Vec<String> {
        let mut services: Vec<String> = self.services.keys().cloned().collect();
        services.sort();
        services
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStats {
    pub service_name: String,
    pub process_count: usize,
    pub total_cpu_usage: f32,
    pub total_memory: u64,
    pub avg_cpu_usage: f32,
    pub avg_memory: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_collector_creation() {
        let collector = ProcessCollector::new();
        assert_eq!(collector.name(), "process");
    }

    #[test]
    fn test_process_collection() {
        let mut collector = ProcessCollector::new();
        let metrics = collector.collect().expect("Failed to collect process metrics");

        assert!(metrics.total_processes > 0, "Should have at least one process");
        assert!(
            metrics.running_processes > 0,
            "Should have at least one running process"
        );
    }

    #[test]
    fn test_service_detection() {
        let service = ProcessCollector::detect_service_type("mysqld", &["mysqld".to_string()]);
        assert_eq!(service, Some("mysql".to_string()));

        let service = ProcessCollector::detect_service_type("redis-server", &[]);
        assert_eq!(service, Some("redis".to_string()));

        let service = ProcessCollector::detect_service_type("unknown", &[]);
        assert_eq!(service, None);
    }

    #[test]
    fn test_process_interval() {
        let collector = ProcessCollector::new();
        assert_eq!(collector.interval(), Duration::from_secs(2));
    }

    #[test]
    fn test_top_processes() {
        let mut collector = ProcessCollector::new();
        let metrics = collector.collect().unwrap();

        let top_5 = metrics.top_cpu(5);
        assert!(top_5.len() <= 5);
        assert!(top_5.len() <= metrics.total_processes);

        // Verify they're sorted by CPU
        if top_5.len() > 1 {
            for i in 0..top_5.len() - 1 {
                assert!(top_5[i].cpu_usage >= top_5[i + 1].cpu_usage);
            }
        }
    }

    #[test]
    fn test_list_services() {
        let mut collector = ProcessCollector::new();
        let metrics = collector.collect().unwrap();

        let services = metrics.list_services();
        // Should be sorted
        let mut sorted = services.clone();
        sorted.sort();
        assert_eq!(services, sorted);
    }
}
