# NEXUS DESIGN SYSTEM

**Version:** 1.0
**Date:** 2025-10-29
**Créé à partir du:** Header Grid Layout

---

## 🎨 Philosophie du Design

Le design system NEXUS est basé sur:
- **Minimalisme géométrique**: Carrés, rectangles, cercles parfaits
- **Noir sur rose**: Contraste maximal, pas de couleurs intermédiaires
- **Iconographie vectorielle**: SVG stroke-based, style épuré
- **Typographie monospace**: Courier New pour l'aspect "terminal/crypto"
- **Séparations visibles**: Lignes de 3px pour délimiter les zones

---

## 📐 Aspect Ratios & Grid System

### Règles de base
```
Carrés: TOUJOURS 80x80px (ou multiples: 160x160, 240x240)
Rectangles: Hauteur fixe 80px, largeur variable (1fr)
Séparations: 3px solid rgba(0, 0, 0, 0.3)
Espacements: Multiples de 8px (8, 16, 24, 32, 40, 48...)
```

### Grid Layout Standard
```css
.nexus-grid {
  display: grid;
  grid-template-columns: 80px 1fr 1fr 80px 1fr;
  height: 80px;
  border: 3px solid rgba(0, 0, 0, 0.3);
}

.nexus-grid-item {
  border-right: 3px solid rgba(0, 0, 0, 0.3);
}
```

---

## 🎯 Iconographie

### Principes
1. **SVG uniquement** - Pas de PNG/JPEG pour les icônes
2. **Stroke-based** - `fill="none"` + `stroke="currentColor"`
3. **ViewBox 24x24** - Standard pour tous les SVG
4. **Stroke-width: 2** - Épaisseur constante
5. **Tailles d'affichage**: 32px (standard), 40px (emphasis), 24px (compact)

### Tailles Standard
```
- Icon Small:  24x24px (contextes compacts)
- Icon Medium: 32x32px (usage standard, header)
- Icon Large:  40x40px (emphasis, CTAs)
- Icon XL:     48x48px (hero sections)
```

### Template SVG
```html
<svg width="32" height="32" viewBox="0 0 24 24"
     fill="none" stroke="currentColor" stroke-width="2"
     stroke-linecap="round" stroke-linejoin="round">
  <!-- Paths ici -->
</svg>
```

---

## 🎭 Icônes NEXUS

### Catalogue d'icônes établi

#### **Auth & User**
```html
<!-- Login (flèche entrant) -->
<svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
  <path d="M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4M10 17l5-5-5-5M15 12H3"/>
</svg>

<!-- Logout (flèche sortant) -->
<svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
  <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4M16 17l5-5-5-5M21 12H9"/>
</svg>

<!-- Register (user + plus) -->
<svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
  <path d="M16 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2M12 7a4 4 0 1 0 0-8 4 4 0 0 0 0 8zM20 8v6M23 11h-6"/>
</svg>
```

#### **Navigation**
- HOME: Maison simple
- CATEGORIES: Grille 3x3
- LISTINGS: Liste à puces
- VENDORS: Groupe d'utilisateurs
- ORDERS: Document avec checkmark

#### **Actions**
- BUY: Panier
- SELL: Tag prix
- EDIT: Crayon
- DELETE: Poubelle
- SEARCH: Loupe
- FILTER: Entonnoir

#### **Status**
- SUCCESS: Checkmark dans cercle
- ERROR: X dans cercle
- WARNING: Triangle avec !
- INFO: i dans cercle
- PENDING: Horloge

#### **Crypto/Privacy**
- MONERO: Logo XMR stylisé
- ESCROW: Coffre
- MULTISIG: 3 clés entrelacées
- TOR: Oignon stylisé
- ENCRYPTED: Cadenas fermé
- DECRYPTED: Cadenas ouvert

---

## 🔘 Points & Toggles

### Point noir (Toggle)
```html
<div class="nexus-dot"></div>
```

```css
.nexus-dot {
  width: 40px;
  height: 40px;
  background: #0d0d0d;
  border-radius: 50%;
  transition: background 0.3s ease, transform 0.2s ease;
  cursor: pointer;
}

.nexus-dot:hover {
  transform: scale(1.1);
}

/* Light mode */
body.light-mode .nexus-dot {
  background: white;
  box-shadow: 0 0 0 2px #0d0d0d;
}
```

### Variantes de taille
- Small: 24px (indicateurs)
- Medium: 32px (boutons secondaires)
- Large: 40px (toggles, actions primaires)
- XL: 48px (hero CTAs)

---

## 📝 Typographie

### Hiérarchie
```css
/* Logo / H1 */
.nexus-logo {
  font-family: 'Courier New', monospace;
  font-size: 2.5rem; /* 40px */
  font-weight: 900;
  letter-spacing: 0.1em;
}

/* Navigation / Labels */
.nexus-nav {
  font-family: 'Courier New', monospace;
  font-size: 0.85rem; /* 13.6px */
  font-weight: 600;
  letter-spacing: 0.05em;
  text-transform: uppercase;
}

/* Body text */
.nexus-text {
  font-family: 'Courier New', monospace;
  font-size: 0.875rem; /* 14px */
  font-weight: 400;
  line-height: 1.6;
}

/* Typewriter / Status */
.nexus-typewriter {
  font-family: 'Courier New', monospace;
  font-size: 0.85rem; /* 13.6px */
  font-weight: 500;
  opacity: 0.7;
}
```

---

## 🎨 Couleurs

### Palette principale
```css
:root {
  /* Backgrounds */
  --nexus-bg-primary: hsl(349, 100%, 55%);    /* Rose vif */
  --nexus-bg-secondary: hsl(349, 100%, 60%);  /* Rose clair */

  /* Foregrounds */
  --nexus-fg-primary: #0d0d0d;               /* Noir */
  --nexus-fg-secondary: rgba(0, 0, 0, 0.7);  /* Noir 70% */
  --nexus-fg-tertiary: rgba(0, 0, 0, 0.5);   /* Noir 50% */

  /* Borders */
  --nexus-border-thick: 3px solid rgba(0, 0, 0, 0.3);
  --nexus-border-medium: 2px solid rgba(0, 0, 0, 0.2);
  --nexus-border-thin: 1px solid rgba(0, 0, 0, 0.15);

  /* Accent (erreurs uniquement) */
  --nexus-accent: #ff1744;                   /* Rouge vif */
}
```

### Règles d'usage
- **Jamais de dégradés** - Couleurs plates uniquement
- **Pas de gris** - Noir avec opacité (rgba)
- **Accent minimal** - Rouge uniquement pour erreurs/alertes
- **Contraste élevé** - Toujours >4.5:1 (WCAG AA)

---

## 🔲 Composants Standards

### Boutons
```html
<!-- Primary -->
<button class="nexus-btn nexus-btn-primary">
  ACTION
</button>

<!-- Secondary -->
<button class="nexus-btn nexus-btn-secondary">
  <svg class="nexus-icon">...</svg>
  ACTION
</button>

<!-- Icon only -->
<button class="nexus-btn-icon">
  <svg class="nexus-icon">...</svg>
</button>
```

### Cards
```html
<div class="nexus-card">
  <div class="nexus-card-header">
    <h3 class="nexus-card-title">TITRE</h3>
    <button class="nexus-btn-icon">...</button>
  </div>
  <div class="nexus-card-body">
    Contenu
  </div>
  <div class="nexus-card-footer">
    Actions
  </div>
</div>
```

### Séparateurs
```html
<!-- Vertical -->
<div class="nexus-separator-v"></div>

<!-- Horizontal -->
<div class="nexus-separator-h"></div>
```

```css
.nexus-separator-v {
  width: 3px;
  background: rgba(0, 0, 0, 0.3);
  height: 100%;
}

.nexus-separator-h {
  height: 3px;
  background: rgba(0, 0, 0, 0.3);
  width: 100%;
}
```

---

## ✅ Checklist de Cohérence

Avant de créer un nouveau composant, vérifier:

- [ ] Utilise-t-il Courier New en monospace ?
- [ ] Les icônes sont-elles des SVG stroke-based ?
- [ ] Les carrés sont-ils parfaitement carrés (80x80) ?
- [ ] Les séparations font-elles 3px ?
- [ ] Les espacements sont-ils des multiples de 8px ?
- [ ] Le contraste est-il suffisant (noir sur rose) ?
- [ ] Les tailles d'icônes suivent-elles les standards (24/32/40/48) ?
- [ ] Le hover change-t-il l'opacité ou la scale, pas la couleur ?
- [ ] Les transitions sont-elles cohérentes (0.2s ease) ?

---

## 📚 Exemples d'Application

### Header Grid (Référence)
✅ **Parfaitement conforme** - Utiliser comme modèle

### Listings Cards
🔄 **À retravailler** - Appliquer grid + icônes SVG

### Forms
🔄 **À retravailler** - Inputs avec bordures 3px, labels uppercase

### Footer
🔄 **À retravailler** - Aligner sur grid 80px

---

## 🚀 Prochaines Étapes

1. Créer bibliothèque d'icônes complète (`nexus-icons.svg`)
2. Créer composants réutilisables (`nexus-components.css`)
3. Auditer toutes les pages existantes
4. Documenter patterns d'animation
5. Créer Storybook/guide interactif

---

**Maintenu par:** Claude Code
**Référence:** Header Grid Layout (templates/partials/nexus/organisms/nav-grid.html)
**CSS:** static/css/nexus-grid-header.css
