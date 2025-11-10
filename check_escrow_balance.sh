#!/bin/bash

ESCROW_ID="11959eae-dda8-4f46-bf31-05ecf6a82f20"

echo "🔍 Checking balance for escrow: $ESCROW_ID"

# Get the session cookie from your browser or environment variable
SESSION_COOKIE=$1

if [ -z "$SESSION_COOKIE" ]; then
    echo "⚠️ Warning: No session cookie provided, trying to use default session name..."
    echo "Please provide your session cookie as argument:"
    echo "Usage: ./check_escrow_balance.sh 'your_session_cookie_here'"
    echo ""
    echo "To get your session cookie:"
    echo "1. Open browser developer tools (F12)"
    echo "2. Go to Application/Storage tab"
    echo "3. Look for Cookies under http://localhost:8080"
    echo "4. Copy the value of 'session' cookie"
    echo ""
    echo "Example usage:"
    echo "  ./check_escrow_balance.sh 'A6Sf9bFH9wmdaUeh1WvxDcamEBSUlESukYp+nRLVnDHCLTQzrZvmlj7E0KXTpB4JtF2lHYvh2I/ebwmwSBCVzJMyJQTOqGGMb5ml2qldiwHTN4vy0ZXt9a8Qmz3Y67yaJveElafi8Azx2Og8iAVnFZZSjKBw1OL0TijuL77+7iKcPK2PCBaGMSr3WANKtbmc7GxLsRPfsRjAd06RooXHxnCyKm5MIb1AOUwcrU38yYgEaGE0oWU8ZJWmFHPsn/mAyOoKcxLOuk4FxuIjQFQ='"
    exit 1
fi

echo "🔍 Verifying escrow balance..."
curl -X POST "http://localhost:8080/api/escrow/$ESCROW_ID/check-balance" \
  -H "Cookie: session=$SESSION_COOKIE" \
  -H "Content-Type: application/json" \
  -H "User-Agent: Monero Marketplace Balance Checker" \
  | python3 -m json.tool

echo ""
echo "💡 Note: The lazy sync process will:" 
echo "  1. Reopen all 3 multisig wallets (buyer, vendor, arbiter)"
echo "  2. Perform multisig info exchange"
echo "  3. Check the balance on the multisig address"
echo "  4. Close all wallets to free RPC slots"
echo ""
echo "⏰ This may take 3-5 seconds as all wallets need to be reopened and synced"