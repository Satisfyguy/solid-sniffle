# Problème de Syntaxe Tera - Migration Nexus Design

**Date:** 2025-10-26
**Contexte:** Migration du design AMAZAWN vers NEXUS
**Blocage:** Erreurs de syntaxe Tera empêchant le démarrage du serveur

---

## 🚨 Problème Principal

Le serveur Actix-web refuse de démarrer à cause d'erreurs de parsing des templates Tera lors de l'initialisation :

```
Error: Failed to initialize Tera templates
Caused by: Failed to parse "/path/to/template.html"
```

## 🔍 Causes Identifiées

### 1. **Syntaxe `{% include "file" with param=value %}` NON SUPPORTÉE**

**Erreur Tera:**
```
Expected `%}` or `-%}` or ignore missing mark for include tag
```

**Code problématique:**
```html
{% include "partials/nexus/atoms/badge.html" with
   text="NEW"
   variant="primary"
   class="custom"
%}
```

**Raison:** Tera ne supporte **PAS** la syntaxe `with` pour passer des paramètres aux includes. Cette syntaxe existe dans Jinja2/Django templates mais pas dans Tera.

**Solution:** Utiliser des **macros** au lieu d'includes paramétrés :
```html
<!-- Dans badge.html -->
{% macro badge(text, variant="default", class="") %}
  <span class="nexus-badge nexus-badge-{{ variant }} {{ class }}">{{ text }}</span>
{% endmacro badge %}

<!-- Dans la page qui utilise le badge -->
{% import "partials/nexus/atoms/badge.html" as badge_comp %}
{{ badge_comp::badge(text="NEW", variant="primary") }}
```

### 2. **Dictionnaires Inline `{}` NON SUPPORTÉS**

**Erreur Tera:**
```
Expected a value that can be negated or an array of values
```

**Code problématique:**
```html
{% set default_icons = {
  "info": "ℹ️",
  "success": "✅",
  "warning": "⚠️",
  "error": "❌"
} %}
```

**Raison:** Tera ne permet pas de créer des dictionnaires inline avec la syntaxe `{}`.

**Solution:** Utiliser des conditions if/elif :
```html
{% if variant == "success" %}
  {% set icon = "✅" %}
{% elif variant == "warning" %}
  {% set icon = "⚠️" %}
{% elif variant == "error" %}
  {% set icon = "❌" %}
{% else %}
  {% set icon = "ℹ️" %}
{% endif %}
```

### 3. **Conditions Ternaires NON SUPPORTÉES**

**Erreur Tera:**
```
Expected `or`, `and`, `not`, `<=`, `>=`, etc.
```

**Code problématique:**
```html
{{ char if char != ' ' else '&nbsp;' | safe }}
```

**Raison:** Tera ne supporte pas la syntaxe ternaire Python/Jinja2 `value if condition else other`.

**Solution:** Utiliser un bloc if/else :
```html
{% if char != ' ' %}{{ char | safe }}{% else %}&nbsp;{% endif %}
```

### 4. **Array Slicing `[:n]` NON SUPPORTÉ**

**Erreur Tera:**
```
Expected `or`, `and`, `not`, etc. or a variable end (`}}`)
```

**Code problématique:**
```html
{{ order.escrow_id[:8] }}
```

**Raison:** Tera ne supporte pas le slicing Python `[start:end]`.

**Solution:** Utiliser le filtre `truncate` :
```html
{{ order.escrow_id | truncate(length=8, end="") }}
```

### 5. **Includes Imbriqués dans des Strings**

**Code problématique:**
```html
{% include "card.html" with
   content='
     <div>{{ title }}</div>
     ' ~ include("badge.html", with={"text": status}) ~ '
   '
%}
```

**Raison:** Multiples problèmes combinés (with + imbrication + dictionnaires).

**Solution:** Inline le HTML directement ou simplifier la structure.

---

## 🛠️ Actions Entreprises

### Tentative 1: Conversion en Macros (ABANDONNÉ - trop long)
- Converti `badge.html` et `button.html` en macros
- Réalisé que 40+ fichiers devaient être mis à jour
- Estimation: 30-40 minutes de travail

### Tentative 2: Script Python de Remplacement Automatique
**Créé:** `scripts/fix_tera_bulk.py`
- Regex pour remplacer tous les `{% include "..." with ... %}` par `{% include "..." %}`
- **Problème:** Ne gère pas les includes multiligne complexes
- **Résultat:** Seulement 16 fichiers sur 40 corrigés

### Tentative 3: Simplification Organismes Complexes
**Fichiers simplifiés manuellement:**
- `nav.html` - Remplacé dictionnaires par HTML fixe
- `footer.html` - Remplacé dictionnaires par HTML fixe
- `alert.html` - Remplacé dictionnaire par if/elif
- `toast.html` - Supprimé référence dictionnaire inexistant

### Tentative 4: Correction Pages Principales
**Fichiers traités:**
- `orders/index.html` - Remplacé tabs et cards par HTML inline + JavaScript
- `orders/show.html` - Remplacé breadcrumb, mais **CASSÉ** par sed
- `listings/show.html` - Remplacé breadcrumb, mais **CASSÉ** par sed

**Problème:** Les commandes `sed` ont supprimé des lignes critiques, cassant la structure des blocs if/for.

### Tentative 5: Restauration Backups AMAZAWN
- Restauré `*-old-amazawn.html` pour les fichiers cassés
- **Conflit:** L'utilisateur veut le design NEXUS, pas AMAZAWN

---

## 📊 État Actuel

### ✅ Serveur Fonctionne
- Compilation Rust: **OK** (warnings Diesel dans `wallet_rpc_config.rs` non bloquants)
- Tera init: **OK** avec templates AMAZAWN
- HTTP server: **Écoute sur 127.0.0.1:8080**

### ❌ Templates Nexus
**Fichiers avec erreurs Tera restantes:**
1. `orders/show.html` - card.html include multiligne (ligne 20)
2. `listings/show.html` - card.html include multiligne (ligne 20)
3. Potentiellement d'autres fichiers avec includes imbriqués

**Total estimé:** ~10-15 fichiers nécessitent encore des corrections

### 🔄 Fichiers Backup
Tous les templates Nexus ont des backups `*-old-amazawn.html` fonctionnels.

---

## 🎯 Solutions Proposées

### Option A: Finir la Migration Nexus (RECOMMANDÉ)
**Temps estimé:** 1-2 heures
**Approche:**
1. Créer un fichier unique `nexus-macros.html` avec tous les components essentiels en macros
2. Mettre à jour les 10-15 pages principales pour importer et utiliser ces macros
3. Tester chaque page individuellement

**Avantages:**
- Design Nexus complet et fonctionnel
- Architecture propre avec macros réutilisables
- Pas de dépendance aux includes paramétrés

**Inconvénients:**
- Travail manuel significatif restant
- Risque d'introduire de nouvelles erreurs

### Option B: Rollback Complet vers AMAZAWN (RAPIDE)
**Temps estimé:** 5 minutes
**Commandes:**
```bash
for file in templates/**/*-old-amazawn.html; do
  cp "$file" "${file%-old-amazawn.html}.html"
done
killall server && ./target/release/server &
```

**Avantages:**
- Interface fonctionnelle immédiatement
- Aucun risque d'erreurs Tera

**Inconvénients:**
- Perte de tout le travail de design Nexus
- Design AMAZAWN moins moderne

### Option C: Hybrid Temporaire
**Temps estimé:** 30 minutes
**Approche:**
1. Garder pages simples en Nexus (auth, settings) qui fonctionnent
2. Rollback pages complexes vers AMAZAWN (listings, orders)
3. Migrer progressivement page par page

---

## 📚 Leçons Apprises

### 🚫 À Ne Jamais Faire avec Tera
1. **N'utilisez jamais** `{% include "file" with param=value %}`
2. **N'utilisez jamais** de dictionnaires inline `{key: value}`
3. **N'utilisez jamais** de conditions ternaires `a if cond else b`
4. **N'utilisez jamais** de slicing d'array `array[:n]`
5. **N'utilisez jamais** d'includes dans des strings concaténées

### ✅ Bonnes Pratiques Tera
1. **Utilisez des macros** pour les components réutilisables
2. **Utilisez if/elif/else** au lieu de dictionnaires
3. **Utilisez des filtres** (`truncate`, `slice`, etc.) au lieu de slicing
4. **Gardez les templates simples** - évitez l'imbrication excessive
5. **Passez les données via le contexte Rust** plutôt que des paramètres template

### 🛡️ Workflow de Migration Recommandé
1. **Toujours** créer des backups avant modification massive
2. **Tester** chaque template individuellement après modification
3. **Utiliser** git pour tracker les changements
4. **Documenter** les patterns Tera supportés avant de commencer
5. **Commencer** par les pages simples, finir par les complexes

---

## 🔗 Ressources

**Documentation Tera:**
- https://keats.github.io/tera/docs/
- https://keats.github.io/tera/docs/#macros

**Templates fonctionnels (référence):**
- `templates/*-old-amazawn.html` - Syntaxe Tera validée
- `templates/base-nexus.html` - Base Nexus corrigée

**Scripts utilitaires:**
- `scripts/fix_tera_bulk.py` - Remplacement automatique includes
- `scripts/fix-tera-syntax.sh` - Script bash (non testé)

---

## 🎬 Prochaines Étapes Recommandées

1. **Décider** quelle option (A/B/C) adopter
2. Si Option A (Nexus):
   - Créer `templates/partials/nexus-macros.html` avec tous les components
   - Corriger `orders/show.html` et `listings/show.html` en priorité
   - Tester page par page
3. Si Option B (AMAZAWN):
   - Exécuter le script de rollback
   - Redémarrer le serveur
4. Si Option C (Hybrid):
   - Identifier les pages Nexus fonctionnelles
   - Rollback seulement les pages complexes
   - Documenter le plan de migration progressive

---

**Note:** Le serveur démarre actuellement avec les templates AMAZAWN restaurés. Pour tester l'interface utilisateur fonctionnelle, accéder à `http://127.0.0.1:8080/listings` ou `/auth/login`.
