#!/bin/bash
# Quick sync status checker

DAEMON_URL="http://127.0.0.1:28081/json_rpc"

echo "🔍 Vérification rapide de la synchronisation..."
echo ""

RESPONSE=$(curl -s "$DAEMON_URL" --data '{"jsonrpc":"2.0","id":"0","method":"get_info"}')

HEIGHT=$(echo "$RESPONSE" | jq -r '.result.height')
TARGET=$(echo "$RESPONSE" | jq -r '.result.target_height')
BLOCKS_LEFT=$((TARGET - HEIGHT))
PERCENT=$(echo "scale=1; ($HEIGHT / $TARGET) * 100" | bc)

echo "📊 Hauteur: $HEIGHT / $TARGET"
echo "📈 Progression: ${PERCENT}%"
echo "⏳ Blocs restants: $BLOCKS_LEFT"
echo ""

if [ "$HEIGHT" -eq "$TARGET" ]; then
    echo "✅ SYNCHRONISATION COMPLÈTE!"
    echo ""
    echo "🎉 Vous pouvez maintenant envoyer votre transaction!"
    echo ""
    echo "Escrow: bb8861a8-0fad-4a82-8d15-8f17b337a856"
    echo "Adresse: 9zjYJFRZB3XNidx66f1D7R9B131JE5G3ZhrzbEL95XCmgD8iBYhSqFmhfzwFk2b1EHWvsxFZ16PyY6VCHRQpd7ivUh8Frnx"
else
    # Calculate ETA
    BLOCKS_PER_SEC=40  # Average speed
    SECONDS_LEFT=$(echo "$BLOCKS_LEFT / $BLOCKS_PER_SEC" | bc)
    HOURS_LEFT=$(echo "scale=1; $SECONDS_LEFT / 3600" | bc)

    echo "⏱️  Temps restant estimé: ~${HOURS_LEFT} heures"
    echo ""
    echo "💤 Le daemon se synchronise en arrière-plan..."
fi
