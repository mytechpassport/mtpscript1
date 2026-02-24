# MTPScript SOC 2 Compliance Documentation

## Overview

MTPScript implements SOC 2 Type II compliance controls for Security, Availability, and Confidentiality (SAC) trust service criteria. This document maps MTPScript features to SOC 2 requirements.

## Security Controls (CC1-CC9)

### Access Control (CC1)
- **VM Isolation**: Each request executes in a fresh VM clone, ensuring complete memory isolation
- **Request Scoping**: Per-request VM cloning prevents cross-request data leakage
- **Gas Limits**: Computational resource limits prevent resource exhaustion attacks

### Communication and Information (CC2)
- **TLS Enforcement**: All HTTP requests require valid TLS certificates
- **Canonical JSON**: Deterministic JSON serialization prevents parsing ambiguities
- **Request Validation**: Structured request parsing with size limits

### Risk Management (CC3)
- **Audit Logging**: All effects log operations with correlation IDs
- **Gas Metering**: Resource usage tracking and limits
- **Idempotency Keys**: Transaction deduplication prevents duplicate operations

### Monitoring (CC4)
- **Request Logging**: Complete audit trail of all API requests
- **Error Logging**: Structured error reporting without stack traces
- **Performance Monitoring**: Gas usage and execution time tracking

### Change Management (CC5)
- **Build Verification**: SHA-256 signed build artifacts
- **Dependency Pinning**: Git-hash pinned dependencies in lock files
- **Reproducible Builds**: Deterministic compilation process

### Logical and Physical Access (CC6)
- **Snapshot Security**: VM snapshots with selective memory wiping
- **File System Isolation**: No direct file system access
- **Network Controls**: Effect-based network access control

### System Operations (CC7)
- **Incident Response**: Structured error handling and logging
- **Backup Procedures**: Snapshot-based recovery mechanisms
- **Capacity Planning**: Gas limits and resource allocation controls

### Confidentiality (CC8)
- **Data Classification**: PCI-classified data handling
- **Memory Wiping**: Secure memory cleanup between requests
- **Encryption**: TLS for all external communications

### Availability (CC9)
- **Fault Tolerance**: VM cloning ensures clean state per request
- **Load Distribution**: Stateless design supports horizontal scaling
- **Resource Limits**: Gas metering prevents cascading failures

## Control Mapping Matrix

| SOC 2 Control | MTPScript Implementation | Evidence |
|---------------|--------------------------|----------|
| CC1.1 | VM cloning per request | `clone_vm()` function |
| CC2.1 | TLS certificate validation | HTTP effect implementation |
| CC3.1 | Gas limit enforcement | `gas_limit` parameter |
| CC4.1 | Structured logging | Log effect with correlation IDs |
| CC5.1 | Build signing | ECDSA-P256 signatures |
| CC6.1 | Memory isolation | VM snapshot cloning |
| CC7.1 | Error boundaries | Result/Option types |
| CC8.1 | Secure memory wipe | `secure_wipe()` function |
| CC9.1 | Stateless design | No persistent state between requests |

## Audit Evidence

### Automated Controls
- Unit tests verify VM isolation
- Integration tests confirm TLS validation
- Gas metering tests validate resource limits
- Determinism tests ensure reproducible execution

### Manual Controls
- Code review for security vulnerabilities
- Dependency security scanning
- Build process verification
- Deployment security review

## Compliance Assessment

### Type II Compliance Scope
- Production AWS Lambda deployment
- Database effects (PostgreSQL)
- HTTP client effects
- Logging and monitoring

### Exclusions
- Local development server (non-production)
- TypeScript migration tooling
- Package manager (pre-production)

## Testing and Validation

### Continuous Monitoring
- Automated test suite runs on all deployments
- Gas usage auditing per request
- Error rate monitoring and alerting
- Performance regression detection

### Annual Assessment
- Third-party SOC 2 audit
- Penetration testing
- Code security review
- Dependency vulnerability assessment

## Contact Information

For compliance inquiries:
- Security Team: security@mtpscript.com
- Compliance Officer: compliance@mtpscript.com
- Audit Evidence: Available upon NDA
