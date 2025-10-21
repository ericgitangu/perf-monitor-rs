#!/bin/bash
# LXC setup script for monitor-rs

set -e

CONTAINER_NAME="monitor-rs"
DISTRO="ubuntu"
RELEASE="jammy"

echo "=== Monitor-RS LXC Setup ==="

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo "Error: This script must be run as root"
    exit 1
fi

# Check if LXC is installed
if ! command -v lxc-create &> /dev/null; then
    echo "Installing LXC..."
    apt-get update
    apt-get install -y lxc lxc-templates
fi

# Create container
echo "Creating LXC container..."
if lxc-info -n "$CONTAINER_NAME" &> /dev/null; then
    echo "Container $CONTAINER_NAME already exists"
    read -p "Destroy and recreate? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        lxc-stop -n "$CONTAINER_NAME" -k || true
        lxc-destroy -n "$CONTAINER_NAME"
    else
        exit 0
    fi
fi

lxc-create -n "$CONTAINER_NAME" -t download -- \
    --dist "$DISTRO" \
    --release "$RELEASE" \
    --arch amd64

# Copy configuration
echo "Configuring container..."
cp monitor-rs.conf /var/lib/lxc/"$CONTAINER_NAME"/config

# Start container
echo "Starting container..."
lxc-start -n "$CONTAINER_NAME"

# Wait for network
sleep 5

# Install Rust and build monitor-rs
echo "Installing dependencies in container..."
lxc-attach -n "$CONTAINER_NAME" -- bash -c '
    apt-get update
    apt-get install -y curl build-essential pkg-config libssl-dev git

    # Install Rust
    curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source /root/.cargo/env

    # Clone and build monitor-rs
    cd /opt
    git clone https://github.com/ericgitangu/perf-monitor-rs.git monitor-rs
    cd monitor-rs
    cargo build --release --features server

    # Install binary
    cp target/release/monitor-rs /usr/local/bin/
    chmod +x /usr/local/bin/monitor-rs

    # Create config directory
    mkdir -p /etc/monitor-rs

    # Generate default config
    monitor-rs generate-config --output /etc/monitor-rs/config.toml
'

# Create systemd service
echo "Creating systemd service..."
lxc-attach -n "$CONTAINER_NAME" -- bash -c '
    cat > /etc/systemd/system/monitor-rs.service <<EOF
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
EOF

    systemctl daemon-reload
    systemctl enable monitor-rs
    systemctl start monitor-rs
'

# Get container IP
CONTAINER_IP=$(lxc-info -n "$CONTAINER_NAME" -iH)

echo ""
echo "=== Setup Complete ==="
echo "Container: $CONTAINER_NAME"
echo "IP Address: $CONTAINER_IP"
echo "Metrics: http://$CONTAINER_IP:9100/metrics"
echo "Health: http://$CONTAINER_IP:9100/health"
echo ""
echo "Useful commands:"
echo "  lxc-attach -n $CONTAINER_NAME    # Attach to container"
echo "  lxc-stop -n $CONTAINER_NAME      # Stop container"
echo "  lxc-start -n $CONTAINER_NAME     # Start container"
echo "  lxc-console -n $CONTAINER_NAME   # Console access"
