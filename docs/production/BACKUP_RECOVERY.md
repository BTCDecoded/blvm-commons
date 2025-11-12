# Backup and Recovery Guide

This guide outlines comprehensive backup and recovery procedures for the BTCDecoded governance system.

## Backup Strategy

### 3-2-1 Rule

- **3 copies** of important data
- **2 different media types**
- **1 offsite backup**

### Backup Types

1. **Full Backup**: Complete system backup
2. **Incremental Backup**: Changes since last backup
3. **Differential Backup**: Changes since last full backup
4. **Continuous Backup**: Real-time data protection

## Database Backups

### PostgreSQL Backups

#### Automated Backup Script

```bash
#!/bin/bash
# PostgreSQL backup script

BACKUP_DIR="/opt/backups/postgresql"
DB_NAME="governance_production"
DB_USER="governance_user"
DATE=$(date +%Y%m%d_%H%M%S)

# Create backup directory
mkdir -p $BACKUP_DIR

# Full backup
pg_dump -h localhost -U $DB_USER -d $DB_NAME \
    --format=custom \
    --compress=9 \
    --file="$BACKUP_DIR/governance_full_$DATE.backup"

# Verify backup
pg_restore --list "$BACKUP_DIR/governance_full_$DATE.backup" > /dev/null
if [ $? -eq 0 ]; then
    echo "Backup verified successfully"
else
    echo "Backup verification failed"
    exit 1
fi

# Compress backup
gzip "$BACKUP_DIR/governance_full_$DATE.backup"

# Keep only last 30 days
find $BACKUP_DIR -name "governance_full_*.backup.gz" -mtime +30 -delete

echo "Backup completed: governance_full_$DATE.backup.gz"
```

#### Point-in-Time Recovery

```bash
#!/bin/bash
# Point-in-time recovery script

BACKUP_FILE="$1"
TARGET_TIME="$2"
RECOVERY_DIR="/opt/recovery"

# Create recovery directory
mkdir -p $RECOVERY_DIR

# Extract backup
gunzip -c $BACKUP_FILE > $RECOVERY_DIR/recovery.backup

# Restore to target time
pg_restore -d governance_recovery \
    --clean \
    --if-exists \
    --create \
    $RECOVERY_DIR/recovery.backup

echo "Point-in-time recovery completed"
```

### SQLite Backups

#### Automated Backup Script

```bash
#!/bin/bash
# SQLite backup script

DB_FILE="/opt/governance-app/data/governance.db"
BACKUP_DIR="/opt/backups/sqlite"
DATE=$(date +%Y%m%d_%H%M%S)

# Create backup directory
mkdir -p $BACKUP_DIR

# Create backup
sqlite3 $DB_FILE ".backup '$BACKUP_DIR/governance_$DATE.db'"

# Verify backup
sqlite3 "$BACKUP_DIR/governance_$DATE.db" "PRAGMA integrity_check;"

# Compress backup
gzip "$BACKUP_DIR/governance_$DATE.db"

# Keep only last 30 days
find $BACKUP_DIR -name "governance_*.db.gz" -mtime +30 -delete

echo "SQLite backup completed: governance_$DATE.db.gz"
```

#### WAL File Backup

```bash
#!/bin/bash
# WAL file backup script

DB_FILE="/opt/governance-app/data/governance.db"
WAL_DIR="/opt/governance-app/data"
BACKUP_DIR="/opt/backups/sqlite/wal"

# Create backup directory
mkdir -p $BACKUP_DIR

# Checkpoint WAL file
sqlite3 $DB_FILE "PRAGMA wal_checkpoint(FULL);"

# Copy WAL files
cp "$WAL_DIR/governance.db-wal" "$BACKUP_DIR/governance_$(date +%Y%m%d_%H%M%S).wal"
cp "$WAL_DIR/governance.db-shm" "$BACKUP_DIR/governance_$(date +%Y%m%d_%H%M%S).shm"

echo "WAL backup completed"
```

## Application Backups

### Configuration Backup

```bash
#!/bin/bash
# Configuration backup script

APP_DIR="/opt/governance-app"
BACKUP_DIR="/opt/backups/application"
DATE=$(date +%Y%m%d_%H%M%S)

# Create backup directory
mkdir -p $BACKUP_DIR

# Backup configuration files
tar -czf "$BACKUP_DIR/config_$DATE.tar.gz" \
    -C $APP_DIR config/ \
    -C $APP_DIR migrations/ \
    -C $APP_DIR keys/

# Backup application binary
cp "$APP_DIR/governance-app" "$BACKUP_DIR/governance-app_$DATE"

# Backup systemd service
cp /etc/systemd/system/governance-app.service "$BACKUP_DIR/governance-app.service"

echo "Application backup completed: config_$DATE.tar.gz"
```

### Log Backup

```bash
#!/bin/bash
# Log backup script

LOG_DIR="/var/log/governance-app"
BACKUP_DIR="/opt/backups/logs"
DATE=$(date +%Y%m%d_%H%M%S)

# Create backup directory
mkdir -p $BACKUP_DIR

# Rotate and compress logs
find $LOG_DIR -name "*.log" -mtime +1 -exec gzip {} \;

# Backup compressed logs
tar -czf "$BACKUP_DIR/logs_$DATE.tar.gz" -C $LOG_DIR .

# Keep only last 90 days
find $BACKUP_DIR -name "logs_*.tar.gz" -mtime +90 -delete

echo "Log backup completed: logs_$DATE.tar.gz"
```

## System Backups

### Full System Backup

```bash
#!/bin/bash
# Full system backup script

BACKUP_DIR="/opt/backups/system"
DATE=$(date +%Y%m%d_%H%M%S)

# Create backup directory
mkdir -p $BACKUP_DIR

# Create system snapshot
tar -czf "$BACKUP_DIR/system_$DATE.tar.gz" \
    --exclude=/proc \
    --exclude=/sys \
    --exclude=/dev \
    --exclude=/tmp \
    --exclude=/var/tmp \
    --exclude=/var/log \
    --exclude=/var/cache \
    --exclude=/opt/backups \
    /

echo "System backup completed: system_$DATE.tar.gz"
```

### Docker Container Backup

```bash
#!/bin/bash
# Docker container backup script

BACKUP_DIR="/opt/backups/docker"
DATE=$(date +%Y%m%d_%H%M%S)

# Create backup directory
mkdir -p $BACKUP_DIR

# Backup container images
docker save governance-app:latest | gzip > "$BACKUP_DIR/governance-app_$DATE.tar.gz"

# Backup container volumes
docker run --rm -v governance_data:/data -v $BACKUP_DIR:/backup alpine \
    tar -czf /backup/volumes_$DATE.tar.gz -C /data .

echo "Docker backup completed: governance-app_$DATE.tar.gz"
```

## Cloud Backups

### AWS S3 Backup

```bash
#!/bin/bash
# AWS S3 backup script

BUCKET_NAME="btcdecoded-governance-backups"
BACKUP_DIR="/opt/backups"
DATE=$(date +%Y%m%d_%H%M%S)

# Upload backups to S3
aws s3 sync $BACKUP_DIR s3://$BUCKET_NAME/backups/$DATE/

# Set lifecycle policy
aws s3api put-bucket-lifecycle-configuration \
    --bucket $BUCKET_NAME \
    --lifecycle-configuration file://lifecycle.json

echo "AWS S3 backup completed"
```

### Google Cloud Storage Backup

```bash
#!/bin/bash
# Google Cloud Storage backup script

BUCKET_NAME="btcdecoded-governance-backups"
BACKUP_DIR="/opt/backups"
DATE=$(date +%Y%m%d_%H%M%S)

# Upload backups to GCS
gsutil -m rsync -r $BACKUP_DIR gs://$BUCKET_NAME/backups/$DATE/

# Set lifecycle policy
gsutil lifecycle set lifecycle.json gs://$BUCKET_NAME

echo "Google Cloud Storage backup completed"
```

## Recovery Procedures

### Database Recovery

#### PostgreSQL Recovery

```bash
#!/bin/bash
# PostgreSQL recovery script

BACKUP_FILE="$1"
DB_NAME="governance_production"

# Stop application
sudo systemctl stop governance-app

# Drop existing database
sudo -u postgres psql -c "DROP DATABASE IF EXISTS $DB_NAME;"

# Create new database
sudo -u postgres psql -c "CREATE DATABASE $DB_NAME;"

# Restore from backup
pg_restore -h localhost -U postgres -d $DB_NAME $BACKUP_FILE

# Start application
sudo systemctl start governance-app

echo "PostgreSQL recovery completed"
```

#### SQLite Recovery

```bash
#!/bin/bash
# SQLite recovery script

BACKUP_FILE="$1"
DB_FILE="/opt/governance-app/data/governance.db"

# Stop application
sudo systemctl stop governance-app

# Backup current database
cp $DB_FILE "$DB_FILE.backup.$(date +%Y%m%d_%H%M%S)"

# Restore from backup
gunzip -c $BACKUP_FILE > $DB_FILE

# Verify database
sqlite3 $DB_FILE "PRAGMA integrity_check;"

# Start application
sudo systemctl start governance-app

echo "SQLite recovery completed"
```

### Application Recovery

```bash
#!/bin/bash
# Application recovery script

BACKUP_FILE="$1"
APP_DIR="/opt/governance-app"

# Stop application
sudo systemctl stop governance-app

# Extract backup
tar -xzf $BACKUP_FILE -C $APP_DIR

# Restore permissions
sudo chown -R governance:governance $APP_DIR
sudo chmod +x $APP_DIR/governance-app

# Start application
sudo systemctl start governance-app

echo "Application recovery completed"
```

### Full System Recovery

```bash
#!/bin/bash
# Full system recovery script

BACKUP_FILE="$1"
RECOVERY_DIR="/opt/recovery"

# Create recovery directory
mkdir -p $RECOVERY_DIR

# Extract backup
tar -xzf $BACKUP_FILE -C $RECOVERY_DIR

# Restore system files
sudo cp -r $RECOVERY_DIR/* /

# Restore permissions
sudo chown -R governance:governance /opt/governance-app

# Start services
sudo systemctl start governance-app
sudo systemctl start postgresql

echo "Full system recovery completed"
```

## Backup Verification

### Automated Verification

```bash
#!/bin/bash
# Backup verification script

BACKUP_DIR="/opt/backups"
LOG_FILE="/var/log/backup-verification.log"

# Verify PostgreSQL backups
for backup in $BACKUP_DIR/postgresql/*.backup.gz; do
    echo "Verifying $backup" >> $LOG_FILE
    gunzip -c $backup | pg_restore --list > /dev/null
    if [ $? -eq 0 ]; then
        echo "PASS: $backup" >> $LOG_FILE
    else
        echo "FAIL: $backup" >> $LOG_FILE
    fi
done

# Verify SQLite backups
for backup in $BACKUP_DIR/sqlite/*.db.gz; do
    echo "Verifying $backup" >> $LOG_FILE
    gunzip -c $backup | sqlite3 -c "PRAGMA integrity_check;" > /dev/null
    if [ $? -eq 0 ]; then
        echo "PASS: $backup" >> $LOG_FILE
    else
        echo "FAIL: $backup" >> $LOG_FILE
    fi
done

echo "Backup verification completed"
```

### Recovery Testing

```bash
#!/bin/bash
# Recovery testing script

TEST_DIR="/opt/recovery-test"
BACKUP_FILE="$1"

# Create test directory
mkdir -p $TEST_DIR

# Test PostgreSQL recovery
if [[ $BACKUP_FILE == *"postgresql"* ]]; then
    echo "Testing PostgreSQL recovery..."
    # Restore to test database
    gunzip -c $BACKUP_FILE | pg_restore -d governance_test
    # Verify data integrity
    psql -d governance_test -c "SELECT COUNT(*) FROM maintainers;"
fi

# Test SQLite recovery
if [[ $BACKUP_FILE == *"sqlite"* ]]; then
    echo "Testing SQLite recovery..."
    # Restore to test database
    gunzip -c $BACKUP_FILE > $TEST_DIR/test.db
    # Verify data integrity
    sqlite3 $TEST_DIR/test.db "PRAGMA integrity_check;"
fi

echo "Recovery testing completed"
```

## Backup Monitoring

### Backup Status Monitoring

```bash
#!/bin/bash
# Backup status monitoring script

BACKUP_DIR="/opt/backups"
ALERT_EMAIL="admin@btcdecoded.org"

# Check backup age
for backup_type in postgresql sqlite application logs; do
    latest_backup=$(find $BACKUP_DIR/$backup_type -name "*.gz" -type f -printf '%T@ %p\n' | sort -n | tail -1 | cut -d' ' -f2)
    if [ -n "$latest_backup" ]; then
        backup_age=$(( $(date +%s) - $(stat -c %Y "$latest_backup") ))
        if [ $backup_age -gt 86400 ]; then  # 24 hours
            echo "WARNING: $backup_type backup is older than 24 hours" | mail -s "Backup Alert" $ALERT_EMAIL
        fi
    else
        echo "ERROR: No $backup_type backup found" | mail -s "Backup Alert" $ALERT_EMAIL
    fi
done
```

### Backup Size Monitoring

```bash
#!/bin/bash
# Backup size monitoring script

BACKUP_DIR="/opt/backups"
MAX_SIZE=10737418240  # 10GB

# Check backup directory size
backup_size=$(du -s $BACKUP_DIR | cut -f1)
if [ $backup_size -gt $MAX_SIZE ]; then
    echo "WARNING: Backup directory size exceeds 10GB" | mail -s "Backup Size Alert" admin@btcdecoded.org
fi
```

## Disaster Recovery

### Disaster Recovery Plan

1. **Assessment Phase**
   - Assess damage and impact
   - Identify available resources
   - Determine recovery strategy
   - Activate recovery team

2. **Recovery Phase**
   - Restore critical systems
   - Recover data from backups
   - Verify system integrity
   - Test functionality

3. **Validation Phase**
   - Verify data integrity
   - Test all functions
   - Validate security
   - Document recovery process

### Recovery Time Objectives (RTO)

- **Critical Systems**: 1 hour
- **Database**: 2 hours
- **Application**: 4 hours
- **Full System**: 8 hours

### Recovery Point Objectives (RPO)

- **Database**: 15 minutes
- **Application Data**: 1 hour
- **Configuration**: 4 hours
- **Logs**: 24 hours

## Conclusion

Effective backup and recovery procedures are essential for maintaining system availability and data integrity. This guide provides comprehensive coverage of backup strategies, recovery procedures, and monitoring techniques.

Regular testing of backup and recovery procedures is essential to ensure they work correctly when needed. Continuous improvement based on lessons learned and changing requirements will ensure the backup and recovery system remains effective and reliable.
