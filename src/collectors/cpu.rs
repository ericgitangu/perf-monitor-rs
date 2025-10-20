use super::MetricCollector;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use sysinfo::{CpuRefreshKind, RefreshKind, System};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuMetrics {
    pub total_usage: f32,
    pub per_core: Vec<f32>,
    pub load_average: (f64, f64, f64),
    pub core_count: usize,
}

pub struct CpuCollector {
    system: System,
}

impl CpuCollector {
    pub fn new() -> Self {
        let mut system = System::new_with_specifics(
            RefreshKind::new().with_cpu(CpuRefreshKind::everything()),
        );

        // Initial refresh to get CPU info
        system.refresh_cpu();

        // Sleep briefly to get accurate first reading
        std::thread::sleep(Duration::from_millis(200));
        system.refresh_cpu();

        Self { system }
    }

    fn calculate_total_usage(&self) -> f32 {
        let cpus = self.system.cpus();
        if cpus.is_empty() {
            return 0.0;
        }

        let sum: f32 = cpus.iter().map(|cpu| cpu.cpu_usage()).sum();
        sum / cpus.len() as f32
    }
}

impl Default for CpuCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricCollector for CpuCollector {
    type Metrics = CpuMetrics;

    fn name(&self) -> &str {
        "cpu"
    }

    fn collect(&mut self) -> crate::Result<Self::Metrics> {
        self.system.refresh_cpu();

        let cpus = self.system.cpus();
        let per_core: Vec<f32> = cpus.iter().map(|cpu| cpu.cpu_usage()).collect();
        let total_usage = self.calculate_total_usage();

        let load_avg = System::load_average();
        let load_average = (load_avg.one, load_avg.five, load_avg.fifteen);

        let core_count = cpus.len();

        Ok(CpuMetrics {
            total_usage,
            per_core,
            load_average,
            core_count,
        })
    }

    fn interval(&self) -> Duration {
        Duration::from_secs(1)
    }

    fn refresh(&mut self) -> crate::Result<()> {
        self.system.refresh_cpu();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_collector_creation() {
        let collector = CpuCollector::new();
        assert_eq!(collector.name(), "cpu");
    }

    #[test]
    fn test_cpu_metrics_collection() {
        let mut collector = CpuCollector::new();
        let metrics = collector.collect().expect("Failed to collect CPU metrics");

        assert!(metrics.total_usage >= 0.0 && metrics.total_usage <= 100.0);
        assert!(!metrics.per_core.is_empty());
        assert_eq!(metrics.per_core.len(), metrics.core_count);

        for usage in &metrics.per_core {
            assert!(*usage >= 0.0 && *usage <= 100.0);
        }

        let (one, five, fifteen) = metrics.load_average;
        assert!(one >= 0.0);
        assert!(five >= 0.0);
        assert!(fifteen >= 0.0);
    }

    #[test]
    fn test_cpu_interval() {
        let collector = CpuCollector::new();
        assert_eq!(collector.interval(), Duration::from_secs(1));
    }
}
