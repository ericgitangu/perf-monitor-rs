pub mod collectors;
pub mod config;
pub mod error;
pub mod export;
pub mod processing;
pub mod ui;

pub use error::{Error, Result};

#[derive(Debug, Clone)]
pub struct MonitorApp {
    pub config: config::Config,
}

impl MonitorApp {
    pub fn new(config: config::Config) -> Self {
        Self { config }
    }
}
