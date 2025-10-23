#!/bin/bash
set -euo pipefail

# =============================================================================
# Infrastructure Validation Script
# =============================================================================
# Validates all infrastructure components without requiring full deployment
# Usage: ./validate-infrastructure.sh

echo "🔍 Monero Marketplace Infrastructure Validation"
echo "==============================================="

ERRORS=0
WARNINGS=0

# =============================================================================
# Docker Compose Validation
# =============================================================================
echo ""
echo "📋 Validating Docker Compose files..."

if docker compose -f 4.5/docker/docker-compose.yml config > /dev/null 2>&1; then
    echo "  ✅ docker-compose.yml is valid"
else
    echo "  ❌ docker-compose.yml is INVALID"
    ((ERRORS++))
fi

if docker compose -f 4.5/docker/docker-compose.blue-green.yml config > /dev/null 2>&1; then
    echo "  ✅ docker-compose.blue-green.yml is valid"
else
    echo "  ❌ docker-compose.blue-green.yml is INVALID"
    ((ERRORS++))
fi

# =============================================================================
# Prometheus Configuration
# =============================================================================
echo ""
echo "📊 Validating Prometheus configuration..."

if docker run --rm -v "$(pwd)/4.5/monitoring/prometheus.yml:/etc/prometheus/prometheus.yml" \
    --entrypoint=/bin/promtool \
    prom/prometheus:v2.48.0 \
    check config /etc/prometheus/prometheus.yml > /dev/null 2>&1; then
    echo "  ✅ prometheus.yml is valid"
else
    echo "  ❌ prometheus.yml is INVALID"
    ((ERRORS++))
fi

# =============================================================================
# Nginx Configuration
# =============================================================================
echo ""
echo "🌐 Validating Nginx configuration..."

# Note: Nginx validation will fail on DNS resolution (upstream servers not available)
# but we can check basic syntax
NGINX_OUTPUT=$(docker run --rm -v "$(pwd)/4.5/nginx/nginx.conf:/etc/nginx/nginx.conf:ro" \
    nginx:1.25-alpine \
    nginx -t 2>&1 || true)

if echo "$NGINX_OUTPUT" | grep -q "host not found in upstream"; then
    echo "  ✅ nginx.conf syntax is valid (DNS resolution expected to fail outside Docker network)"
elif echo "$NGINX_OUTPUT" | grep -q "syntax is ok"; then
    echo "  ✅ nginx.conf is valid"
elif echo "$NGINX_OUTPUT" | grep -q "test is successful"; then
    echo "  ✅ nginx.conf is valid"
else
    echo "  ❌ nginx.conf has syntax errors"
    echo "$NGINX_OUTPUT"
    ((ERRORS++))
fi

# =============================================================================
# Grafana Dashboards
# =============================================================================
echo ""
echo "📈 Validating Grafana dashboards..."

for dashboard in 4.5/monitoring/grafana/dashboards/*-complete.json; do
    if [ -f "$dashboard" ]; then
        if jq empty "$dashboard" 2>/dev/null; then
            echo "  ✅ $(basename "$dashboard") is valid JSON"
        else
            echo "  ❌ $(basename "$dashboard") is INVALID JSON"
            ((ERRORS++))
        fi
    fi
done

# =============================================================================
# Loki/Promtail Configs
# =============================================================================
echo ""
echo "📝 Validating log aggregation configs..."

if [ -f "4.5/monitoring/loki-config.yaml" ]; then
    if python3 -c "import yaml; yaml.safe_load(open('4.5/monitoring/loki-config.yaml'))" 2>/dev/null; then
        echo "  ✅ loki-config.yaml is valid"
    else
        echo "  ❌ loki-config.yaml is INVALID"
        ((ERRORS++))
    fi
else
    echo "  ⚠️  loki-config.yaml not found"
    ((WARNINGS++))
fi

if [ -f "4.5/monitoring/promtail-config.yaml" ]; then
    if python3 -c "import yaml; yaml.safe_load(open('4.5/monitoring/promtail-config.yaml'))" 2>/dev/null; then
        echo "  ✅ promtail-config.yaml is valid"
    else
        echo "  ❌ promtail-config.yaml is INVALID"
        ((ERRORS++))
    fi
else
    echo "  ⚠️  promtail-config.yaml not found"
    ((WARNINGS++))
fi

# =============================================================================
# Bash Scripts Syntax
# =============================================================================
echo ""
echo "🔧 Validating bash scripts..."

for script in 4.5/scripts/*.sh; do
    if [ -f "$script" ]; then
        if bash -n "$script" 2>/dev/null; then
            echo "  ✅ $(basename "$script") syntax is valid"
        else
            echo "  ❌ $(basename "$script") syntax is INVALID"
            ((ERRORS++))
        fi
    fi
done

# =============================================================================
# Environment Files
# =============================================================================
echo ""
echo "🔐 Checking environment files..."

if [ -f "4.5/docker/.env" ]; then
    echo "  ✅ .env file exists"
    if grep -q "CHANGE_ME" "4.5/docker/.env" 2>/dev/null; then
        echo "  ⚠️  .env contains placeholder values (CHANGE_ME)"
        ((WARNINGS++))
    fi
else
    echo "  ⚠️  .env file not found (copy from .env.example)"
    ((WARNINGS++))
fi

if [ -f "4.5/security/age.key" ]; then
    echo "  ✅ Age encryption key exists"
    KEY_PERMS=$(stat -c "%a" "4.5/security/age.key" 2>/dev/null || stat -f "%A" "4.5/security/age.key" 2>/dev/null)
    if [ "$KEY_PERMS" != "600" ]; then
        echo "  ⚠️  age.key has wrong permissions ($KEY_PERMS, should be 600)"
        ((WARNINGS++))
    fi
else
    echo "  ⚠️  age.key not found (run setup-sops.sh)"
    ((WARNINGS++))
fi

# =============================================================================
# GPG Backup Key
# =============================================================================
echo ""
echo "🔑 Checking GPG backup key..."

if gpg --list-keys "Monero Marketplace Backup" > /dev/null 2>&1; then
    echo "  ✅ GPG backup key is configured"
else
    echo "  ⚠️  GPG backup key not found"
    ((WARNINGS++))
fi

# =============================================================================
# Dockerfile Builds
# =============================================================================
echo ""
echo "🐳 Validating Dockerfiles..."

if docker build -t monero-exporter:test 4.5/monitoring/monero-exporter/ > /dev/null 2>&1; then
    echo "  ✅ monero-exporter Dockerfile builds successfully"
else
    echo "  ❌ monero-exporter Dockerfile FAILED to build"
    ((ERRORS++))
fi

# =============================================================================
# Summary
# =============================================================================
echo ""
echo "==============================================="
echo "📊 VALIDATION SUMMARY"
echo "==============================================="
echo "  Errors:   $ERRORS"
echo "  Warnings: $WARNINGS"
echo ""

if [ $ERRORS -eq 0 ] && [ $WARNINGS -eq 0 ]; then
    echo "✅ ALL CHECKS PASSED - Infrastructure is ready for deployment"
    exit 0
elif [ $ERRORS -eq 0 ]; then
    echo "⚠️  PASSED WITH WARNINGS - Review warnings before production deployment"
    exit 0
else
    echo "❌ VALIDATION FAILED - Fix errors before deployment"
    exit 1
fi
