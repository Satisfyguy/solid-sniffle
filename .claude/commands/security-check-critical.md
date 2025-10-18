# Security Theatre Check (Critical Only)

Execute the security theatre detection showing only CRITICAL severity issues.

## Usage
```
/security-check-critical
```

## What it does
Scans for the most dangerous patterns only:
- Hardcoded credentials (passwords, API keys, tokens)
- Unsafe blocks without SAFETY comments
- Critical security vulnerabilities

## When to use
- Quick security audit before deployment
- Focus on blocking issues only
- Pre-production validation
- Security-critical code reviews

## Exit Codes
- Exit 0: No critical issues found
- Exit 1: CRITICAL issues detected (blocking)
