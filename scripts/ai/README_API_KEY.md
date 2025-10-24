# Configuration de la Clé API Anthropic

## 📍 Où Mettre Votre Clé API

Pour utiliser les scripts d'audit de sécurité automatisés avec Claude AI, vous devez configurer votre clé API Anthropic.

## 🔑 Obtenir Votre Clé API

1. **Créer un compte Anthropic** : https://console.anthropic.com/
2. **Générer une clé API** :
   - Aller sur https://console.anthropic.com/settings/keys
   - Cliquer sur "Create Key"
   - Copier la clé (format: `sk-ant-api03-...`)

## ⚙️ Méthode 1 : Variable d'Environnement (RECOMMANDÉ)

### Sur Linux/macOS :

```bash
# Créer le fichier .env à la racine du projet
cd /home/malix/Desktop/monero.marketplace
cp .env.example .env

# Éditer .env et remplacer 'your-api-key-here' par votre vraie clé
nano .env

# Exemple de contenu .env :
# ANTHROPIC_API_KEY=sk-ant-api03-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx

# Charger les variables d'environnement
source .env

# OU export directement (temporaire pour cette session)
export ANTHROPIC_API_KEY="sk-ant-api03-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
```

### Vérification :

```bash
# Vérifier que la clé est chargée
echo $ANTHROPIC_API_KEY
# Devrait afficher : sk-ant-api03-...
```

## ⚙️ Méthode 2 : Argument Direct (MOINS SÉCURISÉ)

```bash
python scripts/ai/claude_security_analyzer.py \
    --file server/src/main.rs \
    --api-key "sk-ant-api03-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
```

⚠️ **ATTENTION** : Cette méthode expose votre clé dans l'historique bash !

## 🚀 Utilisation des Scripts

### 1. Script d'Analyse Complète (claude_security_analyzer.py)

```bash
# Analyser un fichier spécifique
python scripts/ai/claude_security_analyzer.py --file server/src/handlers/escrow.rs

# Analyser un dossier entier (mode deep)
python scripts/ai/claude_security_analyzer.py --dir server/src --mode deep

# Analyser seulement les fichiers modifiés (git)
python scripts/ai/claude_security_analyzer.py --changed-files-only

# Mode quick (Haiku - plus rapide, moins cher)
python scripts/ai/claude_security_analyzer.py --file server/src/main.rs --mode quick
```

### 2. Script de Scan Rapide (claude_quick_scan.py)

```bash
# Scan rapide de tout le projet
python scripts/ai/claude_quick_scan.py

# Scan d'un fichier spécifique
python scripts/ai/claude_quick_scan.py --file server/src/wallet_manager.rs
```

## 📊 Options Disponibles

### claude_security_analyzer.py

```
--file PATH          Analyser un fichier Rust spécifique
--dir PATH           Analyser tous les fichiers .rs dans un dossier
--mode MODE          'deep' (Sonnet 4.5) ou 'quick' (Haiku)
--changed-files-only Analyser seulement les fichiers git modifiés
--post-to-pr         Poster les résultats sur une PR GitHub
--api-key KEY        Clé API Anthropic (ou via ANTHROPIC_API_KEY env var)
--output FILE        Fichier de sortie JSON (défaut: console)
```

### claude_quick_scan.py

```
--file PATH          Analyser un fichier spécifique
--severity LEVEL     Filtrer par niveau (CRITICAL, HIGH, MEDIUM, LOW)
--api-key KEY        Clé API Anthropic
```

## 🔒 Sécurité de la Clé API

### ✅ BONNES PRATIQUES

1. **Fichier .env** :
   - ✅ `.env` est dans `.gitignore` (ligne 28)
   - ✅ Ne JAMAIS committer `.env`
   - ✅ Utiliser `.env.example` comme template

2. **Permissions** :
   ```bash
   chmod 600 .env  # Seul le propriétaire peut lire/écrire
   ```

3. **Rotation** :
   - Changer votre clé API tous les 3 mois
   - Révoquer immédiatement si exposée

4. **Vérification** :
   ```bash
   # Vérifier que .env n'est pas tracké par git
   git status .env
   # Devrait dire "Untracked files" ou ne rien afficher
   ```

### ❌ À NE JAMAIS FAIRE

- ❌ Committer `.env` dans git
- ❌ Partager votre clé API sur Slack/Discord/Email
- ❌ Utiliser la même clé pour dev/prod
- ❌ Stocker la clé en clair dans le code source
- ❌ Passer la clé en argument de commande (visible dans `ps aux`)

## 💰 Coûts Anthropic

### Modèles Utilisés

| Modèle | Usage | Prix Input | Prix Output |
|--------|-------|------------|-------------|
| **Sonnet 4.5** (`deep`) | Analyse approfondie | $3/M tokens | $15/M tokens |
| **Haiku** (`quick`) | Scans rapides | $0.25/M tokens | $1.25/M tokens |

### Estimation Coûts

- **Fichier moyen (500 lignes)** : ~$0.02 (Sonnet) / ~$0.002 (Haiku)
- **Projet complet (50 fichiers)** : ~$1 (Sonnet) / ~$0.10 (Haiku)

💡 **Recommandation** : Utiliser `--mode quick` pour les scans quotidiens, `--mode deep` pour les audits pré-commit.

## 🧪 Test de Configuration

```bash
# Test rapide pour vérifier que tout fonctionne
python scripts/ai/claude_quick_scan.py --file server/src/main.rs

# Si succès, vous devriez voir :
# [INFO] Analyzing server/src/main.rs with Claude Haiku...
# [INFO] Security Score: XX/100
# ...
```

## 🆘 Dépannage

### Erreur : "ANTHROPIC_API_KEY not found"

```bash
# Vérifier que la variable est bien chargée
echo $ANTHROPIC_API_KEY

# Si vide, charger .env
source .env

# OU export manuel
export ANTHROPIC_API_KEY="votre-clé-ici"
```

### Erreur : "anthropic package not installed"

```bash
# Installer les dépendances Python
pip install -r requirements.txt

# OU installer anthropic seul
pip install anthropic>=0.40.0
```

### Erreur : "Invalid API key"

- Vérifier que la clé commence par `sk-ant-api03-`
- Vérifier qu'il n'y a pas d'espaces avant/après
- Régénérer une nouvelle clé sur https://console.anthropic.com/settings/keys

### Erreur : Rate Limit

```bash
# Attendre 60 secondes et réessayer
# OU utiliser --mode quick (Haiku moins limité)
```

## 📚 Ressources

- **Documentation Anthropic API** : https://docs.anthropic.com/
- **Console Anthropic** : https://console.anthropic.com/
- **Pricing** : https://www.anthropic.com/pricing
- **Status Page** : https://status.anthropic.com/

## 🎯 Workflow Recommandé

```bash
# 1. Développement quotidien (quick scan)
python scripts/ai/claude_quick_scan.py --changed-files-only

# 2. Avant commit (analyse fichiers modifiés)
python scripts/ai/claude_security_analyzer.py --changed-files-only --mode deep

# 3. Audit complet (hebdomadaire)
python scripts/ai/claude_security_analyzer.py --dir server/src --mode deep --output audit-$(date +%Y-%m-%d).json

# 4. CI/CD (Pull Request review)
python scripts/ai/claude_security_analyzer.py --changed-files-only --post-to-pr
```

---

**Prochaine étape** : Configurer votre clé API puis tester avec :

```bash
python scripts/ai/claude_quick_scan.py --file server/src/main.rs
```
