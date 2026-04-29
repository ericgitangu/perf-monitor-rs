# Security Policy

## Reporting a vulnerability

If you discover a security issue, please email **developer.ericgitangu@gmail.com**
with a clear description and reproduction steps. Do not open a public GitHub
issue for security concerns.

You'll receive an acknowledgement within 72 hours. Once a fix is in place,
the original report will be credited (unless you prefer to remain anonymous)
and the fix released as a patch.

## Scope

`perf-monitor-rs` runs on user infrastructure with elevated privileges
(reading process tables, querying databases, exposing Prometheus
endpoints). Reports about credential leakage, privilege escalation, or
metric-injection vectors are particularly welcome.
