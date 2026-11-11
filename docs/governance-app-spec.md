# BTCDecoded Governance System - Design Document

## Executive Summary

A Rust GitHub App for the BTCDecoded repositories. The published schedule names signature thresholds and review periods, including 6-of-7 on the constitutional layer.

**Core innovation:** Apply the same cryptographic enforcement to governance that Bitcoin applies to consensus - making power visible, capture expensive, and exit cheap.

## The Problem

**Bitcoin Core today:**
- Any 1 of ~5 maintainers can merge code
- Release signing uses individual PGP keys (trust specific people)
- Governance is informal social consensus
- No structured escalation for disputes
- Single points of failure throughout

**At $2T market cap, this is inadequate.**

## The Solution

**Three-layer defense:**

1. **Development Governance** - GitHub App enforces signature thresholds and review periods
2. **Distribution Governance** - Releases must have valid maintainer multisig
3. **Deployment Governance** - Nodes verify signatures before installing updates

**Even if GitHub governance is bypassed, unsigned releases won't reach users.**

## Repository Hierarchy

### Layer 1: Orange Paper (Consensus Specification)
- **What:** Mathematical specification of Bitcoin consensus rules
- **Threshold:** 6-of-7 maintainer signatures
- **Review period:** 180 days
- **Synchronized with:** Consensus Proof (must move together)

### Layer 2: Consensus Proof (Formal Verification)
- **What:** Formal proofs that specification is correct/consistent
- **Threshold:** 6-of-7 maintainer signatures (same as Layer 1)
- **Review period:** 180 days
- **Synchronized with:** Orange Paper (must move together)

### Layer 3: Protocol Engine (Consensus Implementation)
- **What:** Rust implementation of consensus rules
- **Threshold:** 4-of-5 maintainer signatures
- **Review period:** 90 days
- **Must prove:** Equivalence to Orange Paper specification

### Layer 4: Reference Node (Full Implementation)
- **What:** Complete Bitcoin node (uses Protocol Engine + adds P2P, wallet, RPC)
- **Threshold:** 3-of-5 maintainer signatures
- **Review period:** 60 days
- **Cannot modify:** Protocol Engine (must import as dependency)

### Layer 5: Developer SDK (Extension System)
- **What:** Modules and tools for building on Bitcoin
- **Threshold:** 2-of-3 module maintainer signatures
- **Review period:** 14 days
- **Cannot affect:** Consensus behavior

## Architecture

```
┌─────────────────────────────────────────┐
│   GitHub Organization (5 repos)         │
│   - blvm-spec                           │
│   - blvm-consensus                      │
│   - blvm-protocol                       │
│   - blvm-node                           │
│   - blvm-sdk                             │
└─────────────────────────────────────────┘
            ↓ webhooks
┌─────────────────────────────────────────┐
│   Governance App (Rust)                 │
│   - Signature verification              │
│   - Review period enforcement           │
│   - Cross-repo dependency validation    │
│   - Status checks to GitHub             │
│   - Audit log (PostgreSQL)              │
└─────────────────────────────────────────┘
            ↓ status checks
┌─────────────────────────────────────────┐
│   GitHub PR (merge blocked until)       │
│   ✅ Required signatures collected       │
│   ✅ Review period elapsed               │
│   ✅ Dependencies satisfied              │
└─────────────────────────────────────────┘
            ↓ after merge
┌─────────────────────────────────────────┐
│   Release Signing                       │
│   - Maintainers sign with same keys    │
│   - Multisig threshold required         │
│   - Signature covers binaries + hash   │
└─────────────────────────────────────────┘
            ↓ distribution
┌─────────────────────────────────────────┐
│   Node Verification                     │
│   - Hard-coded governance pubkeys       │
│   - Verifies signatures before install │
│   - Rejects unsigned/invalid releases  │
└─────────────────────────────────────────┘
```

## Core Components

### 1. Governance App (Rust)

**Purpose:** External enforcement engine that GitHub cannot bypass

**Repository:** `governance-app`

**Structure:**
```
governance-app/
├── Cargo.toml
├── README.md
├── .env.example
├── src/
│   ├── main.rs              # Axum server setup
│   ├── lib.rs               # Library exports
│   ├── config.rs            # Configuration loading
│   ├── webhooks/
│   │   ├── mod.rs
│   │   ├── pull_request.rs  # PR opened, synchronized events
│   │   ├── review.rs        # PR review events
│   │   ├── comment.rs       # Issue comment events (signatures)
│   │   └── push.rs          # Direct push detection
│   ├── validation/
│   │   ├── mod.rs
│   │   ├── signatures.rs    # secp256k1 signature verification
│   │   ├── review_period.rs # Time-based enforcement
│   │   ├── cross_layer.rs   # Cross-repo dependency checks
│   │   └── threshold.rs     # Multisig threshold validation
│   ├── enforcement/
│   │   ├── mod.rs
│   │   ├── status_checks.rs # Post status checks to GitHub
│   │   └── merge_block.rs   # Merge blocking logic
│   ├── database/
│   │   ├── mod.rs
│   │   ├── models.rs        # Database models
│   │   ├── queries.rs       # SQL queries
│   │   └── schema.rs        # Schema definitions
│   ├── crypto/
│   │   ├── mod.rs
│   │   ├── signatures.rs    # Signature verification
│   │   └── multisig.rs      # Multisig threshold logic
│   ├── github/
│   │   ├── mod.rs
│   │   ├── client.rs        # GitHub API client
│   │   ├── types.rs         # GitHub API types
│   │   └── webhooks.rs      # Webhook verification
│   └── error.rs             # Error types
├── migrations/
│   ├── 001_initial_schema.sql
│   ├── 002_emergency_mode.sql
│   └── 003_audit_log.sql
├── config/
│   └── app.toml             # Application configuration
└── tests/
    ├── integration/
    │   ├── webhook_tests.rs
    │   └── signature_tests.rs
    └── unit/
        ├── validation_tests.rs
        └── crypto_tests.rs
```

**Key functions:**
- Receive GitHub webhooks (pull_request, review, comment, push)
- Validate cryptographic signatures (secp256k1, Bitcoin-compatible)
- Enforce review periods (time-based blocking)
- Check cross-repo dependencies (graph validation)
- Post status checks to GitHub (block merge button)
- Log all governance actions (immutable audit trail)

**Configuration approach:**
- Environment variables for secrets (GitHub token, database URL)
- TOML file for governance rules source
- Loads governance rules from `governance` repo via GitHub API
- Caches rules locally, refreshes on webhook from `governance` repo

**Database schema:**
```sql
-- Repository configurations (cached from governance repo)
CREATE TABLE repos (
  id SERIAL PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  layer INTEGER NOT NULL,
  signature_threshold TEXT NOT NULL,
  review_period_days INTEGER NOT NULL,
  synchronized_with TEXT[],
  last_updated TIMESTAMP DEFAULT NOW()
);

-- Maintainer keys by layer (cached from governance repo)
CREATE TABLE maintainers (
  id SERIAL PRIMARY KEY,
  github_username TEXT NOT NULL UNIQUE,
  public_key TEXT NOT NULL,
  layer INTEGER NOT NULL,
  active BOOLEAN DEFAULT true,
  last_updated TIMESTAMP DEFAULT NOW()
);

-- Emergency keyholders (cached from governance repo)
CREATE TABLE emergency_keyholders (
  id SERIAL PRIMARY KEY,
  github_username TEXT NOT NULL UNIQUE,
  public_key TEXT NOT NULL,
  active BOOLEAN DEFAULT true,
  last_updated TIMESTAMP DEFAULT NOW()
);

-- Pull request tracking (app state)
CREATE TABLE pull_requests (
  id SERIAL PRIMARY KEY,
  repo_name TEXT NOT NULL,
  pr_number INTEGER NOT NULL,
  opened_at TIMESTAMP NOT NULL,
  layer INTEGER NOT NULL,
  head_sha TEXT NOT NULL,
  signatures JSONB DEFAULT '[]',
  governance_status TEXT DEFAULT 'pending',
  linked_prs JSONB DEFAULT '[]',
  emergency_mode BOOLEAN DEFAULT false,
  created_at TIMESTAMP DEFAULT NOW(),
  updated_at TIMESTAMP DEFAULT NOW(),
  UNIQUE(repo_name, pr_number)
);

-- Emergency mode state (app state)
CREATE TABLE emergency_activations (
  id SERIAL PRIMARY KEY,
  activated_by TEXT NOT NULL,
  reason TEXT NOT NULL,
  evidence TEXT NOT NULL,
  signatures JSONB DEFAULT '[]',
  activated_at TIMESTAMP,
  expires_at TIMESTAMP,
  active BOOLEAN DEFAULT false,
  created_at TIMESTAMP DEFAULT NOW()
);

-- Audit log (immutable record)
CREATE TABLE governance_events (
  id SERIAL PRIMARY KEY,
  event_type TEXT NOT NULL,
  repo_name TEXT,
  pr_number INTEGER,
  maintainer TEXT,
  details JSONB,
  timestamp TIMESTAMP DEFAULT NOW()
);

-- Cross-layer rules (cached from governance repo)
CREATE TABLE cross_layer_rules (
  id SERIAL PRIMARY KEY,
  source_repo TEXT NOT NULL,
  source_pattern TEXT NOT NULL,
  target_repo TEXT NOT NULL,
  target_pattern TEXT NOT NULL,
  validation_type TEXT NOT NULL,
  last_updated TIMESTAMP DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX idx_prs_repo_status ON pull_requests(repo_name, governance_status);
CREATE INDEX idx_prs_opened_at ON pull_requests(opened_at);
CREATE INDEX idx_maintainers_layer ON maintainers(layer, active);
CREATE INDEX idx_events_timestamp ON governance_events(timestamp DESC);
CREATE INDEX idx_emergency_active ON emergency_activations(active, expires_at);
```

### 2. Governance Configuration Repo

**Purpose:** Central source of truth for governance rules

**Repository:** `governance`

**Structure:**
```
governance/
├── README.md                    # Documentation
├── GOVERNANCE.md                # How governance works
├── repos/
│   ├── blvm-spec.yml
│   ├── blvm-consensus.yml
│   ├── blvm-protocol.yml
│   ├── blvm-node.yml
│   └── blvm-sdk.yml
├── maintainers/
│   ├── layer-1-2.yml           # 7 maintainers (constitutional)
│   ├── layer-3.yml             # 5 maintainers (implementation)
│   ├── layer-4.yml             # 5 maintainers (application)
│   └── emergency.yml           # 7 emergency keyholders
├── cross-layer-rules.yml       # Dependency validation rules
├── warnings/                   # Formal warnings (if needed)
│   └── .gitkeep
└── .governance.yml             # Meta: this repo's own governance
```

**Example configuration:**
```yaml
# repos/blvm-spec.yml
layer: 1
governance_level: constitutional
signature_threshold: 6-of-7
review_period_days: 180
synchronized_with:
  - blvm-consensus

cross_layer_rules:
  - if_changed: consensus-rules/**
    then_require_update: blvm-consensus/proofs/**
    validation: equivalence_proof_exists
    error_message: "Consensus rule changes require corresponding proof updates"
```

```yaml
# maintainers/layer-1-2.yml
maintainers:
  - github: alice
    public_key: 0x02a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2
    role: cryptographer
    added: 2025-01-01
  
  - github: bob
    public_key: 0x03b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3
    role: protocol_expert
    added: 2025-01-01
  
  # ... 5 more (total 7)
```

```yaml
# cross-layer-rules.yml
rules:
  - name: consensus_proof_sync
    source_repo: blvm-spec
    source_pattern: consensus-rules/**
    target_repo: blvm-consensus
    target_pattern: proofs/**
    validation: corresponding_file_exists
    bidirectional: true
  
  - name: protocol_engine_equivalence
    source_repo: blvm-protocol
    source_pattern: consensus/**
    target_repo: blvm-spec
    validation: references_latest_version
    required_reference_format: "blvm-spec@v{VERSION}"
```

**This repo's own governance:**
```yaml
# .governance.yml (meta-governance)
governance_source: self
layer: 0  # Meta-layer
signature_threshold: 5-of-7 + 2-of-3  # Maintainers + emergency keyholders
review_period_days: 90
public_comment_period_days: 30
```

**Why separate repo:**
- Governance rules go through governance process
- Changes require higher threshold
- Version-controlled history
- Forkable (alternative governance possible)
- Single source of truth for all governance state

### 3. Local Governance Files

**Each project repo contains `.governance.yml`:**
```yaml
# blvm-spec/.governance.yml
governance_source: https://github.com/btcdecoded/governance
layer: 1

# This file is a pointer to canonical governance
# Actual rules live in governance repo
# App validates this matches canonical config
```

**Purpose:** 
- Quick reference for developers
- App validates against canonical source in `governance` repo
- If mismatch detected, app alerts and uses canonical source

## Workflow Examples

### Normal PR in Protocol Engine (Layer 3)

**Day 0: PR opened**
```
Developer opens PR changing consensus implementation

Governance App receives webhook:
  - Loads rules from governance repo (Layer 3: 4-of-5, 90 days)
  - Creates PR record in database
  - Posts status check:
  
  ❌ Governance: Review Period Not Met
  Required: 90 days | Elapsed: 0 days
  Earliest merge: 2025-04-19
  
  ❌ Governance: Signatures Missing
  Required: 4-of-5 | Current: 0/4
  
  ❌ Governance: Equivalence Proof Missing
  Must prove equivalence to blvm-spec@v2.1.0

GitHub merge button: DISABLED
```

**Days 1-90: Review and signing**
```
Maintainers review code, post comments

Maintainer Alice signs:
  Posts: /governance-sign 0x[signature]
  
Governance App:
  - Verifies signature against Alice's public key
  - Updates database
  - Updates status check: 1/4 signatures ✓

Process repeats for Bob, Carol, Dave
```

**Day 90: All requirements met**
```
✅ Governance: All Requirements Met

Signatures: 4-of-5 ✓
  - alice: 0x1234...
  - bob: 0x5678...
  - carol: 0x9abc...
  - dave: 0xdef0...

Review period: 90/90 days ✓
Equivalence proof: Validated ✓
Dependencies: blvm-spec@v2.1.0 referenced ✓

GitHub merge button: ENABLED
```

### Cross-Repo PR (Orange Paper + Consensus Proof)

**Synchronized changes required:**

```
Developer changes Orange Paper consensus rules

Governance App detects:
  - Files changed match cross-layer rule pattern
  - No corresponding Consensus Proof PR exists
  
Posts comment:
  ⚠️ Cross-Layer Dependency Required
  
  Changes to consensus-rules/** require corresponding
  changes in blvm-consensus repo.
  
  Please open PR in blvm-consensus and link here.

Status check: ❌ BLOCKED

Developer opens PR in blvm-consensus

Governance App:
  - Links the two PRs
  - Both must collect signatures
  - Both must complete review periods
  - Both must reach "ready" state
  
When both ready:
  - App merges both PRs atomically
  - If one fails, neither merges
```

### Emergency Mode Activation

**Scenario: Consensus bug causing chain split**

```
Day 0: Bug discovered

Emergency keyholder creates issue in governance repo:
  Title: EMERGENCY: Chain split at block 850,000
  Evidence: [blockchain explorer links showing split]
  
Emergency keyholders sign:
  5 post: /emergency-activate [signature]

Governance App verifies 5-of-7 threshold:
  ✅ Emergency mode ACTIVATED
  
Changes:
  - Review periods: 180 days → 30 days
  - Signatures: STILL 6-of-7 (unchanged)
  - Formal proofs: STILL required
  - Equivalence testing: STILL required
  
Posts to all repos:
  🚨 EMERGENCY MODE ACTIVE
  Expires: 90 days from now
  Reason: Consensus bug causing chain split

Emergency fix PR:
  - Goes through accelerated 30-day review
  - Still requires 6-of-7 signatures
  - Still requires formal proofs
  - Merges after 30 days (instead of 180)

Day 90: Emergency mode auto-expires
```

### Governance Rule Change

**Changing governance rules themselves:**

```
PR opened in governance repo changing blvm-protocol.yml:
  - signature_threshold: 4-of-5 → 3-of-5

Governance App detects:
  - This is meta-governance change
  - Requires higher threshold: 5-of-7 maintainers + 2-of-3 emergency keyholders
  - Creates GitHub Discussion for public comment
  
Status check:
  ❌ Meta-Governance Change
  Required: 5-of-7 maintainers + 2-of-3 emergency keyholders
  Current: 3-of-7 maintainers, 1-of-3 emergency keyholders
  
  Public comment period: 15/30 days
  Review period: 45/90 days

After all requirements met:
  - PR merges
  - Governance App reloads configuration
  - New rules take effect for future PRs
```

## Meta-Governance: Changing The Rules

**Governance rules themselves go through governance:**

```yaml
governance_rule_changes:
  trigger: PR in governance repo
  
  requirements:
    - signatures: 5-of-7 maintainers + 2-of-3 emergency_keyholders
    - review_period: 90_days
    - public_comment_period: 30_days (GitHub Discussion)
    - rationale_required: true
  
  process:
    1. PR opened changing governance rules
    2. Governance App auto-creates GitHub Discussion
    3. 30-day public comment period
    4. After comments, 90-day review for signatures
    5. App enforces higher threshold
    6. Once merged, app reloads configuration from repo
```

**App watches governance repo:**
- Webhook on push to main branch
- Triggers configuration reload
- Updates database with new rules
- Applies to new PRs immediately
- Existing PRs use rules from when they opened (no retroactive changes)

## Maintainer Lifecycle

### Adding Maintainer

```yaml
process:
  1. nomination:
      - existing maintainer creates GitHub issue in governance repo
      - documents: background, contributions, reason
  
  2. probation:
      - 90 days as reviewer (can comment, cannot sign)
      - demonstrates competence
  
  3. approval:
      - PR to maintainers/layer-X.yml adding new maintainer
      - requires 5-of-7 current maintainers
      - includes public key
  
  4. activation:
      - once merged, app reloads maintainer list
      - new maintainer can now sign PRs
```

### Removing Maintainer

**Voluntary exit:**
```yaml
process:
  - maintainer creates PR removing themselves from maintainers/layer-X.yml
  - 30-day notice period
  - requires 3-of-7 approval
  - app deactivates key after merge
```

**Performance-based removal:**
```yaml
process:
  1. concern:
      - issue created in governance repo
      - documents problems
  
  2. response_period:
      - 30 days for response
  
  3. vote:
      - PR to remove from maintainers/layer-X.yml
      - requires 6-of-7 approval (excluding subject)
  
  4. removal:
      - merged PR triggers app to deactivate key
      - 60-day appeal period
```

### Key Rotation

```yaml
scheduled_rotation:
  - every 2 years
  - PR updating public key in maintainers/layer-X.yml
  - requires 3-of-7 approval
  - app updates to use new key
```

## Graduated Sanctions (Minimal)

### Level 1: Social Pressure
- Concerning behavior called out in PR comments
- Visible to community
- Reputation cost

### Level 2: Formal Warning
```yaml
process:
  1. maintainer creates issue in governance repo
  2. 14-day response period
  3. vote: 4-of-7 maintainers
  4. if approved: create markdown file in governance/warnings/
  5. publicly visible
```

**Example warning file:**
```markdown
# governance/warnings/2025-10-19-alice.md

## Formal Warning: Alice
**Date:** 2025-10-19
**Issued by:** 4-of-7 vote (Bob, Carol, Dave, Eve)
**Reason:** < 20% participation over past 180 days
**Evidence:** [links to PRs]
**Response:** [Alice's response]
```

### Level 3: Removal
- See "Removing Maintainer" process above
- Requires 6-of-7 vote

## Conflict Resolution (Minimal)

**For deadlocked PRs:**

```yaml
if: PR open for 180 days without required signatures

process:
  1. any maintainer requests community input
  2. create GitHub Discussion
  3. 30-day public comment period
  4. maintainers reconsider with community feedback
  5. final vote: still requires original threshold

if_still_deadlocked:
  - PR closed
  - can reopen with new approach
  - or: build as optional module instead
```

## Security Model

### What This Protects Against

✅ **Single maintainer compromise** - Need 6-of-7 or 4-of-5
✅ **GitHub admin bypass** - Cannot create validly-signed release without keys
✅ **Release server compromise** - Signatures won't match, node verification fails
✅ **Social pressure on individual** - Distributed keyholders, visible attempts
✅ **Informal power dynamics** - All actions cryptographically signed

### What This Doesn't Protect Against

❌ **Coordinated 6-of-7 collusion** - A supermajority of keyholders can still approve a change
❌ **Sophisticated multi-key compromise** - Extremely difficult and expensive
❌ **User bypassing verification** - User sovereignty = user's choice

### Defense In Depth

**Layer 1:** GitHub governance (signature requirements, review periods)
**Layer 2:** Release signing (multisig required)
**Layer 3:** Node verification (hard-coded pubkeys)

**Even if Layer 1 fails, Layers 2-3 protect users.**

## Ostrom's Principles - Compliance

### ✅ Principle 1: Clear Boundaries
Layer hierarchy defines who decides what

### ✅ Principle 2: Rules Match Local Conditions
Different thresholds for different risk levels

### ✅ Principle 3: Collective Choice Arrangements
Maintainers can modify rules (higher threshold), community input for deadlocks, fork rights

### ✅ Principle 4: Monitoring
Complete cryptographic audit log

### ✅ Principle 5: Graduated Sanctions
Social pressure → formal warning → removal

### ✅ Principle 6: Conflict Resolution
Community input for deadlocks

### ✅ Principle 7: Recognition of Rights
Fork-ready, distributed keyholders, transparent

**Full Ostrom compliance with minimal overhead.**

## Implementation Requirements

### Rust Dependencies
```toml
[dependencies]
# Web framework
axum = "0.7"
tokio = { version = "1", features = ["full"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["trace"] }

# GitHub API
octocrab = "0.38"

# Database
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-rustls", "chrono", "json"] }

# Cryptography (Bitcoin-compatible)
secp256k1 = { version = "0.28", features = ["rand"] }
bitcoin = "0.31"
sha2 = "0.10"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"
toml = "0.8"

# Time
chrono = { version = "0.4", features = ["serde"] }

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Configuration
config = "0.14"

# HTTP client
reqwest = { version = "0.12", features = ["json"] }

# Glob patterns
glob = "0.3"
```

### Infrastructure
- **Database:** PostgreSQL with regular backups
- **Web server:** Axum for webhooks
- **GitHub App:** Registered with permissions (repos: read, pull_requests: read, checks: write, issues: write)
- **High availability:** Critical path for governance
- **Monitoring:** Alert on failures

### Security
- Maintainer private keys in hardware wallets
- Keys never touch GitHub or app
- Database encrypted at rest
- Webhook signatures verified
- Immutable audit log
- Regular security audits

## Comparison to Bitcoin Core

| Aspect | Bitcoin Core | BTCDecoded |
|--------|--------------|------------|
| Merge authority | 1 of ~5 | 6-of-7 (Layer 1) |
| Capture difficulty | Maintainer process | Several independent keyholders |
| Transparency | Informal | Cryptographic |
| Release signing | Individual PGP | Multisig threshold |
| User protection | Trust | Verification |

**Objectively more secure.**

## Success Criteria

### Technical
- ✅ PRs blocked until requirements met
- ✅ No bypass (even GitHub admins blocked at release level)
- ✅ Cross-repo dependencies enforced
- ✅ Emergency mode works, expires automatically
- ✅ Audit trail immutable

### Organizational
- ✅ Transparent governance
- ✅ Clear signing process
- ✅ No invisible actions
- ✅ Auditable decisions
- ✅ Fork-ready

## What Cursor AI Needs to Build

### Phase 1: Core App
1. **Axum webhook server**
   - Receives GitHub webhooks
   - Verifies webhook signatures
   - Routes to appropriate handlers

2. **Configuration loading**
   - Loads from `governance` repo via GitHub API
   - Caches in PostgreSQL
   - Refreshes on webhook from `governance` repo

3. **Database layer**
   - Schema migrations
   - Models for repos, maintainers, PRs, events
   - Query functions

4. **Signature verification**
   - secp256k1 signature verification
   - Multisig threshold validation
   - Message generation for signing

5. **Review period enforcement**
   - Time-based calculation
   - Emergency mode handling
   - Status check generation

6. **Status checks**
   - Post to GitHub via API
   - Update as signatures collected
   - Clear messaging about requirements

7. **Audit logging**
   - Log all governance events
   - Immutable (append-only)
   - Queryable

### Phase 2: Advanced Features
1. **Cross-repo dependency validation**
   - File pattern matching
   - Link PRs across repos
   - Atomic cross-repo merges

2. **Emergency mode**
   - Activation via signatures
   - Expiration timer
   - Status tracking

3. **Meta-governance**
   - Higher thresholds for governance repo
   - Public comment period tracking
   - Configuration reload on changes

4. **Webhook handlers**
   - pull_request (opened, synchronized)
   - pull_request_review (submitted)
   - issue_comment (signature commands)
   - push (bypass detection)
   - repository events (governance repo changes)

### Phase 3: Tooling & Integration
1. **Release signing CLI**
   - Offline signing tool
   - Signature collection
   - Verification utility

2. **Node verification code**
   - Hard-coded pubkeys
   - Signature verification before install
   - Integration points for each layer

3. **Documentation**
   - README for governance-app
   - Maintainer signing guide
   - User verification guide
   - Governance process docs

### Development Approach for Cursor

**Start with:**
1. Basic webhook server (receives and logs events)
2. Database schema and migrations
3. Configuration loading from `governance` repo
4. Signature verification (core crypto)
5. Simple status checks

**Then add:**
6. Review period enforcement
7. Complete webhook handlers
8. Cross-repo logic
9. Emergency mode
10. Meta-governance

**Testing strategy:**
- Unit tests for crypto (signature verification)
- Integration tests for webhooks (mock GitHub)
- End-to-end tests with test repos
- Manual testing with real PRs in test organization

**Key principles:**
- Fail closed (if validation fails, block merge)
- Idempotent operations (webhooks can arrive multiple times)
- Clear error messages (help developers understand requirements)
- Audit everything (every governance action logged)

## Repositories Required

1. **governance-app** (Rust) - enforcement engine
2. **governance** (YAML config) - governance rules and maintainer keys
3. **Each project repo** needs `.governance.yml` file

**Total: 2 new repos + config files in existing repos**

## Conclusion

This system publishes a signature schedule for how the software changes. A supermajority of keyholders can still approve a change.

At $2T market cap, Bitcoin deserves governance infrastructure matching its technical sophistication.

**The tools exist. The model is proven. The cryptography is sound.**

**What's needed: execution.**