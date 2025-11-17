# Governance Improvements - Preparation Phase

## Overview

This document outlines preparatory work for the three governance improvements that can be done **without requiring full operational infrastructure**. These are foundational steps that set up the infrastructure, tooling, and processes needed for the full implementation.

## Preparation Principles

1. **Infrastructure First**: Set up key management, signing tools, and data structures
2. **Tooling**: Create scripts and utilities for signing and verification
3. **Documentation**: Document processes and procedures
4. **Testing**: Create test infrastructure for validation
5. **No Operational Dependencies**: Work that doesn't require live maintainers, economic nodes, or user nodes

---

## 1. Binary Signing with Math Proof Verification

### Preparation Tasks

#### 1.1 Maintainer Key Infrastructure Setup

**Status**: Partially exists (test keys only)

**Tasks**:
- [ ] **Key Generation Tooling** (1-2 days)
  - Enhance `bllvm-sdk/src/bin/bllvm-keygen` to support maintainer key generation
  - Add key format validation
  - Add key backup/restore functionality
  - Document key generation ceremony

- [ ] **Key Storage Structure** (1 day)
  - Design key storage format (encrypted, HSM-ready)
  - Create key rotation procedures
  - Document key backup requirements
  - Create key inventory system

- [ ] **Key Distribution Mechanism** (2-3 days)
  - Design secure key distribution process
  - Create key exchange protocol
  - Document key sharing procedures
  - Create key verification process

**Deliverables**:
- Enhanced `bllvm-keygen` tool
- Key storage format specification
- Key management documentation
- Key generation ceremony script

#### 1.2 Signing Tooling

**Status**: Basic signing exists (`sign-pr` for PRs)

**Tasks**:
- [ ] **Binary Signing Tool** (2-3 days)
  - Create `bllvm-sign-binary` tool for signing binaries
  - Support multisig threshold signing
  - Create signature aggregation tool
  - Add signature verification tool

- [ ] **Verification Bundle Signing** (2-3 days)
  - Extend `make_verification_bundle.sh` to include signing step
  - Create bundle signature format
  - Add bundle signature verification
  - Integrate with release process

- [ ] **Release Signing Workflow** (1-2 days)
  - Create release signing script
  - Document signing process
  - Create signing checklist
  - Add signing automation (where possible)

**Deliverables**:
- `bllvm-sign-binary` tool
- Enhanced `make_verification_bundle.sh` with signing
- Release signing workflow documentation
- Signing automation scripts

#### 1.3 Verification Infrastructure

**Status**: Basic verification exists

**Tasks**:
- [ ] **Binary Verification Tool** (2-3 days)
  - Create `bllvm-verify-binary` tool
  - Verify binary signatures
  - Verify verification bundle signatures
  - Verify bundle contents match binary
  - Verify Kani proofs in bundle

- [ ] **Node Startup Verification** (3-4 days)
  - Design optional startup verification
  - Create verification cache mechanism
  - Add verification status reporting
  - Document verification process

- [ ] **Verification Bundle Format** (1-2 days)
  - Finalize bundle JSON schema
  - Document bundle structure
  - Create bundle validation tool
  - Add bundle versioning

**Deliverables**:
- `bllvm-verify-binary` tool
- Verification bundle JSON schema
- Node verification integration (optional, opt-in)
- Verification documentation

#### 1.4 OpenTimestamps Integration

**Status**: Basic OTS support exists (needs production hardening)

**Tasks**:
- [ ] **OTS Client Production Hardening** (3-4 days)
  - Replace mock implementations with real OTS client
  - Add error handling and retries
  - Add OTS proof verification
  - Add OTS proof upgrade mechanism

- [ ] **OTS Integration in Release Process** (1-2 days)
  - Integrate OTS stamping into release workflow
  - Add OTS proof to verification bundles
  - Document OTS verification process
  - Create OTS proof storage

**Deliverables**:
- Production-ready OTS client
- OTS integration in release workflow
- OTS verification documentation

#### 1.5 Documentation and Processes

**Tasks**:
- [ ] **Release Signing Documentation** (1-2 days)
  - Document complete signing process
  - Create maintainer signing guide
  - Document verification process for users
  - Create troubleshooting guide

- [ ] **Key Management Documentation** (1 day)
  - Document key generation ceremony
  - Document key storage requirements
  - Document key rotation procedures
  - Create key emergency procedures

**Deliverables**:
- Complete signing process documentation
- Key management documentation
- User verification guide

### Preparation Timeline

**Total: 3-4 weeks** (can be done in parallel with other work)

- Week 1: Key infrastructure + Signing tooling
- Week 2: Verification infrastructure
- Week 3: OTS integration + Documentation
- Week 4: Testing + Polish

---

## 2. Time-Locked Governance Changes

### Preparation Tasks

#### 2.1 Time Lock Tracking Infrastructure

**Status**: Not implemented

**Tasks**:
- [ ] **Database Schema** (1 day)
  - Add time lock tracking tables to governance-app database
  - Design time lock state machine
  - Create time lock query interface
  - Add time lock expiration handling

- [ ] **Time Lock Logic** (2-3 days)
  - Implement time lock calculation (based on tier)
  - Add time lock start/end tracking
  - Create time lock status reporting
  - Add time lock extension mechanism

- [ ] **Governance-App Integration** (2-3 days)
  - Integrate time lock tracking into PR workflow
  - Add time lock status to PR checks
  - Create time lock notifications
  - Add time lock dashboard

**Deliverables**:
- Time lock database schema
- Time lock tracking logic
- Governance-app integration
- Time lock status API

#### 2.2 User Override Signaling Infrastructure

**Status**: Not implemented

**Tasks**:
- [ ] **Signaling Protocol Design** (2-3 days)
  - Design user node signaling protocol
  - Create signal format specification
  - Design signal aggregation mechanism
  - Create signal verification process

- [ ] **Signaling Infrastructure** (3-4 days)
  - Create signaling endpoint (governance-app)
  - Design signal storage and aggregation
  - Create signal query interface
  - Add signal validation logic

- [ ] **Node Signaling Integration** (2-3 days)
  - Design node signaling API
  - Create signaling client library
  - Document signaling process
  - Create signaling test infrastructure

**Deliverables**:
- Signaling protocol specification
- Signaling infrastructure (backend)
- Node signaling client library
- Signaling documentation

#### 2.3 Staged Activation Infrastructure

**Status**: Not implemented

**Tasks**:
- [ ] **Staged Activation Logic** (2-3 days)
  - Design staged activation state machine
  - Create activation stage tracking
  - Add stage transition logic
  - Create stage status reporting

- [ ] **Activation Integration** (1-2 days)
  - Integrate with governance-app
  - Add activation status to PRs
  - Create activation notifications
  - Document activation process

**Deliverables**:
- Staged activation logic
- Activation tracking system
- Activation documentation

#### 2.4 Documentation

**Tasks**:
- [ ] **Time Lock Documentation** (1 day)
  - Document time lock process
  - Document user override mechanism
  - Document staged activation
  - Create troubleshooting guide

**Deliverables**:
- Complete time lock documentation

### Preparation Timeline

**Total: 2-3 weeks**

- Week 1: Time lock tracking + Signaling infrastructure
- Week 2: User override + Staged activation
- Week 3: Integration + Documentation

---

## 3. User Node Signaling

### Preparation Tasks

#### 3.1 Signaling Protocol Design

**Status**: Not implemented

**Tasks**:
- [ ] **Protocol Specification** (3-4 days)
  - Design complete signaling protocol
  - Specify message formats
  - Design Sybil resistance mechanisms
  - Create protocol versioning

- [ ] **Sybil Resistance Design** (2-3 days)
  - Design square root weighting algorithm
  - Design uptime tracking mechanism
  - Design IP deduplication logic
  - Create identity verification process

- [ ] **Protocol Documentation** (1-2 days)
  - Document protocol specification
  - Create protocol examples
  - Document security considerations
  - Create protocol test vectors

**Deliverables**:
- Complete protocol specification
- Sybil resistance design document
- Protocol documentation

#### 3.2 Node Operator Registration

**Status**: Not implemented

**Tasks**:
- [ ] **Registration System Design** (2-3 days)
  - Design operator registration process
  - Design operator identity format
  - Create operator verification process
  - Design operator database schema

- [ ] **Registration Infrastructure** (3-4 days)
  - Create registration endpoint (governance-app)
  - Create operator database
  - Add operator verification logic
  - Create operator management interface

- [ ] **Registration Tooling** (1-2 days)
  - Create operator registration CLI tool
  - Create operator verification tool
  - Document registration process
  - Create registration test infrastructure

**Deliverables**:
- Registration system design
- Registration infrastructure
- Registration tooling
- Registration documentation

#### 3.3 Node Signaling Infrastructure

**Status**: Not implemented

**Tasks**:
- [ ] **Signaling Endpoint** (2-3 days)
  - Create signaling API endpoint
  - Design signal storage
  - Create signal aggregation logic
  - Add signal validation

- [ ] **Node Signaling Client** (3-4 days)
  - Create signaling client library (bllvm-sdk)
  - Integrate with bllvm-node
  - Add signaling configuration
  - Create signaling test infrastructure

- [ ] **Signal Aggregation** (2-3 days)
  - Implement square root weighting
  - Create signal counting logic
  - Add threshold checking
  - Create signal reporting

**Deliverables**:
- Signaling API endpoint
- Node signaling client library
- Signal aggregation system
- Signaling documentation

#### 3.4 Documentation

**Tasks**:
- [ ] **Signaling Documentation** (1-2 days)
  - Document complete signaling process
  - Document operator registration
  - Document Sybil resistance mechanisms
  - Create troubleshooting guide

**Deliverables**:
- Complete signaling documentation

### Preparation Timeline

**Total: 4-5 weeks**

- Week 1: Protocol design + Sybil resistance
- Week 2: Registration system
- Week 3: Signaling infrastructure
- Week 4: Node integration + Aggregation
- Week 5: Documentation + Testing

---

## Combined Preparation Plan

### Phase 1: Foundation (Weeks 1-2)

**Focus**: Key infrastructure and signing tooling

1. **Binary Signing Preparation** (Week 1-2)
   - Key generation tooling
   - Key storage structure
   - Binary signing tool
   - Verification bundle signing

2. **Time Lock Preparation** (Week 1-2)
   - Database schema
   - Time lock logic
   - Basic tracking infrastructure

### Phase 2: Infrastructure (Weeks 3-4)

**Focus**: Verification and signaling infrastructure

1. **Binary Signing Preparation** (Week 3-4)
   - Verification infrastructure
   - OTS integration
   - Node verification (optional)

2. **Time Lock Preparation** (Week 3-4)
   - User override signaling infrastructure
   - Staged activation
   - Integration

### Phase 3: Advanced Preparation (Weeks 5-6)

**Focus**: User node signaling preparation

1. **User Node Signaling Preparation** (Week 5-6)
   - Protocol design
   - Registration system
   - Signaling infrastructure

### Phase 4: Documentation and Testing (Week 7)

**Focus**: Complete documentation and test infrastructure

1. **All Improvements**
   - Complete documentation
   - Test infrastructure
   - Integration testing
   - Process validation

---

## Preparation Deliverables Summary

### Binary Signing
- ✅ Enhanced key generation tooling
- ✅ Binary signing tools
- ✅ Verification tools
- ✅ OTS integration
- ✅ Complete documentation

### Time-Locked Governance
- ✅ Time lock tracking infrastructure
- ✅ User override signaling infrastructure
- ✅ Staged activation system
- ✅ Complete documentation

### User Node Signaling
- ✅ Protocol specification
- ✅ Registration system
- ✅ Signaling infrastructure
- ✅ Complete documentation

---

## Next Steps After Preparation

Once preparation is complete:

1. **Binary Signing**: Ready for maintainer key generation ceremony and first release signing
2. **Time-Locked Governance**: Ready for governance-app integration and first time-locked PR
3. **User Node Signaling**: Ready for operator registration and first signaling test

---

## Notes

- **No Operational Dependencies**: All preparation work can be done without live maintainers, economic nodes, or user nodes
- **Test Infrastructure**: All components should have test infrastructure
- **Documentation First**: Document as you build, not after
- **Incremental**: Can be done incrementally, doesn't need to be all at once

