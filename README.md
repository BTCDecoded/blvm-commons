# Bitcoin Commons Governance System

## ⚠️ UNRELEASED: This system is not yet activated or tested in production

**Current Status: Phase 1 (Infrastructure Building)**

This repository contains the Bitcoin Commons governance and implementation ecosystem, managed by the BTCDecoded GitHub organization. The system implements a constitutional governance model that makes Bitcoin governance **6x harder to capture** than Bitcoin Core's current model.

## 🚨 Important Disclaimers

### Current Status
- ✅ **Infrastructure Complete**: All core components implemented
- ⚠️ **Not Yet Activated**: Governance rules are not enforced
- 🔧 **Test Keys Only**: No real cryptographic enforcement
- 📋 **Development Phase**: System is in rapid AI-assisted development

### What This Means
- **Production Quality**: The codebase is production-quality in many respects
- **Not Battle-Tested**: Has not been tested in real-world scenarios
- **Expect Changes**: Rapid development means frequent updates
- **Use at Your Own Risk**: This is experimental software

### Timeline
- **Phase 2 Activation**: 3-6 months (governance enforcement begins)
- **Phase 3 Full Operation**: 12+ months (mature, stable system)
- **Current Phase**: Infrastructure building and testing

## 📁 Project Structure

For a detailed overview of the project directory structure, see [DIRECTORY_STRUCTURE.md](./DIRECTORY_STRUCTURE.md).

For comprehensive system status and verified implementation status, see [SYSTEM_STATUS.md](./SYSTEM_STATUS.md).

**📚 Documentation**: See [Documentation Index](./docs/INDEX.md) for complete navigation of all documentation.

## 🏗️ Architecture Overview

### Constitutional Governance Model

Bitcoin Commons implements a **5-tier constitutional governance system**:

1. **Tier 1: Routine Maintenance** (3-of-5, 7 days)
   - Bug fixes, documentation, performance optimizations
   - Non-consensus changes only

2. **Tier 2: Feature Changes** (4-of-5, 30 days)
   - New RPC methods, P2P changes, wallet features
   - Must include technical specification

3. **Tier 3: Consensus-Adjacent** (5-of-5, 90 days + economic node veto)
   - Changes affecting consensus validation code
   - Economic nodes can veto (30%+ hashpower or 40%+ economic activity)

4. **Tier 4: Emergency Actions** (4-of-5, 24-hour notification)
   - Critical security patches, network-threatening bugs
   - Real-time economic node oversight, post-mortem required

5. **Tier 5: Governance Changes** (Special process, 180 days)
   - Changes to governance rules themselves
   - Requires economic node signaling (50%+ hashpower, 60%+ economic activity)

### Core Components

- **`bllvm-commons/`** - GitHub App for governance enforcement
- **`developer-sdk/`** - Cryptographic primitives and CLI tools
- **`governance/`** - Governance configuration and documentation

## 🚀 Quick Start (Development Only)

### Prerequisites
- Rust 1.70+
- SQLite3
- Git

### Setup Development Environment

```bash
# Clone repositories (BTCDecoded is the GitHub organization)
git clone https://github.com/btcdecoded/bllvm-commons.git
cd bllvm-commons

# Set up bllvm-commons
cd bllvm-commons
cargo build
cargo test

# Set up developer-sdk
cd ../developer-sdk
cargo build
cargo test
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test suites
cargo test --test economic_nodes_test
cargo test --test governance_fork_test
cargo test --test github_integration_test
cargo test --test e2e_test
```

## 📚 Documentation

### Core Documentation
- [Governance Process](governance/GOVERNANCE.md) - How governance works
- [System Design](governance/DESIGN.md) - Architecture and design decisions
- [Developer Guide](developer-sdk/README.md) - SDK usage and examples

### Development Guides
- [Maintainer Guide](governance/MAINTAINER_GUIDE.md) - For maintainers
- [Economic Node Guide](governance/ECONOMIC_NODE_GUIDE.md) - For economic nodes
- [Deployment Guide](bllvm-commons/DEPLOYMENT.md) - Deployment instructions

### Development Roadmap
- [Phase 1B Plan](governance-system-review.plan.md) - Current development plan
- [Phase Activation](governance/PHASE_ACTIVATION.md) - Activation timeline

## 🔧 Development Status

### Phase 1B: Extended Governance Features (Current)

#### ✅ Completed
- [x] **Track 1**: Economic Node Infrastructure
  - Database schema and migrations
  - Node registry and qualification system
  - Veto signal collection and threshold calculation
  - Integration with Tier 3 validation

- [x] **Track 2**: Governance Fork Capability
  - Configuration export and versioning
  - Adoption tracking and metrics
  - Multiple ruleset support
  - Dashboard for adoption statistics

- [x] **Track 3**: GitHub Status Check Integration
  - Status check posting and updating
  - Merge blocking and enforcement
  - Webhook integration
  - PR classification and tier detection

- [x] **Track 4**: Comprehensive Testing
  - Economic node tests
  - Governance fork tests
  - GitHub integration tests
  - End-to-end scenario tests

- [x] **Track 5**: Disclaimer Documentation
  - Organization-level disclaimers
  - Repository-level warnings
  - Development status documentation

#### 🚧 In Progress
- [ ] **Track 6**: Additional Documentation
  - Maintainer guide
  - Economic node guide
  - Deployment guide

### Next Steps

1. **Complete Documentation**: Finish remaining guides
2. **Security Review**: Begin security audit process
3. **Community Feedback**: Gather input from Bitcoin community
4. **Phase 2 Preparation**: Prepare for governance activation

## 🤝 Contributing

### For Developers
1. **Read the Documentation**: Understand the system architecture
2. **Set Up Development Environment**: Follow setup guides
3. **Run Tests**: Ensure all tests pass
4. **Contribute Code**: Submit pull requests and improvements
5. **Report Issues**: Help identify and fix bugs

### For Organizations
1. **Monitor Development**: Follow progress and updates
2. **Provide Feedback**: Share requirements and use cases
3. **Test in Development**: Experiment with the system
4. **Wait for Phase 2**: Deploy only after official release

### For Researchers
1. **Study the Architecture**: Understand the governance model
2. **Analyze the Code**: Review implementation and design
3. **Provide Feedback**: Share insights and recommendations
4. **Collaborate**: Work with the development team

## 📞 Support

### Development Team
- **GitHub Issues**: Report bugs and feature requests
- **GitHub Discussions**: Ask questions and provide feedback
- **Pull Requests**: Contribute code and improvements

### Security
- **Security Issues**: Report privately to maintainers
- **Vulnerabilities**: Follow responsible disclosure
- **Audit Results**: Will be published when available

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## ⚠️ Final Warning

**This is experimental software in active development. Use at your own risk and do not deploy in production until Phase 2 activation.**

---

**Remember**: This system is designed to make Bitcoin governance more transparent, accountable, and resistant to capture. But it's still in development. Stay informed, provide feedback, and wait for the official release.




