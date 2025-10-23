#!/bin/bash
# Script de compilation pour Monero Marketplace
# Configure les variables d'environnement OpenSSL et compile le projet

export OPENSSL_DIR=/usr
export OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu
export OPENSSL_INCLUDE_DIR=/usr/include/openssl

echo "🔧 Configuration OpenSSL:"
echo "  OPENSSL_DIR=$OPENSSL_DIR"
echo "  OPENSSL_LIB_DIR=$OPENSSL_LIB_DIR"
echo "  OPENSSL_INCLUDE_DIR=$OPENSSL_INCLUDE_DIR"
echo ""

echo "🚀 Compilation du projet Monero Marketplace..."
cargo build "$@"
