# Security Policy

## Supported Versions

Currently supported versions of Open Syria blockchain:

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

**⚠️ IMPORTANT: Do NOT open public issues for security vulnerabilities.**

If you discover a security vulnerability in the Open Syria blockchain, please report it responsibly by:

### 1. Email Report (Preferred)

Send details to: **security@opensyria.org**

Include:
- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)
- Your contact information

### 2. GitHub Security Advisories

Alternatively, use GitHub's private vulnerability reporting:
- Go to: https://github.com/opensyria/blockchain/security/advisories
- Click "Report a vulnerability"
- Fill in the details

### What to Expect

- **Acknowledgment:** Within 48 hours
- **Initial Assessment:** Within 1 week
- **Status Updates:** Every 2 weeks until resolved
- **Disclosure:** Coordinated disclosure after fix is released

### Scope

Security vulnerabilities we're particularly interested in:

**High Priority:**
- Consensus mechanism bypasses
- Signature verification flaws
- Double-spending attacks
- Network protocol vulnerabilities
- State corruption issues
- Private key exposure
- RPC/API authentication bypasses

**Medium Priority:**
- Denial of Service (DoS) attacks
- Resource exhaustion
- Information disclosure
- Transaction validation bypasses

**Out of Scope:**
- Known issues in dependencies (report to upstream)
- Issues requiring physical access to the node
- Social engineering attacks
- Issues in third-party applications

### Bug Bounty Program

**Status:** Not currently available

We plan to launch a bug bounty program after:
- External security audit completion
- Mainnet launch
- Community funding secured

### Security Best Practices

For users and node operators:

**Private Key Security:**
- Never share private keys
- Use hardware wallets for large amounts
- Backup keys securely (encrypted, offline)
- Use multi-signature accounts for high-value operations

**Node Security:**
- Keep software updated
- Use firewalls (limit exposed ports)
- Regular backups of blockchain data
- Monitor logs for suspicious activity
- Run nodes on dedicated/isolated systems

**Network Security:**
- Use TLS for API endpoints
- Implement rate limiting
- Validate all inputs
- Use authentication for administrative functions

### Disclosure Policy

**Coordinated Disclosure:**

1. **Private Disclosure:** You report to us privately
2. **Investigation:** We confirm and develop a fix
3. **Fix Release:** We release patched version
4. **Public Disclosure:** 30 days after fix release
5. **Credit:** You receive credit in security advisory (if desired)

**Timeline:**
- **Day 0:** Vulnerability reported
- **Day 1-7:** Confirmation and assessment
- **Day 7-30:** Develop and test fix
- **Day 30:** Release patched version
- **Day 60:** Public disclosure (if appropriate)

### Security Advisories

Published security advisories:
- GitHub: https://github.com/opensyria/blockchain/security/advisories
- Website: (Coming soon)

### Security Updates

**How to stay informed:**
- Watch the repository for security updates
- Subscribe to GitHub security advisories
- Follow @opensyria on social media (Coming soon)

### Hall of Fame

We recognize researchers who responsibly disclose vulnerabilities:

*No vulnerabilities reported yet.*

---

## Development Security

For contributors:

**Code Review:**
- All PRs require review before merge
- Security-sensitive changes require 2+ reviews
- Automated security scanning via GitHub Actions

**Dependencies:**
- Regular audits with `cargo audit`
- Dependabot enabled for automated updates
- Pin critical dependencies

**Testing:**
- Security test cases required
- Fuzzing for critical components
- Integration tests for consensus logic

**Cryptography:**
- Use only well-established libraries
- No custom crypto implementations
- Regular review of cryptographic code

---

## Contact

- **Security Email:** security@opensyria.org
- **GPG Key:** (Coming soon)
- **Security Advisories:** https://github.com/opensyria/blockchain/security/advisories

---

**Last Updated:** November 18, 2025  
**Policy Version:** 1.0
