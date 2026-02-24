# MTPScript PCI-DSS Compliance Documentation

## Overview

MTPScript implements Payment Card Industry Data Security Standard (PCI-DSS) v4.0 controls for systems handling cardholder data, focusing on secure payment processing and data protection.

## PCI-DSS Scope and Applicability

### Cardholder Data Environment (CDE)
- **In Scope**: Systems processing, storing, or transmitting cardholder data
- **Protected**: VM isolation prevents data leakage between requests
- **Monitored**: Comprehensive audit logging of all data access

### Data Classification
- **Cardholder Data**: Primary Account Number (PAN), expiry dates, cardholder names
- **Sensitive Authentication Data**: CAV2/CVC2, PINs, PIN blocks
- **Prohibited**: Never store sensitive authentication data

## Security Management (Requirement 1-2)

### Network Security Controls (Requirement 1)
- **Firewall Configuration**: Effect system restricts network access
- **Network Segmentation**: VM isolation provides network segmentation
- **Default Deny**: All access must be explicitly declared in effects

### Secure Configuration (Requirement 2)
- **System Hardening**: Minimal attack surface through effect isolation
- **Software Updates**: Automated dependency updates and security patching
- **Configuration Standards**: Build-time security configuration validation

## Cardholder Data Protection (Requirement 3-4)

### Data Encryption (Requirement 3)
- **Data at Rest**: Database effect encryption for stored card data
- **Data in Transit**: TLS 1.2+ required for all card data transmission
- **Strong Cryptography**: AES-256 for data encryption, SHA-256 for integrity

### Data Protection (Requirement 4)
- **PAN Masking**: Only last 4 digits stored/displayed
- **Data Retention**: Minimal data retention policies enforced
- **Secure Transmission**: TLS encryption for all card data

## Vulnerability Management (Requirement 5-7)

### Malware Protection (Requirement 5)
- **Antivirus Software**: Build-time malware scanning
- **Regular Updates**: Automated security signature updates
- **Malware Monitoring**: Runtime behavior monitoring

### Secure Systems (Requirement 6)
- **Security Updates**: Automated patching of dependencies
- **Code Review**: Required security review for payment code
- **Change Control**: Version control and approval workflows

### Access Control (Requirement 7)
- **Access Management**: Effect declarations define data access
- **Need to Know**: Principle of least privilege implementation
- **Access Reviews**: Automated access validation

## Access Control Measures (Requirement 8-10)

### Access Management (Requirement 8)
- **Unique IDs**: Request-level VM isolation provides unique contexts
- **Strong Authentication**: Not applicable (server-to-server)
- **Password Policies**: Not applicable (API-based access)

### Physical Access (Requirement 9)
- **Physical Security**: Cloud provider data center security
- **Media Inventory**: Build artifact inventory and verification
- **Secure Media Transport**: Encrypted deployment artifacts

### Logging and Monitoring (Requirement 10)
- **Audit Logging**: All card data access logged with correlation IDs
- **Log Review**: Automated log analysis and alerting
- **Time Synchronization**: Deterministic timestamp generation
- **Log Integrity**: SHA-256 signed log entries

## Regular Testing (Requirement 11-12)

### Security Testing (Requirement 11)
- **Vulnerability Scans**: Automated security scanning in CI/CD
- **Penetration Testing**: Annual penetration testing requirements
- **Intrusion Detection**: Runtime anomaly detection

### Security Policies (Requirement 12)
- **Information Security Policy**: PCI DSS compliance requirements
- **Risk Assessment**: Annual risk assessment procedures
- **Security Awareness**: Developer security training requirements

## PCI DSS Control Mapping

### Technical Controls
| Requirement | Description | MTPScript Implementation |
|-------------|-------------|--------------------------|
| 1.2.1 | Restrict inbound traffic | Effect-based network controls |
| 1.3.1 | Prohibit direct public access | VM isolation and access controls |
| 2.2.4 | Configure system security parameters | Build-time security hardening |
| 3.4.1 | Render PAN unreadable | Database encryption and masking |
| 3.5.1 | Protect keys used for encryption | Secure key management |
| 4.1.1 | Use strong cryptography | TLS 1.2+ for all communications |
| 6.3.1 | Review custom code | Required security code reviews |
| 7.1.1 | Limit access to system components | Effect declaration scoping |
| 8.2.1 | Employ unique IDs | VM cloning per request |
| 10.2.1 | Implement audit logging | Comprehensive audit trails |
| 10.3.1 | Record audit log entries | Correlation ID tracking |

### Operational Controls
- **Change Management**: Git-based change control with approval
- **Incident Response**: Structured incident response procedures
- **Third-Party Service Providers**: Vendor security assessments
- **Annual Compliance Validation**: PCI DSS assessment procedures

## Cardholder Data Handling Procedures

### Data Processing
- **Collection**: Encrypted transmission from payment processors
- **Processing**: Type-safe processing with validation
- **Storage**: Encrypted storage with access controls
- **Deletion**: Secure deletion and memory wiping

### Data Flow Security
- **Input Validation**: Type system prevents malformed data
- **Processing Isolation**: VM cloning ensures clean execution
- **Output Filtering**: Response sanitization and masking
- **Error Handling**: Secure error responses without data leakage

## Incident Response for PCI

### Breach Detection
- **Monitoring**: Real-time monitoring for suspicious activity
- **Alerting**: Automated alerting for security events
- **Containment**: Immediate isolation of affected systems

### Breach Response
- **Notification**: Required notifications to payment brands and acquirers
- **Investigation**: Forensic analysis using audit logs
- **Remediation**: Security control updates and patches
- **Reporting**: Regulatory reporting requirements

## PCI DSS Validation

### Self-Assessment Questionnaire (SAQ)
- **SAQ Type**: SAQ A (card-not-present merchants)
- **Annual Validation**: Required annual compliance assessment
- **Quarterly Scanning**: Automated vulnerability scanning

### Qualified Security Assessor (QSA)
- **Annual Assessment**: Third-party PCI DSS validation
- **Remediation**: Corrective action for identified issues
- **Attestation**: Formal compliance attestation

## Compensating Controls

### Approved Compensating Controls
- **VM Isolation**: Provides equivalent protection to network segmentation
- **Effect System**: Implements principle of least privilege
- **Deterministic Execution**: Prevents code injection and manipulation

### Documentation Requirements
- **Compensating Control Worksheet**: Detailed justification for each control
- **Risk Analysis**: Risk assessment supporting compensating controls
- **Validation Procedures**: Testing procedures for compensating controls

## Contact Information

For PCI DSS compliance inquiries:
- PCI DSS Compliance Officer: pci@mtpscript.com
- Security Team: security@mtpscript.com
- Payment Processing: payments@mtpscript.com

## Supporting Documentation

- PCI DSS Attestation of Compliance (AOC)
- Self-Assessment Questionnaire (SAQ)
- Compelling Controls Worksheet
- Security Assessment Procedures
