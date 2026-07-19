# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability within termcompositor, please send an email to the project maintainers. All security vulnerabilities will be promptly addressed.

**Please do NOT report security vulnerabilities through public GitHub issues.**

## Reporting Process

1. **Preferred Method**: Use [GitHub Security Advisories](https://github.com/216598762/termcompositor/security/advisories/new)
2. **Alternative**: Email **216598762@proton.me**
3. **Subject Line**: Use `[SECURITY] Brief Description`

## What to Include

- Type of vulnerability (code execution, information disclosure, etc.)
- Affected component (Kitty encoder, Sixel encoder, layer rendering, etc.)
- Severity (Critical/High/Medium/Low)
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

## Response Timeline

| Action | Timeframe |
|--------|-----------|
| Acknowledgment | 24-48 hours |
| Initial Assessment | 3-5 business days |
| Resolution | 1-2 weeks (depending on severity) |

## Disclosure Policy

We follow responsible disclosure practices:

- We will work with reporters to understand and address issues
- We will credit reporters in release notes (unless they prefer anonymity)
- We will not take legal action against reporters who follow this policy

## Scope

### In Scope

- Vulnerabilities in termcompositor code
- Security issues in dependencies
- Code execution vulnerabilities
- Information disclosure
- Buffer overflow or memory safety issues

### Out of Scope

- Vulnerabilities in third-party terminals or terminal emulators
- Issues requiring physical access to the system
- Social engineering attacks
- Denial of service through resource exhaustion

## Security Best Practices

### For Users

- Keep termcompositor updated to the latest version
- Review output before sending to terminals
- Use environment variables for sensitive configuration
- Run with minimal permissions when possible

### For Contributors

- Never commit secrets or API keys
- Use environment variables for sensitive configuration
- Review code for security issues before submitting PRs
- Follow secure coding practices

## Dependency Security

We use `cargo audit` to check for known vulnerabilities in dependencies:

```bash
# Install cargo-audit
cargo install cargo-audit

# Run security audit
cargo audit
```

## Contact

For security inquiries, please contact:

- Email: **216598762@proton.me**
- GitHub Security Advisories: [Create Advisory](https://github.com/216598762/termcompositor/security/advisories/new)

---

*Last updated: July 2026*

*This security policy is adapted from [GitHub's security policy examples](https://docs.github.com/en/code-security/security-advisories/guidance/reports-and-policies/adding-a-security-policy-to-your-repository).*
