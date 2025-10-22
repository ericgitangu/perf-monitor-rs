# Monitor-RS Docker Compose Stack

Complete monitoring stack with Prometheus, Grafana, and auto-provisioned dashboards for monitor-rs.

## 📦 What's Included

- **Prometheus** - Metrics collection and storage (http://localhost:9090)
- **Grafana** - Visualization dashboard (http://localhost:8080)
- **Auto-configured datasource** - Prometheus automatically connected
- **Auto-imported dashboard** - Monitor-RS dashboard ready to use (manual import)
- **Persistent storage** - Data survives container restarts
- **Configurable ports** - Easy to customize via .env file

## 🚀 Quick Start

### Prerequisites

- Docker and Docker Compose installed
- monitor-rs running on host machine at `localhost:9100`

### Step 1: Start monitor-rs Server

In the monitor-rs directory:

```bash
cargo run -- server
```

Or if already compiled:

```bash
./target/release/monitor-rs server
```

Verify it's running:

```bash
curl http://localhost:9100/health
```

### Step 2: Start the Monitoring Stack

Navigate to the docker-compose directory:

```bash
cd examples/docker-compose
```

**(Optional) Customize Ports:**

If you need to change the default ports, create a `.env` file:

```bash
cp .env.example .env
# Edit .env to set custom ports
```

Default ports:
- Grafana: **8080**
- Prometheus: **9090**

Start Prometheus and Grafana:

```bash
docker-compose up -d
```

Check status:

```bash
docker-compose ps
```

You should see:

```
NAME                      STATUS    PORTS
monitor-rs-prometheus     Up        0.0.0.0:9090->9090/tcp
monitor-rs-grafana        Up        0.0.0.0:8080->3000/tcp
```

**Note**: Grafana runs on port **8080** by default to avoid common port conflicts (3000, 3001, etc.).

### Step 3: Access the Dashboard

1. **Open Grafana**: http://localhost:8080
   - Username: `admin`
   - Password: `admin` (you'll be prompted to change it)

2. **Import the Dashboard** (Manual Step Required):

   The dashboard needs to be manually imported:

   - Click **☰ Menu** → **Dashboards** → **Import**
   - Click **Upload JSON file**
   - Select: `/path/to/monitor-rs/examples/grafana-dashboard.json`
   - Choose folder: **Monitor-RS**
   - Select datasource: **Prometheus**
   - Click **Import**

   Alternatively, copy-paste the JSON content from `examples/grafana-dashboard.json`

3. **View Real-time Metrics**:
   - Dashboard auto-refreshes every 10 seconds
   - All 12 panels populated with live data
   - CPU, memory, network, disk, and service metrics

## 🔍 Verify Everything is Working

### Check Prometheus is Scraping

1. Open Prometheus UI: http://localhost:9090
2. Go to **Status → Targets**
3. Verify `monitor-rs` target shows **State: UP** (green)

### Test a Query

In Prometheus UI (**Graph** tab), run:

```promql
cpu_usage_percent
```

You should see current CPU usage data.

### Check Grafana Dashboard

In Grafana, the dashboard should display:
- ✅ CPU usage (total + per-core)
- ✅ Load averages
- ✅ Memory usage
- ✅ Network traffic
- ✅ Disk usage
- ✅ Service metrics (Node, Python, Puma, etc.)

## 🐧 Linux-Specific Configuration

If you're on Linux, update `prometheus.yml` line 25:

```yaml
# Change from:
- 'host.docker.internal:9100'

# To:
- '172.17.0.1:9100'
```

Or use host networking:

```bash
# In docker-compose.yml, add to prometheus service:
network_mode: "host"
```

Then restart:

```bash
docker-compose restart prometheus
```

## 📊 Dashboard Panels

The auto-imported dashboard includes:

1. **CPU Usage** - Overall + 12 per-core graphs
2. **Load Average** - 1m, 5m, 15m trends
3. **Memory Usage** - Memory and swap percentages
4. **Memory Breakdown** - Total, used, available bytes
5. **Network Traffic** - RX/TX rates
6. **Network Interfaces** - Per-interface statistics
7. **Disk Usage** - Overall percentage
8. **Disk Mounts** - Per-mount point usage
9. **Process Statistics** - Total and running processes
10. **Service Process Count** - Processes per service
11. **Service CPU Usage** - CPU per service (Puma, Sidekiq, etc.)
12. **Service Memory Usage** - Memory per service

## 🛠️ Management Commands

### View Logs

```bash
# All services
docker-compose logs -f

# Prometheus only
docker-compose logs -f prometheus

# Grafana only
docker-compose logs -f grafana
```

### Restart Services

```bash
# Restart everything
docker-compose restart

# Restart Prometheus (e.g., after config change)
docker-compose restart prometheus
```

### Stop Stack

```bash
docker-compose down
```

### Stop and Remove Data

```bash
# WARNING: This deletes all metrics and dashboards
docker-compose down -v
```

## 🔧 Configuration

### Prometheus Configuration

Edit `prometheus.yml` to:
- Change scrape intervals
- Add more targets
- Configure alerting

After changes:

```bash
docker-compose restart prometheus
```

### Grafana Configuration

- **Datasource**: `grafana/provisioning/datasources/prometheus.yml`
- **Dashboard provider**: `grafana/provisioning/dashboards/dashboard-provider.yml`
- **Dashboard JSON**: `grafana/provisioning/dashboards/monitor-rs-dashboard.json`

Changes to provisioned dashboards require Grafana restart:

```bash
docker-compose restart grafana
```

## 📈 Adding Custom Dashboards

### Option 1: Via Grafana UI

1. Create new dashboard in Grafana
2. Save with **Save as** → Choose **Monitor-RS** folder
3. Dashboards are persisted in `grafana-data` volume

### Option 2: Via Provisioning

1. Export dashboard as JSON from Grafana
2. Save to `grafana/provisioning/dashboards/my-dashboard.json`
3. Restart Grafana: `docker-compose restart grafana`

## 🚨 Troubleshooting

### Prometheus Can't Reach monitor-rs

**Symptom**: Target shows as DOWN in Prometheus

**Solutions**:

1. **Verify monitor-rs is running**:
   ```bash
   curl http://localhost:9100/metrics
   ```

2. **Check Docker network** (Linux):
   ```bash
   # Find Docker bridge IP
   docker network inspect bridge | grep Gateway

   # Update prometheus.yml with the IP:
   - targets: ['172.17.0.1:9100']
   ```

3. **Use host networking** (Linux only):
   ```yaml
   # In docker-compose.yml under prometheus:
   network_mode: "host"
   # Then change target to: localhost:9100
   ```

### Dashboard Shows "No Data"

**Checks**:

1. Prometheus is scraping successfully (check Targets)
2. Time range in Grafana is set correctly (top-right)
3. Datasource is configured (auto-provisioned on first start)

### Grafana Login Issues

**Reset admin password**:

```bash
docker-compose exec grafana grafana-cli admin reset-admin-password newpassword
```

### Port Already in Use

**Note**: Grafana uses port **8080** by default to avoid conflicts with common development servers.

**Easy Way - Use Environment Variables:**

Create a `.env` file in the `examples/docker-compose/` directory:

```bash
# .env
GRAFANA_PORT=8081      # Change to any available port
PROMETHEUS_PORT=9091   # Change if 9090 is taken
```

Then restart:

```bash
docker-compose down
docker-compose up -d
```

**Alternative - Edit docker-compose.yml:**

```yaml
# In docker-compose.yml, change:
ports:
  - "9091:9090"  # For Prometheus
  - "8081:3000"  # For Grafana
```

## 🎯 Production Considerations

For production deployments:

1. **Change default passwords**:
   ```yaml
   # In docker-compose.yml:
   environment:
     - GF_SECURITY_ADMIN_PASSWORD=<strong-password>
   ```

2. **Use external volumes**:
   ```yaml
   volumes:
     - /var/lib/prometheus:/prometheus
     - /var/lib/grafana:/var/lib/grafana
   ```

3. **Enable HTTPS** with reverse proxy (nginx/traefik)

4. **Set up alerting** (see `../monitor-rs-alerts.yml`)

5. **Configure retention**:
   ```yaml
   # In prometheus command:
   - '--storage.tsdb.retention.time=30d'
   - '--storage.tsdb.retention.size=10GB'
   ```

## 📚 Additional Resources

- [Prometheus Documentation](https://prometheus.io/docs/)
- [Grafana Documentation](https://grafana.com/docs/)
- [PromQL Basics](https://prometheus.io/docs/prometheus/latest/querying/basics/)
- [Monitor-RS Main README](../../README.md)

## 🔗 Related Files

```
examples/docker-compose/
├── docker-compose.yml              # Main compose file
├── prometheus.yml                   # Prometheus scrape config
├── README.md                        # This file
└── grafana/provisioning/
    ├── datasources/
    │   └── prometheus.yml          # Auto-provision Prometheus datasource
    └── dashboards/
        ├── dashboard-provider.yml   # Dashboard provider config
        └── monitor-rs-dashboard.json # Auto-imported dashboard
```

---

**Monitor-RS + Prometheus + Grafana = Complete Monitoring Stack** 🚀

For issues or questions, see the [main project documentation](../../README.md).
