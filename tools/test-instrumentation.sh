#!/bin/bash
# tools/test-instrumentation.sh
# Quick test script to verify instrumentation is working

set -e

echo "=========================================="
echo "Multisig Instrumentation Test"
echo "=========================================="
echo ""

# Check if ENABLE_INSTRUMENTATION is set
if [ -z "$ENABLE_INSTRUMENTATION" ]; then
    echo "⚠️  WARNING: ENABLE_INSTRUMENTATION not set"
    echo "Setting it now for this test..."
    export ENABLE_INSTRUMENTATION=1
fi

echo "✓ ENABLE_INSTRUMENTATION=$ENABLE_INSTRUMENTATION"
echo ""

# Check if Python 3 is available
if ! command -v python3 &> /dev/null; then
    echo "❌ Python 3 not found. Please install Python 3."
    exit 1
fi

echo "✓ Python 3 found: $(python3 --version)"
echo ""

# Check if analysis tool exists
if [ ! -f "tools/analyze_escrow_json.py" ]; then
    echo "❌ Analysis tool not found at tools/analyze_escrow_json.py"
    exit 1
fi

echo "✓ Analysis tool found"
echo ""

# Make analysis tool executable
chmod +x tools/analyze_escrow_json.py

# Check if server compiles with instrumentation module
echo "Checking server compilation..."
if cargo check --package server 2>&1 | grep -q "^error"; then
    echo "❌ Server compilation failed"
    cargo check --package server
    exit 1
fi

echo "✓ Server compiles successfully"
echo ""

# Test analysis tool with sample data
echo "Creating sample instrumentation data..."

cat > test_escrow_sample.json <<'EOF'
[
  {
    "trace_id": "test_escrow-1699999999999",
    "timestamp": 1699999999000,
    "event_type": "SNAPSHOT_PRE_ROUND1",
    "role": "buyer",
    "details": {
      "timestamp": 1699999999000,
      "wallet_id": "test-wallet-123",
      "role": "buyer",
      "is_multisig": false,
      "balance": [0, 0],
      "address": "test_address_buyer",
      "address_hash": "abc123",
      "file_exists": true,
      "collection_time_ms": 50
    }
  },
  {
    "trace_id": "test_escrow-1699999999999",
    "timestamp": 1699999999050,
    "event_type": "RPC_CALL_START",
    "role": "buyer",
    "rpc_port": 18082,
    "details": {
      "method": "prepare_multisig",
      "timestamp": 1699999999050
    }
  },
  {
    "trace_id": "test_escrow-1699999999999",
    "timestamp": 1699999999150,
    "event_type": "RPC_CALL_END",
    "role": "buyer",
    "rpc_port": 18082,
    "details": {
      "method": "prepare_multisig",
      "duration_ms": 100,
      "success": true
    }
  },
  {
    "trace_id": "test_escrow-1699999999999",
    "timestamp": 1699999999200,
    "event_type": "SNAPSHOT_POST_MAKE_MULTISIG",
    "role": "buyer",
    "details": {
      "timestamp": 1699999999200,
      "wallet_id": "test-wallet-123",
      "role": "buyer",
      "is_multisig": true,
      "balance": [0, 0],
      "address": "test_multisig_address",
      "address_hash": "def456",
      "file_exists": true,
      "collection_time_ms": 45
    }
  }
]
EOF

echo "✓ Sample data created"
echo ""

# Test analysis tool
echo "Testing analysis tool..."
if ! python3 tools/analyze_escrow_json.py test_escrow_sample.json > /dev/null 2>&1; then
    echo "❌ Analysis tool failed on sample data"
    python3 tools/analyze_escrow_json.py test_escrow_sample.json
    exit 1
fi

echo "✓ Analysis tool works correctly"
echo ""

# Show sample output
echo "Sample analysis output:"
echo "=========================================="
python3 tools/analyze_escrow_json.py test_escrow_sample.json
echo "=========================================="
echo ""

# Cleanup
rm -f test_escrow_sample.json

echo ""
echo "=========================================="
echo "✅ All instrumentation tests passed!"
echo "=========================================="
echo ""
echo "Next steps:"
echo "1. Integrate instrumentation into wallet_manager.rs"
echo "   See: DOX/guides/INSTRUMENTATION-INTEGRATION-EXAMPLE.md"
echo ""
echo "2. Run server with instrumentation enabled:"
echo "   export ENABLE_INSTRUMENTATION=1"
echo "   cargo run --bin server"
echo ""
echo "3. Trigger escrow operations and analyze results:"
echo "   python3 tools/analyze_escrow_json.py escrow_*.json"
echo ""
