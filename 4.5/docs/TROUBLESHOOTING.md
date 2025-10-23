# Troubleshooting Guide

This guide provides solutions to common issues encountered during the deployment and operation of the Monero Marketplace.

## 1. "Connection refused" to Monero RPC

-   **Cause:** The Monero wallet RPC container is not running, is not accessible, or the port is blocked.
-   **Fix:**
    1.  Check if the container is running:
        ```bash
        docker-compose -f 4.5/docker/docker-compose.yml ps | grep monero-wallet-rpc
        ```
    2.  Check container logs for errors:
        ```bash
        docker-compose -f 4.5/docker/docker-compose.yml logs monero-wallet-rpc-buyer
        ```
    3.  Restart the container:
        ```bash
        docker-compose -f 4.5/docker/docker-compose.yml restart monero-wallet-rpc-buyer
        ```
    4.  Verify firewall rules are not blocking internal Docker network communication.
-   **Prevention:** Ensure `depends_on` is correctly configured in `docker-compose.yml` and health checks are passing before the `server` starts.

## 2. Slow database queries

-   **Cause:** Missing or inefficient database indexes, large tables, or complex queries.
-   **Fix:**
    1.  Analyze slow queries using `EXPLAIN QUERY PLAN` in `sqlite3`.
    2.  Add appropriate indexes as suggested in `DATABASE-OPTIMIZATIONS.md`.
    3.  Refactor complex queries in the application code.
-   **Prevention:** Regularly review database performance metrics and query logs. Implement connection pooling and consider caching for read-heavy data.

## 3. TLS certificate expired

-   **Cause:** Certbot auto-renewal failed, or the cron job is not running.
-   **Fix:**
    1.  Manually renew the certificate:
        ```bash
        sudo certbot renew --force-renewal
        sudo systemctl reload nginx
        ```
    2.  Check Certbot logs for errors: `/var/log/letsencrypt/`.
-   **Prevention:** Verify the Certbot cron job is active and has necessary permissions. Set up monitoring for certificate expiration.

## 4. Docker container won't start

-   **Cause:** Port conflict, incorrect environment variables, missing volumes, or application error during startup.
-   **Fix:**
    1.  Check container logs:
        ```bash
        docker-compose -f 4.5/docker/docker-compose.yml logs <service_name>
        ```
    2.  Check for port conflicts:
        ```bash
        sudo lsof -i :<port_number>
        ```
    3.  Verify `docker-compose.yml` configuration for the service.
-   **Prevention:** Use `docker-compose config` to validate YAML syntax. Implement robust health checks.

## 5. Out of memory (OOM) killed

-   **Cause:** Application consuming too much memory, often due to a memory leak or insufficient allocated resources.
-   **Fix:**
    1.  Check `docker stats` for memory usage of the container.
    2.  Review application logs for `OOM` messages.
    3.  Increase memory limits for the container in `docker-compose.yml`.
    4.  Analyze application code for memory leaks.
-   **Prevention:** Set appropriate memory limits and reservations in `docker-compose.yml`. Implement memory usage monitoring and alerts.

## 6. High latency (>1s responses)

-   **Cause:** High CPU/memory usage, slow database queries, network bottlenecks, or inefficient application code.
-   **Fix:**
    1.  Check system resources (CPU, memory, disk I/O, network) using `docker stats` or host monitoring tools.
    2.  Review Nginx and application access logs for slow requests.
    3.  Optimize database queries and add indexes.
    4.  Implement caching for frequently accessed data.
-   **Prevention:** Use load testing to identify bottlenecks. Implement comprehensive monitoring for response times and resource utilization.

## 7. WebSocket connections dropping

-   **Cause:** Nginx misconfiguration for WebSocket proxying, application errors, or network instability.
-   **Fix:**
    1.  Verify Nginx configuration for WebSocket (`Upgrade` and `Connection` headers).
    2.  Check application logs for WebSocket-related errors.
    3.  Ensure sufficient `ulimit` settings on the host for open files.
-   **Prevention:** Test WebSocket connections thoroughly. Monitor WebSocket connection metrics.

## 8. Prometheus metrics missing

-   **Cause:** Scrape target down, incorrect `prometheus.yml` configuration, or application not exposing metrics endpoint.
-   **Fix:**
    1.  Check Prometheus UI (`http://localhost:9090/targets`) for scrape errors.
    2.  Verify `prometheus.yml` `scrape_configs`.
    3.  Ensure the application's `/metrics` endpoint is accessible.
-   **Prevention:** Validate Prometheus configuration with `promtool check config`. Implement alerts for missing scrape targets.

## 9. Grafana dashboard empty

-   **Cause:** Datasource misconfiguration, Prometheus not running, or incorrect PromQL queries in the dashboard.
-   **Fix:**
    1.  Check Grafana datasource settings (Configuration -> Data sources).
    2.  Verify Prometheus is running and accessible.
    3.  Test PromQL queries directly in Prometheus UI.
-   **Prevention:** Ensure `grafana/datasources/prometheus.yml` is correctly configured and mounted. Validate dashboard JSON files.

## 10. Backup failed

-   **Cause:** Disk full, permissions error, GPG key missing, or script error.
-   **Fix:**
    1.  Check logs of `backup-database.sh` or `backup-wallets.sh` for specific errors.
    2.  Verify disk space on backup volumes.
    3.  Ensure GPG key is present and accessible.
-   **Prevention:** Implement monitoring for backup job failures. Regularly test backup scripts.

## 11. Restore failed

-   **Cause:** Corrupted backup file, incorrect GPG passphrase, or script error.
-   **Fix:**
    1.  Check logs of `restore-database.sh` or `restore-wallet.sh`.
    2.  Verify integrity of the backup file.
    3.  Ensure correct GPG passphrase is used.
-   **Prevention:** Regularly test restore procedures with `test-backup-restore.sh`.

## 12. Port already in use

-   **Cause:** Another process on the host is using the required port.
-   **Fix:**
    1.  Identify the process using the port:
        ```bash
        sudo lsof -i :<port_number>
        ```
    2.  Stop the conflicting process or change the port mapping in `docker-compose.yml`.
-   **Prevention:** Use `netstat -tulnp` or `lsof -i` to check port availability before starting services.

## 13. Permission denied errors

-   **Cause:** Incorrect file or directory permissions for volumes mounted into containers.
-   **Fix:**
    1.  Check permissions of host directories mounted as volumes (e.g., `data/`, `wallets/`).
    2.  Ensure the user inside the container (e.g., `marketplace` user with UID 1000) has appropriate permissions.
        ```bash
        sudo chown -R 1000:1000 /path/to/project/data /path/to/project/wallets
        ```
-   **Prevention:** Set correct permissions during initial setup. Run containers with non-root users.

## 14. Database migration failed

-   **Cause:** SQL syntax error in migration, database schema conflict, or application attempting to run migrations on an already migrated database.
-   **Fix:**
    1.  Check application logs during startup for migration errors.
    2.  Manually inspect the migration files for syntax errors.
    3.  If in development, consider reverting migrations or dropping the database.
-   **Prevention:** Test migrations thoroughly in development and staging environments. Use idempotent migration scripts.

## 15. Wallet sync stuck

-   **Cause:** Monero daemon not running, network connectivity issues, or corrupted wallet files.
-   **Fix:**
    1.  Check Monero wallet RPC logs for errors.
    2.  Verify network connectivity from the wallet container.
    3.  Restart the wallet RPC container.
    4.  If persistent, consider restoring wallet from a recent backup.
-   **Prevention:** Monitor wallet sync status. Ensure stable network connection for the Monero daemon.
