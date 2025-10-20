use super::{CpuMetrics, DiskMetrics, MemoryMetrics, NetworkMetrics, ProcessMetrics};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Aggregated system snapshot combining all metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub timestamp: DateTime<Utc>,
    pub cpu: Option<CpuMetrics>,
    pub memory: Option<MemoryMetrics>,
    pub processes: Option<ProcessMetrics>,
    pub network: Option<NetworkMetrics>,
    pub disk: Option<DiskMetrics>,
}

impl Snapshot {
    pub fn new() -> Self {
        Self {
            timestamp: Utc::now(),
            cpu: None,
            memory: None,
            processes: None,
            network: None,
            disk: None,
        }
    }

    pub fn with_cpu(mut self, metrics: CpuMetrics) -> Self {
        self.cpu = Some(metrics);
        self
    }

    pub fn with_memory(mut self, metrics: MemoryMetrics) -> Self {
        self.memory = Some(metrics);
        self
    }

    pub fn with_processes(mut self, metrics: ProcessMetrics) -> Self {
        self.processes = Some(metrics);
        self
    }

    pub fn with_network(mut self, metrics: NetworkMetrics) -> Self {
        self.network = Some(metrics);
        self
    }

    pub fn with_disk(mut self, metrics: DiskMetrics) -> Self {
        self.disk = Some(metrics);
        self
    }
}

impl Default for Snapshot {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_creation() {
        let snapshot = Snapshot::new();
        assert!(snapshot.cpu.is_none());
        assert!(snapshot.memory.is_none());
    }

    #[test]
    fn test_snapshot_builder() {
        let cpu_metrics = CpuMetrics {
            total_usage: 50.0,
            per_core: vec![45.0, 55.0],
            load_average: (1.0, 1.5, 2.0),
            core_count: 2,
        };

        let memory_metrics = MemoryMetrics {
            total: 16 * 1024 * 1024 * 1024,
            available: 8 * 1024 * 1024 * 1024,
            used: 8 * 1024 * 1024 * 1024,
            free: 8 * 1024 * 1024 * 1024,
            swap_total: 0,
            swap_used: 0,
            swap_free: 0,
            usage_percent: 50.0,
            swap_usage_percent: 0.0,
        };

        let snapshot = Snapshot::new()
            .with_cpu(cpu_metrics.clone())
            .with_memory(memory_metrics.clone());

        assert!(snapshot.cpu.is_some());
        assert!(snapshot.memory.is_some());

        let cpu = snapshot.cpu.unwrap();
        assert_eq!(cpu.total_usage, 50.0);
        assert_eq!(cpu.core_count, 2);
    }
}
