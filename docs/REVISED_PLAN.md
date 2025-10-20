# Revised 1-Week Implementation Plan: Service-Aware Infrastructure Monitor

## Overview
Based on infrastructure audit, we need to monitor:
- **4 Rails applications** (solarhub, moto, momoep, mese)
- **1 Next.js application** (engie-powehub-qa - if accessible)
- **7+ infrastructure services** (MySQL, PostgreSQL, Redis, MongoDB, RabbitMQ, SphinxSearch, Elasticsearch)
- **Background job systems** (Sidekiq with 13+ queues, Celery)
- **System resources** (CPU, memory, disk, network)

## Updated Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Monitor-RS TUI/Server                     │
├─────────────────────────────────────────────────────────────┤
│  System Collectors  │  Service Collectors  │  App Collectors│
├─────────────────────┼──────────────────────┼────────────────┤
│ • CPU              │ • MySQL              │ • Process List  │
│ • Memory           │ • PostgreSQL         │ • Container     │
│ • Disk I/O         │ • Redis              │   Detection     │
│ • Network          │ • MongoDB            │ • Service       │
│                    │ • RabbitMQ           │   Discovery     │
│                    │ • Elasticsearch      │                 │
│                    │ • Sidekiq Stats      │                 │
│                    │ • Celery Stats       │                 │
└────────────────────┴──────────────────────┴─────────────────┘
                            │
                            ▼
              ┌─────────────────────────────┐
              │   Prometheus Exporter       │
              │   /metrics endpoint         │
              └─────────────────────────────┘
```

## Day 1: Core System Collectors (DONE ✓)
- [x] Project setup with dependencies
- [x] CPU collector (per-core, load average)
- [x] Memory collector (RAM, swap)
- [x] Configuration with figment
- [x] Unit tests

## Day 2: Process & Service Discovery (TODAY)
### Process Collector with Service Detection
- [ ] Implement process enumeration with sysinfo
- [ ] Service detection patterns (mysql, redis, postgres, mongod, rabbitmq, etc.)
- [ ] Process filtering and grouping
- [ ] CPU/Memory per process
- [ ] Service health inference (running/stopped)

### Network & Disk Collectors
- [ ] Network interface stats (bytes in/out, packets, errors)
- [ ] Disk I/O (read/write bytes, iops)
- [ ] Disk usage per mount point

## Day 3: Service-Level Collectors (MySQL, Redis, PostgreSQL)
### Database Collectors
**MySQL Collector:**
- [ ] Connection via mysql crate or command-line client
- [ ] Metrics: connections, queries/sec, slow queries, buffer pool
- [ ] Multi-instance support (different ports)
- [ ] Handle connection failures gracefully

**PostgreSQL Collector:**
- [ ] Connection via postgres crate or psql
- [ ] Metrics: connections, queries, cache hit ratio, locks
- [ ] Multi-instance support

**Redis Collector:**
- [ ] Connection via redis crate
- [ ] INFO command parsing
- [ ] Metrics: memory usage, connected clients, ops/sec, keyspace
- [ ] Multi-instance support (different ports/DBs)

### MongoDB Collector (if time permits)
- [ ] Connection via mongodb driver
- [ ] Metrics: connections, ops, locks, storage

## Day 4: Message Queue & Background Job Collectors
### RabbitMQ Collector
- [ ] HTTP Management API integration
- [ ] Metrics: queues, messages, consumers, rates
- [ ] Queue-specific stats

### Sidekiq Collector
- [ ] Redis-based stats collection (Sidekiq stores stats in Redis)
- [ ] Metrics: processed, failed, busy, enqueued per queue
- [ ] Support for multiple queue configurations (momoep has 13+ queues!)
- [ ] Latency tracking

### Celery Collector
- [ ] Redis/RabbitMQ backend inspection
- [ ] Task states, worker status
- [ ] Queue depths

## Day 5: Search & Container Awareness
### Elasticsearch Collector
- [ ] REST API integration (_cluster/health, _nodes/stats)
- [ ] Metrics: cluster status, indices, shards, JVM memory

### Container/Process Awareness
- [ ] Parse /proc/[pid]/cgroup for Docker containers
- [ ] LXC container detection (lxc-info)
- [ ] K8s pod detection (environment variables)
- [ ] Group processes by container/pod

### Service Discovery
- [ ] Auto-detect running services from process list
- [ ] Port scanning (optional) for service discovery
- [ ] Configuration-based service definitions

## Day 6: TUI Implementation
### Core TUI
- [ ] Ratatui setup with crossterm
- [ ] Layout: system overview + service grid
- [ ] CPU/Memory widgets (from Day 1)
- [ ] Process list widget with service filtering
- [ ] Service status widget (green/yellow/red indicators)
- [ ] Navigation and keyboard shortcuts

### Service-Specific Views
- [ ] Database view (connections, queries)
- [ ] Queue view (Sidekiq/Celery/RabbitMQ stats)
- [ ] Network/Disk view
- [ ] Container view (grouped processes)

## Day 7: Prometheus Export & Integration
### Prometheus Metrics Server
- [ ] Axum HTTP server on port 9100
- [ ] System metrics export (CPU, memory, disk, network)
- [ ] Service metrics export (databases, queues, etc.)
- [ ] Process metrics (per-service aggregation)
- [ ] OpenMetrics format
- [ ] /health endpoint

### Deployment Configs
- [ ] Minimal Helm chart with ServiceMonitor
- [ ] LXC container config template
- [ ] Example Prometheus scrape configs
- [ ] Grafana dashboard JSON (optional)

### Documentation & Testing
- [ ] README with architecture diagram
- [ ] Service collector configuration guide
- [ ] Integration test with actual services
- [ ] Example queries for each service type

## Key Implementation Details

### Service Detection Patterns
```rust
const SERVICE_PATTERNS: &[(&str, &str)] = &[
    ("mysql", "mysqld"),
    ("postgres", "postgres"),
    ("redis", "redis-server"),
    ("mongodb", "mongod"),
    ("rabbitmq", "beam.smp.*rabbitmq"),
    ("sidekiq", "sidekiq"),
    ("celery", "celery"),
    ("puma", "puma"),
    ("nginx", "nginx"),
    ("elasticsearch", "elasticsearch"),
    ("sphinx", "searchd"),
];
```

### Collector Trait Extensions
```rust
pub trait ServiceCollector: MetricCollector {
    /// Service-specific health check
    fn health_check(&mut self) -> Result<ServiceHealth>;

    /// Service connection info
    fn connection_info(&self) -> ConnectionInfo;

    /// Service-specific metrics
    fn service_metrics(&mut self) -> Result<HashMap<String, f64>>;
}
```

### Configuration Schema
```toml
[services.mysql]
enabled = true
host = "localhost"
port = 3306
user = "monitor"
password = "secret"
instances = ["solarhub", "accounts"]

[services.redis]
enabled = true
instances = [
    { host = "localhost", port = 6379, name = "main" },
    { host = "localhost", port = 6380, name = "cache" },
]

[services.sidekiq]
enabled = true
redis_url = "redis://localhost:6379/0"
queues = ["default", "ug_mtn", "mtn_open_api_debit"]
```

## Success Criteria (End of Week)

### Minimum Viable Product (MVP)
✅ System monitoring (CPU, memory, disk, network)
✅ Process monitoring with service detection
✅ At least 3 service collectors (MySQL, Redis, PostgreSQL)
✅ TUI with system + service views
✅ Prometheus export for all metrics
✅ Basic Helm chart template

### Stretch Goals
⭐ All 7+ service collectors working
⭐ Container/LXC/K8s awareness
⭐ Sidekiq queue-specific metrics
⭐ Auto-discovery of running services
⭐ Grafana dashboard

## Technology Stack (Updated)

**Core:**
- Rust 2021, sysinfo, crossbeam

**TUI:**
- ratatui, crossterm

**Service Integrations:**
- mysql (async)
- tokio-postgres
- redis
- mongodb driver
- reqwest (for HTTP APIs: RabbitMQ, Elasticsearch)

**Server:**
- tokio, axum, prometheus

**Testing:**
- testcontainers (for integration tests with real services)

## Next Steps After Week 1

1. **Week 2:** Polish, performance optimization, comprehensive testing
2. **Week 3:** Advanced features (alerting, historical data, trends)
3. **Week 4:** Production deployment (full Helm chart, Terraform, monitoring stack)

---

**This revised plan makes monitor-rs a comprehensive infrastructure monitoring tool, not just a system monitor!**
