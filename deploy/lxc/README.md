# LXC Deployment for Monitor-RS

Deploy monitor-rs in an LXC container for isolated system monitoring.

## Prerequisites

- LXC installed on the host system
- Root access
- Internet connectivity for package downloads

## Quick Start

```bash
cd deploy/lxc
sudo ./setup.sh
```

This will:
1. Create an Ubuntu Jammy LXC container
2. Install Rust and dependencies
3. Build monitor-rs from source
4. Install and start the monitor-rs service
5. Expose metrics on port 9100

## Manual Setup

### 1. Create Container

```bash
sudo lxc-create -n monitor-rs -t download -- \
    --dist ubuntu \
    --release jammy \
    --arch amd64
```

### 2. Configure Container

Copy the configuration:

```bash
sudo cp monitor-rs.conf /var/lib/lxc/monitor-rs/config
```

### 3. Start Container

```bash
sudo lxc-start -n monitor-rs
```

### 4. Build and Install

Attach to the container:

```bash
sudo lxc-attach -n monitor-rs
```

Inside the container:

```bash
# Install dependencies
apt-get update
apt-get install -y curl build-essential pkg-config libssl-dev git

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source /root/.cargo/env

# Clone and build
cd /opt
git clone https://github.com/ericgitangu/perf-monitor-rs.git monitor-rs
cd monitor-rs
cargo build --release --features server

# Install
cp target/release/monitor-rs /usr/local/bin/
```

### 5. Create Systemd Service

Create `/etc/systemd/system/monitor-rs.service`:

```ini
[Unit]
Description=Monitor-RS System Monitor
After=network.target

[Service]
Type=simple
User=root
ExecStart=/usr/local/bin/monitor-rs server --listen 0.0.0.0:9100
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

Enable and start:

```bash
systemctl daemon-reload
systemctl enable monitor-rs
systemctl start monitor-rs
```

## Access Metrics

Find container IP:

```bash
sudo lxc-info -n monitor-rs -iH
```

Access endpoints:

```bash
# Metrics
curl http://<CONTAINER_IP>:9100/metrics

# Health check
curl http://<CONTAINER_IP>:9100/health
```

## Configuration

Edit `/etc/monitor-rs/config.toml` in the container to customize:

- Update interval
- Log level
- Database connections
- Queue monitoring

Restart after changes:

```bash
sudo lxc-attach -n monitor-rs -- systemctl restart monitor-rs
```

## Resource Limits

Adjust in `monitor-rs.conf`:

```conf
# CPU: 2 cores
lxc.cgroup2.cpu.max = 200000 100000

# Memory: 512MB
lxc.cgroup2.memory.max = 512M
```

## Management

```bash
# Start
sudo lxc-start -n monitor-rs

# Stop
sudo lxc-stop -n monitor-rs

# Restart
sudo lxc-stop -n monitor-rs && sudo lxc-start -n monitor-rs

# Console access
sudo lxc-console -n monitor-rs

# Attach shell
sudo lxc-attach -n monitor-rs

# View logs
sudo lxc-attach -n monitor-rs -- journalctl -u monitor-rs -f

# Destroy
sudo lxc-destroy -n monitor-rs
```

## Networking

### Bridge Network (Default)

Container gets IP from lxcbr0 bridge. Access via container IP.

### Host Network

For direct host network access, modify config:

```conf
lxc.net.0.type = none
```

Then use `hostNetwork: true` equivalent.

## Production Best Practices

1. **Resource Limits**: Set appropriate CPU/memory limits
2. **Auto-start**: Enable auto-start on boot
3. **Monitoring**: Monitor the container itself
4. **Backups**: Regular backups of container config
5. **Updates**: Keep container OS and monitor-rs updated
6. **Security**: Use AppArmor profiles and capability restrictions

## Troubleshooting

### Container won't start

```bash
sudo lxc-start -n monitor-rs -F  # Foreground mode
sudo lxc-info -n monitor-rs
```

### Service not running

```bash
sudo lxc-attach -n monitor-rs -- systemctl status monitor-rs
sudo lxc-attach -n monitor-rs -- journalctl -u monitor-rs -n 50
```

### Network issues

```bash
sudo lxc-attach -n monitor-rs -- ip addr
sudo lxc-attach -n monitor-rs -- ping -c 3 8.8.8.8
```

## Integration with Prometheus

Add to Prometheus config:

```yaml
scrape_configs:
  - job_name: 'monitor-rs-lxc'
    static_configs:
      - targets: ['<CONTAINER_IP>:9100']
        labels:
          deployment: 'lxc'
          container: 'monitor-rs'
```
