#!/bin/bash
# IPFS Daemon Control Script
# Manages IPFS daemon for Monero Marketplace Reputation System

set -e

IPFS_PID_FILE="$HOME/.ipfs/daemon.pid"
IPFS_LOG_FILE="$HOME/.ipfs/daemon.log"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

function print_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

function print_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

function print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

function check_ipfs_installed() {
    if ! command -v ipfs &> /dev/null; then
        print_error "IPFS is not installed"
        echo "Run: ./scripts/install-ipfs.sh"
        exit 1
    fi
}

function is_daemon_running() {
    if [ -f "$IPFS_PID_FILE" ]; then
        PID=$(cat "$IPFS_PID_FILE")
        if ps -p "$PID" > /dev/null 2>&1; then
            return 0
        else
            rm -f "$IPFS_PID_FILE"
            return 1
        fi
    fi
    return 1
}

function start_daemon() {
    check_ipfs_installed

    if is_daemon_running; then
        print_warn "IPFS daemon is already running (PID: $(cat $IPFS_PID_FILE))"
        return 0
    fi

    print_info "Starting IPFS daemon..."

    # Check if Tor mode is enabled
    if [ "${IPFS_USE_TOR:-false}" = "true" ]; then
        print_info "Tor mode enabled - routing through SOCKS proxy"

        # Verify Tor is running
        if ! curl --socks5-hostname 127.0.0.1:9050 -s https://check.torproject.org/api/ip &>/dev/null; then
            print_error "Tor is not running or not accessible on 127.0.0.1:9050"
            print_info "Start Tor with: sudo systemctl start tor"
            exit 1
        fi

        print_info "✅ Tor is running"

        # Set proxy environment variable
        export ALL_PROXY=socks5h://127.0.0.1:9050
    fi

    # Start daemon in background
    nohup ipfs daemon > "$IPFS_LOG_FILE" 2>&1 &
    echo $! > "$IPFS_PID_FILE"

    # Wait for daemon to start
    sleep 2

    # Verify daemon is running
    if ! is_daemon_running; then
        print_error "Failed to start IPFS daemon"
        print_info "Check logs: tail -f $IPFS_LOG_FILE"
        exit 1
    fi

    # Check API is accessible
    if ! curl -s http://127.0.0.1:5001/api/v0/version &>/dev/null; then
        print_error "IPFS API is not accessible"
        stop_daemon
        exit 1
    fi

    print_info "✅ IPFS daemon started successfully (PID: $(cat $IPFS_PID_FILE))"
    print_info "API: http://127.0.0.1:5001"
    print_info "Gateway: http://127.0.0.1:8080"
    print_info "Logs: tail -f $IPFS_LOG_FILE"
}

function stop_daemon() {
    if ! is_daemon_running; then
        print_warn "IPFS daemon is not running"
        return 0
    fi

    PID=$(cat "$IPFS_PID_FILE")
    print_info "Stopping IPFS daemon (PID: $PID)..."

    kill "$PID"
    rm -f "$IPFS_PID_FILE"

    # Wait for process to stop
    sleep 2

    if ps -p "$PID" > /dev/null 2>&1; then
        print_warn "Daemon did not stop gracefully, forcing kill..."
        kill -9 "$PID" 2>/dev/null || true
    fi

    print_info "✅ IPFS daemon stopped"
}

function restart_daemon() {
    print_info "Restarting IPFS daemon..."
    stop_daemon
    sleep 1
    start_daemon
}

function status_daemon() {
    check_ipfs_installed

    echo "========================================"
    echo "  IPFS Daemon Status"
    echo "========================================"
    echo ""

    # Check daemon status
    if is_daemon_running; then
        PID=$(cat "$IPFS_PID_FILE")
        print_info "Daemon: ${GREEN}RUNNING${NC} (PID: $PID)"
    else
        print_error "Daemon: ${RED}STOPPED${NC}"
        return 1
    fi

    # Check API accessibility
    if curl -s http://127.0.0.1:5001/api/v0/version &>/dev/null; then
        VERSION=$(curl -s http://127.0.0.1:5001/api/v0/version | grep -o '"Version":"[^"]*"' | cut -d'"' -f4)
        print_info "API: ${GREEN}ACCESSIBLE${NC} (version: $VERSION)"
    else
        print_error "API: ${RED}NOT ACCESSIBLE${NC}"
    fi

    # Check peer count
    PEER_COUNT=$(ipfs swarm peers 2>/dev/null | wc -l || echo "0")
    if [ "$PEER_COUNT" -gt 0 ]; then
        print_info "Peers: ${GREEN}$PEER_COUNT connected${NC}"
    else
        print_warn "Peers: ${YELLOW}No peers connected${NC}"
    fi

    # Check repo stats
    REPO_SIZE=$(ipfs repo stat -H 2>/dev/null | grep "RepoSize" | awk '{print $2}' || echo "unknown")
    print_info "Repository size: $REPO_SIZE"

    # Check Tor mode
    if [ "${IPFS_USE_TOR:-false}" = "true" ]; then
        print_info "Tor mode: ${GREEN}ENABLED${NC}"
    else
        print_warn "Tor mode: ${YELLOW}DISABLED${NC} (local testing only)"
    fi

    echo ""
}

function show_logs() {
    if [ ! -f "$IPFS_LOG_FILE" ]; then
        print_error "Log file not found: $IPFS_LOG_FILE"
        exit 1
    fi

    tail -f "$IPFS_LOG_FILE"
}

function show_usage() {
    echo "Usage: $0 {start|stop|restart|status|logs}"
    echo ""
    echo "Commands:"
    echo "  start    - Start IPFS daemon"
    echo "  stop     - Stop IPFS daemon"
    echo "  restart  - Restart IPFS daemon"
    echo "  status   - Show daemon status"
    echo "  logs     - Show daemon logs (tail -f)"
    echo ""
    echo "Environment variables:"
    echo "  IPFS_USE_TOR=true   - Route IPFS through Tor (default: false)"
    echo ""
    echo "Examples:"
    echo "  $ ./scripts/ipfs-daemon.sh start"
    echo "  $ IPFS_USE_TOR=true ./scripts/ipfs-daemon.sh start"
    echo "  $ ./scripts/ipfs-daemon.sh status"
    exit 1
}

# Main command handler
case "${1:-}" in
    start)
        start_daemon
        ;;
    stop)
        stop_daemon
        ;;
    restart)
        restart_daemon
        ;;
    status)
        status_daemon
        ;;
    logs)
        show_logs
        ;;
    *)
        show_usage
        ;;
esac
