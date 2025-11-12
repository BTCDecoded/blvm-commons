# Governance Fork Guide

This guide explains how to use the governance fork system to create, manage, and migrate between different governance rulesets.

## Overview

The governance fork system allows users to:
- Create custom governance rulesets
- Migrate between different rulesets
- Track adoption of governance changes
- Execute governance forks when thresholds are met

## Key Concepts

### Rulesets

A **ruleset** is a complete governance configuration that includes:
- Action tiers and signature requirements
- Economic node configurations
- Maintainer registries
- Repository-specific rules
- Governance fork settings

### Fork Detection

The system automatically detects when:
- Adoption thresholds are met (30% hash rate, 40% economic activity)
- Time-based triggers occur
- Consensus emerges for a particular ruleset
- Manual fork triggers are activated

### Migration

**Migration** is the process of switching from one ruleset to another. This can be:
- **Automatic**: Triggered by fork detection
- **Manual**: Initiated by users or administrators
- **Rollback**: Reverting to a previous ruleset

## CLI Tools

### fork-migrate

The main tool for managing governance rulesets and migrations.

#### List Available Rulesets

```bash
cargo run --release --bin fork-migrate list
cargo run --release --bin fork-migrate list --detailed
```

#### Show Current Ruleset

```bash
cargo run --release --bin fork-migrate current
```

#### Create New Ruleset

```bash
cargo run --release --bin fork-migrate create \
  --name "conservative-governance" \
  --description "Conservative governance with higher thresholds" \
  --version "1.0.0"
```

#### Migrate to Different Ruleset

```bash
cargo run --release --bin fork-migrate migrate \
  --ruleset "conservative-governance" \
  --backup
```

#### Compare Rulesets

```bash
cargo run --release --bin fork-migrate compare \
  --ruleset1 "current" \
  --ruleset2 "conservative-governance"
```

#### Validate Ruleset

```bash
cargo run --release --bin fork-migrate validate \
  --ruleset "conservative-governance"
```

#### Show Migration History

```bash
cargo run --release --bin fork-migrate history
cargo run --release --bin fork-migrate history --limit 20
```

#### Rollback to Previous Ruleset

```bash
cargo run --release --bin fork-migrate rollback \
  --ruleset "previous-ruleset" \
  --force
```

## Fork Detection

### Automatic Detection

The system continuously monitors:
- **Adoption Metrics**: Node count, hash rate, economic activity
- **Time Triggers**: Grace periods, scheduled migrations
- **Consensus**: Overwhelming support for a ruleset
- **Emergency**: Critical governance changes

### Thresholds

Default fork thresholds:
- **Minimum Node Count**: 10 nodes
- **Minimum Hash Rate**: 30% of network
- **Minimum Economic Activity**: 40% of economic activity
- **Minimum Adoption**: 50% total weight
- **Grace Period**: 30 days

### Manual Triggers

Administrators can trigger forks manually:
- Emergency situations
- Scheduled migrations
- Testing and development
- Coordinated upgrades

## Ruleset Management

### Creating Rulesets

1. **Start with Current Configuration**:
   ```bash
   cargo run --release --bin fork-migrate current
   ```

2. **Create New Ruleset**:
   ```bash
   cargo run --release --bin fork-migrate create \
     --name "my-ruleset" \
     --description "Custom governance rules" \
     --version "1.0.0"
   ```

3. **Modify Configuration**:
   - Edit the generated JSON file
   - Update action tiers, thresholds, etc.
   - Validate the changes

4. **Validate Ruleset**:
   ```bash
   cargo run --release --bin fork-migrate validate \
     --ruleset "my-ruleset"
   ```

### Ruleset Structure

A ruleset export contains:

```json
{
  "version": "1.0",
  "ruleset_id": "my-ruleset",
  "ruleset_version": {
    "major": 1,
    "minor": 0,
    "patch": 0
  },
  "created_at": "2024-01-01T00:00:00Z",
  "action_tiers": { /* tier definitions */ },
  "economic_nodes": { /* economic node config */ },
  "maintainers": { /* maintainer registry */ },
  "repositories": { /* repo-specific rules */ },
  "governance_fork": { /* fork settings */ },
  "metadata": {
    "exported_by": "fork-migrate",
    "source_repository": "btcdecoded/governance",
    "commit_hash": "abc123...",
    "export_tool_version": "1.0.0",
    "description": "Custom governance rules"
  }
}
```

## Migration Process

### Automatic Migration

1. **Fork Detection**: System detects threshold conditions
2. **Validation**: Target ruleset is validated
3. **Backup**: Current ruleset is backed up
4. **Migration**: Ruleset is switched
5. **Notification**: Stakeholders are notified
6. **Logging**: Migration is logged

### Manual Migration

1. **Choose Target**: Select ruleset to migrate to
2. **Validate**: Ensure target ruleset is valid
3. **Backup**: Create backup of current state
4. **Migrate**: Execute migration
5. **Verify**: Confirm migration success

### Rollback Process

1. **Identify Target**: Choose previous ruleset
2. **Validate**: Ensure rollback target is valid
3. **Backup**: Backup current state
4. **Rollback**: Execute rollback
5. **Verify**: Confirm rollback success

## Adoption Tracking

### Metrics Collected

- **Node Count**: Number of nodes using each ruleset
- **Hash Rate**: Percentage of network hash rate
- **Economic Activity**: Percentage of economic activity
- **Total Weight**: Combined adoption weight

### Adoption Dashboard

View adoption statistics:
```bash
# In governance-app
curl http://localhost:8080/api/adoption/statistics
```

### Threshold Monitoring

Monitor approaching thresholds:
```bash
# Check if approaching fork conditions
curl http://localhost:8080/api/fork/status
```

## Best Practices

### Ruleset Design

- **Semantic Versioning**: Use proper version numbers
- **Backward Compatibility**: Maintain compatibility when possible
- **Clear Documentation**: Document all changes
- **Testing**: Test rulesets before deployment

### Migration Safety

- **Always Backup**: Create backups before migrations
- **Validate First**: Validate rulesets before migration
- **Gradual Rollout**: Test with small groups first
- **Monitor Closely**: Watch for issues after migration

### Emergency Procedures

- **Quick Rollback**: Have rollback procedures ready
- **Communication**: Notify stakeholders of changes
- **Monitoring**: Watch for system health
- **Documentation**: Log all emergency actions

## Troubleshooting

### Common Issues

**"Ruleset not found"**
- Check ruleset ID spelling
- Verify ruleset exists in exports directory
- Use `list` command to see available rulesets

**"Validation failed"**
- Check JSON syntax
- Verify required fields are present
- Use `validate` command for details

**"Migration failed"**
- Check file permissions
- Verify target ruleset is valid
- Check system logs for errors

**"Adoption not tracking"**
- Verify economic nodes are registered
- Check adoption tracker configuration
- Monitor system logs

### Getting Help

- Check migration history: `fork-migrate history`
- Validate current state: `fork-migrate current`
- Check system logs: `tail -f logs/governance-app.log`
- Contact governance team for assistance

## Security Considerations

### Ruleset Integrity

- **Digital Signatures**: Verify ruleset signatures
- **Hash Verification**: Check ruleset hashes
- **Source Verification**: Verify ruleset sources
- **Access Control**: Limit who can create rulesets

### Migration Security

- **Authentication**: Require authentication for migrations
- **Authorization**: Check permissions before migration
- **Audit Logging**: Log all migration activities
- **Rollback Capability**: Ensure rollback is possible

### Network Security

- **Secure Communication**: Use HTTPS for API calls
- **Certificate Validation**: Verify SSL certificates
- **Access Control**: Restrict access to fork tools
- **Monitoring**: Monitor for suspicious activity

## Production Deployment

### Pre-Production

1. **Test Rulesets**: Test all rulesets thoroughly
2. **Validate Migrations**: Test migration procedures
3. **Document Procedures**: Document all processes
4. **Train Operators**: Train system operators

### Production

1. **Monitor Adoption**: Watch adoption metrics closely
2. **Backup Regularly**: Create regular backups
3. **Log Everything**: Maintain comprehensive logs
4. **Be Ready**: Have rollback procedures ready

### Post-Production

1. **Monitor Performance**: Watch system performance
2. **Collect Feedback**: Gather user feedback
3. **Update Documentation**: Keep docs current
4. **Plan Improvements**: Plan future enhancements
