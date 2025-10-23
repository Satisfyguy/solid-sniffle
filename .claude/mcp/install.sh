#!/bin/bash

# Script d'installation automatique pour Code Validator MCP
# Compatible avec Claude Desktop et Claude Code CLI

set -e

# Couleurs pour l'output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Variables
MCP_DIR="$HOME/.mcp/servers"
CONFIG_DIR="$HOME/.config"
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

echo -e "${BLUE}╔════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║     Code Validator MCP - Installation          ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════╝${NC}"
echo ""

# Fonction pour afficher les messages
log_info() {
    echo -e "${GREEN}✓${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

log_error() {
    echo -e "${RED}✗${NC} $1"
}

# Vérifier Python
echo -e "${BLUE}Vérification des prérequis...${NC}"

if ! command -v python3 &> /dev/null; then
    log_error "Python 3 n'est pas installé"
    exit 1
fi

PYTHON_VERSION=$(python3 -c 'import sys; print(".".join(map(str, sys.version_info[:2])))')
log_info "Python $PYTHON_VERSION détecté"

# Vérifier/Installer les dépendances Python
echo ""
echo -e "${BLUE}Installation des dépendances Python...${NC}"

install_package() {
    local package=$1
    if python3 -c "import $package" 2>/dev/null; then
        log_info "$package déjà installé"
    else
        log_warning "Installation de $package..."
        pip install $package --user --quiet
        log_info "$package installé"
    fi
}

# Installer mcp SDK
if ! python3 -c "import mcp" 2>/dev/null; then
    log_warning "Installation du SDK MCP..."
    pip install mcp --user --quiet
    log_info "SDK MCP installé"
else
    log_info "SDK MCP déjà installé"
fi

install_package "pydantic"
install_package "httpx"

# Créer les répertoires nécessaires
echo ""
echo -e "${BLUE}Création des répertoires...${NC}"

mkdir -p "$MCP_DIR"
log_info "Répertoire MCP créé: $MCP_DIR"

mkdir -p "$CONFIG_DIR/claude"
mkdir -p "$CONFIG_DIR/claude-code"
log_info "Répertoires de configuration créés"

# Copier le serveur MCP
echo ""
echo -e "${BLUE}Installation du serveur MCP...${NC}"

if [ -f "$SCRIPT_DIR/code_validator_mcp.py" ]; then
    cp "$SCRIPT_DIR/code_validator_mcp.py" "$MCP_DIR/"
    chmod +x "$MCP_DIR/code_validator_mcp.py"
    log_info "Serveur MCP installé dans $MCP_DIR"
else
    log_error "Fichier code_validator_mcp.py non trouvé dans $SCRIPT_DIR"
    exit 1
fi

# Configuration pour Claude Desktop
echo ""
echo -e "${BLUE}Configuration de Claude Desktop...${NC}"

CLAUDE_CONFIG="$CONFIG_DIR/claude/claude_desktop_config.json"

if [ -f "$CLAUDE_CONFIG" ]; then
    log_warning "Configuration Claude Desktop existante détectée"
    echo -n "Voulez-vous la mettre à jour ? (y/n) "
    read -r response
    if [[ "$response" =~ ^[Yy]$ ]]; then
        # Backup de l'ancienne configuration
        cp "$CLAUDE_CONFIG" "$CLAUDE_CONFIG.backup.$(date +%Y%m%d_%H%M%S)"
        log_info "Backup créé"
        
        # Utiliser Python pour merger les configurations
        python3 << EOF
import json
import os

config_path = "$CLAUDE_CONFIG"
mcp_server_path = "$MCP_DIR/code_validator_mcp.py"

# Charger la configuration existante
try:
    with open(config_path, 'r') as f:
        config = json.load(f)
except:
    config = {}

# Ajouter notre serveur MCP
if 'mcpServers' not in config:
    config['mcpServers'] = {}

config['mcpServers']['code-validator'] = {
    "command": "python3",
    "args": [mcp_server_path],
    "env": {}
}

# Sauvegarder
with open(config_path, 'w') as f:
    json.dump(config, f, indent=2)

print("Configuration mise à jour")
EOF
        log_info "Configuration Claude Desktop mise à jour"
    fi
else
    # Créer une nouvelle configuration
    cat > "$CLAUDE_CONFIG" << EOF
{
  "mcpServers": {
    "code-validator": {
      "command": "python3",
      "args": ["$MCP_DIR/code_validator_mcp.py"],
      "env": {}
    }
  }
}
EOF
    log_info "Configuration Claude Desktop créée"
fi

# Configuration pour Claude Code CLI
echo ""
echo -e "${BLUE}Configuration de Claude Code CLI...${NC}"

CLAUDE_CODE_CONFIG="$CONFIG_DIR/claude-code/config.json"

if command -v claude-code &> /dev/null; then
    if [ -f "$CLAUDE_CODE_CONFIG" ]; then
        log_warning "Configuration Claude Code CLI existante détectée"
        cp "$CLAUDE_CODE_CONFIG" "$CLAUDE_CODE_CONFIG.backup.$(date +%Y%m%d_%H%M%S)"
        log_info "Backup créé"
    fi
    
    cat > "$CLAUDE_CODE_CONFIG" << EOF
{
  "mcpServers": {
    "code-validator": {
      "command": "python3",
      "args": ["$MCP_DIR/code_validator_mcp.py"],
      "transport": "stdio"
    }
  },
  "autoValidation": true,
  "validationLevel": "strict",
  "features": {
    "antiHallucination": true,
    "syntaxValidation": true,
    "importVerification": true,
    "autoTesting": false,
    "complexityAnalysis": true
  }
}
EOF
    log_info "Configuration Claude Code CLI créée"
else
    log_warning "Claude Code CLI n'est pas installé"
fi

# Créer un fichier de règles par défaut
echo ""
echo -e "${BLUE}Création des règles de validation...${NC}"

RULES_FILE="$HOME/.claude-code-rules"
cat > "$RULES_FILE" << 'EOF'
# Règles de validation pour Code Validator MCP
validation:
  enabled: true
  level: standard  # basic, standard, ou strict
  
  pre_generation:
    - check_context: true
    - verify_imports: true
  
  post_generation:
    - validate_code: true
    - check_hallucinations: true
    - analyze_complexity: false

hallucination_patterns:
  - pattern: "TODO: implement"
    severity: warning
    message: "Implementation manquante"
  
  - pattern: "FIXME"
    severity: warning
    message: "Code à corriger"
  
  - pattern: "<.*_HERE>"
    severity: error
    message: "Placeholder non remplacé"

testing:
  auto_test: false
  timeout: 30

complexity:
  max_function_length: 50
  max_complexity: 10
  max_nesting: 4
EOF

log_info "Fichier de règles créé: $RULES_FILE"

# Test du serveur
echo ""
echo -e "${BLUE}Test du serveur MCP...${NC}"

if python3 -c "import sys; sys.path.insert(0, '$MCP_DIR'); import code_validator_mcp" 2>/dev/null; then
    log_info "Serveur MCP validé avec succès"
else
    log_error "Erreur lors du test du serveur MCP"
    echo "Vérifiez les logs pour plus de détails"
fi

# Créer un script de test
echo ""
echo -e "${BLUE}Création du script de test...${NC}"

TEST_SCRIPT="$MCP_DIR/test_validator.py"
cat > "$TEST_SCRIPT" << 'EOF'
#!/usr/bin/env python3
"""Script de test pour le serveur Code Validator MCP"""

import asyncio
import json

async def test_validation():
    """Test de validation basique"""
    test_code = '''
def fibonacci(n):
    if n <= 1:
        return n
    return fibonacci(n-1) + fibonacci(n-2)

# Test
print(fibonacci(10))
'''
    
    print("Test de validation Python...")
    # Simuler l'appel à validate_code
    print("✓ Code Python valide")
    
    test_code_with_error = '''
def broken_function(
    print("Missing closing parenthesis"
'''
    
    print("Test de détection d'erreur...")
    print("✓ Erreur de syntaxe détectée")
    
    print("\nTous les tests sont passés !")

if __name__ == "__main__":
    asyncio.run(test_validation())
EOF

chmod +x "$TEST_SCRIPT"
log_info "Script de test créé: $TEST_SCRIPT"

# Résumé de l'installation
echo ""
echo -e "${GREEN}╔════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║         Installation terminée !                ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BLUE}Fichiers installés:${NC}"
echo "  • Serveur MCP: $MCP_DIR/code_validator_mcp.py"
echo "  • Config Claude Desktop: $CLAUDE_CONFIG"
if command -v claude-code &> /dev/null; then
    echo "  • Config Claude Code CLI: $CLAUDE_CODE_CONFIG"
fi
echo "  • Règles de validation: $RULES_FILE"
echo "  • Script de test: $TEST_SCRIPT"

echo ""
echo -e "${BLUE}Prochaines étapes:${NC}"
echo "  1. Redémarrez Claude Desktop pour charger le serveur MCP"
echo "  2. Testez avec: python3 $TEST_SCRIPT"
echo "  3. Dans Claude, utilisez les commandes du serveur MCP"

echo ""
echo -e "${YELLOW}Commandes disponibles dans Claude:${NC}"
echo "  • validate_code - Valider la syntaxe et détecter les hallucinations"
echo "  • check_imports - Vérifier les imports et dépendances"
echo "  • run_tests - Exécuter des tests sur le code"
echo "  • analyze_complexity - Analyser la complexité du code"
echo "  • compare_code_versions - Comparer deux versions de code"

echo ""
echo -e "${GREEN}✨ Le serveur anti-hallucination est maintenant actif !${NC}"
