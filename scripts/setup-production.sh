#!/bin/bash

# Production Environment Setup Script for BTCDecoded Governance App
# This script sets up a production environment for the governance system

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
APP_DIR="$PROJECT_ROOT/governance-app"
CONFIG_DIR="$APP_DIR/config"
BACKUP_DIR="/opt/backups"
LOG_DIR="/var/log/governance-app"
DATA_DIR="/opt/governance-app/data"
KEYS_DIR="/opt/governance-app/keys"
SSL_DIR="/opt/governance-app/ssl"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if running as root
check_root() {
    if [[ $EUID -eq 0 ]]; then
        log_error "This script should not be run as root"
        exit 1
    fi
}

# Check system requirements
check_system_requirements() {
    log_info "Checking system requirements..."
    
    # Check OS
    if [[ ! -f /etc/os-release ]]; then
        log_error "Cannot determine operating system"
        exit 1
    fi
    
    source /etc/os-release
    if [[ "$ID" != "ubuntu" ]] && [[ "$ID" != "rhel" ]] && [[ "$ID" != "centos" ]]; then
        log_warning "Unsupported operating system: $ID"
        log_warning "This script is designed for Ubuntu, RHEL, or CentOS"
    fi
    
    # Check available memory
    local mem_gb=$(free -g | awk '/^Mem:/{print $2}')
    if [[ $mem_gb -lt 8 ]]; then
        log_warning "System has less than 8GB RAM ($mem_gb GB)"
        log_warning "Production system should have at least 8GB RAM"
    fi
    
    # Check available disk space
    local disk_gb=$(df -BG / | awk 'NR==2{print $4}' | sed 's/G//')
    if [[ $disk_gb -lt 100 ]]; then
        log_warning "System has less than 100GB free disk space ($disk_gb GB)"
        log_warning "Production system should have at least 100GB free space"
    fi
    
    log_success "System requirements check completed"
}

# Install system dependencies
install_dependencies() {
    log_info "Installing system dependencies..."
    
    # Update package lists
    sudo apt update
    
    # Install essential packages
    sudo apt install -y \
        curl \
        wget \
        git \
        build-essential \
        pkg-config \
        libssl-dev \
        libpq-dev \
        nginx \
        certbot \
        python3-certbot-nginx \
        fail2ban \
        ufw \
        htop \
        vim \
        unzip \
        jq \
        postgresql-client
    
    # Install Rust
    if ! command -v cargo &> /dev/null; then
        log_info "Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source ~/.cargo/env
    else
        log_info "Rust is already installed"
    fi
    
    # Install Docker (optional)
    if ! command -v docker &> /dev/null; then
        log_info "Installing Docker..."
        curl -fsSL https://get.docker.com -o get-docker.sh
        sudo sh get-docker.sh
        sudo usermod -aG docker $USER
        rm get-docker.sh
    else
        log_info "Docker is already installed"
    fi
    
    log_success "Dependencies installed successfully"
}

# Create system user
create_system_user() {
    log_info "Creating system user..."
    
    if id "governance" &>/dev/null; then
        log_info "User 'governance' already exists"
    else
        sudo useradd -r -s /bin/false -d /opt/governance-app governance
        log_success "User 'governance' created"
    fi
}

# Create directories
create_directories() {
    log_info "Creating directories..."
    
    # Create application directories
    sudo mkdir -p "$DATA_DIR"
    sudo mkdir -p "$KEYS_DIR"
    sudo mkdir -p "$SSL_DIR"
    sudo mkdir -p "$BACKUP_DIR"
    sudo mkdir -p "$LOG_DIR"
    
    # Set permissions
    sudo chown -R governance:governance /opt/governance-app
    sudo chmod 755 /opt/governance-app
    sudo chmod 700 "$KEYS_DIR"
    sudo chmod 755 "$DATA_DIR"
    sudo chmod 755 "$LOG_DIR"
    
    log_success "Directories created successfully"
}

# Configure firewall
configure_firewall() {
    log_info "Configuring firewall..."
    
    # Enable UFW
    sudo ufw --force enable
    
    # Configure rules
    sudo ufw default deny incoming
    sudo ufw default allow outgoing
    sudo ufw allow ssh
    sudo ufw allow 80/tcp
    sudo ufw allow 443/tcp
    sudo ufw allow 8080/tcp  # governance-app
    sudo ufw allow 9090/tcp  # Prometheus metrics
    
    # Check status
    sudo ufw status
    
    log_success "Firewall configured successfully"
}

# Configure fail2ban
configure_fail2ban() {
    log_info "Configuring fail2ban..."
    
    # Create fail2ban configuration
    sudo tee /etc/fail2ban/jail.local > /dev/null <<EOF
[DEFAULT]
bantime = 3600
findtime = 600
maxretry = 3

[sshd]
enabled = true
port = ssh
logpath = /var/log/auth.log
maxretry = 3

[nginx-http-auth]
enabled = true
port = http,https
logpath = /var/log/nginx/error.log
maxretry = 3

[governance-app]
enabled = true
port = 8080
logpath = $LOG_DIR/application.log
maxretry = 5
EOF
    
    # Start and enable fail2ban
    sudo systemctl enable fail2ban
    sudo systemctl start fail2ban
    
    log_success "Fail2ban configured successfully"
}

# Install and configure PostgreSQL
install_postgresql() {
    log_info "Installing and configuring PostgreSQL..."
    
    # Install PostgreSQL
    sudo apt install -y postgresql postgresql-contrib
    
    # Start and enable PostgreSQL
    sudo systemctl start postgresql
    sudo systemctl enable postgresql
    
    # Create database and user
    sudo -u postgres psql <<EOF
CREATE DATABASE governance_production;
CREATE USER governance_user WITH PASSWORD 'secure_password_change_me';
GRANT ALL PRIVILEGES ON DATABASE governance_production TO governance_user;
\q
EOF
    
    # Configure PostgreSQL for production
    sudo tee -a /etc/postgresql/*/main/postgresql.conf > /dev/null <<EOF

# Production configuration
shared_buffers = 256MB
effective_cache_size = 1GB
maintenance_work_mem = 64MB
checkpoint_completion_target = 0.9
wal_buffers = 16MB
default_statistics_target = 100
random_page_cost = 1.1
effective_io_concurrency = 200
work_mem = 4MB
min_wal_size = 1GB
max_wal_size = 4GB
EOF
    
    # Restart PostgreSQL
    sudo systemctl restart postgresql
    
    log_success "PostgreSQL installed and configured successfully"
}

# Install and configure Nginx
install_nginx() {
    log_info "Installing and configuring Nginx..."
    
    # Install Nginx
    sudo apt install -y nginx
    
    # Create Nginx configuration
    sudo tee /etc/nginx/sites-available/governance-app > /dev/null <<EOF
server {
    listen 80;
    server_name your-domain.com;  # Replace with your domain
    
    # Security headers
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;
    add_header Content-Security-Policy "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline';" always;
    
    # Rate limiting
    limit_req_zone \$binary_remote_addr zone=api:10m rate=10r/s;
    limit_req_zone \$binary_remote_addr zone=webhook:10m rate=5r/s;
    
    # API endpoints
    location /api/ {
        limit_req zone=api burst=20 nodelay;
        proxy_pass http://localhost:8080;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
        proxy_connect_timeout 30s;
        proxy_send_timeout 30s;
        proxy_read_timeout 30s;
    }
    
    # Webhook endpoints
    location /webhook/ {
        limit_req zone=webhook burst=10 nodelay;
        proxy_pass http://localhost:8080;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
        proxy_connect_timeout 30s;
        proxy_send_timeout 30s;
        proxy_read_timeout 30s;
    }
    
    # Health check endpoints
    location /health {
        proxy_pass http://localhost:8080;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
    }
    
    # Metrics endpoints
    location /metrics {
        proxy_pass http://localhost:8080;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
    }
}
EOF
    
    # Enable site
    sudo ln -sf /etc/nginx/sites-available/governance-app /etc/nginx/sites-enabled/
    sudo rm -f /etc/nginx/sites-enabled/default
    
    # Test configuration
    sudo nginx -t
    
    # Start and enable Nginx
    sudo systemctl start nginx
    sudo systemctl enable nginx
    
    log_success "Nginx installed and configured successfully"
}

# Install monitoring tools
install_monitoring() {
    log_info "Installing monitoring tools..."
    
    # Install Prometheus
    wget https://github.com/prometheus/prometheus/releases/latest/download/prometheus-*.linux-amd64.tar.gz
    tar xvfz prometheus-*.tar.gz
    sudo mv prometheus-*.linux-amd64 /opt/prometheus
    sudo chown -R governance:governance /opt/prometheus
    rm prometheus-*.tar.gz
    
    # Install Node Exporter
    wget https://github.com/prometheus/node_exporter/releases/latest/download/node_exporter-*.linux-amd64.tar.gz
    tar xvfz node_exporter-*.tar.gz
    sudo mv node_exporter-*.linux-amd64/node_exporter /usr/local/bin/
    sudo chmod +x /usr/local/bin/node_exporter
    rm -rf node_exporter-*.linux-amd64*
    
    # Install Grafana
    wget -q -O - https://packages.grafana.com/gpg.key | sudo apt-key add -
    echo "deb https://packages.grafana.com/oss/deb stable main" | sudo tee /etc/apt/sources.list.d/grafana.list
    sudo apt update
    sudo apt install -y grafana
    
    # Start and enable services
    sudo systemctl start prometheus
    sudo systemctl enable prometheus
    sudo systemctl start node_exporter
    sudo systemctl enable node_exporter
    sudo systemctl start grafana-server
    sudo systemctl enable grafana-server
    
    log_success "Monitoring tools installed successfully"
}

# Build application
build_application() {
    log_info "Building application..."
    
    # Navigate to app directory
    cd "$APP_DIR"
    
    # Build in release mode
    cargo build --release
    
    # Copy binary to production location
    sudo cp target/release/governance-app /opt/governance-app/
    sudo chown governance:governance /opt/governance-app/governance-app
    sudo chmod +x /opt/governance-app/governance-app
    
    # Copy configuration files
    sudo cp -r config /opt/governance-app/
    sudo cp -r migrations /opt/governance-app/
    sudo chown -R governance:governance /opt/governance-app/config
    sudo chown -R governance:governance /opt/governance-app/migrations
    
    log_success "Application built successfully"
}

# Create systemd service
create_systemd_service() {
    log_info "Creating systemd service..."
    
    sudo tee /etc/systemd/system/governance-app.service > /dev/null <<EOF
[Unit]
Description=BTCDecoded Governance App
After=network.target postgresql.service
Requires=postgresql.service

[Service]
Type=simple
User=governance
Group=governance
WorkingDirectory=/opt/governance-app
ExecStart=/opt/governance-app/governance-app --config /opt/governance-app/config/production.toml
Restart=always
RestartSec=5
Environment=RUST_LOG=info
Environment=RUST_BACKTRACE=1

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/governance-app/data
ReadWritePaths=/var/log/governance-app
ReadWritePaths=/opt/backups

# Resource limits
LimitNOFILE=65536
LimitNPROC=4096

[Install]
WantedBy=multi-user.target
EOF
    
    # Reload systemd and enable service
    sudo systemctl daemon-reload
    sudo systemctl enable governance-app
    
    log_success "Systemd service created successfully"
}

# Create backup scripts
create_backup_scripts() {
    log_info "Creating backup scripts..."
    
    # Database backup script
    sudo tee /opt/governance-app/scripts/backup-database.sh > /dev/null <<'EOF'
#!/bin/bash
BACKUP_DIR="/opt/backups/database"
DATE=$(date +%Y%m%d_%H%M%S)
DB_NAME="governance_production"

mkdir -p $BACKUP_DIR

# Create backup
pg_dump -h localhost -U governance_user -d $DB_NAME \
    --format=custom \
    --compress=9 \
    --file="$BACKUP_DIR/governance_$DATE.backup"

# Verify backup
pg_restore --list "$BACKUP_DIR/governance_$DATE.backup" > /dev/null
if [ $? -eq 0 ]; then
    echo "Backup verified successfully"
else
    echo "Backup verification failed"
    exit 1
fi

# Compress backup
gzip "$BACKUP_DIR/governance_$DATE.backup"

# Keep only last 30 days
find $BACKUP_DIR -name "governance_*.backup.gz" -mtime +30 -delete

echo "Database backup completed: governance_$DATE.backup.gz"
EOF
    
    # Application backup script
    sudo tee /opt/governance-app/scripts/backup-application.sh > /dev/null <<'EOF'
#!/bin/bash
APP_DIR="/opt/governance-app"
BACKUP_DIR="/opt/backups/application"
DATE=$(date +%Y%m%d_%H%M%S)

mkdir -p $BACKUP_DIR

# Backup application files
tar -czf "$BACKUP_DIR/governance-app_$DATE.tar.gz" \
    -C $APP_DIR \
    --exclude=data \
    --exclude=logs \
    .

# Keep only last 30 days
find $BACKUP_DIR -name "governance-app_*.tar.gz" -mtime +30 -delete

echo "Application backup completed: governance-app_$DATE.tar.gz"
EOF
    
    # Make scripts executable
    sudo chmod +x /opt/governance-app/scripts/backup-database.sh
    sudo chmod +x /opt/governance-app/scripts/backup-application.sh
    sudo chown -R governance:governance /opt/governance-app/scripts
    
    log_success "Backup scripts created successfully"
}

# Create monitoring configuration
create_monitoring_config() {
    log_info "Creating monitoring configuration..."
    
    # Prometheus configuration
    sudo tee /opt/prometheus/prometheus.yml > /dev/null <<EOF
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  - "alert_rules.yml"

alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - localhost:9093

scrape_configs:
  - job_name: 'governance-app'
    static_configs:
      - targets: ['localhost:8080']
    metrics_path: '/metrics'
    scrape_interval: 5s

  - job_name: 'postgresql'
    static_configs:
      - targets: ['localhost:5432']
    scrape_interval: 30s

  - job_name: 'node-exporter'
    static_configs:
      - targets: ['localhost:9100']
    scrape_interval: 15s
EOF
    
    # Alert rules
    sudo tee /opt/prometheus/alert_rules.yml > /dev/null <<EOF
groups:
- name: governance-app
  rules:
  - alert: GovernanceAppDown
    expr: up{job="governance-app"} == 0
    for: 1m
    labels:
      severity: critical
    annotations:
      summary: "Governance app is down"
      description: "The governance app has been down for more than 1 minute"

  - alert: HighErrorRate
    expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.1
    for: 2m
    labels:
      severity: warning
    annotations:
      summary: "High error rate detected"
      description: "Error rate is above 10% for 2 minutes"

  - alert: HighResponseTime
    expr: histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m])) > 1
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "High response time detected"
      description: "95th percentile response time is above 1 second"
EOF
    
    # Start Prometheus
    sudo systemctl start prometheus
    sudo systemctl enable prometheus
    
    log_success "Monitoring configuration created successfully"
}

# Create log rotation configuration
create_log_rotation() {
    log_info "Creating log rotation configuration..."
    
    sudo tee /etc/logrotate.d/governance-app > /dev/null <<EOF
/var/log/governance-app/*.log {
    daily
    missingok
    rotate 30
    compress
    delaycompress
    notifempty
    create 644 governance governance
    postrotate
        systemctl reload governance-app
    endscript
}
EOF
    
    log_success "Log rotation configured successfully"
}

# Create production configuration
create_production_config() {
    log_info "Creating production configuration..."
    
    # Copy example configuration
    sudo cp "$CONFIG_DIR/production.toml.example" "$CONFIG_DIR/production.toml"
    
    # Update configuration with production values
    sudo sed -i "s/your_production_webhook_secret/$(openssl rand -hex 32)/" "$CONFIG_DIR/production.toml"
    sudo sed -i "s/your_production_jwt_secret/$(openssl rand -hex 32)/" "$CONFIG_DIR/production.toml"
    sudo sed -i "s/your-domain.com/$(hostname)/" "$CONFIG_DIR/production.toml"
    
    # Set proper permissions
    sudo chown governance:governance "$CONFIG_DIR/production.toml"
    sudo chmod 600 "$CONFIG_DIR/production.toml"
    
    log_success "Production configuration created successfully"
}

# Main setup function
main() {
    log_info "Starting production environment setup..."
    
    check_root
    check_system_requirements
    install_dependencies
    create_system_user
    create_directories
    configure_firewall
    configure_fail2ban
    install_postgresql
    install_nginx
    install_monitoring
    build_application
    create_systemd_service
    create_backup_scripts
    create_monitoring_config
    create_log_rotation
    create_production_config
    
    log_success "Production environment setup completed successfully!"
    
    echo
    log_info "Next steps:"
    echo "1. Update /opt/governance-app/config/production.toml with your production values"
    echo "2. Generate and install SSL certificates: sudo certbot --nginx -d your-domain.com"
    echo "3. Generate production keys and update configuration"
    echo "4. Start the application: sudo systemctl start governance-app"
    echo "5. Verify the application is running: curl http://localhost:8080/health"
    echo
    log_info "Important files:"
    echo "- Configuration: /opt/governance-app/config/production.toml"
    echo "- Logs: /var/log/governance-app/"
    echo "- Data: /opt/governance-app/data/"
    echo "- Keys: /opt/governance-app/keys/"
    echo "- Backups: /opt/backups/"
    echo
    log_warning "Remember to:"
    echo "- Change default passwords"
    echo "- Generate production keys"
    echo "- Configure SSL certificates"
    echo "- Update firewall rules if needed"
    echo "- Test all functionality"
}

# Run main function
main "$@"
