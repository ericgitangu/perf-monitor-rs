# Kubernetes Deployment for Monitor-RS

Deploy monitor-rs as a DaemonSet in Kubernetes for cluster-wide node monitoring.

## Prerequisites

- Kubernetes cluster (1.19+)
- Helm 3 installed
- kubectl configured
- (Optional) Prometheus Operator for ServiceMonitor

## Quick Start

### Using Helm

```bash
# Add the chart repository (when published)
helm repo add monitor-rs https://charts.monitor-rs.io
helm repo update

# Install with default values
helm install monitor-rs monitor-rs/monitor-rs \
    --namespace monitoring \
    --create-namespace

# Or install from local chart
cd deploy/kubernetes/helm
helm install monitor-rs . \
    --namespace monitoring \
    --create-namespace
```

### Verify Installation

```bash
# Check DaemonSet status
kubectl get daemonset -n monitoring monitor-rs

# Check pods (should be one per node)
kubectl get pods -n monitoring -l app.kubernetes.io/name=monitor-rs

# View logs
kubectl logs -n monitoring -l app.kubernetes.io/name=monitor-rs --tail=50
```

## Configuration

### Custom Values

Create `custom-values.yaml`:

```yaml
# Resource limits
resources:
  limits:
    cpu: 1000m
    memory: 512Mi
  requests:
    cpu: 200m
    memory: 256Mi

# Enable database monitoring
config:
  services:
    mysql:
      enabled: true
      instances:
        - name: "main-db"
          host: "mysql.default.svc.cluster.local"
          port: 3306
          username: "monitor"
          password: "secret"

    redis:
      enabled: true
      instances:
        - name: "cache"
          host: "redis-master.default.svc.cluster.local"
          port: 6379

    sidekiq:
      enabled: true
      redis_url: "redis://redis-master.default.svc.cluster.local:6379/0"
      namespace: "sidekiq"
      queues:
        - "default"
        - "mailers"
        - "high_priority"

# Prometheus ServiceMonitor
serviceMonitor:
  enabled: true
  labels:
    prometheus: kube-prometheus
```

Install with custom values:

```bash
helm install monitor-rs . \
    --namespace monitoring \
    --create-namespace \
    --values custom-values.yaml
```

### Configuration Parameters

| Parameter | Description | Default |
|-----------|-------------|---------|
| `image.repository` | Image repository | `monitor-rs` |
| `image.tag` | Image tag | `0.1.0` |
| `resources.limits.cpu` | CPU limit | `500m` |
| `resources.limits.memory` | Memory limit | `256Mi` |
| `hostNetwork` | Use host network | `true` |
| `hostPID` | Use host PID namespace | `true` |
| `prometheus.enabled` | Enable metrics | `true` |
| `prometheus.port` | Metrics port | `9100` |
| `serviceMonitor.enabled` | Create ServiceMonitor | `true` |
| `config.general.log_level` | Log level | `info` |

See `values.yaml` for all parameters.

## Prometheus Integration

### With Prometheus Operator

ServiceMonitor is automatically created if enabled:

```yaml
serviceMonitor:
  enabled: true
  labels:
    prometheus: kube-prometheus  # Match your Prometheus
```

### Without Prometheus Operator

Add scrape config to Prometheus:

```yaml
scrape_configs:
  - job_name: 'monitor-rs'
    kubernetes_sd_configs:
      - role: pod
        namespaces:
          names:
            - monitoring

    relabel_configs:
      - source_labels: [__meta_kubernetes_pod_label_app_kubernetes_io_name]
        action: keep
        regex: monitor-rs

      - source_labels: [__meta_kubernetes_pod_node_name]
        target_label: node

      - source_labels: [__address__]
        target_label: __address__
        regex: (.+):.*
        replacement: ${1}:9100
```

## Access Metrics

### From Within Cluster

```bash
# Via service
curl http://monitor-rs.monitoring.svc.cluster.local:9100/metrics

# Via pod (on specific node)
kubectl exec -n monitoring POD_NAME -- curl localhost:9100/metrics
```

### Port Forward (for testing)

```bash
# Forward from a specific pod
kubectl port-forward -n monitoring POD_NAME 9100:9100

# Access locally
curl http://localhost:9100/metrics
```

## Multi-Node Metrics

Monitor-RS runs as a DaemonSet, so you get one pod per node:

```bash
# View all instances
kubectl get pods -n monitoring -l app.kubernetes.io/name=monitor-rs -o wide

# Get metrics from specific node
kubectl exec -n monitoring $(kubectl get pod -n monitoring \
    -l app.kubernetes.io/name=monitor-rs \
    --field-selector spec.nodeName=NODE_NAME \
    -o name) -- curl -s localhost:9100/metrics
```

Example Prometheus query for multi-node CPU:

```promql
# Average CPU across all nodes
avg(cpu_usage_percent)

# CPU per node
cpu_usage_percent{node=~".+"}

# Top 5 nodes by CPU
topk(5, cpu_usage_percent)

# Per-core metrics across cluster
cpu_core_usage_percent{core="0"}
```

## Upgrading

```bash
# Update values
helm upgrade monitor-rs . \
    --namespace monitoring \
    --values custom-values.yaml

# Check rollout status
kubectl rollout status daemonset/monitor-rs -n monitoring
```

## Uninstalling

```bash
# Uninstall Helm release
helm uninstall monitor-rs --namespace monitoring

# Delete namespace (optional)
kubectl delete namespace monitoring
```

## Troubleshooting

### Pods not running

```bash
# Describe pod
kubectl describe pod -n monitoring POD_NAME

# Check events
kubectl get events -n monitoring --sort-by='.lastTimestamp'

# View logs
kubectl logs -n monitoring POD_NAME
```

### Metrics not scraped

```bash
# Check ServiceMonitor
kubectl get servicemonitor -n monitoring

# Test metrics endpoint
kubectl exec -n monitoring POD_NAME -- curl -s localhost:9100/metrics | head

# Check Prometheus targets
# Access Prometheus UI -> Status -> Targets
```

### RBAC issues

```bash
# Check service account
kubectl get serviceaccount -n monitoring

# Check cluster role
kubectl get clusterrole monitor-rs -o yaml

# Check binding
kubectl get clusterrolebinding monitor-rs -o yaml
```

### High resource usage

Adjust limits in values:

```yaml
resources:
  limits:
    cpu: 200m      # Lower if needed
    memory: 128Mi  # Lower if needed
```

## Production Best Practices

### 1. Resource Limits

Set appropriate limits based on cluster size:

```yaml
# Small clusters (< 10 nodes)
resources:
  limits:
    cpu: 500m
    memory: 256Mi

# Large clusters (> 50 nodes)
resources:
  limits:
    cpu: 200m
    memory: 128Mi
```

### 2. Node Selection

Deploy only on specific nodes:

```yaml
nodeSelector:
  monitoring: "true"

# Label nodes
kubectl label nodes node1 node2 monitoring=true
```

### 3. Tolerations

Ensure monitoring runs on all nodes including masters:

```yaml
tolerations:
  - effect: NoSchedule
    operator: Exists
  - effect: NoExecute
    operator: Exists
```

### 4. Priority Class

Set high priority for monitoring:

```yaml
priorityClassName: system-node-critical
```

### 5. Update Strategy

Control rollout during updates:

```yaml
updateStrategy:
  type: RollingUpdate
  rollingUpdate:
    maxUnavailable: 1  # Update one node at a time
```

### 6. Security

Use read-only root filesystem:

```yaml
securityContext:
  readOnlyRootFilesystem: true
  runAsNonRoot: true
  runAsUser: 1000
```

### 7. Monitoring the Monitor

Add alerts for monitor-rs itself:

```yaml
# Prometheus alert
- alert: MonitorRSDown
  expr: up{job="monitor-rs"} == 0
  for: 5m
  annotations:
    summary: "Monitor-RS is down on {{ $labels.node }}"
```

## Examples

### Complete Production Deployment

```bash
# production-values.yaml
cat <<EOF > production-values.yaml
image:
  repository: ghcr.io/ericgitangu/monitor-rs
  tag: "0.1.0"
  pullPolicy: Always

resources:
  limits:
    cpu: 300m
    memory: 256Mi
  requests:
    cpu: 100m
    memory: 128Mi

tolerations:
  - effect: NoSchedule
    operator: Exists
  - effect: NoExecute
    operator: Exists

priorityClassName: system-node-critical

serviceMonitor:
  enabled: true
  labels:
    prometheus: kube-prometheus
  interval: 30s

config:
  general:
    log_level: "warn"
    update_interval: "5s"

  services:
    redis:
      enabled: true
      instances:
        - name: "main"
          host: "redis.default.svc.cluster.local"
          port: 6379
EOF

# Deploy
helm install monitor-rs . \
    --namespace monitoring \
    --create-namespace \
    --values production-values.yaml

# Verify
kubectl get daemonset -n monitoring
kubectl get pods -n monitoring -l app.kubernetes.io/name=monitor-rs
```

### Database Monitoring

```yaml
config:
  services:
    mysql:
      enabled: true
      instances:
        - name: "main"
          host: "mysql.default.svc.cluster.local"
          port: 3306
          username: "monitor"
          password: "${MYSQL_PASSWORD}"  # Use secret

    postgresql:
      enabled: true
      instances:
        - name: "accounts"
          host: "postgres.default.svc.cluster.local"
          port: 5432
          username: "monitor"
          password: "${PG_PASSWORD}"
          database: "postgres"
```

### Queue Monitoring (Sidekiq)

```yaml
config:
  services:
    sidekiq:
      enabled: true
      redis_url: "redis://redis.default.svc.cluster.local:6379/0"
      namespace: "sidekiq"
      queues:
        - "default"
        - "mailers"
        - "high_priority"
        - "low_priority"
```

## Integration with Grafana

Import dashboard from `examples/grafana-dashboard.json`:

1. Access Grafana
2. Go to Dashboards → Import
3. Upload `grafana-dashboard.json`
4. Select Prometheus datasource
5. View cluster-wide metrics

## Next Steps

- Set up alerting rules (see `examples/monitor-rs-alerts.yml`)
- Create Grafana dashboards for your metrics
- Configure database/queue monitoring
- Set up log aggregation for monitor-rs logs
- Implement custom service detection rules
