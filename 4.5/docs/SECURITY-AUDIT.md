# Security Audit Documentation

This document details the security audit processes and a checklist for ensuring the Monero Marketplace adheres to production-grade security standards.

## 1. Security Scans

Regular execution of the following security scanning tools is crucial for identifying vulnerabilities and maintaining a secure application.

### 1.1. `cargo audit` (Dependency Vulnerabilities)

`cargo audit` checks Rust project dependencies for known security vulnerabilities. It should be integrated into the CI/CD pipeline.

**Command:**
```bash
cargo audit
```

### 1.2. `trivy` (Container Scanning)

Trivy is a comprehensive security scanner for container images, file systems, and Git repositories. It detects vulnerabilities, misconfigurations, secrets, and more.

**Command (example for Docker image):**
```bash
trivy image --severity HIGH,CRITICAL your-docker-image-name:tag
```

### 1.3. `sqlmap` (SQL Injection)

SQLMap is an open-source penetration testing tool that automates the process of detecting and exploiting SQL injection flaws and taking over database servers.

**Command (example):**
```bash
sqlmap -u "http://your-app.com/api/v1/endpoint?param=value" --batch --risk=3 --level=5
```

### 1.4. `OWASP ZAP` (Web Application Scanning)

OWASP Zed Attack Proxy (ZAP) is one of the world’s most popular free security tools and is actively maintained by a dedicated international team of volunteers. It can help you automatically find security vulnerabilities in your web applications.

**Usage:**
- Run ZAP in daemon mode for automated scans in CI/CD.
- Configure active and passive scans against the deployed application.

### 1.5. `lynis` (System Hardening)

Lynis is a security auditing tool for Unix-like operating systems. It performs an extensive health scan of your systems to support system hardening and compliance testing.

**Command:**
```bash
sudo lynis audit system
```

## 2. Security Hardening Checklist

This checklist ensures that critical security measures are in place and properly configured.

- [ ] **No hardcoded credentials:** All sensitive information (API keys, database passwords, etc.) are loaded from environment variables or a secure secrets management system (e.g., SOPS).
- [ ] **Secrets encrypted at rest:** Sensitive configuration files and data are encrypted when stored (e.g., using SOPS with Age).
- [ ] **TLS 1.3 enforced:** All external communication uses TLS 1.3, with strong ciphers, enforced by the Nginx reverse proxy.
- [ ] **HSTS enabled:** HTTP Strict Transport Security is enabled to prevent downgrade attacks and ensure browsers always connect via HTTPS.
- [ ] **Rate limiting active:** Nginx is configured with rate limiting to protect against brute-force attacks and denial-of-service attempts.
- [ ] **Firewall restricts RPC:** The host firewall (UFW) is configured to deny direct external access to the backend application port (8080) and Monero RPC ports (18082-18084).
- [ ] **Database encrypted (SQLCipher):** The application database is encrypted at rest using SQLCipher.
- [ ] **Backups encrypted (GPG):** All database and wallet backups are encrypted using GPG.
- [ ] **Regular security audits:** Automated scans (`cargo audit`, `trivy`) are integrated into the CI/CD pipeline, and periodic manual penetration testing is performed.
- [ ] **Least privilege principle:** Application runs with a non-root user in Docker containers, and file permissions are set appropriately.
- [ ] **Logging and monitoring:** Comprehensive logging (Loki) and security monitoring (Prometheus alerts) are in place to detect and respond to security incidents.
