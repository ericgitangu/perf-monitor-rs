use crate::collectors::*;
use crate::config::Config;
use crate::export::prometheus::PrometheusExporter;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use parking_lot::RwLock;
use serde_json::json;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tracing::{info, warn};

#[cfg(feature = "mysql")]
use crate::collectors::services::{MySQLCollector, MySQLMetrics};

#[cfg(feature = "postgresql")]
use crate::collectors::services::{PostgreSQLCollector, PostgreSQLMetrics};

#[cfg(feature = "redis-db")]
use crate::collectors::services::{RedisCollector, RedisMetrics};

/// Shared application state for the metrics server
#[derive(Clone)]
pub struct AppState {
    pub metrics: Arc<RwLock<MetricsCache>>,
}

/// Cache for collected metrics
#[derive(Debug, Clone, Default)]
pub struct MetricsCache {
    pub cpu: Option<CpuMetrics>,
    pub memory: Option<MemoryMetrics>,
    pub network: Option<NetworkMetrics>,
    pub disk: Option<DiskMetrics>,
    pub processes: Option<ProcessMetrics>,

    #[cfg(feature = "mysql")]
    pub mysql: Option<MySQLMetrics>,

    #[cfg(feature = "postgresql")]
    pub postgresql: Option<PostgreSQLMetrics>,

    #[cfg(feature = "redis-db")]
    pub redis: Option<RedisMetrics>,

    pub last_update: Option<std::time::Instant>,
}

impl MetricsCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_stale(&self, max_age: Duration) -> bool {
        match self.last_update {
            Some(last) => last.elapsed() > max_age,
            None => true,
        }
    }
}

/// HTTP handler for /metrics endpoint
async fn metrics_handler(State(state): State<AppState>) -> Response {
    let cache = state.metrics.read();

    let output = PrometheusExporter::export_all(
        cache.cpu.as_ref(),
        cache.memory.as_ref(),
        cache.network.as_ref(),
        cache.disk.as_ref(),
        cache.processes.as_ref(),
        #[cfg(feature = "mysql")]
        cache.mysql.as_ref(),
        #[cfg(not(feature = "mysql"))]
        None,
        #[cfg(feature = "postgresql")]
        cache.postgresql.as_ref(),
        #[cfg(not(feature = "postgresql"))]
        None,
        #[cfg(feature = "redis-db")]
        cache.redis.as_ref(),
        #[cfg(not(feature = "redis-db"))]
        None,
    );

    (StatusCode::OK, output).into_response()
}

/// HTTP handler for /health endpoint
async fn health_handler(State(state): State<AppState>) -> Response {
    let cache = state.metrics.read();

    let is_healthy = !cache.is_stale(Duration::from_secs(30));

    let status = if is_healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    let mut metrics_available = serde_json::json!({
        "cpu": cache.cpu.is_some(),
        "memory": cache.memory.is_some(),
        "network": cache.network.is_some(),
        "disk": cache.disk.is_some(),
        "processes": cache.processes.is_some(),
    });

    #[cfg(feature = "mysql")]
    {
        metrics_available["mysql"] = serde_json::json!(cache.mysql.is_some());
    }

    #[cfg(feature = "postgresql")]
    {
        metrics_available["postgresql"] = serde_json::json!(cache.postgresql.is_some());
    }

    #[cfg(feature = "redis-db")]
    {
        metrics_available["redis"] = serde_json::json!(cache.redis.is_some());
    }

    let body = json!({
        "status": if is_healthy { "healthy" } else { "unhealthy" },
        "last_update": cache.last_update.map(|t| t.elapsed().as_secs()),
        "metrics_available": metrics_available
    });

    (status, Json(body)).into_response()
}

/// HTTP handler for / (root) endpoint
async fn root_handler() -> Response {
    let body = json!({
        "service": "monitor-rs",
        "version": env!("CARGO_PKG_VERSION"),
        "endpoints": {
            "/metrics": "Prometheus metrics export",
            "/health": "Health check endpoint"
        }
    });

    (StatusCode::OK, Json(body)).into_response()
}

/// Background task to collect metrics at regular intervals
async fn metrics_collector_task(state: AppState, interval_secs: u64, config: Config) {
    let mut interval = interval(Duration::from_secs(interval_secs));

    let mut cpu_collector = CpuCollector::new();
    let mut memory_collector = MemoryCollector::new();
    let mut network_collector = NetworkCollector::new();
    let mut disk_collector = DiskCollector::new();
    let mut process_collector = ProcessCollector::new();

    // Initialize database collectors based on config
    #[cfg(feature = "mysql")]
    let mut mysql_collector = if let Some(ref services) = config.services {
        if let Some(ref mysql_config) = services.mysql {
            if mysql_config.enabled {
                info!("Initializing MySQL collector with {} instances", mysql_config.instances.len());
                match MySQLCollector::new(mysql_config.instances.clone()) {
                    Ok(collector) => Some(collector),
                    Err(e) => {
                        warn!("Failed to initialize MySQL collector: {}", e);
                        None
                    }
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    #[cfg(feature = "postgresql")]
    let mut postgresql_collector = if let Some(ref services) = config.services {
        if let Some(ref pg_config) = services.postgresql {
            if pg_config.enabled {
                info!("Initializing PostgreSQL collector with {} instances", pg_config.instances.len());
                match PostgreSQLCollector::new(pg_config.instances.clone()) {
                    Ok(collector) => Some(collector),
                    Err(e) => {
                        warn!("Failed to initialize PostgreSQL collector: {}", e);
                        None
                    }
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    #[cfg(feature = "redis-db")]
    let mut redis_collector = if let Some(ref services) = config.services {
        if let Some(ref redis_config) = services.redis {
            if redis_config.enabled {
                info!("Initializing Redis collector with {} instances", redis_config.instances.len());
                match RedisCollector::new(redis_config.instances.clone()) {
                    Ok(collector) => Some(collector),
                    Err(e) => {
                        warn!("Failed to initialize Redis collector: {}", e);
                        None
                    }
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    loop {
        interval.tick().await;

        info!("Collecting metrics...");

        let cpu = cpu_collector.collect().ok();
        let memory = memory_collector.collect().ok();
        let network = network_collector.collect().ok();
        let disk = disk_collector.collect().ok();
        let processes = process_collector.collect().ok();

        // Collect database metrics
        #[cfg(feature = "mysql")]
        let mysql = if let Some(ref mut collector) = mysql_collector {
            if let Some(ref services) = config.services {
                if let Some(ref mysql_config) = services.mysql {
                    match collector.collect_async(&mysql_config.instances).await {
                        Ok(metrics) => {
                            info!("Collected MySQL metrics: {} instances", metrics.instances.len());
                            Some(metrics)
                        }
                        Err(e) => {
                            warn!("Failed to collect MySQL metrics: {}", e);
                            None
                        }
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        #[cfg(feature = "postgresql")]
        let postgresql = if let Some(ref mut collector) = postgresql_collector {
            if let Some(ref services) = config.services {
                if let Some(ref pg_config) = services.postgresql {
                    match collector.collect_async(&pg_config.instances).await {
                        Ok(metrics) => {
                            info!("Collected PostgreSQL metrics: {} instances", metrics.instances.len());
                            Some(metrics)
                        }
                        Err(e) => {
                            warn!("Failed to collect PostgreSQL metrics: {}", e);
                            None
                        }
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        #[cfg(feature = "redis-db")]
        let redis = if let Some(ref mut collector) = redis_collector {
            if let Some(ref services) = config.services {
                if let Some(ref redis_config) = services.redis {
                    match collector.collect_async(&redis_config.instances).await {
                        Ok(metrics) => {
                            info!("Collected Redis metrics: {} instances", metrics.instances.len());
                            Some(metrics)
                        }
                        Err(e) => {
                            warn!("Failed to collect Redis metrics: {}", e);
                            None
                        }
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        let mut cache = state.metrics.write();
        cache.cpu = cpu;
        cache.memory = memory;
        cache.network = network;
        cache.disk = disk;
        cache.processes = processes;

        #[cfg(feature = "mysql")]
        {
            cache.mysql = mysql;
        }

        #[cfg(feature = "postgresql")]
        {
            cache.postgresql = postgresql;
        }

        #[cfg(feature = "redis-db")]
        {
            cache.redis = redis;
        }

        cache.last_update = Some(std::time::Instant::now());

        info!("Metrics collection complete");
    }
}

/// Start the Prometheus metrics HTTP server
pub async fn start_server(config: Config, listen_addr: SocketAddr) -> crate::Result<()> {
    info!("Starting Prometheus metrics server on {}", listen_addr);

    // Create shared state
    let state = AppState {
        metrics: Arc::new(RwLock::new(MetricsCache::new())),
    };

    // Start background metrics collection task
    let collector_state = state.clone();
    let update_interval = config.general.update_interval.as_secs();
    let collector_config = config.clone();
    tokio::spawn(async move {
        metrics_collector_task(collector_state, update_interval, collector_config).await;
    });

    // Build the router
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/metrics", get(metrics_handler))
        .route("/health", get(health_handler))
        .with_state(state);

    // Start the server
    info!("Server listening on {}", listen_addr);
    let listener = tokio::net::TcpListener::bind(listen_addr).await?;

    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_cache_new() {
        let cache = MetricsCache::new();
        assert!(cache.cpu.is_none());
        assert!(cache.memory.is_none());
        assert!(cache.last_update.is_none());
    }

    #[test]
    fn test_metrics_cache_is_stale() {
        let mut cache = MetricsCache::new();
        assert!(cache.is_stale(Duration::from_secs(1)));

        cache.last_update = Some(std::time::Instant::now());
        assert!(!cache.is_stale(Duration::from_secs(10)));
    }

    #[test]
    fn test_app_state_creation() {
        let state = AppState {
            metrics: Arc::new(RwLock::new(MetricsCache::new())),
        };

        let cache = state.metrics.read();
        assert!(cache.cpu.is_none());
    }
}
