# Production Security Guide

This guide outlines security best practices and procedures for the BTCDecoded governance system in production.

## Security Architecture

### Defense in Depth

1. **Network Security**: Firewalls, VPNs, network segmentation
2. **Host Security**: OS hardening, patch management, access controls
3. **Application Security**: Input validation, output encoding, authentication
4. **Data Security**: Encryption, access controls, backup security
5. **Operational Security**: Monitoring, incident response, training

### Security Principles

- **Least Privilege**: Minimum necessary access
- **Defense in Depth**: Multiple security layers
- **Fail Secure**: Secure by default
- **Zero Trust**: Verify everything
- **Continuous Monitoring**: Real-time security monitoring

## Network Security

### Firewall Configuration

```bash
# Basic UFW configuration
sudo ufw default deny incoming
sudo ufw default allow outgoing
sudo ufw allow ssh
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
sudo ufw allow 8080/tcp  # governance-app
sudo ufw enable
```

### Network Segmentation

- **DMZ**: Public-facing services
- **Internal Network**: Application servers
- **Database Network**: Database servers
- **Management Network**: Administrative access

### VPN Access

- **Site-to-Site VPN**: Connect remote sites
- **Client VPN**: Remote access for administrators
- **IPSec**: Secure tunnel protocol
- **OpenVPN**: Open-source VPN solution

## Host Security

### Operating System Hardening

1. **Disable Unnecessary Services**
   ```bash
   sudo systemctl disable bluetooth
   sudo systemctl disable cups
   sudo systemctl disable avahi-daemon
   ```

2. **Configure Automatic Updates**
   ```bash
   sudo apt install unattended-upgrades
   sudo dpkg-reconfigure unattended-upgrades
   ```

3. **Enable Audit Logging**
   ```bash
   sudo apt install auditd
   sudo systemctl enable auditd
   sudo systemctl start auditd
   ```

### Access Controls

1. **SSH Configuration**
   ```bash
   # /etc/ssh/sshd_config
   Port 22
   Protocol 2
   PermitRootLogin no
   PasswordAuthentication no
   PubkeyAuthentication yes
   MaxAuthTries 3
   ClientAliveInterval 300
   ```

2. **User Management**
   ```bash
   # Create application user
   sudo useradd -r -s /bin/false governance
   sudo usermod -L governance
   ```

3. **File Permissions**
   ```bash
   # Set proper permissions
   sudo chmod 600 /opt/governance-app/config/production.toml
   sudo chmod 700 /opt/governance-app/keys
   sudo chown -R governance:governance /opt/governance-app
   ```

## Application Security

### Input Validation

```rust
// Validate webhook payload
pub fn validate_webhook_payload(payload: &[u8], signature: &str) -> Result<(), SecurityError> {
    // Verify signature
    verify_github_signature(payload, signature)?;
    
    // Validate JSON structure
    let webhook: WebhookPayload = serde_json::from_slice(payload)?;
    
    // Validate required fields
    if webhook.repository.is_none() {
        return Err(SecurityError::InvalidPayload);
    }
    
    Ok(())
}
```

### Authentication and Authorization

```rust
// JWT token validation
pub fn validate_jwt_token(token: &str) -> Result<Claims, SecurityError> {
    let validation = Validation::new(Algorithm::HS256);
    let token_data = decode::<Claims>(token, &DecodingKey::from_secret(secret), &validation)?;
    Ok(token_data.claims)
}

// Role-based access control
pub fn check_permission(user: &User, resource: &str, action: &str) -> bool {
    match user.role {
        Role::Admin => true,
        Role::Maintainer => check_maintainer_permission(resource, action),
        Role::User => check_user_permission(resource, action),
    }
}
```

### Data Protection

```rust
// Encrypt sensitive data
pub fn encrypt_sensitive_data(data: &str, key: &[u8]) -> Result<String, SecurityError> {
    let cipher = Aes256Gcm::new(GenericArray::from_slice(key));
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher.encrypt(&nonce, data.as_bytes())?;
    Ok(format!("{}:{}", nonce.to_base64(), ciphertext.to_base64()))
}

// Hash passwords
pub fn hash_password(password: &str) -> Result<String, SecurityError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
    Ok(password_hash.to_string())
}
```

## Database Security

### PostgreSQL Security

1. **Connection Security**
   ```sql
   -- Configure SSL
   ssl = on
   ssl_cert_file = 'server.crt'
   ssl_key_file = 'server.key'
   ssl_ca_file = 'ca.crt'
   ```

2. **User Permissions**
   ```sql
   -- Create application user with minimal privileges
   CREATE USER governance_user WITH PASSWORD 'secure_password';
   GRANT CONNECT ON DATABASE governance_production TO governance_user;
   GRANT USAGE ON SCHEMA public TO governance_user;
   GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO governance_user;
   ```

3. **Row Level Security**
   ```sql
   -- Enable RLS on sensitive tables
   ALTER TABLE maintainers ENABLE ROW LEVEL SECURITY;
   CREATE POLICY maintainer_policy ON maintainers
       FOR ALL TO governance_user
       USING (active = true);
   ```

### SQLite Security

1. **File Permissions**
   ```bash
   chmod 600 /opt/governance-app/data/governance.db
   chown governance:governance /opt/governance-app/data/governance.db
   ```

2. **Encryption**
   ```rust
   // Use SQLCipher for encryption
   let database_url = "sqlite:///opt/governance-app/data/governance.db?cipher=aes-256-cbc&key=your_encryption_key";
   ```

## Key Management

### Key Storage

1. **Hardware Security Modules (HSM)**
   - Store private keys in HSM
   - Use HSM for cryptographic operations
   - Implement key backup and recovery

2. **Encrypted Storage**
   ```bash
   # Encrypt key files
   gpg --symmetric --cipher-algo AES256 private_key.pem
   chmod 600 private_key.pem.gpg
   ```

3. **Key Rotation**
   ```rust
   // Implement key rotation
   pub async fn rotate_keys(&self) -> Result<(), KeyError> {
       let new_keys = self.generate_new_keys().await?;
       self.update_keys(new_keys).await?;
       self.archive_old_keys().await?;
       Ok(())
   }
   ```

### Key Distribution

1. **Secure Channels**
   - Use encrypted communication
   - Implement key escrow
   - Use multiple distribution methods

2. **Key Verification**
   ```rust
   // Verify key integrity
   pub fn verify_key_integrity(key: &[u8], checksum: &str) -> bool {
       let computed_checksum = sha256(key);
       computed_checksum == checksum
   }
   ```

## Monitoring and Logging

### Security Monitoring

1. **Intrusion Detection**
   ```bash
   # Install and configure fail2ban
   sudo apt install fail2ban
   sudo systemctl enable fail2ban
   sudo systemctl start fail2ban
   ```

2. **Log Monitoring**
   ```yaml
   # Logstash configuration for security logs
   input {
     file {
       path => "/var/log/auth.log"
       type => "auth"
     }
   }
   
   filter {
     if [type] == "auth" {
       grok {
         match => { "message" => "%{SYSLOGTIMESTAMP:timestamp} %{IPORHOST:host} %{PROG:program}: %{GREEDYDATA:message}" }
       }
     }
   }
   ```

3. **Security Alerts**
   ```yaml
   # Prometheus alert rules
   - alert: FailedLoginAttempts
     expr: rate(auth_failed_logins[5m]) > 10
     for: 2m
     labels:
       severity: warning
     annotations:
       summary: "High number of failed login attempts"
   ```

### Audit Logging

```rust
// Comprehensive audit logging
pub struct AuditLogger {
    logger: Logger,
}

impl AuditLogger {
    pub fn log_security_event(&self, event: SecurityEvent) -> Result<(), LogError> {
        let audit_entry = AuditEntry {
            timestamp: Utc::now(),
            event_type: event.event_type,
            user_id: event.user_id,
            resource: event.resource,
            action: event.action,
            result: event.result,
            details: event.details,
        };
        
        self.logger.info(&serde_json::to_string(&audit_entry)?);
        Ok(())
    }
}
```

## Incident Response

### Security Incident Classification

1. **Critical**: Data breach, system compromise
2. **High**: Unauthorized access, service disruption
3. **Medium**: Security policy violation, suspicious activity
4. **Low**: Minor security alerts, configuration issues

### Response Procedures

1. **Immediate Response**
   - Isolate affected systems
   - Preserve evidence
   - Notify security team
   - Document incident

2. **Investigation**
   - Analyze logs and evidence
   - Determine scope and impact
   - Identify attack vector
   - Assess damage

3. **Recovery**
   - Remove threats
   - Restore systems
   - Implement fixes
   - Monitor for recurrence

## Compliance and Auditing

### Security Standards

1. **ISO 27001**: Information security management
2. **SOC 2**: Security, availability, processing integrity
3. **PCI DSS**: Payment card industry security
4. **GDPR**: General data protection regulation

### Audit Procedures

1. **Internal Audits**
   - Quarterly security reviews
   - Vulnerability assessments
   - Penetration testing
   - Compliance checks

2. **External Audits**
   - Annual third-party audits
   - Security certifications
   - Compliance assessments
   - Risk assessments

## Security Training

### Training Program

1. **Security Awareness**
   - Phishing simulation
   - Password security
   - Social engineering
   - Incident reporting

2. **Technical Training**
   - Secure coding practices
   - Security tools usage
   - Incident response procedures
   - Threat intelligence

3. **Regular Updates**
   - Monthly security briefings
   - Quarterly training sessions
   - Annual certification renewal
   - Continuous learning

## Conclusion

Security is a continuous process that requires ongoing attention and improvement. This guide provides a foundation for implementing comprehensive security measures, but it should be regularly updated based on new threats, technologies, and best practices.

Regular security assessments, training, and monitoring are essential for maintaining effective security posture and protecting the governance system from evolving threats.
