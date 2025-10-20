mod defaults;

use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub use defaults::*;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    #[serde(default = "defaults::default_general")]
    pub general: GeneralConfig,

    #[serde(default = "defaults::default_export")]
    pub export: ExportConfig,

    #[serde(default = "defaults::default_ui")]
    pub ui: UiConfig,

    #[serde(default)]
    pub services: Option<ServicesConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ServicesConfig {
    #[cfg(feature = "mysql")]
    #[serde(default)]
    pub mysql: Option<MySQLServiceConfig>,

    #[cfg(feature = "postgresql")]
    #[serde(default)]
    pub postgresql: Option<PostgreSQLServiceConfig>,

    #[cfg(feature = "redis-db")]
    #[serde(default)]
    pub redis: Option<RedisServiceConfig>,
}

#[cfg(feature = "mysql")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MySQLServiceConfig {
    pub enabled: bool,
    pub instances: Vec<crate::collectors::services::MySQLInstanceConfig>,
}

#[cfg(feature = "postgresql")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PostgreSQLServiceConfig {
    pub enabled: bool,
    pub instances: Vec<crate::collectors::services::PostgreSQLInstanceConfig>,
}

#[cfg(feature = "redis-db")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RedisServiceConfig {
    pub enabled: bool,
    pub instances: Vec<crate::collectors::services::RedisInstanceConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GeneralConfig {
    #[serde(default = "defaults::default_update_interval")]
    #[serde(with = "humantime_serde")]
    pub update_interval: Duration,

    #[serde(default = "defaults::default_log_level")]
    pub log_level: String,

    #[serde(default)]
    pub host_root: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExportConfig {
    #[serde(default = "defaults::default_export_enabled")]
    pub enabled: bool,

    #[serde(default = "defaults::default_export_port")]
    pub port: u16,

    #[serde(default = "defaults::default_export_host")]
    pub host: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UiConfig {
    #[serde(default = "defaults::default_theme")]
    pub theme: String,

    #[serde(default = "defaults::default_refresh_rate")]
    pub refresh_rate: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: defaults::default_general(),
            export: defaults::default_export(),
            ui: defaults::default_ui(),
            services: None,
        }
    }
}

impl Config {
    pub fn load() -> crate::Result<Self> {
        let config = Figment::new()
            .merge(Toml::file("config.toml").nested())
            .merge(Toml::file("monitor.toml").nested())
            .merge(Env::prefixed("MONITOR_"))
            .extract()?;

        Ok(config)
    }

    pub fn load_from_file(path: &str) -> crate::Result<Self> {
        let config = Figment::new()
            .merge(Toml::file(path).nested())
            .merge(Env::prefixed("MONITOR_"))
            .extract()?;

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.general.log_level, "info");
        assert_eq!(config.export.port, 9100);
        assert_eq!(config.ui.theme, "default");
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let serialized = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();
        assert_eq!(deserialized.export.port, config.export.port);
    }
}
