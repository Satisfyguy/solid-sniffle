# Guide d'Utilisation des Icônes NEXUS

**Version:** 1.0
**Date:** 2025-10-29

---

## 🎯 Objectif

Ce guide explique comment utiliser le système d'icônes NEXUS de manière cohérente dans toute l'application.

---

## 📚 Setup

### 1. Inclure les fichiers requis

Dans votre `base-nexus.html` ou template principal:

```html
<head>
  <!-- CSS des icônes -->
  <link rel="stylesheet" href="/static/css/nexus-icons.css">
</head>

<body>
  <!-- Sprite SVG (invisible) - À inclure UNE SEULE FOIS -->
  <div style="display: none;">
    {% include "partials/nexus/icons/sprite.html" %}
  </div>

  <!-- Votre contenu -->
</body>
```

**Alternative:** Charger le sprite via AJAX ou l'inclure directement:

```html
<body>
  <script>
    fetch('/static/icons/nexus-icons.svg')
      .then(r => r.text())
      .then(text => {
        const div = document.createElement('div');
        div.style.display = 'none';
        div.innerHTML = text;
        document.body.insertBefore(div, document.body.firstChild);
      });
  </script>
</body>
```

---

## 🎨 Utilisation de Base

### Syntaxe Standard

```html
<svg class="nexus-icon">
  <use href="/static/icons/nexus-icons.svg#icon-NAME"></use>
</svg>
```

### Exemples

```html
<!-- Login icon -->
<svg class="nexus-icon">
  <use href="/static/icons/nexus-icons.svg#icon-login"></use>
</svg>

<!-- Cart icon (large) -->
<svg class="nexus-icon nexus-icon-lg">
  <use href="/static/icons/nexus-icons.svg#icon-cart"></use>
</svg>

<!-- Search icon (small) -->
<svg class="nexus-icon nexus-icon-sm">
  <use href="/static/icons/nexus-icons.svg#icon-search"></use>
</svg>
```

---

## 📏 Tailles Disponibles

```html
<!-- Small: 24×24px -->
<svg class="nexus-icon nexus-icon-sm">
  <use href="/static/icons/nexus-icons.svg#icon-user"></use>
</svg>

<!-- Medium: 32×32px (défaut) -->
<svg class="nexus-icon">
  <use href="/static/icons/nexus-icons.svg#icon-user"></use>
</svg>

<!-- Large: 40×40px -->
<svg class="nexus-icon nexus-icon-lg">
  <use href="/static/icons/nexus-icons.svg#icon-user"></use>
</svg>

<!-- Extra Large: 48×48px -->
<svg class="nexus-icon nexus-icon-xl">
  <use href="/static/icons/nexus-icons.svg#icon-user"></use>
</svg>
```

**Quand utiliser chaque taille:**
- `sm` (24px): Icônes inline dans le texte, badges, tags
- `md` (32px): Boutons, navigation, actions standards
- `lg` (40px): CTAs, toggles, actions importantes
- `xl` (48px): Hero sections, headers, emphase maximale

---

## 🎭 Utilisation dans les Composants

### Dans un Bouton

```html
<!-- Bouton avec icône et texte -->
<button class="nexus-btn nexus-btn-primary">
  <svg class="nexus-icon">
    <use href="/static/icons/nexus-icons.svg#icon-cart"></use>
  </svg>
  ACHETER
</button>

<!-- Bouton icon seul -->
<button class="nexus-btn-icon" aria-label="Rechercher">
  <svg class="nexus-icon">
    <use href="/static/icons/nexus-icons.svg#icon-search"></use>
  </svg>
</button>
```

### Dans un Lien

```html
<a href="/login" class="nexus-link">
  <svg class="nexus-icon nexus-icon-sm">
    <use href="/static/icons/nexus-icons.svg#icon-login"></use>
  </svg>
  Se connecter
</a>
```

### Dans une Card

```html
<div class="nexus-card">
  <svg class="nexus-card-icon">
    <use href="/static/icons/nexus-icons.svg#icon-shield"></use>
  </svg>
  <h3 class="nexus-card-title">ESCROW SÉCURISÉ</h3>
  <p>Protection 2-of-3 multisig...</p>
</div>
```

### Dans une Liste

```html
<ul class="nexus-list">
  <li class="nexus-list-item">
    <svg class="nexus-icon nexus-icon-sm">
      <use href="/static/icons/nexus-icons.svg#icon-check"></use>
    </svg>
    Paiement confirmé
  </li>
  <li class="nexus-list-item">
    <svg class="nexus-icon nexus-icon-sm">
      <use href="/static/icons/nexus-icons.svg#icon-clock"></use>
    </svg>
    En attente d'expédition
  </li>
</ul>
```

---

## 🎨 États et Couleurs

### Couleurs de Status

```html
<!-- Success (vert) -->
<svg class="nexus-icon nexus-icon-success">
  <use href="/static/icons/nexus-icons.svg#icon-check-circle"></use>
</svg>

<!-- Error (rouge) -->
<svg class="nexus-icon nexus-icon-error">
  <use href="/static/icons/nexus-icons.svg#icon-x-circle"></use>
</svg>

<!-- Warning (orange) -->
<svg class="nexus-icon nexus-icon-warning">
  <use href="/static/icons/nexus-icons.svg#icon-alert-triangle"></use>
</svg>

<!-- Info (bleu) -->
<svg class="nexus-icon nexus-icon-info">
  <use href="/static/icons/nexus-icons.svg#icon-info"></use>
</svg>

<!-- Pending (gris) -->
<svg class="nexus-icon nexus-icon-pending">
  <use href="/static/icons/nexus-icons.svg#icon-clock"></use>
</svg>
```

### Couleur Personnalisée

```html
<!-- L'icône hérite de la couleur du texte -->
<div style="color: #ff1744;">
  <svg class="nexus-icon">
    <use href="/static/icons/nexus-icons.svg#icon-trash"></use>
  </svg>
  Supprimer
</div>
```

---

## 🔄 Animation

```html
<!-- Icône tournante (loading) -->
<svg class="nexus-icon nexus-icon-spin">
  <use href="/static/icons/nexus-icons.svg#icon-settings"></use>
</svg>
```

---

## 📋 Catalogue Complet

### Auth & User
- `icon-login` - Se connecter
- `icon-logout` - Se déconnecter
- `icon-register` - S'inscrire
- `icon-user` - Utilisateur
- `icon-users` - Utilisateurs multiples / Vendors

### Navigation
- `icon-home` - Accueil
- `icon-grid` - Catégories / Grille
- `icon-list` - Listings / Liste
- `icon-file` - Orders / Document

### Actions
- `icon-cart` - Panier / Acheter
- `icon-tag` - Tag / Vendre
- `icon-edit` - Éditer
- `icon-trash` - Supprimer
- `icon-search` - Rechercher
- `icon-filter` - Filtrer
- `icon-plus` - Ajouter
- `icon-minus` - Retirer
- `icon-x` - Fermer

### Status
- `icon-check` - Validé
- `icon-check-circle` - Succès
- `icon-x-circle` - Erreur
- `icon-alert-triangle` - Alerte
- `icon-info` - Information
- `icon-clock` - En attente

### Crypto / Privacy
- `icon-lock` - Chiffré / Verrouillé
- `icon-unlock` - Déchiffré / Déverrouillé
- `icon-shield` - Protection / Escrow
- `icon-key` - Clé
- `icon-eye` - Visible
- `icon-eye-off` - Caché

### UI
- `icon-settings` - Paramètres
- `icon-menu` - Menu
- `icon-chevron-down` - Bas
- `icon-chevron-up` - Haut
- `icon-chevron-right` - Droite
- `icon-chevron-left` - Gauche
- `icon-arrow-right` - Flèche droite
- `icon-copy` - Copier
- `icon-download` - Télécharger
- `icon-upload` - Uploader
- `icon-external-link` - Lien externe

---

## ✅ Checklist d'Utilisation

Avant d'utiliser une icône, vérifier:

- [ ] L'icône existe dans `nexus-icons.svg`
- [ ] La taille est appropriée au contexte (sm/md/lg/xl)
- [ ] La couleur suit les règles (noir par défaut, status colors uniquement pour états)
- [ ] L'icône a un `aria-label` si elle est seule (accessibilité)
- [ ] La syntaxe `<use href="...">` est correcte

---

## 🚫 Erreurs Communes

### ❌ NE PAS FAIRE

```html
<!-- ❌ Inline SVG (pas réutilisable) -->
<svg width="32" height="32" viewBox="0 0 24 24">
  <path d="M15 3h4a2..."/>
</svg>

<!-- ❌ Image PNG (pas scalable) -->
<img src="icon-login.png" width="32">

<!-- ❌ Font icon (pas notre style) -->
<i class="fa fa-user"></i>

<!-- ❌ Taille personnalisée non-standard -->
<svg class="nexus-icon" style="width: 27px; height: 27px;">
  ...
</svg>
```

### ✅ FAIRE

```html
<!-- ✅ Utiliser le sprite -->
<svg class="nexus-icon">
  <use href="/static/icons/nexus-icons.svg#icon-login"></use>
</svg>

<!-- ✅ Tailles standards -->
<svg class="nexus-icon nexus-icon-lg">
  <use href="/static/icons/nexus-icons.svg#icon-login"></use>
</svg>
```

---

## 🔧 Ajouter une Nouvelle Icône

1. **Trouver l'icône** (format SVG stroke-based)
2. **Convertir au format NEXUS:**
   - `viewBox="0 0 24 24"`
   - `fill="none"`
   - `stroke="currentColor"`
   - `stroke-width="2"`
   - `stroke-linecap="round"`
   - `stroke-linejoin="round"`

3. **Ajouter à `nexus-icons.svg`:**

```xml
<symbol id="icon-NEW-NAME" viewBox="0 0 24 24">
  <path d="..." fill="none" stroke="currentColor" stroke-width="2"
        stroke-linecap="round" stroke-linejoin="round"/>
</symbol>
```

4. **Documenter dans ce guide** (section Catalogue)
5. **Tester dans différents contextes** (boutons, cards, listes)

---

## 📖 Ressources

- **Sprite SVG:** `/static/icons/nexus-icons.svg`
- **CSS:** `/static/css/nexus-icons.css`
- **Charte complète:** `DOX/NEXUS-DESIGN-SYSTEM.md`
- **Exemples:** `templates/examples/icons-showcase.html` (à créer)

---

## 🆘 Support

Si une icône manque ou si vous avez besoin d'aide:

1. Vérifier le catalogue ci-dessus
2. Consulter `NEXUS-DESIGN-SYSTEM.md`
3. Créer une issue sur le repo avec label `icons`

---

**Maintenu par:** Claude Code
**Dernière mise à jour:** 2025-10-29
