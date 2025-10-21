use crate::collectors::*;
use std::fmt::Write;

/// Prometheus metrics exporter
pub struct PrometheusExporter;

impl PrometheusExporter {
    pub fn new() -> Self {
        Self
    }

    /// Export all metrics in Prometheus/OpenMetrics format
    pub fn export_all(
        cpu: Option<&CpuMetrics>,
        memory: Option<&MemoryMetrics>,
        network: Option<&NetworkMetrics>,
        disk: Option<&DiskMetrics>,
        processes: Option<&ProcessMetrics>,
    ) -> String {
        let mut output = String::new();

        // Header
        writeln!(
            &mut output,
            "# Monitor-RS Metrics Export - OpenMetrics Format"
        )
        .ok();
        writeln!(&mut output).ok();

        if let Some(cpu) = cpu {
            Self::export_cpu(&mut output, cpu);
        }

        if let Some(memory) = memory {
            Self::export_memory(&mut output, memory);
        }

        if let Some(network) = network {
            Self::export_network(&mut output, network);
        }

        if let Some(disk) = disk {
            Self::export_disk(&mut output, disk);
        }

        if let Some(processes) = processes {
            Self::export_processes(&mut output, processes);
        }

        output
    }

    fn export_cpu(output: &mut String, cpu: &CpuMetrics) {
        writeln!(output, "# HELP cpu_usage_percent CPU usage percentage").ok();
        writeln!(output, "# TYPE cpu_usage_percent gauge").ok();
        writeln!(output, "cpu_usage_percent {}", cpu.total_usage).ok();
        writeln!(output).ok();

        writeln!(
            output,
            "# HELP cpu_cores_total Total number of CPU cores"
        )
        .ok();
        writeln!(output, "# TYPE cpu_cores_total gauge").ok();
        writeln!(output, "cpu_cores_total {}", cpu.core_count).ok();
        writeln!(output).ok();

        writeln!(
            output,
            "# HELP cpu_load_average System load average"
        )
        .ok();
        writeln!(output, "# TYPE cpu_load_average gauge").ok();
        writeln!(
            output,
            "cpu_load_average{{period=\"1m\"}} {}",
            cpu.load_average.0
        )
        .ok();
        writeln!(
            output,
            "cpu_load_average{{period=\"5m\"}} {}",
            cpu.load_average.1
        )
        .ok();
        writeln!(
            output,
            "cpu_load_average{{period=\"15m\"}} {}",
            cpu.load_average.2
        )
        .ok();
        writeln!(output).ok();

        writeln!(
            output,
            "# HELP cpu_core_usage_percent Per-core CPU usage percentage"
        )
        .ok();
        writeln!(output, "# TYPE cpu_core_usage_percent gauge").ok();
        for (i, usage) in cpu.per_core.iter().enumerate() {
            writeln!(output, "cpu_core_usage_percent{{core=\"{}\"}} {}", i, usage).ok();
        }
        writeln!(output).ok();
    }

    fn export_memory(output: &mut String, memory: &MemoryMetrics) {
        writeln!(output, "# HELP memory_total_bytes Total memory in bytes").ok();
        writeln!(output, "# TYPE memory_total_bytes gauge").ok();
        writeln!(output, "memory_total_bytes {}", memory.total).ok();
        writeln!(output).ok();

        writeln!(output, "# HELP memory_used_bytes Used memory in bytes").ok();
        writeln!(output, "# TYPE memory_used_bytes gauge").ok();
        writeln!(output, "memory_used_bytes {}", memory.used).ok();
        writeln!(output).ok();

        writeln!(
            output,
            "# HELP memory_available_bytes Available memory in bytes"
        )
        .ok();
        writeln!(output, "# TYPE memory_available_bytes gauge").ok();
        writeln!(output, "memory_available_bytes {}", memory.available).ok();
        writeln!(output).ok();

        writeln!(
            output,
            "# HELP memory_usage_percent Memory usage percentage"
        )
        .ok();
        writeln!(output, "# TYPE memory_usage_percent gauge").ok();
        writeln!(output, "memory_usage_percent {}", memory.usage_percent).ok();
        writeln!(output).ok();

        writeln!(output, "# HELP swap_total_bytes Total swap in bytes").ok();
        writeln!(output, "# TYPE swap_total_bytes gauge").ok();
        writeln!(output, "swap_total_bytes {}", memory.swap_total).ok();
        writeln!(output).ok();

        writeln!(output, "# HELP swap_used_bytes Used swap in bytes").ok();
        writeln!(output, "# TYPE swap_used_bytes gauge").ok();
        writeln!(output, "swap_used_bytes {}", memory.swap_used).ok();
        writeln!(output).ok();

        writeln!(
            output,
            "# HELP swap_usage_percent Swap usage percentage"
        )
        .ok();
        writeln!(output, "# TYPE swap_usage_percent gauge").ok();
        writeln!(output, "swap_usage_percent {}", memory.swap_usage_percent).ok();
        writeln!(output).ok();

        writeln!(output, "# HELP memory_free_bytes Free memory in bytes").ok();
        writeln!(output, "# TYPE memory_free_bytes gauge").ok();
        writeln!(output, "memory_free_bytes {}", memory.free).ok();
        writeln!(output).ok();

        writeln!(output, "# HELP swap_free_bytes Free swap in bytes").ok();
        writeln!(output, "# TYPE swap_free_bytes gauge").ok();
        writeln!(output, "swap_free_bytes {}", memory.swap_free).ok();
        writeln!(output).ok();
    }

    fn export_network(output: &mut String, network: &NetworkMetrics) {
        writeln!(
            output,
            "# HELP network_received_bytes_total Total bytes received"
        )
        .ok();
        writeln!(output, "# TYPE network_received_bytes_total counter").ok();
        writeln!(
            output,
            "network_received_bytes_total {}",
            network.total_received_bytes
        )
        .ok();
        writeln!(output).ok();

        writeln!(
            output,
            "# HELP network_transmitted_bytes_total Total bytes transmitted"
        )
        .ok();
        writeln!(output, "# TYPE network_transmitted_bytes_total counter").ok();
        writeln!(
            output,
            "network_transmitted_bytes_total {}",
            network.total_transmitted_bytes
        )
        .ok();
        writeln!(output).ok();

        writeln!(
            output,
            "# HELP network_received_rate_bytes_per_second Receive rate in bytes per second"
        )
        .ok();
        writeln!(output, "# TYPE network_received_rate_bytes_per_second gauge").ok();
        writeln!(
            output,
            "network_received_rate_bytes_per_second {}",
            network.total_received_rate
        )
        .ok();
        writeln!(output).ok();

        writeln!(
            output,
            "# HELP network_transmitted_rate_bytes_per_second Transmit rate in bytes per second"
        )
        .ok();
        writeln!(
            output,
            "# TYPE network_transmitted_rate_bytes_per_second gauge"
        )
        .ok();
        writeln!(
            output,
            "network_transmitted_rate_bytes_per_second {}",
            network.total_transmitted_rate
        )
        .ok();
        writeln!(output).ok();

        // Per-interface metrics
        writeln!(
            output,
            "# HELP network_interface_received_bytes_total Bytes received per interface"
        )
        .ok();
        writeln!(
            output,
            "# TYPE network_interface_received_bytes_total counter"
        )
        .ok();
        for (name, iface) in &network.interfaces {
            writeln!(
                output,
                "network_interface_received_bytes_total{{interface=\"{}\"}} {}",
                name, iface.received_bytes
            )
            .ok();
        }
        writeln!(output).ok();

        writeln!(
            output,
            "# HELP network_interface_transmitted_bytes_total Bytes transmitted per interface"
        )
        .ok();
        writeln!(
            output,
            "# TYPE network_interface_transmitted_bytes_total counter"
        )
        .ok();
        for (name, iface) in &network.interfaces {
            writeln!(
                output,
                "network_interface_transmitted_bytes_total{{interface=\"{}\"}} {}",
                name, iface.transmitted_bytes
            )
            .ok();
        }
        writeln!(output).ok();

        writeln!(
            output,
            "# HELP network_interface_received_rate_bytes_per_second Receive rate per interface"
        )
        .ok();
        writeln!(
            output,
            "# TYPE network_interface_received_rate_bytes_per_second gauge"
        )
        .ok();
        for (name, iface) in &network.interfaces {
            writeln!(
                output,
                "network_interface_received_rate_bytes_per_second{{interface=\"{}\"}} {}",
                name, iface.received_rate
            )
            .ok();
        }
        writeln!(output).ok();

        writeln!(
            output,
            "# HELP network_interface_transmitted_rate_bytes_per_second Transmit rate per interface"
        )
        .ok();
        writeln!(
            output,
            "# TYPE network_interface_transmitted_rate_bytes_per_second gauge"
        )
        .ok();
        for (name, iface) in &network.interfaces {
            writeln!(
                output,
                "network_interface_transmitted_rate_bytes_per_second{{interface=\"{}\"}} {}",
                name, iface.transmitted_rate
            )
            .ok();
        }
        writeln!(output).ok();
    }

    fn export_disk(output: &mut String, disk: &DiskMetrics) {
        writeln!(
            output,
            "# HELP disk_total_bytes Total disk capacity in bytes"
        )
        .ok();
        writeln!(output, "# TYPE disk_total_bytes gauge").ok();
        writeln!(output, "disk_total_bytes {}", disk.total_space).ok();
        writeln!(output).ok();

        writeln!(
            output,
            "# HELP disk_used_bytes Used disk space in bytes"
        )
        .ok();
        writeln!(output, "# TYPE disk_used_bytes gauge").ok();
        writeln!(output, "disk_used_bytes {}", disk.total_used).ok();
        writeln!(output).ok();

        writeln!(
            output,
            "# HELP disk_available_bytes Available disk space in bytes"
        )
        .ok();
        writeln!(output, "# TYPE disk_available_bytes gauge").ok();
        writeln!(output, "disk_available_bytes {}", disk.total_available).ok();
        writeln!(output).ok();

        writeln!(
            output,
            "# HELP disk_usage_percent Overall disk usage percentage"
        )
        .ok();
        writeln!(output, "# TYPE disk_usage_percent gauge").ok();
        writeln!(
            output,
            "disk_usage_percent {}",
            disk.overall_usage_percent
        )
        .ok();
        writeln!(output).ok();

        // Per-mount metrics
        writeln!(
            output,
            "# HELP disk_mount_total_bytes Total capacity per mount point"
        )
        .ok();
        writeln!(output, "# TYPE disk_mount_total_bytes gauge").ok();
        for disk_info in &disk.disks {
            writeln!(
                output,
                "disk_mount_total_bytes{{mount=\"{}\",type=\"{}\"}} {}",
                disk_info.mount_point, disk_info.disk_kind, disk_info.total_space
            )
            .ok();
        }
        writeln!(output).ok();

        writeln!(
            output,
            "# HELP disk_mount_used_bytes Used space per mount point"
        )
        .ok();
        writeln!(output, "# TYPE disk_mount_used_bytes gauge").ok();
        for disk_info in &disk.disks {
            writeln!(
                output,
                "disk_mount_used_bytes{{mount=\"{}\",type=\"{}\"}} {}",
                disk_info.mount_point, disk_info.disk_kind, disk_info.used_space
            )
            .ok();
        }
        writeln!(output).ok();

        writeln!(
            output,
            "# HELP disk_mount_available_bytes Available space per mount point"
        )
        .ok();
        writeln!(output, "# TYPE disk_mount_available_bytes gauge").ok();
        for disk_info in &disk.disks {
            writeln!(
                output,
                "disk_mount_available_bytes{{mount=\"{}\",type=\"{}\"}} {}",
                disk_info.mount_point, disk_info.disk_kind, disk_info.available_space
            )
            .ok();
        }
        writeln!(output).ok();

        writeln!(
            output,
            "# HELP disk_mount_usage_percent Usage percentage per mount point"
        )
        .ok();
        writeln!(output, "# TYPE disk_mount_usage_percent gauge").ok();
        for disk_info in &disk.disks {
            writeln!(
                output,
                "disk_mount_usage_percent{{mount=\"{}\",type=\"{}\"}} {}",
                disk_info.mount_point, disk_info.disk_kind, disk_info.usage_percent
            )
            .ok();
        }
        writeln!(output).ok();
    }

    fn export_processes(output: &mut String, processes: &ProcessMetrics) {
        writeln!(
            output,
            "# HELP processes_total Total number of processes"
        )
        .ok();
        writeln!(output, "# TYPE processes_total gauge").ok();
        writeln!(output, "processes_total {}", processes.total_processes).ok();
        writeln!(output).ok();

        writeln!(
            output,
            "# HELP processes_running Number of running processes"
        )
        .ok();
        writeln!(output, "# TYPE processes_running gauge").ok();
        writeln!(
            output,
            "processes_running {}",
            processes.running_processes
        )
        .ok();
        writeln!(output).ok();

        // Service-level metrics
        let services = processes.list_services();

        writeln!(
            output,
            "# HELP service_process_count Number of processes per service"
        )
        .ok();
        writeln!(output, "# TYPE service_process_count gauge").ok();
        for service_name in &services {
            if let Some(stats) = processes.service_stats(service_name) {
                writeln!(
                    output,
                    "service_process_count{{service=\"{}\"}} {}",
                    service_name, stats.process_count
                )
                .ok();
            }
        }
        writeln!(output).ok();

        writeln!(
            output,
            "# HELP service_cpu_usage_percent CPU usage per service"
        )
        .ok();
        writeln!(output, "# TYPE service_cpu_usage_percent gauge").ok();
        for service_name in &services {
            if let Some(stats) = processes.service_stats(service_name) {
                writeln!(
                    output,
                    "service_cpu_usage_percent{{service=\"{}\"}} {}",
                    service_name, stats.total_cpu_usage
                )
                .ok();
            }
        }
        writeln!(output).ok();

        writeln!(
            output,
            "# HELP service_memory_bytes Memory usage per service in bytes"
        )
        .ok();
        writeln!(output, "# TYPE service_memory_bytes gauge").ok();
        for service_name in &services {
            if let Some(stats) = processes.service_stats(service_name) {
                writeln!(
                    output,
                    "service_memory_bytes{{service=\"{}\"}} {}",
                    service_name, stats.total_memory
                )
                .ok();
            }
        }
        writeln!(output).ok();
    }
}

impl Default for PrometheusExporter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_cpu_metrics() {
        let cpu = CpuMetrics {
            total_usage: 45.5,
            core_count: 8,
            per_core: vec![40.0, 50.0, 45.0, 42.0, 48.0, 46.0, 44.0, 43.0],
            load_average: (1.5, 1.2, 0.9),
        };

        let output = PrometheusExporter::export_all(Some(&cpu), None, None, None, None);

        assert!(output.contains("cpu_usage_percent 45.5"));
        assert!(output.contains("cpu_cores_total 8"));
        assert!(output.contains("cpu_load_average{period=\"1m\"} 1.5"));
        assert!(output.contains("cpu_core_usage_percent{core=\"0\"} 40"));
    }

    #[test]
    fn test_export_memory_metrics() {
        let memory = MemoryMetrics {
            total: 16_000_000_000,
            used: 8_000_000_000,
            available: 8_000_000_000,
            usage_percent: 50.0,
            swap_total: 4_000_000_000,
            swap_used: 1_000_000_000,
            swap_usage_percent: 25.0,
            free: 2_000_000_000,
            swap_free: 3_000_000_000,
        };

        let output = PrometheusExporter::export_all(None, Some(&memory), None, None, None);

        assert!(output.contains("memory_total_bytes 16000000000"));
        assert!(output.contains("memory_used_bytes 8000000000"));
        assert!(output.contains("memory_usage_percent 50"));
        assert!(output.contains("swap_total_bytes 4000000000"));
    }

    #[test]
    fn test_export_empty_metrics() {
        let output = PrometheusExporter::export_all(None, None, None, None, None);

        assert!(output.contains("Monitor-RS Metrics Export"));
        assert!(!output.contains("cpu_usage_percent"));
        assert!(!output.contains("memory_total_bytes"));
    }
}
