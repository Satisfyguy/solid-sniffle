#!/bin/bash

# Monero Daemon Sync Monitor
# Monitors testnet daemon synchronization progress

DAEMON_URL="http://127.0.0.1:28081/json_rpc"
REFRESH_INTERVAL=30  # seconds
LOG_FILE="sync_monitor.log"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo "========================================" | tee -a "$LOG_FILE"
echo "Monero Testnet Daemon Sync Monitor" | tee -a "$LOG_FILE"
echo "Started: $(date)" | tee -a "$LOG_FILE"
echo "========================================" | tee -a "$LOG_FILE"
echo ""

# Track initial state
PREV_HEIGHT=0
START_TIME=$(date +%s)

while true; do
    # Get daemon info
    RESPONSE=$(curl -s "$DAEMON_URL" --data '{"jsonrpc":"2.0","id":"0","method":"get_info"}' 2>/dev/null)

    if [ $? -ne 0 ]; then
        echo -e "${RED}[ERROR]${NC} Cannot connect to daemon at $DAEMON_URL" | tee -a "$LOG_FILE"
        sleep $REFRESH_INTERVAL
        continue
    fi

    # Parse JSON response
    HEIGHT=$(echo "$RESPONSE" | jq -r '.result.height // 0')
    TARGET=$(echo "$RESPONSE" | jq -r '.result.target_height // 0')
    INCOMING=$(echo "$RESPONSE" | jq -r '.result.incoming_connections_count // 0')
    OUTGOING=$(echo "$RESPONSE" | jq -r '.result.outgoing_connections_count // 0')

    if [ "$HEIGHT" -eq 0 ] || [ "$TARGET" -eq 0 ]; then
        echo -e "${RED}[ERROR]${NC} Invalid response from daemon" | tee -a "$LOG_FILE"
        sleep $REFRESH_INTERVAL
        continue
    fi

    # Calculate stats
    BLOCKS_LEFT=$((TARGET - HEIGHT))
    PERCENT=$(echo "scale=2; ($HEIGHT / $TARGET) * 100" | bc)

    # Calculate sync speed
    if [ "$PREV_HEIGHT" -gt 0 ]; then
        BLOCKS_SYNCED=$((HEIGHT - PREV_HEIGHT))
        BLOCKS_PER_SEC=$(echo "scale=2; $BLOCKS_SYNCED / $REFRESH_INTERVAL" | bc)

        # ETA calculation
        if [ "$(echo "$BLOCKS_PER_SEC > 0" | bc)" -eq 1 ]; then
            SECONDS_LEFT=$(echo "scale=0; $BLOCKS_LEFT / $BLOCKS_PER_SEC" | bc)
            HOURS_LEFT=$(echo "scale=1; $SECONDS_LEFT / 3600" | bc)
            MINUTES_LEFT=$(echo "scale=0; ($SECONDS_LEFT % 3600) / 60" | bc)

            ETA="~${HOURS_LEFT}h ${MINUTES_LEFT}m"
        else
            ETA="calculating..."
        fi
    else
        BLOCKS_PER_SEC="0.00"
        ETA="calculating..."
    fi

    PREV_HEIGHT=$HEIGHT

    # Determine status color
    if [ "$(echo "$PERCENT >= 90" | bc)" -eq 1 ]; then
        STATUS_COLOR=$GREEN
        STATUS="ALMOST DONE"
    elif [ "$(echo "$PERCENT >= 50" | bc)" -eq 1 ]; then
        STATUS_COLOR=$BLUE
        STATUS="SYNCING"
    else
        STATUS_COLOR=$YELLOW
        STATUS="SYNCING"
    fi

    # Clear screen and display
    clear
    echo "=========================================="
    echo "   Monero Testnet Daemon Sync Monitor"
    echo "=========================================="
    echo ""
    echo -e "${STATUS_COLOR}Status: $STATUS${NC}"
    echo ""
    echo "📊 Progress:"
    echo "   Height:        $HEIGHT / $TARGET"
    echo "   Percent:       ${PERCENT}%"
    echo "   Blocks left:   $BLOCKS_LEFT"
    echo ""
    echo "⚡ Speed:"
    echo "   Blocks/sec:    $BLOCKS_PER_SEC"
    echo "   ETA:           $ETA"
    echo ""
    echo "🌐 Network:"
    echo "   Incoming:      $INCOMING peers"
    echo "   Outgoing:      $OUTGOING peers"
    echo ""

    # Progress bar
    PROGRESS_WIDTH=50
    FILLED=$(echo "scale=0; ($PERCENT / 100) * $PROGRESS_WIDTH" | bc)
    EMPTY=$((PROGRESS_WIDTH - FILLED))

    echo -n "["
    for ((i=0; i<FILLED; i++)); do echo -n "="; done
    for ((i=0; i<EMPTY; i++)); do echo -n " "; done
    echo "] ${PERCENT}%"
    echo ""

    # Check if synced
    if [ "$HEIGHT" -eq "$TARGET" ]; then
        TOTAL_TIME=$(($(date +%s) - START_TIME))
        HOURS=$((TOTAL_TIME / 3600))
        MINUTES=$(((TOTAL_TIME % 3600) / 60))

        echo -e "${GREEN}=========================================="
        echo "   ✅ SYNCHRONIZATION COMPLETE!"
        echo "==========================================${NC}"
        echo ""
        echo "Total time: ${HOURS}h ${MINUTES}m"
        echo ""
        echo "🎉 You can now send transactions to your escrow!"
        echo ""
        echo "Escrow ID: bb8861a8-0fad-4a82-8d15-8f17b337a856"
        echo "Address: 9zjYJFRZB3XNidx66f1D7R9B131JE5G3ZhrzbEL95XCmgD8iBYhSqFmhfzwFk2b1EHWvsxFZ16PyY6VCHRQpd7ivUh8Frnx"
        echo ""

        # Log completion
        echo "[$(date)] Sync complete! Height: $HEIGHT" >> "$LOG_FILE"

        # Play notification sound (if available)
        if command -v paplay &> /dev/null; then
            paplay /usr/share/sounds/freedesktop/stereo/complete.oga 2>/dev/null &
        fi

        echo "Press Ctrl+C to exit"
        sleep infinity
    fi

    echo "Last update: $(date)"
    echo "Refresh: ${REFRESH_INTERVAL}s | Press Ctrl+C to exit"

    # Log to file every 5 minutes
    CURRENT_TIME=$(date +%s)
    if [ $((CURRENT_TIME % 300)) -lt $REFRESH_INTERVAL ]; then
        echo "[$(date)] Height: $HEIGHT/$TARGET (${PERCENT}%) | Speed: ${BLOCKS_PER_SEC} b/s | ETA: $ETA" >> "$LOG_FILE"
    fi

    sleep $REFRESH_INTERVAL
done
