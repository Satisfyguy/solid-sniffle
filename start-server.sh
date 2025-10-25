#!/bin/bash
# Script de lancement du serveur Monero Marketplace

# Configuration OpenSSL
export OPENSSL_DIR=/usr
export OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu
export OPENSSL_INCLUDE_DIR=/usr/include/openssl

# Variables d'environnement pour le serveur
export DATABASE_URL="sqlite:///home/malix/Desktop/monero.marketplace/data/marketplace-dev.db?mode=rwc"
export DB_ENCRYPTION_KEY="development_encryption_key_32_bytes_minimum_required_for_sqlcipher"
export SESSION_SECRET_KEY="8dca8a38790f2ce50422553309fa4f756dfd50d7c67a0aba2009d688b64ea811"
export RUST_LOG="debug"
export MONERO_NETWORK="testnet"

# Créer le dossier data s'il n'existe pas
mkdir -p /home/malix/Desktop/monero.marketplace/data

echo "🚀 Lancement du serveur Monero Marketplace..."
echo "📊 Base de données: $DATABASE_URL"
echo "🔐 Chiffrement: SQLCipher activé"
echo "🌐 Réseau: $MONERO_NETWORK"
echo "📝 Logs: $RUST_LOG"
echo ""

# Lancer le serveur
./target/release/server

