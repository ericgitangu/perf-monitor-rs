use mongodb::{Client, options::ClientOptions};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MongoDBInstanceConfig {
    pub host: String,
    pub port: u16,
    #[serde(skip_serializing)]
    pub username: Option<String>,
    #[serde(skip_serializing)]
    pub password: Option<String>,
    pub database: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MongoDBDatabaseStats {
    pub name: String,
    pub collections: i64,
    pub documents: i64,
    pub data_size_bytes: i64,
    pub index_size_bytes: i64,
    pub storage_size_bytes: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MongoDBInstanceMetrics {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub version: String,
    pub uptime_seconds: i64,
    pub current_connections: i64,
    pub available_connections: i64,
    pub active_connections: i64,
    pub total_created_connections: i64,
    pub ops_per_second: f64,
    pub ops_insert: i64,
    pub ops_query: i64,
    pub ops_update: i64,
    pub ops_delete: i64,
    pub ops_getmore: i64,
    pub ops_command: i64,
    pub lock_percent: f64,
    pub replication_lag_seconds: Option<i64>,
    pub replica_set_role: Option<String>,
    pub databases: Vec<MongoDBDatabaseStats>,
    pub available: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MongoDBMetrics {
    pub instances: Vec<MongoDBInstanceMetrics>,
    pub total_connections: i64,
    pub total_ops_per_second: f64,
    pub total_data_size_bytes: i64,
}

pub struct MongoDBCollector {
    clients: HashMap<String, Client>,
    previous_stats: HashMap<String, (i64, Instant)>, // (total_ops, time)
}

impl MongoDBCollector {
    pub async fn new(instances: Vec<MongoDBInstanceConfig>) -> crate::Result<Self> {
        let mut clients = HashMap::new();

        for instance in instances {
            let connection_string = if let (Some(username), Some(password)) = (&instance.username, &instance.password) {
                format!(
                    "mongodb://{}:{}@{}:{}/{}",
                    username, password, instance.host, instance.port, instance.database
                )
            } else {
                format!("mongodb://{}:{}/{}", instance.host, instance.port, instance.database)
            };

            match ClientOptions::parse(&connection_string).await {
                Ok(options) => {
                    match Client::with_options(options) {
                        Ok(client) => {
                            clients.insert(instance.name.clone(), client);
                        }
                        Err(e) => {
                            tracing::error!("Failed to create MongoDB client for {}: {}", instance.name, e);
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to parse MongoDB connection string for {}: {}", instance.name, e);
                }
            }
        }

        Ok(Self {
            clients,
            previous_stats: HashMap::new(),
        })
    }

    pub async fn collect(&mut self, configs: &[MongoDBInstanceConfig]) -> crate::Result<MongoDBMetrics> {
        let mut instances = Vec::new();
        let mut total_connections = 0;
        let mut total_ops_per_second = 0.0;
        let mut total_data_size_bytes = 0;

        for config in configs {
            let metrics = self.collect_instance(config).await;
            total_connections += metrics.current_connections;
            total_ops_per_second += metrics.ops_per_second;

            for db in &metrics.databases {
                total_data_size_bytes += db.data_size_bytes;
            }

            instances.push(metrics);
        }

        Ok(MongoDBMetrics {
            instances,
            total_connections,
            total_ops_per_second,
            total_data_size_bytes,
        })
    }

    async fn collect_instance(&mut self, config: &MongoDBInstanceConfig) -> MongoDBInstanceMetrics {
        let client = match self.clients.get(&config.name) {
            Some(c) => c,
            None => {
                return MongoDBInstanceMetrics {
                    name: config.name.clone(),
                    host: config.host.clone(),
                    port: config.port,
                    version: String::new(),
                    uptime_seconds: 0,
                    current_connections: 0,
                    available_connections: 0,
                    active_connections: 0,
                    total_created_connections: 0,
                    ops_per_second: 0.0,
                    ops_insert: 0,
                    ops_query: 0,
                    ops_update: 0,
                    ops_delete: 0,
                    ops_getmore: 0,
                    ops_command: 0,
                    lock_percent: 0.0,
                    replication_lag_seconds: None,
                    replica_set_role: None,
                    databases: Vec::new(),
                    available: false,
                    error: Some("Client not initialized".to_string()),
                };
            }
        };

        // Get admin database for server status
        let admin_db = client.database("admin");

        match admin_db.run_command(mongodb::bson::doc! { "serverStatus": 1 }, None).await {
            Ok(server_status) => {
                let version = server_status.get_str("version").unwrap_or("unknown").to_string();
                let uptime = server_status.get_i64("uptime").unwrap_or(0);

                // Connections
                let connections = server_status.get_document("connections").ok();
                let current_connections = connections.and_then(|c| c.get_i64("current").ok()).unwrap_or(0);
                let available_connections = connections.and_then(|c| c.get_i64("available").ok()).unwrap_or(0);
                let active_connections = connections.and_then(|c| c.get_i64("active").ok()).unwrap_or(0);
                let total_created = connections.and_then(|c| c.get_i64("totalCreated").ok()).unwrap_or(0);

                // Operation counters
                let opcounters = server_status.get_document("opcounters").ok();
                let ops_insert = opcounters.and_then(|o| o.get_i64("insert").ok()).unwrap_or(0);
                let ops_query = opcounters.and_then(|o| o.get_i64("query").ok()).unwrap_or(0);
                let ops_update = opcounters.and_then(|o| o.get_i64("update").ok()).unwrap_or(0);
                let ops_delete = opcounters.and_then(|o| o.get_i64("delete").ok()).unwrap_or(0);
                let ops_getmore = opcounters.and_then(|o| o.get_i64("getmore").ok()).unwrap_or(0);
                let ops_command = opcounters.and_then(|o| o.get_i64("command").ok()).unwrap_or(0);

                let total_ops = ops_insert + ops_query + ops_update + ops_delete + ops_getmore + ops_command;

                // Calculate ops/sec
                let now = Instant::now();
                let ops_per_second = if let Some((prev_ops, prev_time)) = self.previous_stats.get(&config.name) {
                    let elapsed = now.duration_since(*prev_time).as_secs_f64();
                    if elapsed > 0.0 {
                        (total_ops - prev_ops) as f64 / elapsed
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };
                self.previous_stats.insert(config.name.clone(), (total_ops, now));

                // Global lock percentage
                let global_lock = server_status.get_document("globalLock").ok();
                let lock_percent = global_lock
                    .and_then(|gl| gl.get_f64("lockTime").ok())
                    .unwrap_or(0.0) / uptime as f64 * 100.0;

                // Replication info
                let repl = server_status.get_document("repl").ok();
                let replica_set_role = repl.and_then(|r| r.get_str("setName").ok()).map(|s| s.to_string());
                let replication_lag = None; // Would need to calculate from oplog

                // Database stats
                let mut databases = Vec::new();
                if let Ok(db_list) = admin_db.run_command(mongodb::bson::doc! { "listDatabases": 1 }, None).await {
                    if let Ok(db_array) = db_list.get_array("databases") {
                        for db_doc in db_array {
                            if let Some(db) = db_doc.as_document() {
                                if let Ok(db_name) = db.get_str("name") {
                                    if db_name != "admin" && db_name != "local" && db_name != "config" {
                                        let db_obj = client.database(db_name);
                                        if let Ok(stats) = db_obj.run_command(mongodb::bson::doc! { "dbStats": 1 }, None).await {
                                            databases.push(MongoDBDatabaseStats {
                                                name: db_name.to_string(),
                                                collections: stats.get_i64("collections").unwrap_or(0),
                                                documents: stats.get_i64("objects").unwrap_or(0),
                                                data_size_bytes: stats.get_i64("dataSize").unwrap_or(0),
                                                index_size_bytes: stats.get_i64("indexSize").unwrap_or(0),
                                                storage_size_bytes: stats.get_i64("storageSize").unwrap_or(0),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                MongoDBInstanceMetrics {
                    name: config.name.clone(),
                    host: config.host.clone(),
                    port: config.port,
                    version,
                    uptime_seconds: uptime,
                    current_connections,
                    available_connections,
                    active_connections,
                    total_created_connections: total_created,
                    ops_per_second,
                    ops_insert,
                    ops_query,
                    ops_update,
                    ops_delete,
                    ops_getmore,
                    ops_command,
                    lock_percent,
                    replication_lag_seconds: replication_lag,
                    replica_set_role,
                    databases,
                    available: true,
                    error: None,
                }
            }
            Err(e) => {
                MongoDBInstanceMetrics {
                    name: config.name.clone(),
                    host: config.host.clone(),
                    port: config.port,
                    version: String::new(),
                    uptime_seconds: 0,
                    current_connections: 0,
                    available_connections: 0,
                    active_connections: 0,
                    total_created_connections: 0,
                    ops_per_second: 0.0,
                    ops_insert: 0,
                    ops_query: 0,
                    ops_update: 0,
                    ops_delete: 0,
                    ops_getmore: 0,
                    ops_command: 0,
                    lock_percent: 0.0,
                    replication_lag_seconds: None,
                    replica_set_role: None,
                    databases: Vec::new(),
                    available: false,
                    error: Some(format!("Failed to get server status: {}", e)),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mongodb_instance_config_serialization() {
        let config = MongoDBInstanceConfig {
            host: "localhost".to_string(),
            port: 27017,
            username: Some("monitor".to_string()),
            password: Some("secret".to_string()),
            database: "admin".to_string(),
            name: "test".to_string(),
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(!json.contains("secret")); // Password should not be serialized
        assert!(json.contains("localhost"));
        assert!(json.contains("27017"));
    }

    #[test]
    fn test_mongodb_metrics_aggregation() {
        let metrics = MongoDBMetrics {
            instances: vec![
                MongoDBInstanceMetrics {
                    name: "test1".to_string(),
                    host: "localhost".to_string(),
                    port: 27017,
                    version: "6.0.0".to_string(),
                    uptime_seconds: 1000,
                    current_connections: 50,
                    available_connections: 200,
                    active_connections: 30,
                    total_created_connections: 100,
                    ops_per_second: 1000.0,
                    ops_insert: 100,
                    ops_query: 200,
                    ops_update: 50,
                    ops_delete: 10,
                    ops_getmore: 20,
                    ops_command: 500,
                    lock_percent: 0.5,
                    replication_lag_seconds: None,
                    replica_set_role: None,
                    databases: vec![],
                    available: true,
                    error: None,
                },
            ],
            total_connections: 50,
            total_ops_per_second: 1000.0,
            total_data_size_bytes: 0,
        };

        assert_eq!(metrics.total_connections, 50);
        assert!((metrics.total_ops_per_second - 1000.0).abs() < 0.1);
    }
}
