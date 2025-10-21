# Monitor-RS Documentation

**Service-Aware Infrastructure Monitoring in Rust**

Welcome to the Monitor-RS documentation! This guide provides visualizations and an overview of the project architecture, deployment options, and usage patterns.

---

## 📊 Project Overview

```mermaid
mindmap
  root((Monitor-RS))
    Collectors
      System
        CPU per-core
        Memory + Swap
        Network per-interface
        Disk per-mount
        Process detection
      Database
        MySQL
        PostgreSQL
        Redis
      Queue
        Sidekiq 13+ queues
        RabbitMQ
        Celery
    Interfaces
      CLI
        snapshot
        server
        tui
        generate-config
      TUI
        Real-time dashboard
        Multi-panel layout
        Keyboard controls
      HTTP API
        /metrics
        /health
        /
    Export
      Prometheus
        40+ metrics
        OpenMetrics format
        Per-core labels
        Per-interface labels
      Grafana
        12 panels
        Ready dashboard
    Deployment
      Kubernetes
        Helm chart
        DaemonSet
        ServiceMonitor
      LXC
        Automated setup
        systemd service
      Bare Metal
        Binary install
        systemd service
```

---

## 🏗️ Architecture

### System Architecture

```mermaid
C4Context
    title System Context Diagram for Monitor-RS

    Person(user, "User", "Infrastructure engineer")
    System(monitorrs, "Monitor-RS", "Infrastructure monitoring system")
    System_Ext(prometheus, "Prometheus", "Metrics storage")
    System_Ext(grafana, "Grafana", "Visualization")
    System_Ext(os, "Operating System", "Linux system")

    Rel(user, monitorrs, "Uses", "CLI/TUI/HTTP")
    Rel(monitorrs, os, "Collects metrics from", "sysinfo/procfs")
    Rel(monitorrs, prometheus, "Exports metrics to", "HTTP")
    Rel(prometheus, grafana, "Data source", "PromQL")
    Rel(user, grafana, "Views dashboards", "HTTPS")

    UpdateLayoutConfig($c4ShapeInRow="3", $c4BoundaryInRow="1")
```

### Component Architecture

```mermaid
graph TB
    subgraph "Monitor-RS Application"
        CLI[CLI Interface<br/>clap]
        TUI[Terminal UI<br/>ratatui]
        HTTP[HTTP Server<br/>axum:9100]

        CLI --> Core
        TUI --> Core
        HTTP --> Core

        subgraph "Core Engine"
            Core[Collectors Manager]
            Core --> SysCol[System Collectors]
            Core --> DBCol[Database Collectors]
            Core --> QCol[Queue Collectors]
        end

        subgraph "System Collectors"
            SysCol --> CPU[CPU<br/>per-core]
            SysCol --> MEM[Memory<br/>+ swap]
            SysCol --> NET[Network<br/>per-interface]
            SysCol --> DISK[Disk<br/>per-mount]
            SysCol --> PROC[Process<br/>+ service detect]
        end

        subgraph "Database Collectors"
            DBCol --> MySQL[MySQL<br/>QPS, connections]
            DBCol --> PG[PostgreSQL<br/>TPS, cache hit]
            DBCol --> Redis[Redis<br/>ops/sec, memory]
        end

        subgraph "Queue Collectors"
            QCol --> Sidekiq[Sidekiq<br/>13+ queues]
            QCol --> RabbitMQ[RabbitMQ<br/>queue depth]
            QCol --> Celery[Celery<br/>active tasks]
        end

        Core --> Export[Prometheus Export<br/>OpenMetrics]
    end

    subgraph "External Systems"
        OS[Operating System<br/>Linux/WSL2]
        DBs[(Databases)]
        Queues[(Message Queues)]
        Prom[Prometheus<br/>Scraper]
        Graf[Grafana<br/>Dashboard]
    end

    CPU --> OS
    MEM --> OS
    NET --> OS
    DISK --> OS
    PROC --> OS

    MySQL --> DBs
    PG --> DBs
    Redis --> DBs

    Sidekiq --> Queues
    RabbitMQ --> Queues
    Celery --> Queues

    Export --> HTTP
    HTTP --> Prom
    Prom --> Graf

    style Core fill:#326ce5,stroke:#fff,stroke-width:2px,color:#fff
    style Export fill:#e25822,stroke:#fff,stroke-width:2px,color:#fff
    style HTTP fill:#28a745,stroke:#fff,stroke-width:2px,color:#fff
```

---

## 📦 Deployment Patterns

### Kubernetes Deployment

```mermaid
graph TB
    subgraph "Kubernetes Cluster"
        subgraph "Namespace: monitoring"
            DS[DaemonSet<br/>monitor-rs]
            SVC[Headless Service]
            CM[ConfigMap<br/>config.toml]
            SA[ServiceAccount]
            SM[ServiceMonitor]
        end

        subgraph "Node 1"
            POD1[monitor-rs Pod<br/>Host Network<br/>Host PID]
            POD1 --> SYS1[Host System<br/>Metrics]
        end

        subgraph "Node 2"
            POD2[monitor-rs Pod<br/>Host Network<br/>Host PID]
            POD2 --> SYS2[Host System<br/>Metrics]
        end

        subgraph "Node N"
            POD3[monitor-rs Pod<br/>Host Network<br/>Host PID]
            POD3 --> SYS3[Host System<br/>Metrics]
        end

        DS --> POD1
        DS --> POD2
        DS --> POD3

        CM --> POD1
        CM --> POD2
        CM --> POD3

        SA --> POD1
        SA --> POD2
        SA --> POD3

        POD1 --> SVC
        POD2 --> SVC
        POD3 --> SVC

        SVC --> SM
    end

    subgraph "Prometheus Operator"
        SM --> PROM[Prometheus]
    end

    subgraph "Grafana"
        PROM --> GRAF[Grafana<br/>Dashboard]
    end

    style DS fill:#326ce5,stroke:#fff,stroke-width:2px,color:#fff
    style SM fill:#e25822,stroke:#fff,stroke-width:2px,color:#fff
    style PROM fill:#e6522c,stroke:#fff,stroke-width:2px,color:#fff
    style GRAF fill:#f46800,stroke:#fff,stroke-width:2px,color:#fff
```

### LXC Deployment

```mermaid
graph LR
    subgraph "Host System"
        LXC[LXC Container<br/>Ubuntu Jammy]

        subgraph "Container Resources"
            CPU[CPU Limit<br/>1 core]
            MEM[Memory Limit<br/>256MB]
            NET[Network<br/>Bridge]
        end

        subgraph "Container Mounts"
            PROC["proc mount - ro, bind"]
            SYS["sys mount - ro, bind"]
        end

        subgraph "Inside Container"
            MON[monitor-rs<br/>Binary]
            SYSD[systemd<br/>Service]
            CONF[config.toml]
        end

        LXC --> CPU
        LXC --> MEM
        LXC --> NET

        LXC --> PROC
        LXC --> SYS

        LXC --> MON
        MON --> SYSD
        CONF --> MON
    end

    subgraph "Monitoring"
        MON --> |:9100| PROM[Prometheus]
        PROM --> GRAF[Grafana]
    end

    style LXC fill:#326ce5,stroke:#fff,stroke-width:2px,color:#fff
    style MON fill:#28a745,stroke:#fff,stroke-width:2px,color:#fff
    style PROM fill:#e6522c,stroke:#fff,stroke-width:2px,color:#fff
    style GRAF fill:#f46800,stroke:#fff,stroke-width:2px,color:#fff
```

---

## 🔄 Data Collection Flow

### Metric Collection Pipeline

```mermaid
sequenceDiagram
    participant User
    participant CLI
    participant Collectors
    participant System
    participant Prometheus
    participant Grafana

    User->>CLI: cargo run -- server
    CLI->>Collectors: Initialize all collectors

    loop Every 1 second
        Collectors->>System: Collect metrics
        System-->>Collectors: CPU, Memory, Network, Disk
        Collectors->>Collectors: Store in cache
    end

    Prometheus->>CLI: GET /metrics
    CLI->>Collectors: Get cached metrics
    Collectors-->>CLI: Return metrics
    CLI->>Prometheus: OpenMetrics format

    Prometheus->>Prometheus: Store metrics
    Grafana->>Prometheus: PromQL queries
    Prometheus-->>Grafana: Query results
    Grafana-->>User: Display dashboard
```

### TUI Update Flow

```mermaid
sequenceDiagram
    participant User
    participant TUI
    participant Collectors
    participant Terminal

    User->>TUI: cargo run
    TUI->>Terminal: Enable raw mode
    TUI->>Terminal: Enter alternate screen
    TUI->>Collectors: Initialize collectors

    loop Every 1 second
        TUI->>Collectors: Collect all metrics
        Collectors-->>TUI: Metrics data
        TUI->>TUI: Update panels
        TUI->>Terminal: Render UI
    end

    User->>TUI: Press 'r'
    TUI->>Collectors: Force collect
    Collectors-->>TUI: Fresh metrics
    TUI->>Terminal: Render UI

    User->>TUI: Press 'q'
    TUI->>Terminal: Leave alternate screen
    TUI->>Terminal: Disable raw mode
    TUI-->>User: Exit
```

---

## 📈 Metrics Hierarchy

```mermaid
graph TD
    Root[Monitor-RS Metrics] --> System[System Metrics]
    Root --> Services[Service Metrics]

    System --> CPU[CPU]
    System --> Memory[Memory]
    System --> Network[Network]
    System --> Disk[Disk]
    System --> Process[Processes]

    CPU --> CPU1[cpu_usage_percent]
    CPU --> CPU2[cpu_cores_total]
    CPU --> CPU3[cpu_load_average]
    CPU --> CPU4[cpu_core_usage_percent]

    Memory --> MEM1[memory_total_bytes]
    Memory --> MEM2[memory_used_bytes]
    Memory --> MEM3[memory_usage_percent]
    Memory --> MEM4[swap_*]

    Network --> NET1[network_received_bytes_total]
    Network --> NET2[network_transmitted_bytes_total]
    Network --> NET3[network_*_rate_bytes_per_second]
    Network --> NET4[network_interface_*]

    Disk --> DISK1[disk_total_bytes]
    Disk --> DISK2[disk_used_bytes]
    Disk --> DISK3[disk_usage_percent]
    Disk --> DISK4[disk_mount_*]

    Process --> PROC1[processes_total]
    Process --> PROC2[processes_running]

    Services --> SVC[Service Metrics]
    SVC --> SVC1[service_process_count]
    SVC --> SVC2[service_cpu_usage_percent]
    SVC --> SVC3[service_memory_bytes]

    style Root fill:#326ce5,stroke:#fff,stroke-width:2px,color:#fff
    style System fill:#28a745,stroke:#fff,stroke-width:2px,color:#fff
    style Services fill:#e25822,stroke:#fff,stroke-width:2px,color:#fff
```

---

## 🎯 Use Case Scenarios

### Development Workflow

```mermaid
journey
    title Developer Using Monitor-RS
    section Local Development
      Install monitor-rs: 5: Developer
      Run TUI: 5: Developer
      View real-time metrics: 4: Developer
      Identify bottleneck: 5: Developer
    section Debugging
      Run snapshot: 5: Developer
      Analyze service metrics: 4: Developer
      Check database stats: 5: Developer
      Fix performance issue: 5: Developer
    section Testing
      Start Prometheus server: 5: Developer
      Configure scraping: 4: Developer
      View in Grafana: 5: Developer
      Verify metrics: 5: Developer
```

### Production Deployment

```mermaid
journey
    title Production Deployment Flow
    section Preparation
      Review requirements: 5: SRE
      Choose deployment method: 5: SRE
      Prepare configuration: 4: SRE
    section Kubernetes
      Install Helm chart: 5: SRE
      Verify DaemonSet: 5: SRE
      Check ServiceMonitor: 4: SRE
      Validate metrics: 5: SRE
    section Monitoring
      Import Grafana dashboard: 5: SRE
      Configure alerts: 4: SRE
      Set up notifications: 4: SRE
      Monitor production: 5: SRE
```

---

## 📊 Performance Characteristics

### Resource Usage

```mermaid
pie title CPU Overhead Distribution
    "Collection" : 50
    "TUI Rendering" : 30
    "HTTP Server" : 20
```

```mermaid
pie title Memory Footprint
    "Base Runtime" : 33
    "Collectors" : 27
    "TUI State" : 20
    "HTTP Server" : 20
```

---

## 🔗 Integration Points

```mermaid
graph LR
    A[Monitor-RS] --> B[Prometheus]
    A --> C[Grafana]
    A --> D[Kubernetes]
    A --> E[LXC]
    A --> F[systemd]

    B --> G[AlertManager]
    B --> H[Thanos]

    C --> I[Dashboard]
    C --> J[Alerts]

    D --> K[ServiceMonitor]
    D --> L[DaemonSet]

    style A fill:#326ce5,stroke:#fff,stroke-width:3px,color:#fff
    style B fill:#e6522c,stroke:#fff,stroke-width:2px,color:#fff
    style C fill:#f46800,stroke:#fff,stroke-width:2px,color:#fff
    style D fill:#326ce5,stroke:#fff,stroke-width:2px,color:#fff
```

---

## 📚 Documentation Structure

```
docs/
├── README.md                    # This file (visualizations & overview)
├── INDEX.md                     # Documentation index
├── week1/                       # Week 1 implementation
│   ├── OVERVIEW.md             # Status overview
│   ├── COMPLETED.md            # Completed features
│   └── REMAINING.md            # Remaining work
├── guides/                      # User guides
│   └── QUICKSTART.md           # Quick start guide
├── architecture/                # Architecture docs (planned)
└── deployment/                  # Deployment guides (planned)

Root Documentation:
├── README.md                    # Main project README (900+ lines)
├── IMPLEMENTATION_SUMMARY.md    # Complete implementation details
├── CHANGELOG.md                 # Version history
└── deploy/                      # Deployment configs
    ├── kubernetes/              # K8s Helm chart + guide
    └── lxc/                     # LXC config + guide
```

---

## 🚀 Quick Links

- **[Main README](../README.md)** - Project overview and quick start
- **[Implementation Summary](../IMPLEMENTATION_SUMMARY.md)** - Complete Week 1 details
- **[CHANGELOG](../CHANGELOG.md)** - Version history
- **[Kubernetes Deployment](../deploy/kubernetes/README.md)** - K8s guide
- **[LXC Deployment](../deploy/lxc/README.md)** - LXC guide
- **[Quick Start Guide](guides/QUICKSTART.md)** - Get started in 5 minutes

---

## 📞 Support

- **Issues:** [GitHub Issues](https://github.com/ericgitangu/perf-monitor-rs/issues)
- **Discussions:** [GitHub Discussions](https://github.com/ericgitangu/perf-monitor-rs/discussions)
- **Repository:** [GitHub](https://github.com/ericgitangu/perf-monitor-rs)

---

*Monitor-RS - Service-aware infrastructure monitoring in Rust 🦀*

*Built with ❤️ by [Eric Gitangu](https://github.com/ericgitangu)*
