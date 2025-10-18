#!/bin/bash

# Script d'installation de la skill market-prod pour Gemini CLI
# Usage: ./install-gemini-skill.sh

set -e

echo "🚀 Installation de la skill market-prod pour Gemini CLI..."

# Vérifier si Gemini CLI est installé
if ! command -v gemini-cli &> /dev/null; then
    echo "❌ Gemini CLI n'est pas installé."
    echo "📥 Installation de Gemini CLI..."
    
    # Installation de Gemini CLI (ajustez selon votre méthode d'installation)
    if command -v npm &> /dev/null; then
        npm install -g @google/gemini-cli
    elif command -v pip &> /dev/null; then
        pip install gemini-cli
    else
        echo "❌ Ni npm ni pip ne sont disponibles. Installez Gemini CLI manuellement."
        exit 1
    fi
fi

echo "✅ Gemini CLI détecté: $(gemini-cli --version)"

# Créer le répertoire des skills si nécessaire
SKILLS_DIR="$HOME/.gemini-cli/skills"
mkdir -p "$SKILLS_DIR"

# Copier la skill
echo "📋 Copie de la skill market-prod..."
cp .claude/skills/market-prod.yaml "$SKILLS_DIR/"

# Rendre le fichier exécutable (si nécessaire)
chmod +x "$SKILLS_DIR/market-prod.yaml"

echo "✅ Skill market-prod installée avec succès!"
echo ""
echo "🎯 Utilisation:"
echo "   gemini-cli market-prod 'Comment implémenter la Phase 1 du plan?'"
echo "   gemini-cli market-prod 'Quels sont les quality gates pour la Phase 2?'"
echo "   gemini-cli market-prod 'Comment configurer les feature flags?'"
echo ""
echo "📚 La skill contient le plan de transformation complet avec:"
echo "   - 5 phases de développement"
echo "   - Quality gates détaillés"
echo "   - Stratégie Production-First"
echo "   - Métriques de performance"
echo "   - Processus de déploiement"
echo ""
echo "🔧 Prochaines étapes:"
echo "   1. Testez la skill: gemini-cli market-prod 'Quelle est la Phase 1?'"
echo "   2. Commencez l'implémentation selon le plan"
echo "   3. Respectez les quality gates à chaque étape"
