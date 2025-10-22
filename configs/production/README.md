# Infrastructure Monitoring Examples

This directory contains production-ready monitoring configurations for the entire application infrastructure stack.

## Overview

These configurations demonstrate comprehensive monitoring for a multi-service architecture including:

- **Ruby on Rails applications** (solarhub, momoep, moto, mese)
- **Python/FastAPI microservice** (ALMS accounts service)
- **Databases** (MySQL 8.0.18, MongoDB 4.2, PostgreSQL, Redis 3)
- **Search engines** (ThinkingSphinx 5.6.0)
- **Web servers** (Puma)
- **Background job processors** (Sidekiq, Celery)
- **Message queues** (RabbitMQ)

## Configuration Files

### 1. solarhub-config.toml

**Application Type:** Ruby on Rails
**Complexity:** Medium
**Key Features:**
- MySQL primary + replica monitoring
- MongoDB primary + replica monitoring
- Redis for cache and Sidekiq
- ThinkingSphinx search monitoring
- 3 Puma web server instances
- Sidekiq with 9 specialized queues
- ALMS integration for accounts

**Use Case:** Standard Rails application with search capabilities

```bash
monitor-rs --config examples/infrastructure/solarhub-config.toml
```

### 2. momoep-config.toml

**Application Type:** Ruby on Rails - Payment Processing Platform
**Complexity:** High
**Key Features:**
- MySQL primary + 2 replicas (high availability for transactions)
- MongoDB for payment logs and analytics
- Redis with 3 databases (cache, sidekiq, sessions)
- ThinkingSphinx for payment search
- 4 Puma web server instances (high load)
- **Sidekiq with 25+ specialized payment queues**
- External payment gateway health checks (MTN MoMo, Airtel Money)
- Aggressive alerting thresholds (10s replication lag, 60s queue latency)

**Payment Queues:**
- Payment lifecycle: initiation, authorization, capture, settlement, refund, reversal
- Security: fraud detection, KYC verification, compliance checks
- Provider integration: MTN, Airtel, Orange, Vodafone
- Notifications: webhooks (in/out), SMS, email, push
- Reconciliation: daily, transaction matching, settlement
- Analytics: payment analytics, merchant reporting, revenue calculation

**Use Case:** High-volume, mission-critical payment processing

```bash
monitor-rs --config examples/infrastructure/momoep-config.toml
```

### 3. moto-config.toml

**Application Type:** Ruby on Rails
**Complexity:** Low-Medium
**Key Features:**
- MySQL primary + replica
- MongoDB for documents
- Redis for cache and Sidekiq
- ThinkingSphinx search
- 2 Puma web servers
- Sidekiq with 6 standard queues

**Use Case:** Standard Rails application

```bash
monitor-rs --config examples/infrastructure/moto-config.toml
```

### 4. mese-config.toml

**Application Type:** Ruby on Rails
**Complexity:** Low-Medium
**Key Features:**
- MySQL primary + replica
- MongoDB for documents
- Redis for cache and Sidekiq
- ThinkingSphinx search
- 2 Puma web servers
- Sidekiq with 6 standard queues

**Use Case:** Standard Rails application

```bash
monitor-rs --config examples/infrastructure/mese-config.toml
```

### 5. accounts-alms-config.toml

**Application Type:** Python/FastAPI Microservice
**Complexity:** Medium
**Key Features:**
- **PostgreSQL** (not MySQL) for RDBMS
- Redis for sessions and caching
- RabbitMQ for message queuing
- **Celery** (not Sidekiq) for background tasks
- FastAPI health/ready endpoints
- Account-specific alerting (failed logins, verification timeouts)

**Celery Queues:**
- Account management: creation, verification, updates
- Security: password resets, email/SMS verification
- Operations: notifications, audit logging

**Use Case:** Centralized account management service for all platforms

```bash
monitor-rs --config examples/infrastructure/accounts-alms-config.toml
```

## Common Infrastructure Components

### MySQL 8.0.18 Configuration

All MySQL instances use the following tuning parameters:

```bash
--innodb-buffer-pool-instances=4
--innodb-buffer-pool-size=256M
```

**Monitored Metrics:**
- Connections (current, max, percentage utilization)
- Query performance (slow queries, queries per second)
- Replication lag (critical for data consistency)
- InnoDB buffer pool efficiency
- Table/index statistics

### MongoDB 4.2

**Monitored Metrics:**
- Connections (current, available, active)
- Operations per second (insert, query, update, delete, getmore, command)
- Lock percentage
- Database stats (collections, documents, storage size, index size)
- Replication lag and role

### Redis 3

**Monitored Metrics:**
- Memory usage and eviction rate
- Keys (total, expired, evicted per second)
- Hit/miss ratio for cache efficiency
- Connected clients
- Command statistics

### ThinkingSphinx 5.6.0

**Protocol:** MySQL Wire Protocol (Port 9306)
**NOT Elasticsearch!**

**Monitored Metrics:**
- Version and uptime
- Queries total and per second
- Average query time (milliseconds)
- Index statistics (document count, size in bytes)
- Worker threads running

### Puma Web Server

**Stats API:** `http://host:9292/stats?token=SECRET`

**Monitored Metrics:**
- Workers (total, booted, old)
- Thread pool usage (running, max, capacity, **backlog**)
- Requests count
- Per-worker details (PID, phase, booted status)

**Critical Metric:** Backlog indicates request queuing (target: < 30-50)

### Sidekiq (Rails)

**Backend:** Redis

**Monitored Metrics:**
- Queue sizes and latency
- Processed/failed job counts
- Dead jobs count
- Retry jobs count
- Worker utilization

### Celery (Python)

**Broker:** RabbitMQ
**Backend:** Redis

**Monitored Metrics:**
- Queue lengths
- Task execution times
- Worker status
- Failed task rate

## Environment Variables

All configurations use environment variable substitution for sensitive data:

```bash
# Database Passwords
export MYSQL_MONITOR_PASSWORD="..."
export POSTGRES_MONITOR_PASSWORD="..."
export MONGODB_MONITOR_PASSWORD="..."
export REDIS_PASSWORD="..."

# Service Tokens
export PUMA_STATS_TOKEN="..."
export ALMS_API_KEY="..."

# External APIs
export MTN_MOMO_API_URL="https://..."
export AIRTEL_MONEY_API_URL="https://..."
```

## Alerting Thresholds

### Conservative (Solarhub, Moto, Mese)

```toml
[alerts.mysql]
connections_percent = 80
slow_queries_per_second = 10
replication_lag_seconds = 30

[alerts.sidekiq]
queue_latency_seconds = 300
dead_jobs_threshold = 100
```

### Aggressive (Momoep - Payment Platform)

```toml
[alerts.mysql]
connections_percent = 75
slow_queries_per_second = 5
replication_lag_seconds = 10  # Very low tolerance

[alerts.sidekiq]
queue_latency_seconds = 60    # Payment queues must be fast
dead_jobs_threshold = 10      # Low tolerance for failed payments
```

## Multi-Environment Deployment

### Production

```bash
monitor-rs --config examples/infrastructure/solarhub-config.toml \
  --env production
```

### Staging

Copy configuration and adjust:

```toml
[general]
environment = "staging"
collection_interval = "30s"  # Less frequent
retention_period = "3d"       # Shorter retention
```

### Development

```toml
[general]
environment = "development"
collection_interval = "60s"
retention_period = "1d"

[prometheus]
enabled = false  # Optional in dev
```

## Docker Compose Example

See `examples/docker-compose/` for containerized deployment examples.

## Prometheus Integration

All configurations export metrics in OpenMetrics format on `/metrics`:

- **solarhub:** http://localhost:9090/metrics
- **momoep:** http://localhost:9091/metrics
- **moto:** http://localhost:9092/metrics
- **mese:** http://localhost:9093/metrics
- **alms:** http://localhost:9094/metrics

### Prometheus Scrape Configuration

```yaml
scrape_configs:
  - job_name: 'solarhub'
    static_configs:
      - targets: ['localhost:9090']

  - job_name: 'momoep'
    static_configs:
      - targets: ['localhost:9091']

  - job_name: 'moto'
    static_configs:
      - targets: ['localhost:9092']

  - job_name: 'mese'
    static_configs:
      - targets: ['localhost:9093']

  - job_name: 'alms'
    static_configs:
      - targets: ['localhost:9094']
```

## Architecture Diagram

```
┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐
│  Solarhub   │  │   Momoep    │  │    Moto     │  │    Mese     │
│   (Rails)   │  │   (Rails)   │  │   (Rails)   │  │   (Rails)   │
└──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘
       │                │                │                │
       └────────────────┴────────────────┴────────────────┘
                              │
                    ┌─────────┴─────────┐
                    │                   │
              ┌─────▼─────┐       ┌────▼────┐
              │   ALMS    │       │  Redis  │
              │ (FastAPI) │       │ Cluster │
              └─────┬─────┘       └─────────┘
                    │
       ┌────────────┼────────────┐
       │            │            │
  ┌────▼────┐  ┌───▼───┐  ┌─────▼─────┐
  │  MySQL  │  │ Mongo │  │  Sphinx   │
  │ Cluster │  │   DB  │  │  Search   │
  └─────────┘  └───────┘  └───────────┘
```

## Service Dependencies

### Solarhub
- MySQL (primary database)
- MongoDB (documents)
- Redis (cache, sessions, Sidekiq)
- ThinkingSphinx (search)
- ALMS (accounts)

### Momoep
- MySQL (transaction database)
- MongoDB (logs, analytics)
- Redis (cache, sessions, Sidekiq - critical)
- ThinkingSphinx (payment search)
- ALMS (accounts)
- External: MTN MoMo, Airtel Money, Orange Money, Vodafone Cash

### ALMS
- PostgreSQL (account database)
- Redis (sessions, cache)
- RabbitMQ (message queue)

## Troubleshooting

### High Puma Backlog

```bash
# Check thread pool configuration
# Increase max_threads in config/puma.rb
threads 5, 10  # min 5, max 10 (increase max)
workers 4      # Increase worker count
```

### Sidekiq Queue Latency

```bash
# Check Redis memory
redis-cli INFO memory

# Add more Sidekiq workers
bundle exec sidekiq -c 25  # 25 concurrent threads

# Split critical queues to dedicated workers
bundle exec sidekiq -q payment_critical -c 10
```

### MySQL Replication Lag

```bash
# Check replication status
SHOW SLAVE STATUS\G

# Common causes:
# - Slow queries on replica
# - Network issues
# - Disk I/O bottleneck
```

### MongoDB High Lock Percentage

```bash
# Check for long-running operations
db.currentOp()

# Ensure indexes exist for queries
db.collection.getIndexes()
```

## Related Documentation

- [Week 1 Implementation Summary](../../docs/summary.md)
- [APM Guide](../../docs/guides/APM.md) (coming soon)
- [Docker Compose Examples](../docker-compose/)
