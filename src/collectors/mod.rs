pub mod cpu;
pub mod disk;
pub mod memory;
pub mod network;
pub mod process;
pub mod services;
pub mod snapshot;

use std::time::Duration;

pub use cpu::{CpuCollector, CpuMetrics};
pub use disk::{DiskCollector, DiskInfo, DiskMetrics};
pub use memory::{MemoryCollector, MemoryMetrics};
pub use network::{NetworkCollector, NetworkInterfaceMetrics, NetworkMetrics};
pub use process::{ProcessCollector, ProcessInfo, ProcessMetrics, ServiceStats};
pub use snapshot::Snapshot;

/// Trait for metric collectors
pub trait MetricCollector: Send + Sync {
    type Metrics: Send + Sync;

    /// Name of the collector
    fn name(&self) -> &str;

    /// Collect current metrics
    fn collect(&mut self) -> crate::Result<Self::Metrics>;

    /// Recommended collection interval
    fn interval(&self) -> Duration {
        Duration::from_secs(1)
    }

    /// Refresh/update internal state if needed
    fn refresh(&mut self) -> crate::Result<()> {
        Ok(())
    }
}
