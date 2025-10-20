# Monitor-RS Quick Start Guide

## ⚡ Installation & Setup

```bash
# You're already in the project directory
cd /home/egitangu/Development/performance_benchmarker

# Build the project (debug mode)
$HOME/.cargo/bin/cargo build

# Or build optimized release version
$HOME/.cargo/bin/cargo build --release

# Run tests to verify everything works
$HOME/.cargo/bin/cargo test
```

## 🚀 Basic Usage

### 1. View System Snapshot

**Command:**
```bash
$HOME/.cargo/bin/cargo run -- snapshot
```

**Output:**
```
=== System Snapshot ===
Timestamp: 2025-10-20 21:28:51 UTC

--- CPU ---
Total Usage: 4.13%
Core Count: 12

--- Memory ---
Total: 15.62 GB
Used: 3.33 GB (21.33%)

--- Processes ---
Total: 200

--- Detected Services ---
  node - 27 process(es), Memory: 3.52 GB
  python - 2 process(es), Memory: 18.25 MB
```

### 2. Generate Configuration

**Command:**
```bash
$HOME/.cargo/bin/cargo run -- generate-config --output myconfig.toml
```

**Creates:**
```toml
[general]
update_interval = "1s"
log_level = "info"

[export]
enabled = true
port = 9100
host = "0.0.0.0"

[ui]
theme = "default"
refresh_rate = 1000
```

### 3. Use Custom Configuration

**Command:**
```bash
$HOME/.cargo/bin/cargo run -- --config myconfig.toml snapshot
```

### 4. Run Tests

**Command:**
```bash
$HOME/.cargo/bin/cargo test
```

**Expected:**
```
running 18 tests
...
test result: ok. 18 passed
```

## 📁 Project Structure

```
/home/egitangu/Development/performance_benchmarker/
├── Cargo.toml           # Dependencies and project config
├── README.md            # Full documentation
├── PROGRESS.md          # Implementation tracking
├── SUMMARY.md           # What we've built
├── QUICKSTART.md        # This file
├── example-config.toml  # Generated config example
├── src/
│   ├── main.rs          # CLI entry point
│   ├── lib.rs           # Library API
│   ├── error.rs         # Error types
│   ├── config/          # Configuration system
│   │   ├── mod.rs
│   │   └── defaults.rs
│   └── collectors/      # Metric collectors
│       ├── mod.rs       # Collector trait
│       ├── cpu.rs       # CPU monitoring
│       ├── memory.rs    # Memory monitoring
│       ├── process.rs   # Process & service detection
│       └── snapshot.rs  # Aggregated snapshots
├── tests/               # Integration tests
├── benches/             # Performance benchmarks
├── deploy/              # Deployment configs (TODO)
└── docs/                # Additional documentation
    └── REVISED_PLAN.md  # Full 1-week plan
```

## 🎯 What's Monitoring Your System

### Currently Detected

**System Resources:**
- 12 CPU cores
- 15.62 GB RAM
- 8 GB Swap

**Processes:**
- 200 total processes
- 27 Node.js processes (your Next.js apps!)
- 2 Python processes

### Services to Discover

Once you start your infrastructure services, Monitor-RS will detect:

**Databases:**
- MySQL (mysqld)
- PostgreSQL (postgres)
- Redis (redis-server)
- MongoDB (mongod)

**Web Servers:**
- Puma (puma)
- Nginx (nginx)

**Background Jobs:**
- Sidekiq (sidekiq)
- Celery (celery)

**Queues:**
- RabbitMQ (beam.*rabbitmq)

**Search:**
- Elasticsearch (elasticsearch)
- SphinxSearch (searchd)

## 🔧 Development Workflow

### Make Changes

```bash
# Edit a collector
vim src/collectors/cpu.rs

# Run tests
$HOME/.cargo/bin/cargo test

# Check if it compiles
$HOME/.cargo/bin/cargo check

# Run snapshot to see changes
$HOME/.cargo/bin/cargo run -- snapshot
```

### Watch Mode (if cargo-watch installed)

```bash
# Install cargo-watch
$HOME/.cargo/bin/cargo install cargo-watch

# Auto-run tests on file changes
$HOME/.cargo/bin/cargo watch -x test

# Auto-run snapshot on changes
$HOME/.cargo/bin/cargo watch -x 'run -- snapshot'
```

## 🐛 Troubleshooting

### "Command not found: cargo"

Use full path:
```bash
$HOME/.cargo/bin/cargo build
```

Or add to PATH:
```bash
export PATH="$HOME/.cargo/bin:$PATH"
cargo build
```

### Build Errors

```bash
# Clean and rebuild
$HOME/.cargo/bin/cargo clean
$HOME/.cargo/bin/cargo build
```

### Tests Failing

```bash
# Run specific test
$HOME/.cargo/bin/cargo test test_cpu_collection

# Show test output
$HOME/.cargo/bin/cargo test -- --nocapture
```

## 📊 Monitoring Specific Services

### Monitor Your Next.js App

```bash
# Run snapshot to see Node.js processes
$HOME/.cargo/bin/cargo run -- snapshot

# Look for "node" in detected services
# Output shows: "node - 27 process(es)"
```

### Monitor MySQL (when running)

```bash
# Start your MySQL instances
# Then run snapshot
$HOME/.cargo/bin/cargo run -- snapshot

# Will show: "mysql - X process(es), CPU: X%, Memory: XGB"
```

### Monitor Sidekiq Workers (when running)

```bash
# Start your Rails apps with Sidekiq
# Run snapshot
$HOME/.cargo/bin/cargo run -- snapshot

# Will show: "sidekiq - X process(es), CPU: X%, Memory: XGB"
```

## 🚦 Next Steps

### Day 3 (Network & Disk + Databases)

**To implement:**
1. Network collector (bytes in/out)
2. Disk I/O collector
3. MySQL collector (connections, queries/sec)
4. PostgreSQL collector
5. Redis collector

**To test, you'll need running services:**
```bash
# Example: Start MySQL locally
docker run -d -p 3306:3306 --name mysql \
  -e MYSQL_ROOT_PASSWORD=password \
  mysql:8.0

# Monitor-RS will detect it!
$HOME/.cargo/bin/cargo run -- snapshot
```

### Day 4 (Message Queues)

**To implement:**
1. RabbitMQ collector (HTTP Management API)
2. Sidekiq collector (Redis-based stats)
3. Celery collector
4. Elasticsearch collector

### Day 5 (TUI)

**To implement:**
1. Ratatui-based terminal UI
2. Real-time updates
3. Multi-panel layout
4. Service status grid

### Day 6 (Prometheus)

**To implement:**
1. HTTP server on port 9100
2. `/metrics` endpoint
3. OpenMetrics format export

### Day 7 (Deployment)

**To implement:**
1. Helm chart
2. LXC configuration
3. Integration tests
4. Documentation

## 💡 Pro Tips

**1. Generate config first:**
```bash
$HOME/.cargo/bin/cargo run -- generate-config
# Edit config.toml
# Then use: cargo run -- --config config.toml snapshot
```

**2. Watch resource usage:**
```bash
# Run monitor in a loop
while true; do \
  $HOME/.cargo/bin/cargo run -- snapshot; \
  sleep 5; \
done
```

**3. Save snapshots:**
```bash
# Redirect to file
$HOME/.cargo/bin/cargo run -- snapshot > snapshot-$(date +%Y%m%d-%H%M%S).txt
```

**4. Filter processes (coming soon):**
```bash
# Will add: cargo run -- snapshot --filter mysql
# To show only MySQL-related processes
```

## 📚 Resources

**Documentation:**
- README.md - Full documentation
- PROGRESS.md - Implementation status
- SUMMARY.md - Achievement summary
- docs/REVISED_PLAN.md - Complete plan

**Code:**
- src/collectors/ - All collectors
- src/config/ - Configuration system
- tests/ - Test suite

**Configuration:**
- example-config.toml - Generated example
- Cargo.toml - Project dependencies

## ✅ Verification Checklist

- [ ] Rust installed (`$HOME/.cargo/bin/cargo --version`)
- [ ] Project builds (`$HOME/.cargo/bin/cargo build`)
- [ ] Tests pass (`$HOME/.cargo/bin/cargo test`)
- [ ] Snapshot works (`$HOME/.cargo/bin/cargo run -- snapshot`)
- [ ] Config generates (`$HOME/.cargo/bin/cargo run -- generate-config`)
- [ ] Services detected (Node.js: ✅, Python: ✅)

## 🎉 Success!

You now have a working Rust-based infrastructure monitor that:
- ✅ Monitors CPU, Memory, Processes
- ✅ Detects 14 service types
- ✅ Groups processes by service
- ✅ Provides service-level statistics
- ✅ Configurable via TOML/env/CLI

**Ready to continue with Day 3!**

---

**Quick Reference:**
```bash
# Most common commands
$HOME/.cargo/bin/cargo test                    # Run tests
$HOME/.cargo/bin/cargo run -- snapshot         # View system
$HOME/.cargo/bin/cargo run -- generate-config  # Make config
$HOME/.cargo/bin/cargo build --release         # Optimized build
```
