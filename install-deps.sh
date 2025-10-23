#!/bin/bash
# Installation des dépendances système pour le serveur Monero Marketplace
# À exécuter avec: bash install-deps.sh

set -e

echo "📦 Installation des dépendances système..."

# Mise à jour des paquets
echo "🔄 Mise à jour de la liste des paquets..."
sudo apt update

# Installation des dépendances
echo "⬇️  Installation de pkg-config et libssl-dev..."
sudo apt install -y pkg-config libssl-dev build-essential

echo "✅ Dépendances installées avec succès!"
echo ""
echo "📝 Dépendances installées:"
echo "  - pkg-config (pour détecter les bibliothèques)"
echo "  - libssl-dev (développement OpenSSL)"
echo "  - build-essential (outils de compilation)"
echo ""
echo "🚀 Vous pouvez maintenant compiler le serveur avec:"
echo "   cd server && cargo build"
