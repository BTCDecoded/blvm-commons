# Production Maintenance Guide

This guide outlines comprehensive maintenance procedures for the BTCDecoded governance system in production.

## Maintenance Schedule

### Daily Maintenance (5 minutes)

1. **System Health Check**
   ```bash
   # Check service status
   sudo systemctl status governance-app
   sudo systemctl status postgresql
   sudo systemctl status nginx
   
   # Check disk space
   df -h
   
   # Check memory usage
   free -h
   
   # Check CPU load
   uptime
   ```

2. **Log Review**
   ```bash
   # Check error logs
   sudo journalctl -u governance-app --since "1 day ago" | grep ERROR
   
   # Check security logs
   sudo grep "Failed password" /var/log/auth.log | tail -10
   
   # Check application logs
   tail -100 /var/log/governance-app/application.log
   ```

3. **Backup Verification**
   ```bash
   # Check backup status
   ls -la /opt/backups/
   
   # Verify latest backup
   ./scripts/verify-backup.sh
   ```

### Weekly Maintenance (30 minutes)

1. **Performance Review**
   ```bash
   # Check system performance
   iostat -x 1 5
   vmstat 1 5
   netstat -i
   
   # Check database performance
   sudo -u postgres psql -c "SELECT * FROM pg_stat_activity;"
   sudo -u postgres psql -c "SELECT * FROM pg_stat_database;"
   ```

2. **Security Review**
   ```bash
   # Check for security updates
   sudo apt list --upgradable
   
   # Check failed login attempts
   sudo grep "Failed password" /var/log/auth.log | wc -l
   
   # Check firewall status
   sudo ufw status
   ```

3. **Log Rotation**
   ```bash
   # Rotate application logs
   sudo logrotate -f /etc/logrotate.d/governance-app
   
   # Clean old logs
   find /var/log -name "*.log.*" -mtime +30 -delete
   ```

### Monthly Maintenance (2 hours)

1. **System Updates**
   ```bash
   # Update system packages
   sudo apt update
   sudo apt upgrade -y
   
   # Update application
   cd /opt/governance-app
   git pull
   cargo build --release
   sudo systemctl restart governance-app
   ```

2. **Database Maintenance**
   ```bash
   # PostgreSQL maintenance
   sudo -u postgres psql -c "VACUUM ANALYZE;"
   sudo -u postgres psql -c "REINDEX DATABASE governance_production;"
   
   # SQLite maintenance
   sqlite3 /opt/governance-app/data/governance.db "VACUUM;"
   sqlite3 /opt/governance-app/data/governance.db "ANALYZE;"
   ```

3. **Security Audit**
   ```bash
   # Run security scan
   sudo lynis audit system
   
   # Check for vulnerabilities
   sudo apt audit
   
   # Review access logs
   sudo grep "sudo" /var/log/auth.log | tail -20
   ```

### Quarterly Maintenance (4 hours)

1. **Comprehensive Review**
   - Review all system metrics
   - Analyze performance trends
   - Update documentation
   - Review security policies

2. **Capacity Planning**
   - Analyze resource usage
   - Plan for future growth
   - Update monitoring thresholds
   - Review backup strategies

3. **Disaster Recovery Testing**
   - Test backup restoration
   - Verify recovery procedures
   - Update recovery documentation
   - Train staff on procedures

## System Updates

### Operating System Updates

1. **Pre-Update Checklist**
   ```bash
   # Create system snapshot
   sudo lvcreate -L 10G -s -n system_snapshot /dev/vg0/root
   
   # Backup critical files
   sudo cp -r /etc /opt/backups/etc_$(date +%Y%m%d)
   sudo cp -r /opt/governance-app /opt/backups/app_$(date +%Y%m%d)
   ```

2. **Update Process**
   ```bash
   # Update package lists
   sudo apt update
   
   # Check what will be updated
   sudo apt list --upgradable
   
   # Perform update
   sudo apt upgrade -y
   
   # Reboot if required
   sudo reboot
   ```

3. **Post-Update Verification**
   ```bash
   # Check service status
   sudo systemctl status governance-app
   
   # Verify functionality
   curl -f http://localhost:8080/health
   
   # Check logs for errors
   sudo journalctl -u governance-app --since "10 minutes ago"
   ```

### Application Updates

1. **Pre-Update Preparation**
   ```bash
   # Stop application
   sudo systemctl stop governance-app
   
   # Backup current version
   sudo cp /opt/governance-app/governance-app /opt/governance-app/governance-app.backup
   
   # Backup configuration
   sudo cp -r /opt/governance-app/config /opt/governance-app/config.backup
   ```

2. **Update Process**
   ```bash
   # Pull latest code
   cd /opt/governance-app
   git pull origin main
   
   # Build new version
   cargo build --release
   
   # Run database migrations
   cargo run --bin governance-app -- --migrate
   
   # Start application
   sudo systemctl start governance-app
   ```

3. **Post-Update Verification**
   ```bash
   # Check service status
   sudo systemctl status governance-app
   
   # Verify health endpoints
   curl -f http://localhost:8080/health
   curl -f http://localhost:8080/health/db
   curl -f http://localhost:8080/health/github
   
   # Test functionality
   ./scripts/test-functionality.sh
   ```

## Database Maintenance

### PostgreSQL Maintenance

1. **Regular Maintenance**
   ```bash
   # Vacuum database
   sudo -u postgres psql -d governance_production -c "VACUUM;"
   
   # Analyze statistics
   sudo -u postgres psql -d governance_production -c "ANALYZE;"
   
   # Reindex database
   sudo -u postgres psql -d governance_production -c "REINDEX DATABASE governance_production;"
   ```

2. **Performance Tuning**
   ```bash
   # Check database size
   sudo -u postgres psql -c "SELECT pg_size_pretty(pg_database_size('governance_production'));"
   
   # Check table sizes
   sudo -u postgres psql -d governance_production -c "SELECT schemaname,tablename,pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) as size FROM pg_tables ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;"
   
   # Check index usage
   sudo -u postgres psql -d governance_production -c "SELECT schemaname,tablename,indexname,idx_scan,idx_tup_read,idx_tup_fetch FROM pg_stat_user_indexes ORDER BY idx_scan DESC;"
   ```

3. **Backup and Recovery**
   ```bash
   # Create backup
   sudo -u postgres pg_dump -d governance_production -f /opt/backups/postgresql/governance_$(date +%Y%m%d_%H%M%S).sql
   
   # Test backup
   sudo -u postgres psql -d governance_test -f /opt/backups/postgresql/governance_$(date +%Y%m%d_%H%M%S).sql
   ```

### SQLite Maintenance

1. **Regular Maintenance**
   ```bash
   # Vacuum database
   sqlite3 /opt/governance-app/data/governance.db "VACUUM;"
   
   # Analyze database
   sqlite3 /opt/governance-app/data/governance.db "ANALYZE;"
   
   # Check integrity
   sqlite3 /opt/governance-app/data/governance.db "PRAGMA integrity_check;"
   ```

2. **Performance Optimization**
   ```bash
   # Check database size
   ls -lh /opt/governance-app/data/governance.db
   
   # Check WAL file size
   ls -lh /opt/governance-app/data/governance.db-wal
   
   # Checkpoint WAL file
   sqlite3 /opt/governance-app/data/governance.db "PRAGMA wal_checkpoint(FULL);"
   ```

## Security Maintenance

### Security Updates

1. **System Security Updates**
   ```bash
   # Check for security updates
   sudo apt list --upgradable | grep -i security
   
   # Install security updates
   sudo apt update
   sudo apt upgrade -y
   
   # Check for kernel updates
   uname -r
   sudo apt list --installed | grep linux-image
   ```

2. **Application Security Updates**
   ```bash
   # Update Rust dependencies
   cd /opt/governance-app
   cargo audit
   cargo update
   
   # Update system dependencies
   sudo apt update
   sudo apt upgrade -y
   ```

### Security Monitoring

1. **Log Analysis**
   ```bash
   # Check failed login attempts
   sudo grep "Failed password" /var/log/auth.log | tail -20
   
   # Check sudo usage
   sudo grep "sudo" /var/log/auth.log | tail -20
   
   # Check system errors
   sudo journalctl -p err --since "1 day ago"
   ```

2. **Vulnerability Scanning**
   ```bash
   # Run Lynis security audit
   sudo lynis audit system
   
   # Check for known vulnerabilities
   sudo apt audit
   
   # Scan for malware
   sudo clamscan -r /opt/governance-app
   ```

## Performance Monitoring

### System Performance

1. **Resource Monitoring**
   ```bash
   # Check CPU usage
   top -bn1 | grep "Cpu(s)"
   
   # Check memory usage
   free -h
   
   # Check disk I/O
   iostat -x 1 5
   
   # Check network usage
   netstat -i
   ```

2. **Application Performance**
   ```bash
   # Check application metrics
   curl -s http://localhost:8080/metrics | grep -E "(http_requests_total|http_request_duration_seconds)"
   
   # Check database performance
   sudo -u postgres psql -c "SELECT * FROM pg_stat_activity WHERE state = 'active';"
   
   # Check log performance
   tail -1000 /var/log/governance-app/application.log | grep -E "(ERROR|WARN|slow)"
   ```

### Performance Optimization

1. **Database Optimization**
   ```bash
   # Update table statistics
   sudo -u postgres psql -d governance_production -c "ANALYZE;"
   
   # Rebuild indexes
   sudo -u postgres psql -d governance_production -c "REINDEX DATABASE governance_production;"
   
   # Check for unused indexes
   sudo -u postgres psql -d governance_production -c "SELECT schemaname,tablename,indexname,idx_scan FROM pg_stat_user_indexes WHERE idx_scan = 0;"
   ```

2. **Application Optimization**
   ```bash
   # Check application configuration
   cat /opt/governance-app/config/production.toml
   
   # Check environment variables
   env | grep -E "(GOVERNANCE|DATABASE|GITHUB)"
   
   # Check system limits
   ulimit -a
   ```

## Backup Maintenance

### Backup Verification

1. **Daily Verification**
   ```bash
   # Check backup age
   find /opt/backups -name "*.gz" -mtime -1 -ls
   
   # Verify backup integrity
   ./scripts/verify-backup.sh
   
   # Check backup size
   du -sh /opt/backups/*
   ```

2. **Weekly Verification**
   ```bash
   # Test backup restoration
   ./scripts/test-backup-restore.sh
   
   # Check backup retention
   find /opt/backups -name "*.gz" -mtime +30 -ls
   
   # Clean old backups
   find /opt/backups -name "*.gz" -mtime +30 -delete
   ```

### Backup Optimization

1. **Compression Optimization**
   ```bash
   # Check compression ratios
   ls -lh /opt/backups/*.gz | awk '{print $5, $9}'
   
   # Test different compression levels
   gzip -9 -c /opt/governance-app/data/governance.db > test_compression.gz
   ```

2. **Storage Optimization**
   ```bash
   # Check disk usage
   df -h /opt/backups
   
   # Move old backups to archive
   find /opt/backups -name "*.gz" -mtime +90 -exec mv {} /opt/archive/ \;
   
   # Compress old backups
   find /opt/backups -name "*.sql" -mtime +7 -exec gzip {} \;
   ```

## Monitoring Maintenance

### Monitoring System Health

1. **Prometheus Maintenance**
   ```bash
   # Check Prometheus status
   sudo systemctl status prometheus
   
   # Check Prometheus configuration
   promtool check config /etc/prometheus/prometheus.yml
   
   # Check Prometheus targets
   curl -s http://localhost:9090/api/v1/targets | jq '.data.activeTargets[] | select(.health != "up")'
   ```

2. **Grafana Maintenance**
   ```bash
   # Check Grafana status
   sudo systemctl status grafana-server
   
   # Check Grafana configuration
   grafana-cli admin reset-admin-password newpassword
   
   # Backup Grafana configuration
   sudo cp -r /var/lib/grafana /opt/backups/grafana_$(date +%Y%m%d)
   ```

### Alert Management

1. **Alert Review**
   ```bash
   # Check active alerts
   curl -s http://localhost:9090/api/v1/alerts | jq '.data.alerts[] | select(.state == "firing")'
   
   # Check alert rules
   promtool check rules /etc/prometheus/alert_rules.yml
   
   # Test alert rules
   promtool test rules /etc/prometheus/alert_rules.yml
   ```

2. **Alert Tuning**
   ```bash
   # Review alert thresholds
   grep -r "expr:" /etc/prometheus/alert_rules.yml
   
   # Update alert rules
   sudo nano /etc/prometheus/alert_rules.yml
   sudo systemctl reload prometheus
   ```

## Troubleshooting

### Common Issues

1. **Service Won't Start**
   ```bash
   # Check service status
   sudo systemctl status governance-app
   
   # Check logs
   sudo journalctl -u governance-app -n 50
   
   # Check configuration
   sudo -u governance /opt/governance-app/governance-app --config-check
   ```

2. **Database Connection Issues**
   ```bash
   # Check database status
   sudo systemctl status postgresql
   
   # Check database logs
   sudo journalctl -u postgresql -n 50
   
   # Test database connection
   sudo -u postgres psql -c "SELECT 1;"
   ```

3. **Performance Issues**
   ```bash
   # Check system resources
   top -bn1
   free -h
   df -h
   
   # Check application metrics
   curl -s http://localhost:8080/metrics
   
   # Check database performance
   sudo -u postgres psql -c "SELECT * FROM pg_stat_activity;"
   ```

### Emergency Procedures

1. **Service Outage**
   ```bash
   # Restart services
   sudo systemctl restart governance-app
   sudo systemctl restart postgresql
   sudo systemctl restart nginx
   
   # Check service status
   sudo systemctl status governance-app
   ```

2. **Database Issues**
   ```bash
   # Restart database
   sudo systemctl restart postgresql
   
   # Check database integrity
   sudo -u postgres psql -c "SELECT pg_is_in_recovery();"
   
   # Restore from backup if needed
   ./scripts/restore-database.sh
   ```

3. **Security Incident**
   ```bash
   # Isolate system
   sudo ufw deny in
   sudo ufw deny out
   
   # Preserve evidence
   sudo cp -r /var/log /opt/evidence/logs_$(date +%Y%m%d_%H%M%S)
   
   # Notify security team
   echo "Security incident detected" | mail -s "Security Alert" security@btcdecoded.org
   ```

## Documentation Updates

### Maintenance Documentation

1. **Update Procedures**
   - Document new procedures
   - Update existing procedures
   - Remove outdated procedures
   - Validate procedure accuracy

2. **Knowledge Base**
   - Add troubleshooting guides
   - Update FAQ
   - Document lessons learned
   - Share best practices

3. **Training Materials**
   - Update training guides
   - Create new training materials
   - Conduct training sessions
   - Assess training effectiveness

## Conclusion

Regular maintenance is essential for maintaining system reliability, security, and performance. This guide provides comprehensive coverage of maintenance procedures, but it should be regularly updated based on lessons learned and changing requirements.

Continuous improvement of maintenance procedures, regular training of maintenance staff, and effective monitoring of system health are essential for maintaining effective maintenance capabilities.
