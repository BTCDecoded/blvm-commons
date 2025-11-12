# Production Monitoring Guide

This guide outlines the comprehensive monitoring strategy for the BTCDecoded governance system in production.

## Overview

Effective monitoring is essential for maintaining system reliability, security, and performance. This guide covers all aspects of monitoring from infrastructure to application-level metrics.

## Monitoring Architecture

### Core Components

- **Prometheus**: Metrics collection and storage
- **Grafana**: Visualization and dashboards
- **AlertManager**: Alert routing and notification
- **ELK Stack**: Log aggregation and analysis
- **Jaeger**: Distributed tracing
- **Uptime Robot**: External monitoring

### Data Flow

```
Applications → Prometheus → Grafana
     ↓
   Logs → ELK Stack → Kibana
     ↓
  Traces → Jaeger → Grafana
     ↓
  Alerts → AlertManager → Notifications
```

## Infrastructure Monitoring

### System Metrics

#### CPU Monitoring
- **CPU Usage**: Overall CPU utilization
- **CPU Load**: 1, 5, 15 minute load averages
- **CPU Cores**: Per-core utilization
- **CPU Temperature**: Hardware temperature monitoring

#### Memory Monitoring
- **Memory Usage**: Total memory utilization
- **Memory Available**: Available memory
- **Swap Usage**: Swap file utilization
- **Memory Pressure**: Memory pressure indicators

#### Disk Monitoring
- **Disk Usage**: Disk space utilization
- **Disk I/O**: Read/write operations
- **Disk Latency**: I/O response times
- **Disk Health**: SMART status monitoring

#### Network Monitoring
- **Network Traffic**: Inbound/outbound traffic
- **Network Latency**: Round-trip times
- **Network Errors**: Packet loss, errors
- **Network Connections**: Active connections

### Database Monitoring

#### PostgreSQL Metrics
- **Connection Count**: Active connections
- **Query Performance**: Query execution times
- **Lock Contention**: Database locks
- **Replication Lag**: Replication delay
- **Cache Hit Ratio**: Buffer cache efficiency
- **Transaction Rate**: Transactions per second
- **Deadlocks**: Deadlock occurrences

#### SQLite Metrics
- **Database Size**: Database file size
- **Page Count**: Total pages
- **Free Pages**: Available pages
- **Cache Hit Ratio**: Page cache efficiency
- **WAL Size**: Write-ahead log size
- **Checkpoint Frequency**: Checkpoint operations

## Application Monitoring

### Governance App Metrics

#### Request Metrics
- **Request Rate**: Requests per second
- **Request Duration**: Response times
- **Request Size**: Payload sizes
- **Error Rate**: Error percentage
- **Status Codes**: HTTP status code distribution

#### Business Metrics
- **PRs Processed**: Pull requests processed
- **Signatures Collected**: Signatures received
- **Economic Nodes**: Active economic nodes
- **Vetoes**: Economic node vetoes
- **Governance Events**: Governance actions

#### Performance Metrics
- **Memory Usage**: Application memory
- **Goroutine Count**: Active goroutines
- **GC Pauses**: Garbage collection pauses
- **Heap Size**: Memory heap size
- **CPU Usage**: Application CPU usage

### GitHub Integration Metrics

#### Webhook Metrics
- **Webhook Rate**: Webhooks received per second
- **Webhook Processing Time**: Processing duration
- **Webhook Failures**: Failed webhook processing
- **Signature Validation**: Signature validation success rate
- **Event Types**: Webhook event type distribution

#### API Metrics
- **API Rate**: GitHub API calls per second
- **API Quota**: Rate limit utilization
- **API Errors**: API error rates
- **Response Times**: API response times
- **Retry Attempts**: API retry attempts

### Economic Node Metrics

#### Node Metrics
- **Active Nodes**: Number of active nodes
- **Node Types**: Distribution by type
- **Node Weights**: Total weight distribution
- **Node Health**: Node health status
- **Node Updates**: Node update frequency

#### Veto Metrics
- **Veto Rate**: Vetoes per hour
- **Veto Weight**: Total veto weight
- **Veto Duration**: Veto processing time
- **Veto Success**: Successful veto processing
- **Veto Failures**: Failed veto processing

## Security Monitoring

### Authentication Metrics
- **Login Attempts**: Authentication attempts
- **Failed Logins**: Failed authentication
- **Account Lockouts**: Account lockout events
- **Password Changes**: Password change events
- **MFA Usage**: Multi-factor authentication usage

### Authorization Metrics
- **Access Attempts**: Authorization attempts
- **Permission Denials**: Access denied events
- **Privilege Escalation**: Privilege escalation attempts
- **Role Changes**: Role modification events
- **Access Patterns**: Unusual access patterns

### Security Events
- **Intrusion Attempts**: Intrusion detection alerts
- **Malware Detection**: Malware alerts
- **Suspicious Activity**: Anomaly detection
- **Data Exfiltration**: Data loss prevention
- **Compliance Violations**: Policy violations

## Log Monitoring

### Application Logs

#### Log Levels
- **ERROR**: Error conditions
- **WARN**: Warning conditions
- **INFO**: Informational messages
- **DEBUG**: Debug information
- **TRACE**: Trace information

#### Log Categories
- **Access Logs**: HTTP access logs
- **Error Logs**: Application errors
- **Audit Logs**: Security audit logs
- **Performance Logs**: Performance metrics
- **Business Logs**: Business logic logs

### System Logs

#### Operating System
- **System Events**: System-level events
- **Kernel Messages**: Kernel log messages
- **Service Logs**: Service-specific logs
- **Cron Logs**: Scheduled task logs
- **Boot Logs**: System boot logs

#### Security Logs
- **Authentication Logs**: Login/logout events
- **Authorization Logs**: Access control events
- **Firewall Logs**: Network security events
- **Intrusion Logs**: Intrusion detection events
- **Compliance Logs**: Compliance-related events

## Alerting Strategy

### Alert Severity Levels

#### Critical (P0)
- **System Down**: Complete system outage
- **Security Breach**: Confirmed security breach
- **Data Loss**: Data corruption or loss
- **Key Compromise**: Key security compromise
- **Service Unavailable**: Critical service unavailable

#### High (P1)
- **Performance Degradation**: Significant performance issues
- **High Error Rate**: Elevated error rates
- **Resource Exhaustion**: Resource utilization high
- **Security Alerts**: High-priority security alerts
- **Service Degradation**: Service quality issues

#### Medium (P2)
- **Warning Conditions**: Warning-level conditions
- **Capacity Issues**: Capacity planning alerts
- **Configuration Issues**: Configuration problems
- **Minor Security Alerts**: Low-priority security alerts
- **Performance Warnings**: Performance warnings

#### Low (P3)
- **Informational**: Informational alerts
- **Maintenance**: Maintenance notifications
- **Capacity Planning**: Capacity planning alerts
- **Trend Alerts**: Trend-based alerts
- **Status Updates**: Status change notifications

### Alert Rules

#### Infrastructure Alerts
```yaml
# CPU Usage Alert
- alert: HighCPUUsage
  expr: cpu_usage_percent > 80
  for: 5m
  labels:
    severity: warning
  annotations:
    summary: "High CPU usage detected"
    description: "CPU usage is above 80% for 5 minutes"

# Memory Usage Alert
- alert: HighMemoryUsage
  expr: memory_usage_percent > 90
  for: 2m
  labels:
    severity: critical
  annotations:
    summary: "High memory usage detected"
    description: "Memory usage is above 90% for 2 minutes"

# Disk Space Alert
- alert: LowDiskSpace
  expr: disk_usage_percent > 85
  for: 1m
  labels:
    severity: warning
  annotations:
    summary: "Low disk space detected"
    description: "Disk usage is above 85%"
```

#### Application Alerts
```yaml
# High Error Rate Alert
- alert: HighErrorRate
  expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.1
  for: 2m
  labels:
    severity: critical
  annotations:
    summary: "High error rate detected"
    description: "Error rate is above 10% for 2 minutes"

# High Response Time Alert
- alert: HighResponseTime
  expr: histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m])) > 1
  for: 5m
  labels:
    severity: warning
  annotations:
    summary: "High response time detected"
    description: "95th percentile response time is above 1 second"

# Service Down Alert
- alert: ServiceDown
  expr: up == 0
  for: 1m
  labels:
    severity: critical
  annotations:
    summary: "Service is down"
    description: "Service has been down for 1 minute"
```

### Notification Channels

#### Email Notifications
- **Critical Alerts**: Immediate email to on-call team
- **High Alerts**: Email to relevant team members
- **Medium Alerts**: Email to team leads
- **Low Alerts**: Daily digest email

#### Slack Notifications
- **Critical Alerts**: Immediate Slack message to #alerts
- **High Alerts**: Slack message to #alerts
- **Medium Alerts**: Slack message to #monitoring
- **Low Alerts**: Daily digest to #monitoring

#### PagerDuty Integration
- **Critical Alerts**: Immediate PagerDuty escalation
- **High Alerts**: PagerDuty escalation after 15 minutes
- **Medium Alerts**: PagerDuty escalation after 1 hour
- **Low Alerts**: No PagerDuty escalation

## Dashboard Configuration

### Infrastructure Dashboard

#### System Overview
- **CPU Usage**: CPU utilization over time
- **Memory Usage**: Memory utilization over time
- **Disk Usage**: Disk space utilization
- **Network Traffic**: Network I/O over time
- **Load Average**: System load over time

#### Database Overview
- **Connection Count**: Database connections
- **Query Performance**: Query execution times
- **Cache Hit Ratio**: Cache efficiency
- **Lock Contention**: Database locks
- **Replication Lag**: Replication delay

### Application Dashboard

#### Request Metrics
- **Request Rate**: Requests per second
- **Response Time**: Response time percentiles
- **Error Rate**: Error rate over time
- **Status Codes**: HTTP status code distribution
- **Request Size**: Payload size distribution

#### Business Metrics
- **PRs Processed**: Pull requests over time
- **Signatures Collected**: Signatures over time
- **Economic Nodes**: Active nodes over time
- **Vetoes**: Veto events over time
- **Governance Events**: Governance actions over time

### Security Dashboard

#### Authentication Metrics
- **Login Attempts**: Authentication attempts over time
- **Failed Logins**: Failed authentication over time
- **Account Lockouts**: Lockout events over time
- **MFA Usage**: Multi-factor authentication usage
- **Access Patterns**: Unusual access patterns

#### Security Events
- **Intrusion Attempts**: Intrusion detection alerts
- **Malware Detection**: Malware alerts
- **Suspicious Activity**: Anomaly detection
- **Compliance Violations**: Policy violations
- **Security Score**: Overall security health

## Monitoring Tools Setup

### Prometheus Configuration

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  - "alert_rules.yml"

alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - alertmanager:9093

scrape_configs:
  - job_name: 'governance-app'
    static_configs:
      - targets: ['governance-app:8080']
    metrics_path: '/metrics'
    scrape_interval: 5s

  - job_name: 'postgresql'
    static_configs:
      - targets: ['postgresql:5432']
    scrape_interval: 30s

  - job_name: 'node-exporter'
    static_configs:
      - targets: ['node-exporter:9100']
    scrape_interval: 15s
```

### Grafana Configuration

```yaml
datasources:
  - name: Prometheus
    type: prometheus
    access: proxy
    url: http://prometheus:9090
    isDefault: true

dashboards:
  - name: Infrastructure
    path: /etc/grafana/provisioning/dashboards/infrastructure.json
  - name: Application
    path: /etc/grafana/provisioning/dashboards/application.json
  - name: Security
    path: /etc/grafana/provisioning/dashboards/security.json
```

### ELK Stack Configuration

```yaml
# Elasticsearch
elasticsearch:
  image: docker.elastic.co/elasticsearch/elasticsearch:8.8.0
  environment:
    - discovery.type=single-node
    - xpack.security.enabled=false
  ports:
    - "9200:9200"

# Logstash
logstash:
  image: docker.elastic.co/logstash/logstash:8.8.0
  volumes:
    - ./logstash.conf:/usr/share/logstash/pipeline/logstash.conf
  ports:
    - "5044:5044"

# Kibana
kibana:
  image: docker.elastic.co/kibana/kibana:8.8.0
  environment:
    - ELASTICSEARCH_HOSTS=http://elasticsearch:9200
  ports:
    - "5601:5601"
```

## Performance Monitoring

### Key Performance Indicators (KPIs)

#### Availability
- **Uptime**: System availability percentage
- **MTTR**: Mean Time To Recovery
- **MTBF**: Mean Time Between Failures
- **SLA Compliance**: Service Level Agreement compliance

#### Performance
- **Response Time**: Average response time
- **Throughput**: Requests per second
- **Error Rate**: Error percentage
- **Resource Utilization**: CPU, memory, disk usage

#### Security
- **Security Incidents**: Number of security incidents
- **Vulnerability Count**: Open vulnerabilities
- **Compliance Score**: Compliance rating
- **Audit Findings**: Audit findings count

### Performance Baselines

#### Normal Operation
- **Response Time**: < 200ms (95th percentile)
- **Error Rate**: < 0.1%
- **CPU Usage**: < 70%
- **Memory Usage**: < 80%
- **Disk Usage**: < 80%

#### Peak Load
- **Response Time**: < 500ms (95th percentile)
- **Error Rate**: < 1%
- **CPU Usage**: < 90%
- **Memory Usage**: < 90%
- **Disk Usage**: < 85%

## Capacity Planning

### Resource Forecasting

#### CPU Forecasting
- **Historical Usage**: Analyze CPU usage trends
- **Growth Rate**: Calculate growth rate
- **Peak Usage**: Identify peak usage patterns
- **Future Requirements**: Project future needs

#### Memory Forecasting
- **Memory Growth**: Track memory growth over time
- **Peak Memory**: Identify peak memory usage
- **Memory Leaks**: Detect memory leaks
- **Future Requirements**: Project memory needs

#### Storage Forecasting
- **Data Growth**: Track data growth rate
- **Log Growth**: Monitor log file growth
- **Backup Growth**: Track backup storage needs
- **Future Requirements**: Project storage needs

### Scaling Strategies

#### Horizontal Scaling
- **Load Balancing**: Distribute load across instances
- **Auto-scaling**: Automatically scale based on load
- **Microservices**: Break down monolithic applications
- **Container Orchestration**: Use Kubernetes or similar

#### Vertical Scaling
- **CPU Upgrade**: Increase CPU capacity
- **Memory Upgrade**: Increase memory capacity
- **Storage Upgrade**: Increase storage capacity
- **Network Upgrade**: Increase network capacity

## Troubleshooting Guide

### Common Issues

#### High CPU Usage
1. **Check Process List**: Identify high CPU processes
2. **Analyze CPU Profiling**: Use profiling tools
3. **Check for Loops**: Look for infinite loops
4. **Optimize Code**: Optimize inefficient code

#### High Memory Usage
1. **Check Memory Usage**: Identify memory consumers
2. **Analyze Memory Leaks**: Use memory profiling tools
3. **Check Garbage Collection**: Analyze GC behavior
4. **Optimize Memory Usage**: Reduce memory footprint

#### High Disk Usage
1. **Check Disk Space**: Identify space consumers
2. **Analyze Log Files**: Check log file sizes
3. **Check Database Size**: Monitor database growth
4. **Implement Log Rotation**: Rotate log files

#### Network Issues
1. **Check Network Connectivity**: Test network connections
2. **Analyze Network Traffic**: Use network monitoring tools
3. **Check DNS Resolution**: Verify DNS configuration
4. **Check Firewall Rules**: Verify firewall configuration

### Diagnostic Tools

#### System Tools
- **top/htop**: Process monitoring
- **iostat**: I/O statistics
- **netstat**: Network statistics
- **ss**: Socket statistics
- **lsof**: List open files

#### Application Tools
- **pprof**: Go profiling
- **strace**: System call tracing
- **tcpdump**: Network packet analysis
- **wireshark**: Network protocol analysis
- **jq**: JSON processing

## Maintenance Procedures

### Daily Maintenance
- **Check System Health**: Review system metrics
- **Review Alerts**: Check for new alerts
- **Check Logs**: Review error logs
- **Verify Backups**: Ensure backups are working

### Weekly Maintenance
- **Performance Review**: Analyze performance metrics
- **Capacity Planning**: Review capacity trends
- **Security Review**: Check security metrics
- **Update Documentation**: Update monitoring docs

### Monthly Maintenance
- **Dashboard Review**: Review and update dashboards
- **Alert Tuning**: Tune alert thresholds
- **Capacity Planning**: Plan for future capacity
- **Tool Updates**: Update monitoring tools

## Conclusion

Effective monitoring is essential for maintaining system reliability, security, and performance. This guide provides comprehensive coverage of all monitoring aspects, from infrastructure to application-level metrics.

Regular review and updates of monitoring configurations, alert rules, and dashboards are essential for maintaining effective monitoring capabilities. Continuous improvement based on lessons learned and changing requirements will ensure the monitoring system remains effective and relevant.
