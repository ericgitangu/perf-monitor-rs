use super::{ExportConfig, GeneralConfig, UiConfig};
use std::time::Duration;

pub fn default_update_interval() -> Duration {
    Duration::from_secs(1)
}

pub fn default_log_level() -> String {
    "info".to_string()
}

pub fn default_export_enabled() -> bool {
    true
}

pub fn default_export_port() -> u16 {
    9100
}

pub fn default_export_host() -> String {
    "0.0.0.0".to_string()
}

pub fn default_theme() -> String {
    "default".to_string()
}

pub fn default_refresh_rate() -> u64 {
    1000 // milliseconds
}

pub fn default_general() -> GeneralConfig {
    GeneralConfig {
        update_interval: default_update_interval(),
        log_level: default_log_level(),
        host_root: None,
    }
}

pub fn default_export() -> ExportConfig {
    ExportConfig {
        enabled: default_export_enabled(),
        port: default_export_port(),
        host: default_export_host(),
    }
}

pub fn default_ui() -> UiConfig {
    UiConfig {
        theme: default_theme(),
        refresh_rate: default_refresh_rate(),
    }
}
