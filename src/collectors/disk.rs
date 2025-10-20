use super::MetricCollector;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use sysinfo::{DiskKind, Disks};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub file_system: String,
    pub disk_kind: String,
    pub total_space: u64,
    pub available_space: u64,
    pub used_space: u64,
    pub usage_percent: f64,
    pub is_removable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskMetrics {
    pub disks: Vec<DiskInfo>,
    pub total_space: u64,
    pub total_available: u64,
    pub total_used: u64,
    pub overall_usage_percent: f64,
}

pub struct DiskCollector {
    disks: Disks,
}

impl DiskCollector {
    pub fn new() -> Self {
        let mut disks = Disks::new_with_refreshed_list();
        disks.refresh();

        Self { disks }
    }

    fn format_disk_kind(kind: DiskKind) -> String {
        match kind {
            DiskKind::HDD => "HDD".to_string(),
            DiskKind::SSD => "SSD".to_string(),
            _ => "Unknown".to_string(),
        }
    }

    fn calculate_usage_percent(used: u64, total: u64) -> f64 {
        if total == 0 {
            0.0
        } else {
            (used as f64 / total as f64) * 100.0
        }
    }
}

impl Default for DiskCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricCollector for DiskCollector {
    type Metrics = DiskMetrics;

    fn name(&self) -> &str {
        "disk"
    }

    fn collect(&mut self) -> crate::Result<Self::Metrics> {
        self.disks.refresh();

        let mut disk_infos = Vec::new();
        let mut total_space = 0u64;
        let mut total_available = 0u64;
        let mut total_used = 0u64;

        for disk in &self.disks {
            let total = disk.total_space();
            let available = disk.available_space();
            let used = total.saturating_sub(available);

            let info = DiskInfo {
                name: disk.name().to_string_lossy().to_string(),
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                file_system: disk.file_system().to_string_lossy().to_string(),
                disk_kind: Self::format_disk_kind(disk.kind()),
                total_space: total,
                available_space: available,
                used_space: used,
                usage_percent: Self::calculate_usage_percent(used, total),
                is_removable: disk.is_removable(),
            };

            total_space += total;
            total_available += available;
            total_used += used;

            disk_infos.push(info);
        }

        let overall_usage = Self::calculate_usage_percent(total_used, total_space);

        Ok(DiskMetrics {
            disks: disk_infos,
            total_space,
            total_available,
            total_used,
            overall_usage_percent: overall_usage,
        })
    }

    fn interval(&self) -> Duration {
        Duration::from_secs(5) // Disk stats change less frequently
    }

    fn refresh(&mut self) -> crate::Result<()> {
        self.disks.refresh();
        Ok(())
    }
}

impl DiskMetrics {
    /// Format bytes to human-readable format
    pub fn format_bytes(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        format!("{:.2} {}", size, UNITS[unit_index])
    }

    /// Get disks with usage above threshold
    pub fn disks_above_threshold(&self, threshold_percent: f64) -> Vec<&DiskInfo> {
        self.disks
            .iter()
            .filter(|d| d.usage_percent >= threshold_percent)
            .collect()
    }

    /// Get the most full disk
    pub fn most_full_disk(&self) -> Option<&DiskInfo> {
        self.disks
            .iter()
            .max_by(|a, b| {
                a.usage_percent
                    .partial_cmp(&b.usage_percent)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// Get total capacity in human-readable format
    pub fn total_capacity_formatted(&self) -> String {
        Self::format_bytes(self.total_space)
    }

    /// Get total used in human-readable format
    pub fn total_used_formatted(&self) -> String {
        Self::format_bytes(self.total_used)
    }

    /// Get total available in human-readable format
    pub fn total_available_formatted(&self) -> String {
        Self::format_bytes(self.total_available)
    }
}

impl DiskInfo {
    /// Get formatted total space
    pub fn total_formatted(&self) -> String {
        DiskMetrics::format_bytes(self.total_space)
    }

    /// Get formatted used space
    pub fn used_formatted(&self) -> String {
        DiskMetrics::format_bytes(self.used_space)
    }

    /// Get formatted available space
    pub fn available_formatted(&self) -> String {
        DiskMetrics::format_bytes(self.available_space)
    }

    /// Check if disk usage is critical (>90%)
    pub fn is_critical(&self) -> bool {
        self.usage_percent >= 90.0
    }

    /// Check if disk usage is warning level (>80%)
    pub fn is_warning(&self) -> bool {
        self.usage_percent >= 80.0 && self.usage_percent < 90.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disk_collector_creation() {
        let collector = DiskCollector::new();
        assert_eq!(collector.name(), "disk");
    }

    #[test]
    fn test_disk_collection() {
        let mut collector = DiskCollector::new();
        let metrics = collector.collect().expect("Failed to collect disk metrics");

        // Should have at least one disk
        assert!(!metrics.disks.is_empty(), "Should have at least one disk");

        // Verify totals make sense
        assert!(metrics.total_space > 0, "Total space should be greater than 0");
        assert!(
            metrics.total_used <= metrics.total_space,
            "Used space should not exceed total"
        );
    }

    #[test]
    fn test_disk_interval() {
        let collector = DiskCollector::new();
        assert_eq!(collector.interval(), Duration::from_secs(5));
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(DiskMetrics::format_bytes(512), "512.00 B");
        assert_eq!(DiskMetrics::format_bytes(1024), "1.00 KB");
        assert_eq!(DiskMetrics::format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(DiskMetrics::format_bytes(1024 * 1024 * 1024), "1.00 GB");
        assert_eq!(
            DiskMetrics::format_bytes(1536 * 1024 * 1024 * 1024),
            "1.50 TB"
        );
    }

    #[test]
    fn test_usage_calculations() {
        let mut collector = DiskCollector::new();
        let metrics = collector.collect().unwrap();

        for disk in &metrics.disks {
            assert!(
                disk.usage_percent >= 0.0 && disk.usage_percent <= 100.0,
                "Usage percent should be between 0 and 100"
            );
            assert_eq!(
                disk.total_space,
                disk.used_space + disk.available_space,
                "Total should equal used + available"
            );
        }
    }

    #[test]
    fn test_most_full_disk() {
        let mut collector = DiskCollector::new();
        let metrics = collector.collect().unwrap();

        if !metrics.disks.is_empty() {
            let most_full = metrics.most_full_disk();
            assert!(most_full.is_some());

            let most_full = most_full.unwrap();
            // Should be the highest usage
            for disk in &metrics.disks {
                assert!(disk.usage_percent <= most_full.usage_percent);
            }
        }
    }

    #[test]
    fn test_threshold_filtering() {
        let mut collector = DiskCollector::new();
        let metrics = collector.collect().unwrap();

        let high_usage = metrics.disks_above_threshold(80.0);
        for disk in high_usage {
            assert!(disk.usage_percent >= 80.0);
        }
    }

    #[test]
    fn test_disk_status_checks() {
        let disk = DiskInfo {
            name: "test".to_string(),
            mount_point: "/".to_string(),
            file_system: "ext4".to_string(),
            disk_kind: "SSD".to_string(),
            total_space: 100 * 1024 * 1024 * 1024,
            available_space: 5 * 1024 * 1024 * 1024,
            used_space: 95 * 1024 * 1024 * 1024,
            usage_percent: 95.0,
            is_removable: false,
        };

        assert!(disk.is_critical());
        assert!(!disk.is_warning());
    }
}
