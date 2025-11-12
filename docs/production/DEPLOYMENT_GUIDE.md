# Production Deployment Guide

This guide provides step-by-step instructions for deploying the BTCDecoded governance system to production.

## Prerequisites

### System Requirements

- **Operating System**: Ubuntu 22.04 LTS or RHEL 8+
- **CPU**: 4+ cores, 2.4GHz+
- **Memory**: 8GB+ RAM
- **Storage**: 100GB+ SSD storage
- **Network**: Stable internet connection with static IP

### Software Requirements

- **Docker**: 20.10+
- **Docker Compose**: 2.0+
- **Rust**: 1.70+
- **PostgreSQL**: 14+ (if using PostgreSQL backend)
- **Nginx**: 1.18+ (for reverse proxy)
- **Certbot**: For SSL certificates

### Security Requirements

- **Firewall**: Configured with minimal required ports
- **SSH**: Key-based authentication only
- **Updates**: System fully updated and patched
- **Monitoring**: Security monitoring enabled
- **Backups**: Automated backup system configured

## Pre-Deployment Checklist

### 1. Security Hardening

- [ ] System hardened according to security guidelines
- [ ] Firewall configured with minimal required ports
- [ ] SSH configured for key-based authentication only
- [ ] Fail2ban installed and configured
- [ ] Log monitoring configured
- [ ] Intrusion detection system enabled

### 2. Network Configuration

- [ ] Static IP address assigned
- [ ] DNS records configured
- [ ] SSL certificates obtained and installed
- [ ] Load balancer configured (if applicable)
- [ ] CDN configured (if applicable)

### 3. Database Setup

- [ ] PostgreSQL installed and configured (if using PostgreSQL)
- [ ] Database user created with minimal privileges
- [ ] Database configured for production use
- [ ] Backup system configured
- [ ] Monitoring configured

### 4. Key Management

- [ ] Production keys generated via key ceremony
- [ ] Keys securely stored and distributed
- [ ] Key rotation schedule established
- [ ] Emergency key recovery procedures tested

## Deployment Steps

### Step 1: System Preparation

1. **Update System**
   ```bash
   sudo apt update && sudo apt upgrade -y
   ```

2. **Install Required Software**
   ```bash
   # Install Docker
   curl -fsSL https://get.docker.com -o get-docker.sh
   sudo sh get-docker.sh
   sudo usermod -aG docker $USER

   # Install Docker Compose
   sudo curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
   sudo chmod +x /usr/local/bin/docker-compose

   # Install Nginx
   sudo apt install nginx -y

   # Install Certbot
   sudo apt install certbot python3-certbot-nginx -y
   ```

3. **Configure Firewall**
   ```bash
   sudo ufw enable
   sudo ufw allow ssh
   sudo ufw allow 80
   sudo ufw allow 443
   sudo ufw allow 8080  # For governance-app
   ```

### Step 2: Database Setup

1. **Install PostgreSQL** (if using PostgreSQL backend)
   ```bash
   sudo apt install postgresql postgresql-contrib -y
   sudo systemctl start postgresql
   sudo systemctl enable postgresql
   ```

2. **Create Database and User**
   ```bash
   sudo -u postgres psql
   CREATE DATABASE governance_production;
   CREATE USER governance_user WITH PASSWORD 'secure_password';
   GRANT ALL PRIVILEGES ON DATABASE governance_production TO governance_user;
   \q
   ```

3. **Configure PostgreSQL for Production**
   ```bash
   sudo nano /etc/postgresql/14/main/postgresql.conf
   # Set appropriate values for:
   # - shared_buffers
   # - effective_cache_size
   # - maintenance_work_mem
   # - checkpoint_completion_target
   # - wal_buffers
   # - default_statistics_target
   ```

### Step 3: Application Deployment

1. **Clone Repository**
   ```bash
   git clone https://github.com/btcdecoded/governance-app.git
   cd governance-app
   ```

2. **Create Production Configuration**
   ```bash
   cp config/production.toml.example config/production.toml
   nano config/production.toml
   ```

3. **Configure Environment Variables**
   ```bash
   cp .env.example .env
   nano .env
   ```

4. **Build Application**
   ```bash
   cargo build --release
   ```

5. **Run Database Migrations**
   ```bash
   cargo run --bin governance-app -- --migrate
   ```

### Step 4: Web Server Configuration

1. **Configure Nginx**
   ```bash
   sudo nano /etc/nginx/sites-available/governance-app
   ```

   ```nginx
   server {
       listen 80;
       server_name your-domain.com;

       location / {
           proxy_pass http://localhost:8080;
           proxy_set_header Host $host;
           proxy_set_header X-Real-IP $remote_addr;
           proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
           proxy_set_header X-Forwarded-Proto $scheme;
       }
   }
   ```

2. **Enable Site**
   ```bash
   sudo ln -s /etc/nginx/sites-available/governance-app /etc/nginx/sites-enabled/
   sudo nginx -t
   sudo systemctl reload nginx
   ```

3. **Obtain SSL Certificate**
   ```bash
   sudo certbot --nginx -d your-domain.com
   ```

### Step 5: Service Configuration

1. **Create Systemd Service**
   ```bash
   sudo nano /etc/systemd/system/governance-app.service
   ```

   ```ini
   [Unit]
   Description=BTCDecoded Governance App
   After=network.target postgresql.service

   [Service]
   Type=simple
   User=governance
   WorkingDirectory=/opt/governance-app
   ExecStart=/opt/governance-app/target/release/governance-app
   Restart=always
   RestartSec=5
   Environment=RUST_LOG=info
   Environment=CONFIG_PATH=/opt/governance-app/config/production.toml

   [Install]
   WantedBy=multi-user.target
   ```

2. **Create Application User**
   ```bash
   sudo useradd -r -s /bin/false governance
   sudo mkdir -p /opt/governance-app
   sudo chown -R governance:governance /opt/governance-app
   ```

3. **Install Application**
   ```bash
   sudo cp target/release/governance-app /opt/governance-app/
   sudo cp -r config /opt/governance-app/
   sudo cp -r migrations /opt/governance-app/
   sudo chown -R governance:governance /opt/governance-app
   ```

4. **Start Service**
   ```bash
   sudo systemctl daemon-reload
   sudo systemctl enable governance-app
   sudo systemctl start governance-app
   ```

### Step 6: Monitoring Setup

1. **Install Monitoring Tools**
   ```bash
   # Install Prometheus
   wget https://github.com/prometheus/prometheus/releases/latest/download/prometheus-*.linux-amd64.tar.gz
   tar xvfz prometheus-*.tar.gz
   sudo mv prometheus-*.linux-amd64 /opt/prometheus

   # Install Grafana
   wget -q -O - https://packages.grafana.com/gpg.key | sudo apt-key add -
   echo "deb https://packages.grafana.com/oss/deb stable main" | sudo tee /etc/apt/sources.list.d/grafana.list
   sudo apt update
   sudo apt install grafana -y
   ```

2. **Configure Prometheus**
   ```bash
   sudo nano /opt/prometheus/prometheus.yml
   ```

   ```yaml
   global:
     scrape_interval: 15s

   scrape_configs:
     - job_name: 'governance-app'
       static_configs:
         - targets: ['localhost:8080']
   ```

3. **Start Monitoring Services**
   ```bash
   sudo systemctl start prometheus
   sudo systemctl enable prometheus
   sudo systemctl start grafana-server
   sudo systemctl enable grafana-server
   ```

### Step 7: Backup Configuration

1. **Configure Database Backups**
   ```bash
   sudo nano /opt/backup-db.sh
   ```

   ```bash
   #!/bin/bash
   BACKUP_DIR="/opt/backups"
   DATE=$(date +%Y%m%d_%H%M%S)
   DB_NAME="governance_production"

   mkdir -p $BACKUP_DIR
   pg_dump $DB_NAME > $BACKUP_DIR/governance_$DATE.sql
   gzip $BACKUP_DIR/governance_$DATE.sql

   # Keep only last 30 days of backups
   find $BACKUP_DIR -name "governance_*.sql.gz" -mtime +30 -delete
   ```

2. **Configure Application Backups**
   ```bash
   sudo nano /opt/backup-app.sh
   ```

   ```bash
   #!/bin/bash
   BACKUP_DIR="/opt/backups"
   APP_DIR="/opt/governance-app"
   DATE=$(date +%Y%m%d_%H%M%S)

   mkdir -p $BACKUP_DIR
   tar -czf $BACKUP_DIR/governance-app_$DATE.tar.gz $APP_DIR

   # Keep only last 30 days of backups
   find $BACKUP_DIR -name "governance-app_*.tar.gz" -mtime +30 -delete
   ```

3. **Schedule Backups**
   ```bash
   sudo crontab -e
   # Add:
   # 0 2 * * * /opt/backup-db.sh
   # 0 3 * * * /opt/backup-app.sh
   ```

## Post-Deployment Verification

### 1. Health Checks

1. **Application Health**
   ```bash
   curl -f http://localhost:8080/health
   ```

2. **Database Connectivity**
   ```bash
   curl -f http://localhost:8080/health/db
   ```

3. **GitHub Integration**
   ```bash
   curl -f http://localhost:8080/health/github
   ```

### 2. Functionality Tests

1. **Webhook Processing**
   - Test webhook endpoint with sample payload
   - Verify webhook signature validation
   - Check event processing

2. **Status Check Integration**
   - Create test PR
   - Verify status checks are posted
   - Test merge blocking functionality

3. **Economic Node Integration**
   - Test economic node registration
   - Verify veto functionality
   - Check adoption tracking

### 3. Security Verification

1. **SSL Certificate**
   ```bash
   openssl s_client -connect your-domain.com:443 -servername your-domain.com
   ```

2. **Firewall Configuration**
   ```bash
   sudo ufw status
   nmap -p 80,443,8080 your-domain.com
   ```

3. **Service Security**
   ```bash
   sudo systemctl status governance-app
   sudo journalctl -u governance-app -f
   ```

## Maintenance Procedures

### Daily Maintenance

1. **Check Service Status**
   ```bash
   sudo systemctl status governance-app
   ```

2. **Review Logs**
   ```bash
   sudo journalctl -u governance-app --since "1 day ago"
   ```

3. **Check Disk Space**
   ```bash
   df -h
   ```

4. **Verify Backups**
   ```bash
   ls -la /opt/backups/
   ```

### Weekly Maintenance

1. **Update System**
   ```bash
   sudo apt update && sudo apt upgrade -y
   ```

2. **Review Security Logs**
   ```bash
   sudo journalctl -u fail2ban
   sudo grep "Failed password" /var/log/auth.log
   ```

3. **Check Database Performance**
   ```bash
   sudo -u postgres psql -c "SELECT * FROM pg_stat_activity;"
   ```

4. **Verify Monitoring**
   - Check Grafana dashboards
   - Review Prometheus metrics
   - Verify alerting rules

### Monthly Maintenance

1. **Security Audit**
   - Review access logs
   - Check for suspicious activity
   - Verify key rotation schedule

2. **Performance Review**
   - Analyze performance metrics
   - Review resource usage
   - Optimize configuration

3. **Backup Verification**
   - Test backup restoration
   - Verify backup integrity
   - Update backup procedures

## Troubleshooting

### Common Issues

1. **Service Won't Start**
   ```bash
   sudo systemctl status governance-app
   sudo journalctl -u governance-app -n 50
   ```

2. **Database Connection Issues**
   ```bash
   sudo systemctl status postgresql
   sudo -u postgres psql -c "SELECT * FROM pg_stat_activity;"
   ```

3. **Webhook Issues**
   - Check webhook URL configuration
   - Verify GitHub App permissions
   - Review webhook logs

4. **Performance Issues**
   - Check system resources
   - Review database performance
   - Analyze application logs

### Emergency Procedures

1. **Service Outage**
   ```bash
   sudo systemctl restart governance-app
   sudo systemctl restart nginx
   sudo systemctl restart postgresql
   ```

2. **Database Issues**
   ```bash
   sudo systemctl restart postgresql
   sudo -u postgres psql -c "SELECT pg_reload_conf();"
   ```

3. **Security Incident**
   - Isolate affected systems
   - Preserve evidence
   - Notify stakeholders
   - Follow incident response plan

## Security Considerations

### Network Security

- **Firewall**: Configure minimal required ports
- **SSL/TLS**: Use strong encryption
- **VPN**: Consider VPN for administrative access
- **DDoS Protection**: Implement DDoS mitigation

### Application Security

- **Authentication**: Use strong authentication
- **Authorization**: Implement proper access controls
- **Input Validation**: Validate all inputs
- **Output Encoding**: Encode all outputs

### Data Security

- **Encryption**: Encrypt data at rest and in transit
- **Backups**: Secure backup storage
- **Key Management**: Secure key storage and rotation
- **Access Control**: Limit data access

### Operational Security

- **Monitoring**: Continuous security monitoring
- **Logging**: Comprehensive audit logging
- **Incident Response**: Prepared incident response
- **Training**: Regular security training

## Compliance and Auditing

### Compliance Requirements

- **Data Protection**: GDPR, CCPA compliance
- **Security Standards**: ISO 27001, SOC 2
- **Financial Regulations**: PCI DSS if applicable
- **Industry Standards**: Relevant industry standards

### Auditing

- **Internal Audits**: Regular internal audits
- **External Audits**: Third-party security audits
- **Penetration Testing**: Regular penetration testing
- **Vulnerability Scanning**: Regular vulnerability scans

## Conclusion

This deployment guide provides comprehensive instructions for deploying the BTCDecoded governance system to production. Follow all steps carefully and ensure proper security measures are in place.

Regular maintenance and monitoring are essential for maintaining system security and performance. Keep this guide updated as the system evolves and new requirements emerge.
