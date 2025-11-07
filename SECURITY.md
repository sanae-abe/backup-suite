# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 1.0.x   | :white_check_mark: |
| < 1.0   | :x:                |

## Security Features

backup-suite implements enterprise-grade security features:

### 🔒 Encryption
- **AES-256-GCM**: Authenticated encryption for confidential data protection
- **Argon2 Key Derivation**: Secure password-based key generation (NIST SP 800-63B compliant)
- **Password Policy**: Strength evaluation and auto-generation capabilities
- **Secure Memory Management**: Sensitive data erasure with zeroize

### 🛡️ Additional Security Measures
- **Audit Log**: Event logging with HMAC-SHA256 tampering detection (ISO 27001 A.12.6 compliant)
- **Integrity Verification**: SHA-256 hash-based backup verification
- **Path Traversal Protection**: safe_join implementation
- **Memory Safety**: Rust's type system prevents buffer overflows and memory leaks

## Reporting a Vulnerability

If you discover a security vulnerability in backup-suite, please report it responsibly:

### 📧 Contact Information
- **Email**: sanae-abe@m3.com
- **Subject**: `[backup-suite] Security Vulnerability Report`

### 📝 What to Include
Please provide the following information in your report:

1. **Description**: Clear description of the vulnerability
2. **Affected Version(s)**: Which version(s) are affected
3. **Steps to Reproduce**: Detailed steps to reproduce the issue
4. **Potential Impact**: Assessment of the security impact
5. **Proof of Concept**: Code or commands demonstrating the vulnerability (if applicable)
6. **Suggested Fix**: Your recommendations for fixing the issue (optional)

### ⏱️ Response Timeline
- **Initial Response**: Within 48 hours
- **Status Update**: Within 7 days
- **Security Patch**: Target 30 days for critical vulnerabilities

### 🔐 Disclosure Policy
- **Please do not open a public issue for security vulnerabilities**
- We will work with you to address the issue before public disclosure
- We will credit you in the security advisory (unless you prefer to remain anonymous)
- After the fix is released, we will publish a security advisory on GitHub

## Security Best Practices for Users

### 🔑 Password Management
- Use strong, unique passwords for encrypted backups
- Consider using `--generate-password` for auto-generated strong passwords
- Store passwords securely (password manager recommended)
- Never commit passwords to version control

### 📁 Backup Security
- Enable encryption for sensitive data: `backup-suite run --encrypt --password "your-password"`
- Verify backup integrity regularly: `backup-suite verify`
- Protect backup destination with appropriate file permissions
- Use incremental backups to minimize exposure window

### 🔄 Updates
- Keep backup-suite updated to the latest version
- Subscribe to GitHub releases for security notifications
- Review CHANGELOG.md for security-related updates

## Security Audit

backup-suite undergoes regular security reviews:

- **Static Analysis**: cargo-clippy with security lints
- **Dependency Auditing**: cargo-audit, cargo-deny
- **Vulnerability Scanning**: Dependabot alerts enabled
- **Testing**: 343 tests including 23 security-focused tests

### Latest Security Audit (v1.0.0)
- ✅ All dependencies audited (no known vulnerabilities)
- ✅ 23 security tests passed
- ✅ Memory safety verified (Rust type system)
- ✅ Cryptographic implementation reviewed

## Known Security Considerations

### Current Limitations
1. **Local Storage Only**: v1.0.0 supports local backups only
   - Cloud storage support planned for future versions
   - Users must secure their backup destination independently

2. **Password Storage**: Passwords are not stored
   - Users must remember or securely store encryption passwords
   - Lost passwords cannot be recovered (by design)

3. **Metadata Exposure**: Backup metadata is not encrypted
   - File names and directory structure are visible
   - Consider this when backing up sensitive data

## Responsible Disclosure

We follow responsible disclosure principles:

1. **Private Reporting**: Report vulnerabilities privately via email
2. **Coordinated Disclosure**: Work with us before public disclosure
3. **Credit**: We acknowledge security researchers who report responsibly
4. **Transparency**: Public advisories after fixes are released

## Contact

For security-related questions or concerns:
- **Security Email**: sanae-abe@m3.com
- **General Issues**: https://github.com/sanae-abe/backup-suite/issues (non-security only)

---

Thank you for helping keep backup-suite and its users safe!
