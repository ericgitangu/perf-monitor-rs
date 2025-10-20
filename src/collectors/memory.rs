use super::MetricCollector;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use sysinfo::{MemoryRefreshKind, RefreshKind, System};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetrics {
    pub total: u64,
    pub available: u64,
    pub used: u64,
    pub free: u64,
    pub swap_total: u64,
    pub swap_used: u64,
    pub swap_free: u64,
    pub usage_percent: f64,
    pub swap_usage_percent: f64,
}

pub struct MemoryCollector {
    system: System,
}

impl MemoryCollector {
    pub fn new() -> Self {
        let mut system = System::new_with_specifics(
            RefreshKind::new().with_memory(MemoryRefreshKind::everything()),
        );

        system.refresh_memory();

        Self { system }
    }

    fn calculate_usage_percent(used: u64, total: u64) -> f64 {
        if total == 0 {
            return 0.0;
        }
        (used as f64 / total as f64) * 100.0
    }
}

impl Default for MemoryCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricCollector for MemoryCollector {
    type Metrics = MemoryMetrics;

    fn name(&self) -> &str {
        "memory"
    }

    fn collect(&mut self) -> crate::Result<Self::Metrics> {
        self.system.refresh_memory();

        let total = self.system.total_memory();
        let available = self.system.available_memory();
        let used = self.system.used_memory();
        let free = self.system.free_memory();

        let swap_total = self.system.total_swap();
        let swap_used = self.system.used_swap();
        let swap_free = self.system.free_swap();

        let usage_percent = Self::calculate_usage_percent(used, total);
        let swap_usage_percent = Self::calculate_usage_percent(swap_used, swap_total);

        Ok(MemoryMetrics {
            total,
            available,
            used,
            free,
            swap_total,
            swap_used,
            swap_free,
            usage_percent,
            swap_usage_percent,
        })
    }

    fn interval(&self) -> Duration {
        Duration::from_secs(1)
    }

    fn refresh(&mut self) -> crate::Result<()> {
        self.system.refresh_memory();
        Ok(())
    }
}

impl MemoryMetrics {
    pub fn format_bytes(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        format!("{:.2} {}", size, UNITS[unit_index])
    }

    pub fn total_formatted(&self) -> String {
        Self::format_bytes(self.total)
    }

    pub fn used_formatted(&self) -> String {
        Self::format_bytes(self.used)
    }

    pub fn available_formatted(&self) -> String {
        Self::format_bytes(self.available)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_collector_creation() {
        let collector = MemoryCollector::new();
        assert_eq!(collector.name(), "memory");
    }

    #[test]
    fn test_memory_metrics_collection() {
        let mut collector = MemoryCollector::new();
        let metrics = collector.collect().expect("Failed to collect memory metrics");

        assert!(metrics.total > 0, "Total memory should be greater than 0");
        assert!(
            metrics.used <= metrics.total,
            "Used memory should not exceed total"
        );
        assert!(
            metrics.available <= metrics.total,
            "Available memory should not exceed total"
        );
        assert!(
            metrics.usage_percent >= 0.0 && metrics.usage_percent <= 100.0,
            "Usage percent should be between 0 and 100"
        );

        if metrics.swap_total > 0 {
            assert!(
                metrics.swap_used <= metrics.swap_total,
                "Used swap should not exceed total swap"
            );
            assert!(
                metrics.swap_usage_percent >= 0.0 && metrics.swap_usage_percent <= 100.0,
                "Swap usage percent should be between 0 and 100"
            );
        }
    }

    #[test]
    fn test_memory_interval() {
        let collector = MemoryCollector::new();
        assert_eq!(collector.interval(), Duration::from_secs(1));
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(MemoryMetrics::format_bytes(512), "512.00 B");
        assert_eq!(MemoryMetrics::format_bytes(1024), "1.00 KB");
        assert_eq!(MemoryMetrics::format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(MemoryMetrics::format_bytes(1024 * 1024 * 1024), "1.00 GB");
        assert_eq!(
            MemoryMetrics::format_bytes(1536 * 1024 * 1024),
            "1.50 GB"
        );
    }

    #[test]
    fn test_calculate_usage_percent() {
        assert_eq!(MemoryCollector::calculate_usage_percent(0, 100), 0.0);
        assert_eq!(MemoryCollector::calculate_usage_percent(50, 100), 50.0);
        assert_eq!(MemoryCollector::calculate_usage_percent(100, 100), 100.0);
        assert_eq!(MemoryCollector::calculate_usage_percent(0, 0), 0.0);
    }
}
