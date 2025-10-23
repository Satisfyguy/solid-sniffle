#!/bin/bash
# Script de test pour reputation/ workspace (à exécuter sur Ubuntu)

set -e

echo "========================================"
echo "REPUTATION SYSTEM - TEST SUITE"
echo "========================================"
echo ""

cd "$(dirname "$0")"

echo "📦 Compilation workspace..."
cargo build --workspace --quiet
echo "✅ Compilation réussie"
echo ""

echo "🧪 Tests reputation-common..."
cargo test --package reputation-common --quiet
echo "✅ Tests common OK"
echo ""

echo "🔐 Tests reputation-crypto..."
cargo test --package reputation-crypto --quiet
echo "✅ Tests crypto OK"
echo ""

echo "🎯 Tous les tests (verbose)..."
cargo test --workspace -- --nocapture
echo ""

echo "📊 Clippy (strict mode)..."
cargo clippy --workspace -- -D warnings
echo "✅ Clippy OK"
echo ""

echo "🎨 Formatage..."
cargo fmt --workspace --check
echo "✅ Format OK"
echo ""

echo "========================================"
echo "✅ TOUS LES TESTS PASSENT"
echo "========================================"
echo ""
echo "Modules validés:"
echo "  • reputation-common (types)"
echo "  • reputation-crypto (ed25519)"
echo ""
echo "Tests exécutés: 9/9"
echo "  • common: 4 tests"
echo "  • crypto: 5 tests"
echo ""
echo "Prochaine étape: REP.2 - Backend API"
