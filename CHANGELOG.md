# Changelog

All notable changes to Monitor-RS will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- **Docker Image** - Added curl to runtime dependencies for health check support
  - Enables container health checks via HTTP endpoints
  - Minimal image size impact (~2MB)

### Post-Week 1 Enhancements (Days 8+) - Production Infrastructure

## [0.2.0] - 2025-10-22

### Added - Post-Week 1: Real Infrastructure Support

#### MongoDB Collector
- **MongoDB Monitoring** (`src/collectors/services/mongodb.rs` - 352 lines)
  - Async client with mongodb v2.8
  - Multi-instance support
  - Metrics collected:
    - Connections (current, available, active, total_created)
    - Operations per second (insert, query, update, delete, getmore, command)
    - Lock percentage
    - Replication lag and role
    - Database statistics (collections, documents, data/index/storage size)
    - Version and uptime
  - Connection management
  - Password protection
  - 2 unit tests

#### ThinkingSphinx Collector
- **Sphinx Search Monitoring** (`src/collectors/services/sphinx.rs` - 309 lines)
  - **NOT Elasticsearch** - Uses MySQL wire protocol on port 9306
  - Multi-instance support
  - Metrics collected:
    - Queries total and per second (delta-based)
    - Average query time in milliseconds
    - Index statistics (document count, size in bytes)
    - Worker threads running
    - Version, uptime, connections
  - Previous stats tracking for QPS calculation
  - 2 unit tests

#### Puma Web Server Collector
- **Puma Metrics** (`src/collectors/services/puma.rs` - 298 lines)
  - HTTP-based stats API with reqwest
  - Supports clustered and single mode
  - Metrics collected:
    - Workers (total, booted, old)
    - Thread pool usage (running, max, capacity)
    - **Backlog** (critical metric)
    - Requests count
    - Per-worker details (PID, phase, booted, last_checkin)
  - Token authentication
  - 2 unit tests

#### Production Infrastructure Examples
- **5 Real Infrastructure Configs** (`examples/infrastructure/`)
  - `solarhub-config.toml` - Standard Rails with MySQL, MongoDB, Redis, Sphinx, Puma, Sidekiq
  - `momoep-config.toml` - Payment platform with 25+ Sidekiq queues, HA MySQL, aggressive alerting
  - `moto-config.toml` - Standard Rails monitoring
  - `mese-config.toml` - Standard Rails monitoring
  - `accounts-alms-config.toml` - Python/FastAPI with PostgreSQL, RabbitMQ, Celery
  - `README.md` (450+ lines) - Complete infrastructure guide

#### APM Documentation
- **Application Performance Monitoring Guide** (`docs/guides/APM.md` - 650+ lines)
  - Architecture monitoring (multi-service, Prometheus aggregation)
  - Service dependency graphs
  - Performance bottleneck detection (5 patterns)
  - Database monitoring deep dive (MySQL, MongoDB, Redis)
  - Queue monitoring (Sidekiq 25+ queues, Celery)
  - Web server monitoring (Puma backlog, thread pool)
  - Search engine monitoring (ThinkingSphinx)
  - Alerting strategies and severity levels
  - 2 troubleshooting workflows (decision trees)
  - Best practices (baselines, runbooks, autoscaling)

### Changed

#### Documentation
- **README.md** - Updated with real infrastructure stack
  - Added production-ready examples section
  - Updated feature list (MongoDB, Sphinx, Puma)
  - Updated architecture diagram
  - Updated statistics (14 collectors, 50+ metrics, 5 examples)

#### Dependencies
- Added `mongodb = "2.8"` (optional)
- Added `reqwest = "0.11"` with json feature (optional)
- Updated `server` feature to include `reqwest`
- Updated `databases` feature to include `mongodb-db`

#### Module Structure
- Registered MongoDB, Sphinx, and Puma collectors in `src/collectors/services/mod.rs`
- Added Puma to server feature dependencies

### Statistics (Post-Week 1)
- **New Collectors:** 3 (MongoDB, Sphinx, Puma)
- **Total Collectors:** 14
- **New Config Files:** 6 (5 infrastructure + 1 README)
- **New Documentation:** 1 major guide (APM)
- **Lines of Code Added:** ~1,500
- **Total Project Size:** ~14,500 lines

---

## [0.1.0] - 2025-10-21

### Added - Day 7: Deployment & Integration

#### Kubernetes Deployment
- **Helm Chart** for Kubernetes deployment
  - DaemonSet configuration for cluster-wide monitoring
  - ServiceMonitor for Prometheus Operator integration
  - RBAC permissions (ServiceAccount, ClusterRole, ClusterRoleBinding)
  - ConfigMap for flexible configuration
  - Resource limits and security contexts
  - Rolling update strategy
  - Host network and PID namespace support
- **Deployment Templates**
  - `daemonset.yaml` - Main workload template
  - `service.yaml` - Headless service for pod discovery
  - `servicemonitor.yaml` - Prometheus Operator integration
  - `configmap.yaml` - Dynamic configuration from values
  - Complete RBAC templates
- **Helm Values** - Comprehensive configuration options
  - Resource management (CPU, memory limits)
  - Security settings (non-root, read-only filesystem)
  - Service monitoring configuration
  - Database and queue monitoring settings
  - Node selection and tolerations

#### LXC Deployment
- **LXC Container Configuration** (`deploy/lxc/monitor-rs.conf`)
  - Resource limits (CPU, memory)
  - Host /proc and /sys mounting for metrics
  - Network configuration (bridge mode)
  - Auto-start configuration
  - Security capabilities
- **Automated Setup Script** (`deploy/lxc/setup.sh`)
  - Automated container creation
  - Rust installation
  - Monitor-RS build and installation
  - Systemd service creation
  - Network configuration
- **LXC Documentation** - Complete deployment guide
  - Quick start instructions
  - Manual setup steps
  - Configuration options
  - Management commands
  - Troubleshooting guide

#### Documentation
- **Kubernetes Deployment Guide** - Complete K8s documentation
  - Quick start with Helm
  - Configuration parameters
  - Prometheus integration (with/without Operator)
  - Multi-node metrics collection
  - Upgrade and uninstall procedures
  - Production best practices
  - Troubleshooting guide
  - Example configurations
- **LXC Deployment Guide** - Complete LXC documentation
  - Prerequisites and setup
  - Configuration options
  - Service management
  - Prometheus integration
  - Production best practices
  - Troubleshooting
- **CHANGELOG.md** - This file (iterative changelog)

### Added - Day 6: Prometheus Metrics Export

#### Metrics Export
- **Prometheus Exporter** (`src/export/prometheus.rs`)
  - OpenMetrics format compliance
  - 40+ metrics exported
  - CPU metrics (total, per-core, load averages)
  - Memory metrics (total, used, available, swap)
  - Network metrics (total, per-interface, rates)
  - Disk metrics (total, per-mount, usage percentages)
  - Process and service metrics (counts, CPU, memory)
  - Label support for multi-dimensional metrics

#### HTTP Server
- **Async HTTP Server** (`src/export/server.rs`)
  - Built with axum framework
  - Three endpoints:
    - `GET /` - Service information and version
    - `GET /metrics` - Prometheus metrics export
    - `GET /health` - Health check with staleness detection
  - Background metrics collection task
  - Shared state with thread-safe RwLock
  - Configurable update intervals
  - Graceful metric caching

#### Configuration Examples
- **Prometheus Scrape Config** (`examples/prometheus.yml`)
  - Static targets configuration
  - Kubernetes service discovery
  - Relabel configs for K8s pods
  - Health check endpoint scraping
- **Alert Rules** (`examples/monitor-rs-alerts.yml`)
  - 13 alert rules across 3 severity levels
  - CPU, memory, disk, swap alerts
  - Service-level alerting
  - Health check and staleness alerts
- **Grafana Dashboard** (`examples/grafana-dashboard.json`)
  - 12 panels covering all metrics
  - CPU usage with per-core breakdown
  - Memory usage and breakdown
  - Network traffic graphs
  - Disk usage and space
  - Process and service metrics
  - Ready-to-import JSON

#### Integration
- Updated `main.rs` with async runtime (`#[tokio::main]`)
- Server command fully functional
- Feature-gated Prometheus support
- Error handling for missing features

### Added - Day 5: Terminal UI

#### Interactive TUI
- **TUI Application** (`src/ui/app.rs`)
  - Built with ratatui v0.26 and crossterm v0.27
  - Real-time metrics display with 1-second updates
  - Multi-panel layout:
    - CPU panel (usage, cores, load average)
    - Memory panel (total, used, swap)
    - Network panel (RX/TX totals and rates)
    - Disk panel (usage, status indicators)
    - Services panel (top services by memory)
  - Keyboard controls:
    - `q` or `Esc` - Quit application
    - `r` - Force refresh metrics
  - Auto-refresh every 1 second
  - Graceful terminal cleanup
  - Event-driven architecture

#### UI Components
- `src/ui/app.rs` - Main application logic
- `src/ui/mod.rs` - Module exports
- `src/ui/widgets/` - Widget components (placeholder)
- `src/ui/layouts/` - Layout definitions (placeholder)

### Added - Day 4: Queue Collectors

#### Sidekiq Collector
- **Sidekiq Monitoring** (`src/collectors/services/sidekiq.rs`)
  - Redis-based statistics collection
  - Multi-queue support (13+ queues for momoep)
  - Metrics collected:
    - Processed jobs count
    - Failed jobs count
    - Busy workers count
    - Queue latency per queue
  - Namespace support for multi-tenant setups
  - 5 unit tests

**Queues Supported:**
- `default`, `ug_mtn`, `mtn_open_api_debit`, `airtel_open_api_debit`
- `cellulant_checkout_api_debit`, `moov_api`, `paystack`, `paga`
- `mz_vodacom_debit`, `tz_vodacom`, `zm_zamtel_open_api_debit`
- `check_status`

#### RabbitMQ Collector
- **RabbitMQ Monitoring** (`src/collectors/services/rabbitmq.rs`)
  - HTTP Management API client structure
  - Multi-queue support
  - Metrics collected:
    - Queue depth (messages ready)
    - Message rates (publish, deliver, ack)
    - Consumer count per queue
    - Memory usage
  - 4 unit tests

#### Celery Collector
- **Celery Monitoring** (`src/collectors/services/celery.rs`)
  - Broker-agnostic design (Redis/RabbitMQ)
  - Multi-queue support
  - Metrics collected:
    - Active tasks count
    - Scheduled tasks count
    - Worker statistics
    - Queue lengths
  - 4 unit tests

### Added - Day 3: Database Collectors

#### MySQL Collector
- **MySQL Monitoring** (`src/collectors/services/mysql.rs`)
  - Async client with mysql_async
  - Multi-instance support
  - Metrics collected:
    - Connection count (current, max)
    - Queries per second (QPS)
    - Slow queries count
    - Buffer pool statistics
    - Replication status
  - Connection pooling
  - 2 unit tests

#### PostgreSQL Collector
- **PostgreSQL Monitoring** (`src/collectors/services/postgresql.rs`)
  - Async client with tokio-postgres
  - Multi-instance support
  - Metrics collected:
    - Connection count (active, idle, total)
    - Transactions per second (TPS)
    - Cache hit ratio
    - Active locks count
    - Database size
  - Connection pooling
  - 2 unit tests

#### Redis Collector
- **Redis Monitoring** (`src/collectors/services/redis.rs`)
  - Async client with redis crate
  - Multi-instance support
  - Metrics collected:
    - Operations per second
    - Memory usage (used, peak, fragmentation)
    - Hit rate (hits, misses, ratio)
    - Keyspace statistics
    - Replication info
  - INFO command parsing
  - 3 unit tests

### Added - Days 1-2: Core System Monitoring

#### System Collectors
- **CPU Collector** (`src/collectors/cpu.rs`)
  - Total CPU usage percentage
  - Per-core usage tracking
  - Load average (1m, 5m, 15m)
  - Core count detection
  - 3 unit tests

- **Memory Collector** (`src/collectors/memory.rs`)
  - Total, used, available, free memory
  - Swap statistics (total, used, free)
  - Usage percentages
  - Cached and buffer memory
  - 5 unit tests

- **Network Collector** (`src/collectors/network.rs`)
  - Total bytes received/transmitted
  - Per-interface statistics
  - Receive/transmit rates (bytes/sec)
  - Multiple interface support
  - Rate calculation over time
  - 6 unit tests

- **Disk Collector** (`src/collectors/disk.rs`)
  - Total, used, available space
  - Per-mount statistics
  - Usage percentages
  - Disk type detection (HDD, SSD)
  - Critical/warning thresholds (90%, 95%)
  - 8 unit tests

- **Process Collector** (`src/collectors/process.rs`)
  - Total and running process counts
  - Top processes by CPU and memory
  - Service detection (14 types):
    - MySQL, PostgreSQL, Redis, MongoDB
    - RabbitMQ, Sidekiq, Celery, Elasticsearch
    - SphinxSearch, Node.js, Puma, Nginx
    - Python, Ruby
  - Service aggregation (CPU, memory, count)
  - 6 unit tests

#### Infrastructure
- **Error Handling** (`src/error.rs`)
  - Custom error types with thiserror
  - Result type alias
  - IO, parse, and collection errors

- **Configuration** (`src/config/mod.rs`)
  - TOML-based configuration with figment
  - Environment variable support
  - Service-specific configurations
  - Default values
  - 2 unit tests

- **Snapshot System** (`src/collectors/snapshot.rs`)
  - Aggregated system snapshots
  - Builder pattern for flexibility
  - Timestamp tracking
  - 2 unit tests

#### CLI
- **Command-Line Interface** (`src/main.rs`)
  - Built with clap v4
  - Commands:
    - `tui` - Interactive terminal UI (default)
    - `server` - Prometheus metrics server
    - `snapshot` - One-time system snapshot
    - `generate-config` - Generate default config
  - Logging with tracing
  - Configuration file support

### Testing
- **58 tests passing** (100% success rate)
  - System collectors: 28 tests
  - Database collectors: 7 tests
  - Queue collectors: 13 tests
  - Prometheus export: 6 tests
  - Config/Snapshot: 4 tests

### Infrastructure
- Rust 2021 edition
- Async/await with tokio
- Comprehensive error handling
- Feature flags for optional components
- ~13,500 lines of code
- 43 source files

---

## Version History

### [0.1.0] - 2025-10-21 - Week 1 Complete (100%)

**Week 1 Achievement: Complete infrastructure monitoring solution**

- 11 collectors (5 system + 3 database + 3 queue)
- Interactive TUI with ratatui
- Prometheus metrics export (40+ metrics)
- HTTP server with health checks
- Kubernetes Helm chart
- LXC deployment configuration
- 58 passing tests
- Complete documentation

**Stats:**
- Lines of Code: ~13,500
- Source Files: 43
- Tests: 58 (100% passing)
- Collectors: 11
- Metrics Exported: 40+
- Documentation Pages: 15+

---

## Contributing

When contributing to this project, please:

1. Update CHANGELOG.md with your changes
2. Follow the format: [Added/Changed/Deprecated/Removed/Fixed/Security]
3. Include version number and date
4. Add tests for new features
5. Update documentation

## Links

- [Repository](https://github.com/ericgitangu/perf-monitor-rs)
- [Issues](https://github.com/ericgitangu/perf-monitor-rs/issues)
- [Releases](https://github.com/ericgitangu/perf-monitor-rs/releases)

---

*Monitor-RS - Service-aware infrastructure monitoring in Rust 🦀*
