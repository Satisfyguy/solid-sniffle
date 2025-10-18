# Claude Code Skills - Guide d'Utilisation

## 📋 Vue d'ensemble

Les **skills** Claude Code sont des outils personnalisés définis en YAML qui étendent les capacités de l'assistant. Ce projet contient une skill puissante pour détecter le "security theatre" dans le code Rust.

## 🎯 Skill disponible : `check-security-theatre`

### Description
Détecte automatiquement les patterns de "security theatre" dans votre code Rust avec :
- **5 niveaux de sévérité** : CRITICAL, HIGH, MEDIUM, LOW, INFO
- **Support Git** : Scanner uniquement les fichiers staged
- **Métriques** : Ratio issues/lignes de code
- **Exceptions configurables** : Fichier `.security-theatre-ignore`
- **Contexte intelligent** : Ignore les patterns légitimes (tests, examples)

### Localisation
```
.claude/skills/check-security-theatre.yaml
```

## 🚀 Comment utiliser la skill

### Méthode 1 : Via les commandes slash (Recommandé)

J'ai créé 4 commandes slash pour vous faciliter la vie :

#### `/security-check`
Scan complet avec toutes les sévérités (INFO et plus)
```
/security-check
```

#### `/security-check-verbose`
Scan complet avec détails de chaque issue
```
/security-check-verbose
```

#### `/security-check-staged`
Scan uniquement des fichiers Git staged (rapide)
```
/security-check-staged
```

#### `/security-check-critical`
Scan des issues CRITICAL uniquement (bloquant)
```
/security-check-critical
```

### Méthode 2 : Invocation directe de la skill

Vous pouvez invoquer la skill directement en demandant à Claude Code :

```
Execute the check-security-theatre skill with verbose=true
```

Ou avec paramètres spécifiques :
```
Execute the check-security-theatre skill with:
- scan_path: "server/src"
- min_severity: "HIGH"
- verbose: true
```

### Méthode 3 : Via l'interface Claude Code

Dans l'interface Claude Code Desktop :
1. Tapez `/` pour ouvrir le menu des commandes
2. Cherchez "check-security-theatre"
3. Sélectionnez la skill
4. Configurez les paramètres si nécessaire

## ⚙️ Paramètres de la skill

| Paramètre | Type | Défaut | Description |
|-----------|------|--------|-------------|
| `scan_path` | string | `"."` | Chemin du répertoire à scanner |
| `verbose` | boolean | `false` | Afficher tous les détails des issues |
| `git_staged_only` | boolean | `false` | Scanner uniquement les fichiers staged |
| `min_severity` | string | `"INFO"` | Niveau minimum (CRITICAL/HIGH/MEDIUM/LOW/INFO) |

### Exemples d'utilisation avec paramètres

**Scanner uniquement le serveur avec verbose :**
```
Execute check-security-theatre with scan_path="server" and verbose=true
```

**Vérifier les fichiers staged avant commit :**
```
Execute check-security-theatre with git_staged_only=true
```

**Audit de sécurité strict (CRITICAL + HIGH uniquement) :**
```
Execute check-security-theatre with min_severity="HIGH"
```

## 📊 Patterns détectés

### CRITICAL (Bloquant absolu)
- ❌ **Credentials hardcodés** : `password = "secret123"`
- ❌ **Unsafe sans justification** : `unsafe { ... }` sans commentaire `// SAFETY:`

### HIGH (Très dangereux)
- ⚠️ **Patterns interdits** : `.unwrap()`, `.expect("")`
- ⚠️ **Code mort** : `unimplemented!`, `todo!`, `panic!("")`

### MEDIUM (Problématique)
- 🟡 **Debug code** : `println!()`, `dbg!()`
- 🟡 **Magic numbers** : Nombres hardcodés > 1000

### LOW (Code smell)
- 🔵 **Placeholders** : `// TODO`, `// FIXME`, `// XXX`
- 🔵 **Tests désactivés** : `#[ignore]`, `#[cfg(never)]`

### INFO (Suggestions)
- 💡 **Suppositions** : `// should work`, `// probably works`
- 💡 **Placeholders temporaires** : `// TEMP`, `// REMOVE THIS`

## 🛡️ Exceptions contextuelles

La skill ignore automatiquement certains patterns légitimes :

### Tests
```rust
#[cfg(test)]
mod tests {
    // .unwrap() et panic! sont OK ici
    assert_eq!(result.unwrap(), 42);
}
```

### Examples
```rust
// examples/demo.rs
fn main() {
    println!("Hello"); // println! est OK dans les examples
}
```

### Debug builds
```rust
#[cfg(debug_assertions)]
fn debug_info() {
    println!("Debug mode"); // OK en debug
}
```

## 📝 Fichier d'exceptions : `.security-theatre-ignore`

Pour ajouter des exceptions personnalisées :

```bash
# Format: file_pattern:line_regex_pattern

# Ignorer println! dans l'outil CLI
cli/src/test_tool.rs:println!

# Ignorer .expect() dans les tests d'intégration
server/tests/.*:expect\(".*"\)

# Ignorer magic numbers dans les constantes de migration
server/migrations/.*:const.*=.*\d{4,}
```

**⚠️ Important :** Toute exception doit être justifiée dans le commit message.

## 🔄 Workflow recommandé

### 1. Développement
Pendant le développement, utilisez la skill pour valider votre code :

```
Execute check-security-theatre with scan_path="server/src/handlers"
```

### 2. Avant commit
Vérifiez les fichiers staged :

```
/security-check-staged
```

### 3. Pre-commit hook
La skill est automatiquement exécutée par le pre-commit hook :

```bash
./scripts/pre-commit.sh
# Inclut check-security-theatre.sh
```

### 4. Code review
Audit complet avant merge :

```
/security-check-verbose
```

### 5. Pre-production
Vérification critique avant déploiement :

```
/security-check-critical
```

## 📈 Interprétation des résultats

### Exit codes
- **0** : Aucune issue ou issues de sévérité < HIGH
- **1** : Issues CRITICAL ou HIGH détectées (bloquant)

### Métriques
```
📊 Metrics:
   Total lines: 5247
   Issues/100 lines: 1.24
```

**Cibles recommandées :**
- 🟢 Excellent : < 0.5 issues/100 lignes
- 🟡 Acceptable : 0.5 - 2.0 issues/100 lignes
- 🔴 À améliorer : > 2.0 issues/100 lignes

### Rapport par sévérité
```
📈 Issues by Severity:
  CRITICAL: 0
  HIGH: 3
  MEDIUM: 12
  LOW: 8
  INFO: 15
```

**Plan d'action :**
1. Corriger CRITICAL immédiatement (bloquant)
2. Corriger HIGH avant commit
3. Planifier MEDIUM pour le prochain sprint
4. Adresser LOW/INFO de manière opportuniste

## 🎓 Exemples pratiques

### Exemple 1 : Développement d'une nouvelle feature
```bash
# Avant de commencer
/security-check-staged

# Développement...
# (écrire du code)

# Après modifications
Execute check-security-theatre with scan_path="server/src/handlers/auth.rs" and verbose=true

# Avant commit
/security-check-staged
git add .
git commit -m "feat: Add authentication handler"
```

### Exemple 2 : Audit de sécurité complet
```bash
# Scan complet avec tous les détails
/security-check-verbose

# Si issues trouvées, scan par sévérité
Execute check-security-theatre with min_severity="HIGH"

# Corriger les issues
# ...

# Re-vérifier
/security-check
```

### Exemple 3 : Debug d'un échec pre-commit
```bash
# Pre-commit a échoué
# Vérifier les fichiers staged
/security-check-staged

# Détails des issues
Execute check-security-theatre with git_staged_only=true and verbose=true

# Corriger ou ajouter exception
echo "server/src/temp.rs:println!" >> .security-theatre-ignore

# Re-commit
git commit -m "fix: Address security theatre issues"
```

## 🔗 Intégration avec le workflow Git

### Pre-commit hook
Le hook `.git/hooks/pre-commit` exécute automatiquement :

```bash
#!/bin/bash
# ... autres vérifications ...

# Security theatre detection
if ! ./scripts/check-security-theatre.sh; then
    echo "❌ Security theatre detected!"
    echo "Run: /security-check-staged for details"
    exit 1
fi
```

### CI/CD Integration (à venir)
```yaml
# .github/workflows/security.yml
- name: Security Theatre Check
  run: |
    ./scripts/check-security-theatre.sh --min-severity HIGH
```

## 💡 Conseils et astuces

### 1. Utiliser la skill pendant le développement
Ne pas attendre le pre-commit hook - utilisez la skill **pendant** le développement :
```
Execute check-security-theatre with scan_path="current_file.rs"
```

### 2. Filtrer par sévérité pour focus
Concentrez-vous sur les issues critiques d'abord :
```
/security-check-critical
```

### 3. Mode verbose pour comprendre le contexte
Quand vous ne comprenez pas une issue :
```
Execute check-security-theatre with verbose=true and scan_path="problematic_file.rs"
```

### 4. Scanner un module spécifique
Validation rapide d'un module :
```
Execute check-security-theatre with scan_path="server/src/crypto"
```

### 5. Combiner avec Git pour tracking
```bash
# Avant modifications
/security-check > before.txt

# Après modifications
/security-check > after.txt

# Comparer
diff before.txt after.txt
```

## 🐛 Troubleshooting

### "No staged Rust files to check"
**Cause :** Aucun fichier Rust dans `git add`
**Solution :**
```bash
git add server/src/main.rs
/security-check-staged
```

### "Not a git repository"
**Cause :** Exécution hors d'un repo Git
**Solution :** Utiliser `scan_path` au lieu de `git_staged_only`

### Skill non trouvée
**Cause :** Claude Code ne détecte pas la skill
**Solution :**
1. Vérifier que le fichier existe : `.claude/skills/check-security-theatre.yaml`
2. Redémarrer Claude Code Desktop
3. Vérifier la syntaxe YAML avec `yamllint`

### Faux positifs
**Cause :** Pattern légitime détecté comme issue
**Solution :** Ajouter à `.security-theatre-ignore` avec justification

## 📚 Ressources complémentaires

- [CLAUDE.md](../CLAUDE.md) - Instructions complètes pour Claude Code
- [SECURITY-THEATRE-PREVENTION.md](../docs/SECURITY-THEATRE-PREVENTION.md) - Documentation détaillée
- [.cursorrules](../.cursorrules) - Règles de développement
- [pre-commit.sh](../scripts/pre-commit.sh) - Pipeline de validation

## 🔄 Mise à jour de la skill

Pour modifier la skill, éditez :
```
.claude/skills/check-security-theatre.yaml
```

Après modification :
1. Redémarrer Claude Code (si nécessaire)
2. Tester avec `/security-check-verbose`
3. Valider avec un fichier contenant des issues connues

## 🎯 Roadmap

### Version actuelle : 2.0
- ✅ 5 niveaux de sévérité
- ✅ Support Git staged files
- ✅ Métriques et rapports
- ✅ Exceptions contextuelles

### Version future : 3.0 (planifiée)
- 🔲 Support multi-langages (Python, TypeScript)
- 🔲 Métriques historiques
- 🔲 Auto-fix pour issues simples
- 🔲 Intégration IDE (VSCode extension)

---

**Dernière mise à jour :** 2025-10-18
**Mainteneur :** Monero Marketplace Team
**License :** MIT (Educational purpose only)
