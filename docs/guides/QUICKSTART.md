# Quick Start Guide

Get Monitor-RS running in 5 minutes.

## Prerequisites

- Rust 1.75+ (install via rustup)
- Linux environment (WSL2 supported)

## Installation

```bash
# Navigate to project
cd /home/egitangu/Development/performance_benchmarker

# Build the project
cargo build --release

# Or just run in debug mode
cargo build
```

## Basic Usage

### 1. View System Snapshot

```bash
cargo run -- snapshot
```

**Output:**
```
=== System Snapshot ===
--- CPU ---
Total Usage: 25.17%
Core Count: 12

--- Memory ---
Total: 15.62 GB
Used: 4.24 GB (27.14%)

--- Network ---
Total RX: 318.74 MB
Total TX: 225.88 MB

--- Disk ---
Total: 30.92 TB
Used: 3.19 TB (10.31%)

--- Detected Services ---
  node - 28 processes, 3.68 GB
  python - 2 processes, 18.25 MB
```

### 2. Generate Configuration

```bash
cargo run -- generate-config --output config.toml
```

Edit `config.toml` to customize settings.

### 3. Use Custom Configuration

```bash
cargo run -- --config config.toml snapshot
```

## Configuration

Basic `config.toml`:

```toml
[general]
update_interval = "1s"
log_level = "info"

[export]
enabled = true
port = 9100

[ui]
theme = "default"
refresh_rate = 1000
```

## Environment Variables

```bash
# Override log level
MONITOR_LOG_LEVEL=debug cargo run -- snapshot

# Override export port
MONITOR_EXPORT_PORT=9200 cargo run -- snapshot
```

## Running Tests

```bash
# All tests (32 passing)
cargo test

# Specific test
cargo test test_cpu_collection

# With output
cargo test -- --nocapture
```

## What's Monitored

**System:**
- CPU (per-core usage, load average)
- Memory (RAM + swap)
- Network (per-interface stats, rates)
- Disk (usage, mount points)
- Processes (200+ tracked)

**Services Detected:**
- Node.js, Python, Ruby
- MySQL, PostgreSQL, Redis, MongoDB
- Sidekiq, Celery, RabbitMQ
- Elasticsearch, SphinxSearch
- Puma, Nginx

## Next Steps

- **[Full README](../../README.md)** - Complete documentation
- **[Week 1 Progress](../week1/OVERVIEW.md)** - Implementation status
- **[Remaining Work](../week1/REMAINING.md)** - What's next

## Troubleshooting

**"Command not found: cargo"**

Use full path:
```bash
$HOME/.cargo/bin/cargo build
```

**Build errors:**

```bash
cargo clean
cargo build
```

**No services detected:**

Services are detected from running processes. Start your services (MySQL, Redis, etc.) to see them.

---

**Quick Commands:**
```bash
cargo test                           # Run tests
cargo run -- snapshot                # View system
cargo run -- generate-config         # Create config
cargo build --release                # Optimized build
```
