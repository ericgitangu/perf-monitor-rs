use super::MetricCollector;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use sysinfo::{NetworkData, Networks};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterfaceMetrics {
    pub name: String,
    pub received_bytes: u64,
    pub transmitted_bytes: u64,
    pub received_packets: u64,
    pub transmitted_packets: u64,
    pub received_errors: u64,
    pub transmitted_errors: u64,
    pub received_rate: f64,      // bytes/sec
    pub transmitted_rate: f64,   // bytes/sec
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    pub interfaces: HashMap<String, NetworkInterfaceMetrics>,
    pub total_received_bytes: u64,
    pub total_transmitted_bytes: u64,
    pub total_received_rate: f64,
    pub total_transmitted_rate: f64,
}

pub struct NetworkCollector {
    networks: Networks,
    previous_metrics: Option<NetworkMetrics>,
    last_update: std::time::Instant,
}

impl NetworkCollector {
    pub fn new() -> Self {
        let mut networks = Networks::new_with_refreshed_list();
        networks.refresh();

        Self {
            networks,
            previous_metrics: None,
            last_update: std::time::Instant::now(),
        }
    }

    fn calculate_rate(&self, current: u64, previous: u64, elapsed_secs: f64) -> f64 {
        if elapsed_secs > 0.0 {
            let diff = current.saturating_sub(previous);
            diff as f64 / elapsed_secs
        } else {
            0.0
        }
    }

    fn collect_interface_metrics(&self, name: &str, data: &NetworkData) -> NetworkInterfaceMetrics {
        NetworkInterfaceMetrics {
            name: name.to_string(),
            received_bytes: data.total_received(),
            transmitted_bytes: data.total_transmitted(),
            received_packets: data.total_packets_received(),
            transmitted_packets: data.total_packets_transmitted(),
            received_errors: data.total_errors_on_received(),
            transmitted_errors: data.total_errors_on_transmitted(),
            received_rate: 0.0,      // Will be calculated
            transmitted_rate: 0.0,   // Will be calculated
        }
    }
}

impl Default for NetworkCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricCollector for NetworkCollector {
    type Metrics = NetworkMetrics;

    fn name(&self) -> &str {
        "network"
    }

    fn collect(&mut self) -> crate::Result<Self::Metrics> {
        self.networks.refresh();

        let now = std::time::Instant::now();
        let elapsed = now.duration_since(self.last_update).as_secs_f64();

        let mut interfaces = HashMap::new();
        let mut total_received = 0u64;
        let mut total_transmitted = 0u64;

        for (interface_name, data) in &self.networks {
            let mut metrics = self.collect_interface_metrics(interface_name, data);

            // Calculate rates if we have previous data
            if let Some(ref prev) = self.previous_metrics {
                if let Some(prev_interface) = prev.interfaces.get(interface_name) {
                    metrics.received_rate = self.calculate_rate(
                        metrics.received_bytes,
                        prev_interface.received_bytes,
                        elapsed,
                    );
                    metrics.transmitted_rate = self.calculate_rate(
                        metrics.transmitted_bytes,
                        prev_interface.transmitted_bytes,
                        elapsed,
                    );
                }
            }

            total_received += metrics.received_bytes;
            total_transmitted += metrics.transmitted_bytes;

            interfaces.insert(interface_name.to_string(), metrics);
        }

        // Calculate total rates
        let (total_rx_rate, total_tx_rate) = if let Some(ref prev) = self.previous_metrics {
            (
                self.calculate_rate(total_received, prev.total_received_bytes, elapsed),
                self.calculate_rate(total_transmitted, prev.total_transmitted_bytes, elapsed),
            )
        } else {
            (0.0, 0.0)
        };

        let current_metrics = NetworkMetrics {
            interfaces,
            total_received_bytes: total_received,
            total_transmitted_bytes: total_transmitted,
            total_received_rate: total_rx_rate,
            total_transmitted_rate: total_tx_rate,
        };

        self.previous_metrics = Some(current_metrics.clone());
        self.last_update = now;

        Ok(current_metrics)
    }

    fn interval(&self) -> Duration {
        Duration::from_secs(1)
    }

    fn refresh(&mut self) -> crate::Result<()> {
        self.networks.refresh();
        Ok(())
    }
}

impl NetworkMetrics {
    /// Format bytes per second to human-readable format
    pub fn format_rate(bytes_per_sec: f64) -> String {
        const UNITS: &[&str] = &["B/s", "KB/s", "MB/s", "GB/s"];
        let mut rate = bytes_per_sec;
        let mut unit_index = 0;

        while rate >= 1024.0 && unit_index < UNITS.len() - 1 {
            rate /= 1024.0;
            unit_index += 1;
        }

        format!("{:.2} {}", rate, UNITS[unit_index])
    }

    /// Get the most active interface by total traffic
    pub fn most_active_interface(&self) -> Option<&NetworkInterfaceMetrics> {
        self.interfaces
            .values()
            .max_by_key(|i| i.received_bytes + i.transmitted_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_collector_creation() {
        let collector = NetworkCollector::new();
        assert_eq!(collector.name(), "network");
    }

    #[test]
    fn test_network_collection() {
        let mut collector = NetworkCollector::new();
        let metrics = collector.collect().expect("Failed to collect network metrics");

        // Should have at least loopback interface
        assert!(!metrics.interfaces.is_empty(), "Should have at least one network interface");
    }

    #[test]
    fn test_network_interval() {
        let collector = NetworkCollector::new();
        assert_eq!(collector.interval(), Duration::from_secs(1));
    }

    #[test]
    fn test_format_rate() {
        assert_eq!(NetworkMetrics::format_rate(512.0), "512.00 B/s");
        assert_eq!(NetworkMetrics::format_rate(1024.0), "1.00 KB/s");
        assert_eq!(NetworkMetrics::format_rate(1024.0 * 1024.0), "1.00 MB/s");
        assert_eq!(NetworkMetrics::format_rate(1.5 * 1024.0 * 1024.0), "1.50 MB/s");
    }

    #[test]
    fn test_rate_calculation() {
        let mut collector = NetworkCollector::new();

        // First collection (no rates yet)
        let first = collector.collect().unwrap();
        assert_eq!(first.total_received_rate, 0.0);
        assert_eq!(first.total_transmitted_rate, 0.0);

        // Wait a bit and collect again
        std::thread::sleep(Duration::from_millis(100));
        collector.refresh().unwrap();

        // Second collection should potentially have rates
        let _second = collector.collect().unwrap();
        // Rates might be 0 if no traffic, but collection should succeed
    }

    #[test]
    fn test_most_active_interface() {
        let mut collector = NetworkCollector::new();
        let metrics = collector.collect().unwrap();

        if !metrics.interfaces.is_empty() {
            let active = metrics.most_active_interface();
            assert!(active.is_some());
        }
    }
}
