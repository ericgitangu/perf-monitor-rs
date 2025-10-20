# Week 1: Remaining Work

This document outlines all remaining work to complete Week 1 implementation.

**Current Status:** 40% Complete (Days 1-3 partial done)
**Remaining:** 60% (Day 3 part 2 + Days 4-7)

---

## 🔄 Day 3 Part 2: Database Collectors (20% of Week)

**Status:** Not Started
**Priority:** HIGH - Foundation for service monitoring
**Estimated Time:** 6-8 hours

### MySQL Collector
**Dependencies:** `mysql_async = "0.34"`

**Implementation Tasks:**
- [ ] Add mysql_async dependency
- [ ] Create `src/collectors/services/mysql.rs`
- [ ] Connection pool management
- [ ] Multi-instance support (different ports)
- [ ] Metrics collection:
  - [ ] Active connections
  - [ ] Queries per second
  - [ ] Slow query count
  - [ ] Buffer pool usage
  - [ ] Replication status (if applicable)
- [ ] Error handling for connection failures
- [ ] Unit tests with mock connections
- [ ] Integration tests (optional, requires MySQL)

**Configuration Schema:**
```toml
[services.mysql]
enabled = true
instances = [
  { host = "localhost", port = 3306, user = "monitor", password = "secret", database = "mysql", name = "solarhub" },
  { host = "localhost", port = 3307, user = "monitor", password = "secret", database = "mysql", name = "accounts" },
]
```

### PostgreSQL Collector
**Dependencies:** `tokio-postgres = "0.7"`

**Implementation Tasks:**
- [ ] Add tokio-postgres dependency
- [ ] Create `src/collectors/services/postgresql.rs`
- [ ] Connection management
- [ ] Multi-instance support
- [ ] Metrics collection:
  - [ ] Active connections
  - [ ] Cache hit ratio
  - [ ] Locks
  - [ ] Transaction statistics
  - [ ] Database size
- [ ] Error handling
- [ ] Unit tests
- [ ] Integration tests (optional)

**Configuration Schema:**
```toml
[services.postgresql]
enabled = true
instances = [
  { host = "localhost", port = 5432, user = "monitor", password = "secret", database = "postgres", name = "accounts" },
]
```

### Redis Collector
**Dependencies:** `redis = { version = "0.25", features = ["tokio-comp"] }`

**Implementation Tasks:**
- [ ] Add redis dependency
- [ ] Create `src/collectors/services/redis.rs`
- [ ] Connection handling
- [ ] Multi-instance support (different ports/DBs)
- [ ] INFO command parsing
- [ ] Metrics collection:
  - [ ] Memory usage
  - [ ] Connected clients
  - [ ] Operations per second
  - [ ] Keyspace statistics
  - [ ] Replication info
  - [ ] Hit rate
- [ ] Error handling
- [ ] Unit tests
- [ ] Integration tests (optional)

**Configuration Schema:**
```toml
[services.redis]
enabled = true
instances = [
  { host = "localhost", port = 6379, db = 0, name = "main" },
  { host = "localhost", port = 6380, db = 0, name = "cache" },
]
```

---

## ⏳ Day 4: Message Queue & Job Collectors (15% of Week)

**Status:** Not Started
**Priority:** HIGH - Critical for payment processing monitoring (momoep)
**Estimated Time:** 6-8 hours

### RabbitMQ Collector
**Dependencies:** `reqwest = "0.12"` (for HTTP Management API)

**Implementation Tasks:**
- [ ] Create `src/collectors/services/rabbitmq.rs`
- [ ] HTTP Management API client
- [ ] Multi-instance support
- [ ] Metrics collection:
  - [ ] Queue depths per queue
  - [ ] Message rates (publish, deliver, ack)
  - [ ] Consumer counts
  - [ ] Unacked messages
  - [ ] Connection count
- [ ] Error handling
- [ ] Tests

**Configuration Schema:**
```toml
[services.rabbitmq]
enabled = true
management_url = "http://localhost:15672"
username = "guest"
password = "guest"
queues = ["default", "high_priority", "low_priority"]
```

### Sidekiq Collector
**Dependencies:** `redis` (reuse from Day 3)

**Implementation Tasks:**
- [ ] Create `src/collectors/services/sidekiq.rs`
- [ ] Redis-based stats collection (Sidekiq stores stats in Redis)
- [ ] Multi-queue support (momoep has 13+ queues!)
- [ ] Metrics collection:
  - [ ] Processed jobs (total, per queue)
  - [ ] Failed jobs (total, per queue)
  - [ ] Busy workers
  - [ ] Enqueued jobs (per queue)
  - [ ] Latency per queue
  - [ ] Retry queue size
- [ ] Special handling for momoep queues:
  - `ug_mtn`, `mtn_open_api_debit`, `airtel_open_api_debit`
  - `cellulant_checkout_api_debit`, `moov_api`, `paystack`, `paga`
  - `mz_vodacom_debit`, `tz_vodacom`, `zm_zamtel_open_api_debit`
  - `check_status`, `default`
- [ ] Error handling
- [ ] Tests

**Configuration Schema:**
```toml
[services.sidekiq]
enabled = true
redis_url = "redis://localhost:6379/0"
queues = [
  "default", "ug_mtn", "mtn_open_api_debit", "airtel_open_api_debit",
  "cellulant_checkout_api_debit", "moov_api", "paystack", "paga",
  "mz_vodacom_debit", "tz_vodacom", "zm_zamtel_open_api_debit", "check_status"
]
```

### Celery Collector
**Dependencies:** `redis` or `amqp` (broker dependent)

**Implementation Tasks:**
- [ ] Create `src/collectors/services/celery.rs`
- [ ] Broker inspection (Redis or RabbitMQ)
- [ ] Metrics collection:
  - [ ] Active tasks
  - [ ] Scheduled tasks
  - [ ] Worker status
  - [ ] Task success/failure rates
  - [ ] Queue depths
- [ ] Error handling
- [ ] Tests

**Configuration Schema:**
```toml
[services.celery]
enabled = true
broker_url = "redis://localhost:6379/0"
queues = ["celery", "main-queue", "hourly-tasks-queue"]
```

### Elasticsearch Collector
**Dependencies:** `reqwest` (for REST API)

**Implementation Tasks:**
- [ ] Create `src/collectors/services/elasticsearch.rs`
- [ ] REST API client (`_cluster/health`, `_nodes/stats`)
- [ ] Metrics collection:
  - [ ] Cluster health (green/yellow/red)
  - [ ] Node count
  - [ ] Index count
  - [ ] Shard statistics
  - [ ] JVM memory usage
  - [ ] Query performance
- [ ] Error handling
- [ ] Tests

**Configuration Schema:**
```toml
[services.elasticsearch]
enabled = true
url = "http://localhost:9200"
indices = ["logs", "metrics"]
```

---

## ⏳ Day 5: Terminal UI (15% of Week)

**Status:** Not Started
**Priority:** MEDIUM - Nice to have for local monitoring
**Estimated Time:** 8-10 hours

### Ratatui Integration
**Dependencies:** Already added (ratatui, crossterm)

**Implementation Tasks:**
- [ ] Create `src/ui/app.rs` - Application state machine
- [ ] Create `src/ui/event.rs` - Event handling system
- [ ] Create `src/ui/theme.rs` - Theme system
- [ ] Terminal setup/cleanup
- [ ] Event loop with keyboard handling

### Layout System
**Tasks:**
- [ ] Create `src/ui/layouts/default.rs`
- [ ] Multi-panel layout:
  - [ ] System overview panel (CPU, Memory, Disk, Network)
  - [ ] Service status grid
  - [ ] Process list panel
  - [ ] Detail view panel
- [ ] Responsive resizing
- [ ] Panel switching

### Widgets
**Tasks:**
- [ ] `src/ui/widgets/cpu.rs` - CPU bars + sparklines
- [ ] `src/ui/widgets/memory.rs` - Memory gauges
- [ ] `src/ui/widgets/disk.rs` - Disk usage bars
- [ ] `src/ui/widgets/network.rs` - Network rate charts
- [ ] `src/ui/widgets/process_list.rs` - Scrollable process table
- [ ] `src/ui/widgets/service_grid.rs` - Service status grid

### Interactivity
**Tasks:**
- [ ] Keyboard navigation (arrow keys, tab, etc.)
- [ ] Process sorting (CPU, Memory, Name)
- [ ] Process filtering (by service, regex)
- [ ] Search functionality (/)
- [ ] Help overlay (F1 or ?)
- [ ] Quit handling (q, Ctrl-C)

### Real-Time Updates
**Tasks:**
- [ ] Background collector thread
- [ ] Periodic UI refresh (configurable)
- [ ] Delta calculations for rates
- [ ] Smooth animations

---

## ⏳ Day 6: Prometheus Metrics Export (15% of Week)

**Status:** Not Started
**Priority:** HIGH - Critical for integration with existing monitoring
**Estimated Time:** 6-8 hours

### HTTP Server
**Dependencies:** Already added (tokio, axum, prometheus)

**Implementation Tasks:**
- [ ] Create `src/export/prometheus.rs`
- [ ] Axum HTTP server setup
- [ ] Metric registration system
- [ ] Metric update mechanism

### Metrics Endpoints
**Tasks:**
- [ ] `/metrics` endpoint - OpenMetrics format
- [ ] `/health` endpoint - Liveness probe
- [ ] CORS middleware (optional)
- [ ] Request logging

### System Metrics Export
**Tasks:**
- [ ] CPU metrics:
  - `node_cpu_usage_percent{core="N"}`
  - `node_load_average{period="1m|5m|15m"}`
- [ ] Memory metrics:
  - `node_memory_total_bytes`
  - `node_memory_used_bytes`
  - `node_memory_available_bytes`
  - `node_swap_total_bytes`
  - `node_swap_used_bytes`
- [ ] Network metrics:
  - `node_network_receive_bytes_total{interface="eth0"}`
  - `node_network_transmit_bytes_total{interface="eth0"}`
  - `node_network_receive_errors_total{interface="eth0"}`
- [ ] Disk metrics:
  - `node_filesystem_size_bytes{mountpoint="/"}`
  - `node_filesystem_avail_bytes{mountpoint="/"}`
  - `node_filesystem_used_percent{mountpoint="/"}`

### Service Metrics Export
**Tasks:**
- [ ] Process metrics:
  - `process_cpu_percent{pid="1234",service="mysql",name="mysqld"}`
  - `process_memory_bytes{pid="1234",service="mysql"}`
- [ ] MySQL metrics:
  - `mysql_connections{instance="solarhub"}`
  - `mysql_queries_per_second{instance="solarhub"}`
  - `mysql_slow_queries{instance="solarhub"}`
- [ ] Redis metrics:
  - `redis_memory_used_bytes{instance="main"}`
  - `redis_ops_per_second{instance="main"}`
  - `redis_connected_clients{instance="main"}`
- [ ] Sidekiq metrics:
  - `sidekiq_queue_depth{queue="ug_mtn"}`
  - `sidekiq_processed_total{queue="ug_mtn"}`
  - `sidekiq_failed_total{queue="ug_mtn"}`
  - `sidekiq_latency_seconds{queue="ug_mtn"}`

### Testing
**Tasks:**
- [ ] Integration tests with curl/httpie
- [ ] Prometheus scrape config example
- [ ] Validate OpenMetrics format
- [ ] Load testing

---

## ⏳ Day 7: Deployment & Integration (15% of Week)

**Status:** Not Started
**Priority:** MEDIUM - Production deployment preparation
**Estimated Time:** 8-10 hours

### Kubernetes Helm Chart
**Tasks:**
- [ ] Complete `deploy/kubernetes/helm/monitor-rs/Chart.yaml`
- [ ] Complete `deploy/kubernetes/helm/monitor-rs/values.yaml`
- [ ] Templates:
  - [ ] `templates/daemonset.yaml` - Full implementation
  - [ ] `templates/service.yaml` - ClusterIP service
  - [ ] `templates/servicemonitor.yaml` - Prometheus Operator
  - [ ] `templates/serviceaccount.yaml` - RBAC
  - [ ] `templates/clusterrole.yaml` - Permissions
  - [ ] `templates/clusterrolebinding.yaml` - Binding
  - [ ] `templates/configmap.yaml` - Configuration
- [ ] Security contexts (unprivileged, read-only root)
- [ ] Resource limits (CPU, memory)
- [ ] Node selector and tolerations
- [ ] Testing with minikube/kind

### LXC Container Configuration
**Tasks:**
- [ ] Complete `deploy/lxc/container-config.conf`
- [ ] Unprivileged container setup
- [ ] Host filesystem access (read-only)
- [ ] Systemd service unit
- [ ] Ansible playbook for multi-container deployment
- [ ] Cloud-init configuration
- [ ] Testing on Ubuntu 24.04 LXC

### Integration Testing
**Tasks:**
- [ ] End-to-end tests with real services
- [ ] Docker Compose test stack
- [ ] Testcontainers integration
- [ ] CI/CD pipeline (GitHub Actions)
- [ ] Cross-platform testing (Linux, macOS, Windows)

### Documentation
**Tasks:**
- [ ] Deployment guide (K8s)
- [ ] Deployment guide (LXC)
- [ ] Deployment guide (Standalone)
- [ ] Prometheus query examples
- [ ] Grafana dashboard JSON
- [ ] Troubleshooting guide
- [ ] Performance tuning guide

### Example Configurations
**Tasks:**
- [ ] Production configuration example
- [ ] Development configuration example
- [ ] Prometheus scrape config
- [ ] Grafana dashboard template
- [ ] Alert rules for Prometheus

---

## 📊 Effort Breakdown

| Task Category | Estimated Hours | % of Remaining |
|---------------|-----------------|----------------|
| Database Collectors (Day 3) | 6-8 hours | 33% |
| Message Queues (Day 4) | 6-8 hours | 33% |
| TUI (Day 5) | 8-10 hours | 20% |
| Prometheus (Day 6) | 6-8 hours | 20% |
| Deployment (Day 7) | 8-10 hours | 20% |
| **Total** | **34-44 hours** | **100%** |

**Note:** With ~6-8 hours/day, this is approximately 5-7 days of focused work.

---

## 🎯 Prioritization Recommendations

### Must-Have (MVP)
1. ✅ System collectors (Done)
2. 🔄 Database collectors (Day 3) - **NEXT**
3. ⏳ Sidekiq collector (Day 4) - **HIGH** for momoep monitoring
4. ⏳ Prometheus export (Day 6) - **HIGH** for integration
5. ⏳ Basic Helm chart (Day 7) - **MEDIUM**

### Nice-to-Have
- Full TUI (Day 5) - Can use `snapshot` command instead
- RabbitMQ collector - Only if actively used
- Celery collector - Only for accounts service
- Elasticsearch collector - Only if actively monitored
- LXC deployment - Alternative to K8s

### Optional/Future
- Advanced TUI features (themes, multiple views)
- Grafana dashboards (can be created later)
- Alerting system (can use Prometheus alerts)
- Historical data storage
- Web dashboard

---

## 🔗 Related Documents

- [Week 1 Overview](OVERVIEW.md)
- [Completed Features](COMPLETED.md)
- [Architecture Design](../architecture/DESIGN.md)
- [Quick Start Guide](../guides/QUICKSTART.md)

---

**Last Updated:** 2025-10-20
**Next Priority:** Complete database collectors (MySQL, PostgreSQL, Redis)
