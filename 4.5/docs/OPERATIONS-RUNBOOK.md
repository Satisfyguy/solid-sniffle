# Operations Runbook

This runbook provides detailed procedures for daily, weekly, and monthly operational tasks, as well as incident response guidelines for the Monero Marketplace.

## 1. Daily Operations (10 minutes)

Perform these checks at the beginning of each day to ensure system health.

-   **Morning Checks:**
    -   **Health Status:** Run the `docker-health-check.sh` script to verify all services are healthy.
        ```bash
        cd /path/to/project/4.5/scripts
        ./docker-health-check.sh
        ```
    -   **Dashboard Review:** Log in to Grafana (`http://localhost:3000`) and review the "System Overview", "HTTP Overview", and "Escrow Overview" dashboards for any anomalies or unusual metrics.
    -   **Alerts Check:** Verify that Alertmanager (`http://localhost:9093`) shows no active critical alerts.
    -   **Log Review:** Briefly check recent application logs for errors or warnings.
        ```bash
        docker-compose -f /path/to/project/4.5/docker/docker-compose.yml logs --since 1h | grep -E "ERROR|WARN"
        ```

## 2. Weekly Tasks

These tasks should be performed once a week.

-   **Security Audit Logs Review:** Review logs from security tools (e.g., `cargo audit` reports, Trivy scans) for any new findings.
-   **Disk Space Check:** Monitor disk usage on all volumes, especially `/app/data` and `/backups`.
    ```bash
    df -h
    ```
-   **Test Backup Restoration:** Periodically run the `test-backup-restore.sh` script to ensure backup integrity and restoration procedures are functional.
    ```bash
    cd /path/to/project/4.5/scripts
    ./test-backup-restore.sh
    ```
-   **Prometheus Alerts Review:** Review and fine-tune Prometheus alert rules based on operational experience.
-   **Update Dependencies:** Check for and apply minor version updates for Docker images and Rust dependencies.

## 3. Monthly Tasks

These tasks should be performed once a month.

-   **Full DR Test:** Conduct a full disaster recovery simulation (e.g., Scenario 3: Complete Server Loss) in a staging environment.
-   **Security Scan:** Perform a comprehensive security scan using tools like `trivy` for containers and `lynis` for system hardening.
-   **Certificate Renewal Verification:** Ensure SSL certificates are renewing automatically and are valid for the next 30 days.
-   **Performance Review:** Analyze Grafana dashboards for long-term performance trends, especially p95 response times and resource utilization.

## 4. Incident Response

This section provides guidance for responding to common incidents.

### 4.1. High CPU Usage (>90%)

-   **Symptoms:** Slow application response, `top`/`htop` shows high CPU for `server` container.
-   **Diagnosis Commands:**
    ```bash
    docker stats marketplace-server
    docker-compose -f /path/to/project/4.5/docker/docker-compose.yml logs server | grep "ERROR"
    ```
-   **Remediation Steps:**
    1.  Check application logs for recent errors or unusual activity.
    2.  Consider scaling up resources (CPU limits in `docker-compose.yml`).
    3.  If a specific endpoint is causing the spike, consider rate-limiting it further in Nginx.
    4.  Restart the `server` container if no clear cause is found.

### 4.2. Database Locked Error

-   **Symptoms:** Application logs show `SQLITE_BUSY` or `database is locked` errors.
-   **Diagnosis Commands:**
    ```bash
    docker-compose -f /path/to/project/4.5/docker/docker-compose.yml logs server | grep "locked"
    ls -l /path/to/project/data/marketplace.db*
    ```
-   **Remediation Steps:**
    1.  Stop all services: `docker-compose -f /path/to/project/4.5/docker/docker-compose.yml down`.
    2.  Remove SQLite journal/WAL files: `rm /path/to/project/data/marketplace.db-wal /path/to/project/data/marketplace.db-shm`.
    3.  Restart services: `docker-compose -f /path/to/project/4.5/docker/docker-compose.yml up -d`.

### 4.3. Wallet RPC Unreachable

-   **Symptoms:** Monero RPC related errors in application logs, `monero-wallet-rpc-*` containers unhealthy.
-   **Diagnosis Commands:**
    ```bash
    docker-compose -f /path/to/project/4.5/docker/docker-compose.yml ps
    docker-compose -f /path/to/project/4.5/docker/docker-compose.yml logs monero-wallet-rpc-buyer
    ```
-   **Remediation Steps:**
    1.  Restart the affected wallet RPC container: `docker-compose -f /path/to/project/4.5/docker/docker-compose.yml restart monero-wallet-rpc-buyer`.
    2.  If restart fails, check container logs for specific errors.
    3.  Consider restoring the wallet from backup if files are corrupted (refer to Disaster Recovery Plan).

### 4.4. Disk Space Low

-   **Symptoms:** Alerts from Prometheus, `df -h` shows low free space on `/app/data` or `/backups`.
-   **Diagnosis Commands:**
    ```bash
    df -h
    docker system df
    ```
-   **Remediation Steps:**
    1.  Clean up old Docker images/volumes: `docker system prune -a`.
    2.  Manually remove old backups if retention policy failed: `find /backups -type f -mtime +90 -delete`.
    3.  Increase disk size of the underlying volume.

### 4.5. Memory Leak

-   **Symptoms:** Gradual increase in memory usage over time, eventual OOM kills or slow performance.
-   **Diagnosis Commands:**
    ```bash
    docker stats --no-stream
    # Inside container: valgrind or other memory profiling tools (requires debug build)
    ```
-   **Remediation Steps:**
    1.  Restart the affected service (e.g., `marketplace-server`).
    2.  Analyze recent code changes for potential memory leaks.
    3.  Implement memory limits in `docker-compose.yml` to prevent a single service from consuming all host memory.

### 4.6. SSL Certificate Expired

-   **Symptoms:** Browser warnings about invalid certificates, Nginx logs show SSL errors.
-   **Diagnosis Commands:**
    ```bash
    sudo certbot certificates
    sudo nginx -t
    ```
-   **Remediation Steps:**
    1.  Manually renew certificate: `sudo certbot renew --force-renewal`.
    2.  Reload Nginx: `sudo systemctl reload nginx`.
    3.  Verify cron job for auto-renewal is correctly configured.

### 4.7. DDoS Attack

-   **Symptoms:** Extreme spike in request rate, high CPU/network usage, application unresponsive.
-   **Diagnosis Commands:**
    ```bash
    nginx access.log | awk '{print $1}' | sort | uniq -c | sort -nr | head -n 10
    ```
-   **Remediation Steps:**
    1.  Enable Nginx rate limiting (if not already active or increase limits).
    2.  Implement WAF (Web Application Firewall) rules.
    3.  Contact your hosting provider for assistance with traffic filtering.

### 4.8. Data Breach

-   **Symptoms:** Unauthorized access, data exfiltration, suspicious activity in logs.
-   **Diagnosis Commands:**
    1.  Isolate affected systems immediately.
    2.  Review all access logs (SSH, application, database) for unusual logins or commands.
    3.  Check `SECURITY-AUDIT.md` for recent vulnerability scan results.
-   **Remediation Steps:**
    1.  Engage incident response team.
    2.  Rotate all credentials (database, API keys, SSH keys).
    3.  Perform a full forensic analysis.
    4.  Notify affected users and authorities as required by law.

### 4.9. Service Outage

-   **Symptoms:** All services down, application completely unreachable.
-   **Diagnosis Commands:**
    ```bash
    docker-compose -f /path/to/project/4.5/docker/docker-compose.yml ps
    docker-compose -f /path/to/project/4.5/docker/docker-compose.yml logs
    ```
-   **Remediation Steps:**
    1.  Attempt to restart the entire stack: `docker-compose -f /path/to/project/4.5/docker/docker-compose.yml up -d`.
    2.  If it fails, check individual service logs for root cause.
    3.  If host machine is down, refer to Scenario 3: Complete Server Loss.

### 4.10. Backup Corruption

-   **Symptoms:** `test-backup-restore.sh` fails, `restore-database.sh` or `restore-wallet.sh` reports integrity check failures.
-   **Diagnosis Commands:**
    ```bash
    gpg --verify <backup.gpg>
    sqlite3 <restored.db> "PRAGMA integrity_check;"
    ```
-   **Remediation Steps:**
    1.  Attempt to restore an older backup.
    2.  Verify GPG key integrity.
    3.  Review backup script logs for errors during creation.
    4.  If all backups are corrupted, manual data recovery may be necessary (last resort).
