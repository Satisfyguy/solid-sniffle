#!/bin/bash
# Script interactif pour configurer la clé API Anthropic
# Usage: bash scripts/ai/setup-api-key.sh

set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
ENV_FILE="$PROJECT_ROOT/.env"
ENV_EXAMPLE="$PROJECT_ROOT/.env.example"

echo "============================================"
echo "  Configuration Clé API Anthropic"
echo "  Monero Marketplace - AI Security Audits"
echo "============================================"
echo ""

# Vérifier si .env existe déjà
if [ -f "$ENV_FILE" ]; then
    echo "⚠️  Le fichier .env existe déjà."
    echo ""
    read -p "Voulez-vous le remplacer ? (y/N): " -n 1 -r
    echo ""
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "❌ Configuration annulée."
        exit 0
    fi
fi

# Demander la clé API
echo "📋 Instructions :"
echo "1. Allez sur https://console.anthropic.com/settings/keys"
echo "2. Cliquez sur 'Create Key'"
echo "3. Copiez la clé (format: sk-ant-api03-...)"
echo ""
echo "🔑 Entrez votre clé API Anthropic :"
read -r API_KEY

# Valider le format de base
if [[ ! $API_KEY =~ ^sk-ant-api ]]; then
    echo "❌ Format de clé invalide. La clé doit commencer par 'sk-ant-api'"
    exit 1
fi

# Créer le fichier .env
cat > "$ENV_FILE" << EOF
# Anthropic API Key for Claude Security Analyzer
# Generated on $(date)
# DO NOT COMMIT THIS FILE - it's in .gitignore
ANTHROPIC_API_KEY=$API_KEY
EOF

# Sécuriser les permissions
chmod 600 "$ENV_FILE"

echo ""
echo "✅ Fichier .env créé avec succès !"
echo "✅ Permissions définies à 600 (lecture/écriture propriétaire uniquement)"
echo ""

# Vérifier .gitignore
if grep -q "^\.env$" "$PROJECT_ROOT/.gitignore" 2>/dev/null; then
    echo "✅ .env est bien dans .gitignore (sécurisé)"
else
    echo "⚠️  ATTENTION : .env n'est pas dans .gitignore !"
    echo "   Ajoutez-le manuellement pour éviter de committer votre clé."
fi

echo ""
echo "🧪 Test de la configuration..."
echo ""

# Charger .env et tester
source "$ENV_FILE"

# Vérifier que Python et pip sont installés
if ! command -v python3 &> /dev/null; then
    echo "❌ Python 3 n'est pas installé."
    echo "   Installation: sudo apt install python3 python3-pip"
    exit 1
fi

# Vérifier que les dépendances sont installées
if ! python3 -c "import anthropic" 2>/dev/null; then
    echo "📦 Installation des dépendances Python..."
    pip install -r "$PROJECT_ROOT/requirements.txt"
fi

# Test rapide de l'API
echo "🚀 Test de connexion à l'API Anthropic..."
python3 << 'PYEOF'
import os
import sys
try:
    import anthropic
    client = anthropic.Anthropic(api_key=os.environ.get("ANTHROPIC_API_KEY"))
    # Test simple avec Haiku (moins cher)
    response = client.messages.create(
        model="claude-3-5-haiku-20250219",
        max_tokens=10,
        messages=[{"role": "user", "content": "Say 'API OK'"}]
    )
    print(f"✅ API fonctionne ! Réponse: {response.content[0].text}")
    sys.exit(0)
except anthropic.AuthenticationError:
    print("❌ Clé API invalide. Vérifiez votre clé sur https://console.anthropic.com/settings/keys")
    sys.exit(1)
except Exception as e:
    print(f"❌ Erreur : {e}")
    sys.exit(1)
PYEOF

if [ $? -eq 0 ]; then
    echo ""
    echo "🎉 Configuration réussie !"
    echo ""
    echo "📚 Prochaines étapes :"
    echo ""
    echo "1. Charger les variables d'environnement :"
    echo "   source .env"
    echo ""
    echo "2. Tester l'analyseur de sécurité :"
    echo "   python scripts/ai/claude_quick_scan.py --file server/src/main.rs"
    echo ""
    echo "3. Analyser un fichier en profondeur :"
    echo "   python scripts/ai/claude_security_analyzer.py --file server/src/handlers/escrow.rs"
    echo ""
    echo "4. Lire la documentation complète :"
    echo "   cat scripts/ai/README_API_KEY.md"
    echo ""
else
    echo ""
    echo "⚠️  La configuration a échoué. Vérifiez votre clé API."
    echo "   Documentation : https://docs.anthropic.com/"
fi
