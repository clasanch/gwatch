# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

If you discover a security vulnerability in gwatch, please report it responsibly:

1. **Do NOT** open a public GitHub issue for security vulnerabilities
2. Email: claudio[@]sanchez[.]simplelogin[.]com
3. Include:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

### What to expect

- Acknowledgment within 48 hours
- Status update within 7 days
- We aim to release fixes within 30 days for critical issues

### Scope

Security issues we are interested in:
- Remote code execution
- Path traversal vulnerabilities
- Command injection via file paths
- Sensitive data exposure

Out of scope:
- Issues requiring physical access
- Social engineering
- Denial of service (unless trivially exploitable)

## Security Best Practices

When using gwatch:
- Only run in repositories you trust
- Review the `editor.command` config before using
- Be cautious with custom ignore patterns
