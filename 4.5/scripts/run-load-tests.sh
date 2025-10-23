#!/bin/bash
set -euo pipefail

# ============================================================================
# Load Testing Validation Script
# ============================================================================
# Purpose: Execute k6 load tests and validate SLA performance targets
# Requirements: k6 installed (https://k6.io/docs/get-started/installation/)
# ============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOAD_TESTS_DIR="$SCRIPT_DIR/../load-tests"
RESULTS_DIR="$LOAD_TESTS_DIR/results"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
RESULTS_FILE="$RESULTS_DIR/performance-${TIMESTAMP}.json"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# ============================================================================
# Pre-Flight Checks
# ============================================================================
log_info "Load Testing Validation Starting..."

if ! command -v k6 &> /dev/null; then
    log_error "k6 not found. Install: https://k6.io/docs/get-started/installation/"
    log_info "Quick install (Ubuntu): sudo apt-key adv --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys C5AD17C747E3415A3642D57D77C6C491D6AC1D00"
    log_info "                        echo 'deb https://dl.k6.io/deb stable main' | sudo tee /etc/apt/sources.list.d/k6.list"
    log_info "                        sudo apt-get update && sudo apt-get install k6"
    exit 1
fi

# Check if application is running
if ! curl -s -o /dev/null -w "%{http_code}" http://localhost:8080/api/health | grep -q "200"; then
    log_warn "Application not responding on http://localhost:8080"
    log_warn "Please start the application: cd 4.5/docker && docker compose up -d"
    exit 1
fi

mkdir -p "$RESULTS_DIR"

# ============================================================================
# Execute Load Tests
# ============================================================================
log_info "Starting Performance Validation Test..."
log_info "Duration: ~22 minutes"
log_info "Scenarios: Baseline (5min) → Load (10min) → Stress (5min) → Spike (2min)"
log_info ""

cd "$LOAD_TESTS_DIR"

# Run k6 test
if k6 run \
    --out json="$RESULTS_FILE" \
    --summary-export="$RESULTS_DIR/summary-${TIMESTAMP}.json" \
    scenarios/performance-validation.js; then

    log_info ""
    log_info "============================================"
    log_info "Load Test: PASSED ✓"
    log_info "============================================"
    log_info "Results saved to: $RESULTS_FILE"
    log_info ""

    # Parse results (basic summary)
    if command -v jq &> /dev/null; then
        log_info "Summary Statistics:"
        jq -r '.metrics | to_entries[] | select(.key | contains("http_req_duration")) | "\(.key): p95=\(.value.values.p95)ms, p99=\(.value.values.p99)ms"' "$RESULTS_DIR/summary-${TIMESTAMP}.json" 2>/dev/null || true
    fi

    exit 0
else
    log_error ""
    log_error "============================================"
    log_error "Load Test: FAILED ✗"
    log_error "============================================"
    log_error "Some thresholds were not met."
    log_error "Review output above for details."
    log_error ""
    exit 1
fi
