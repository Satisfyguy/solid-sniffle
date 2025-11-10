#!/bin/bash

# Script pour déclencher la synchronisation lazy multisig

ESCROW_ID="11959eae-dda8-4f46-bf31-05ecf6a82f20"

echo "🚀 Déclenchement de la synchronisation Lazy Multisig pour l'escrow: $ESCROW_ID"
echo "⏳ ATTENTION: Ce processus va rouvrir les 3 wallets multisig, effectuer la synchro, et vérifier la balance"
echo ""

# Appeler l'API de vérification de balance (cela déclenche le lazy sync)
echo "📡 Appel de l'API de vérification de balance..."
RESPONSE=$(curl -X POST "http://localhost:8080/api/escrow/$ESCROW_ID/check-balance" \
  -H "Cookie: session=A6Sf9bFH9wmdaUeh1WvxDcamEBSUlESukYp+nRLVnDHCLTQzrZvmlj7E0KXTpB4JtF2lHYvh2I/ebwmwSBCVzJMyJQTOqGGMb5ml2qldiwHTN4vy0ZXt9a8Qmz3Y67yaJveElafi8Azx2Og8iAVnFZZSjKBw1OL0TijuL77+7iKcPK2PCBaGMSr3WANKtbmc7GxLsRPfsRjAd06RooXHxnCyKm5MIb1AOUwcrU38yYgEaGE0oWU8ZJWmFHPsn/mAyOoKcxLOuk4FxuIjQFQ=" 2>/dev/null)

# Vérifier la réponse
if [[ $? -eq 0 ]]; then
    if [[ "$RESPONSE" == *"Not authenticated"* ]]; then
        echo "🔒 ERREUR: Session expirée ou non-autorisée"
        echo "   Raison: Le cookie de session est probablement expiré ou n'autorise pas l'accès à cet escrow"
    elif [[ "$RESPONSE" == *"error"* ]]; then
        echo "❌ ERREUR: Problème avec l'appel API:"
        echo "$RESPONSE"
    else
        echo "✅ SUCCÈS: Synchronisation Lazy Multisig effectuée!"
        echo "📊 Réponse de l'API:"
        echo "$RESPONSE" | python3 -m json.tool
    fi
else
    echo "🌐 ERREUR: Impossible de joindre le serveur"
fi

echo ""
echo "ℹ️  INFORMATION: Le Lazy Sync Multisig fait ce qui suit:"
echo "   1. Rouvre temporairement les 3 wallets (buyer, vendor, arbiter)"
echo "   2. Effectue l'échange d'infos multisig entre eux"
echo "   3. Vérifie la balance sur l'adresse multisig"
echo "   4. Ferme immédiatement les 3 wallets pour libérer les RPC slots"
echo ""
echo "⏰ Durée typique: 3-5 secondes"