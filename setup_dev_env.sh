#!/usr/bin/env bash
# ============================================================
#  Setup Rust + Cursor + Claude Code on Ubuntu
#  Author: Satisfy Guy
#  Version: 1.2 (Ubuntu Edition - Enhanced)
# ============================================================

set -e

# Couleurs pour une meilleure lisibilité
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() { echo -e "${BLUE}ℹ️  $1${NC}"; }
log_success() { echo -e "${GREEN}✅ $1${NC}"; }
log_warning() { echo -e "${YELLOW}⚠️  $1${NC}"; }
log_error() { echo -e "${RED}❌ $1${NC}"; exit 1; }

echo "🚀 Initialisation de la configuration Rust + Cursor + Claude pour Ubuntu..."

# --- Vérification des privilèges sudo ---
if ! sudo -n true 2>/dev/null; then
  log_warning "Ce script nécessite les privilèges sudo. Tu seras peut-être invité à entrer ton mot de passe."
fi

# --- Détection du shell ---
SHELL_NAME=$(basename "$SHELL")
if [[ "$SHELL_NAME" == "zsh" ]]; then
  SHELL_RC="$HOME/.zshrc"
elif [[ "$SHELL_NAME" == "bash" ]]; then
  SHELL_RC="$HOME/.bashrc"
else
  SHELL_RC="$HOME/.profile"
  log_warning "Shell non reconnu ($SHELL_NAME). Utilisation de ~/.profile"
fi

log_info "Configuration détectée : $SHELL_NAME ($SHELL_RC)"

# --- Mises à jour du système ---
log_info "Mise à jour du système..."
sudo apt update -y && sudo apt upgrade -y

# --- Installation des dépendances système ---
log_info "Installation des dépendances système..."
sudo apt install -y build-essential curl git pkg-config libssl-dev

# --- Installation Python et pip ---
if ! command -v python3 &>/dev/null; then
  log_info "Installation de Python3..."
  sudo apt install -y python3 python3-venv
else
  log_success "Python3 déjà installé ($(python3 --version))"
fi

if ! command -v pip3 &>/dev/null; then
  log_info "Installation de pip..."
  sudo apt install -y python3-pip
else
  log_success "pip déjà installé ($(pip3 --version))"
fi

# --- Installation Rust ---
if ! command -v rustup &>/dev/null; then
  log_info "Installation de Rust..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
  source "$HOME/.cargo/env"
  log_success "Rust installé avec succès"
else
  log_success "Rust déjà installé ($(rustc --version))"
  log_info "Mise à jour de Rust..."
  rustup update
fi

# S'assurer que cargo est dans le PATH
export PATH="$HOME/.cargo/bin:$PATH"

# --- Composants Rust ---
log_info "Installation des composants Rust..."
rustup component add rustfmt clippy rust-src rust-analyzer 2>/dev/null || true

if ! command -v cargo-watch &>/dev/null; then
  log_info "Installation de cargo-watch..."
  cargo install cargo-watch
else
  log_success "cargo-watch déjà installé"
fi

# Autres outils utiles
log_info "Installation d'outils Rust supplémentaires..."
cargo install cargo-edit --quiet 2>/dev/null || log_warning "cargo-edit déjà installé ou échec d'installation"
cargo install cargo-outdated --quiet 2>/dev/null || log_warning "cargo-outdated déjà installé ou échec d'installation"

# --- Installation Claude Code ---
if ! command -v claude &>/dev/null; then
  log_info "Installation de Claude Code..."
  pip3 install --upgrade claude-code --user
  
  # Ajouter ~/.local/bin au PATH si nécessaire
  if [[ ":$PATH:" != ".*:"$HOME"/.local/bin:"* ]]; then
    export PATH="$HOME/.local/bin:$PATH"
    if ! grep -q ".local/bin" "$SHELL_RC"; then
      echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$SHELL_RC"
    fi
  fi
  log_success "Claude Code installé"
else
  log_success "Claude Code déjà installé"
  log_info "Mise à jour de Claude Code..."
  pip3 install --upgrade claude-code --user
fi

# --- Configuration Cursor ---
CONFIG_DIR="$HOME/.config/Cursor/User"
mkdir -p "$CONFIG_DIR"

log_info "Application de la configuration Cursor..."
cat > "$CONFIG_DIR/settings.json" <<'EOF'
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.procMacro.enable": true,
  "rust-analyzer.cargo.runBuildScripts": true,
  "rust-analyzer.inlayHints.enable": true,
  "rust-analyzer.inlayHints.chainingHints.enable": true,
  "rust-analyzer.inlayHints.parameterHints.enable": true,
  "rust-analyzer.hover.actions.enable": true,
  "rust-analyzer.lens.enable": true,
  "editor.formatOnSave": true,
  "editor.codeActionsOnSave": {
    "source.fixAll": "explicit",
    "source.organizeImports": "explicit"
  },
  "files.insertFinalNewline": true,
  "files.trimTrailingWhitespace": true,
  "editor.wordWrap": "on",
  "editor.minimap.enabled": false,
  "editor.bracketPairColorization.enabled": true,
  "editor.guides.bracketPairs": true,
  "cursor.aiModel": "claude-sonnet-4-5-20250929",
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer",
    "editor.tabSize": 4
  }
}
EOF

log_success "Configuration Cursor appliquée"

# --- Snippets Rust ---
SNIPPET_DIR="$HOME/.config/Cursor/User/snippets"
mkdir -p "$SNIPPET_DIR"

log_info "Installation des snippets Rust..."
cat > "$SNIPPET_DIR/rust.json" <<'EOF'
{
  "rust_module": {
    "prefix": "modtemplate",
    "body": [
      "//! ${1:Module Name}",
      "",
      "use super::*",
      "",
      "pub fn ${2:function_name}() -> ${3:Result<(), Box<dyn std::error::Error>>} {",
      "    ${4:// TODO: implement}",
      "    Ok(())",
      "}",
      "",
      "#[cfg(test)]",
      "mod tests {",
      "    use super::*
      "",
      "    #[test]",
      "    fn ${5:test_name}() {",
      "        ${6:// TODO: write test}",
      "    }",
      "}"
    ],
    "description": "Template module Rust avec tests"
  },
  "rust_test": {
    "prefix": "testfn",
    "body": [
      "#[test]",
      "fn ${1:test_name}() {",
      "    ${2:assert_eq!(1, 1);}",
      "}"
    ],
    "description": "Template test function"
  },
  "rust_main": {
    "prefix": "mainrs",
    "body": [
      "fn main() -> Result<(), Box<dyn std::error::Error>> {",
      "    ${1:// TODO: implement}",
      "    Ok(())",
      "}"
    ],
    "description": "Main function avec gestion d'erreur"
  },
  "rust_derive": {
    "prefix": "derive",
    "body": [
      "#[derive(Debug, Clone, PartialEq)]",
      "${1:pub }struct ${2:Name} {",
      "    ${3:field}: ${4:Type},",
      "}"
    ],
    "description": "Struct avec derives courants"
  },
  "rust_error": {
    "prefix": "errortype",
    "body": [
      "#[derive(Debug, thiserror::Error)]",
      "pub enum ${1:Error} {",
      "    #[error(\"${2:error description}\")]",
      "    ${3:Variant},",
      "}"
    ],
    "description": "Type d'erreur avec thiserror"
  }
}
EOF

log_success "Snippets Rust installés"

# --- Alias pratiques ---
if ! grep -q "# === Cursor + Claude aliases ===" "$SHELL_RC"; then
  log_info "Ajout des alias dans $SHELL_RC..."
  cat >> "$SHELL_RC" <<'EOF'

# === Cursor + Claude aliases ===
alias cargoc='cargo check && claude "Summarize compiler warnings in plain English"'
alias cargot='cargo test -- --nocapture | claude "Explain the test results simply"'
alias cargor='cargo run 2>&1 | claude "Explain any errors or output"'
alias cargob='cargo build --release && claude "Summarize the build process"'
alias rustdoc='cargo doc --open'
alias rustfmt='cargo fmt -- --check'
alias rustclip='cargo clippy -- -D warnings'

# Rust environnement
export RUST_BACKTRACE=1
export PATH="$HOME/.cargo/bin:$PATH"
EOF
  log_success "Alias ajoutés"
else
  log_success "Alias déjà présents"
fi

# --- Vérification finale ---
log_info "Vérification de l'installation..."

check_command() {
  if command -v "$1" &>/dev/null; then
    log_success "$1 : $(command -v $1)"
  else
    log_warning "$1 : non trouvé"
  fi
}

check_command rustc
check_command cargo
check_command rustfmt
check_command clippy-driver
check_command rust-analyzer
check_command claude

# --- Template de projet Rust ---
log_info "Création d'un template de projet Rust..."
TEMPLATE_DIR="$HOME/rust-template"

if [[ ! -d "$TEMPLATE_DIR" ]]; then
  cargo new "$TEMPLATE_DIR" --vcs git
  
  # Ajout d'un Cargo.toml amélioré
  cat > "$TEMPLATE_DIR/Cargo.toml" <<'EOF'
[package]
name = "rust-template"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1.0"
thiserror = "1.0"

[dev-dependencies]
criterion = "0.5"

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
EOF

  log_success "Template de projet créé dans $TEMPLATE_DIR"
else
  log_info "Template de projet déjà existant"
fi

# --- Fin du setup ---
echo ""
echo "═══════════════════════════════════════════════════════════"
log_success "Installation terminée avec succès !"
echo "═══════════════════════════════════════════════════════════"
echo ""
log_info "Prochaines étapes :"
echo "  1️⃣  Redémarre ton terminal ou exécute : source $SHELL_RC"
echo "  2️⃣  Lance Cursor et ouvre un projet Rust"
echo "  3️⃣  Teste avec : cargo check"
echo ""
log_info "Alias disponibles :"
echo "  • cargoc  : Check + analyse IA"
echo "  • cargot  : Tests + explication IA"
echo "  • cargor  : Run + analyse d'erreurs"
echo "  • cargob  : Build release + résumé"
echo "  • rustdoc : Ouvre la documentation"
echo ""
log_info "Template de projet disponible dans : $TEMPLATE_DIR"
echo ""
echo "🚀 Bon dev avec Rust + Cursor + Claude !"
