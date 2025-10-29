# MVP - GEOMETRIC UI TRANSFORMATION
## Plan de Transformation Complète - 7 Jours

**Version:** 1.0
**Date:** 2025-10-29
**Statut:** Planning Phase
**Objectif:** Créer un MVP fonctionnel de l'interface géométrique interactive en 7 jours

---

## 📋 EXECUTIVE SUMMARY

### Vision
Transformer le NEXUS Marketplace en une expérience visuelle unique avec une navigation géométrique interactive inspirée d'un design abstrait coloré. La page d'accueil devient un canvas interactif où chaque forme géométrique est un bouton de navigation.

### Approche MVP
- **Durée:** 7 jours
- **Scope:** Homepage géométrique + page Listings hybride
- **Validation:** Test utilisateur pour décider de la suite (3 semaines supplémentaires ou ajustement)
- **Risque:** Mesuré - MVP avant engagement complet

### Compromis Acceptés
1. **Quick Buy Toast:** Message toast après 3-5s pour utilisateurs pressés
2. **Vendor Dashboard:** Mode classique avec branding géométrique seulement
3. **Pages Internes:** Approche hybride (branding géométrique + contenu classique)
4. **User Menu:** Avatar géométrique avec dropdown standard
5. **Onboarding:** Modal tutoriel obligatoire première visite

---

## 🎨 DESIGN SYSTEM

### Palette de Couleurs (Exacte de l'image de référence)

```css
--geo-pink: #E91E8C;
--geo-pink-dark: #C01A75;
--geo-yellow: #D4A44A;
--geo-yellow-dark: #B8873D;
--geo-turquoise: #4CA0A0;
--geo-turquoise-dark: #3E8282;
--geo-blue: #3E6DB5;
--geo-blue-dark: #325A96;
--geo-beige: #EFE4D8;
```

### Mapping Navigation → Formes Géométriques

| Forme | Couleur | Position | Fonction | Taille |
|-------|---------|----------|----------|--------|
| **Forme 1** | Rose (#E91E8C) | Haut-gauche | LOGIN | Circle 80px |
| **Forme 2** | Jaune (#D4A44A) | Haut-centre | REGISTER | Arc 120px |
| **Forme 3** | Bleu (#3E6DB5) | Centre | LISTINGS | Large arch 200px |
| **Forme 4** | Rose (#E91E8C) | Droite | ORDERS | Circle 100px |
| **Forme 5** | Turquoise (#4CA0A0) | Bas-centre | ESCROW | Abstract shape |
| **Forme 6** | Turquoise (#4CA0A0) | Bas-gauche | SETTINGS | Cactus shape |

### Animations & Effets

**Hover:**
- Scale: 1.0 → 1.15 (0.4s ease-out)
- Couleur: base → dark variant
- Label: opacity 0 → 1 (fade-up)

**Click:**
- Morphing shape (0.8s cubic-bezier)
- Page slide transition

**Background:**
- Parallax sur mouvement souris
- 15-20 formes décoratives
- Vitesse variable (data-speed: 0.02 à 0.08)

---

## 📁 STRUCTURE DE FICHIERS

### Nouveaux Fichiers (10 fichiers)

#### CSS (5 fichiers)
```
static/css/
├── geometric-variables.css   (Design tokens)
├── geometric-base.css        (Base styles, canvas, shapes)
├── geometric-animations.css  (Keyframes, transitions)
├── geometric-components.css  (Modal, toast, user menu)
└── geometric-listings.css    (Hybrid listings page)
```

#### HTML Templates (3 fichiers)
```
templates/
├── base-geometric.html                (Base template géométrique)
├── geometric-home.html                (Homepage interactive)
└── listings/geometric-listings.html   (Listings hybride)
```

#### JavaScript (5 fichiers)
```
static/js/
├── geometric-nav.js         (Navigation principale)
├── geometric-onboarding.js  (Modal tutoriel première visite)
├── geometric-toast.js       (Quick buy toast)
├── geometric-user-menu.js   (User avatar dropdown)
└── geometric-utils.js       (Fonctions utilitaires)
```

### Fichiers à Modifier

#### Backend
- `server/src/main.rs` ou `server/src/routes/frontend.rs`
  - Route `GET /` → `geometric-home.html`
  - Route `GET /listings` → `geometric-listings.html`
  - Préserver: auth session, CSRF tokens, user context

---

## 🗓️ TIMELINE - 7 JOURS

### **Jour 1-2: Design System & Homepage Structure**

**Objectifs:**
- ✅ Créer système de design CSS complet
- ✅ Structurer homepage SVG avec 6 formes navigation
- ✅ Implémenter animations de base

**Livrables:**
1. `geometric-variables.css` (complet)
2. `geometric-base.css` (complet)
3. `geometric-animations.css` (complet)
4. `geometric-home.html` (structure SVG)
5. `base-geometric.html` (template de base)

**Code Clé:**

**geometric-variables.css:**
```css
:root {
  /* Couleurs principales */
  --geo-pink: #E91E8C;
  --geo-pink-dark: #C01A75;
  --geo-yellow: #D4A44A;
  --geo-yellow-dark: #B8873D;
  --geo-turquoise: #4CA0A0;
  --geo-turquoise-dark: #3E8282;
  --geo-blue: #3E6DB5;
  --geo-blue-dark: #325A96;
  --geo-beige: #EFE4D8;

  /* Tailles de formes */
  --geo-shape-xs: 60px;
  --geo-shape-sm: 80px;
  --geo-shape-md: 120px;
  --geo-shape-lg: 200px;
  --geo-shape-xl: 400px;

  /* Border radius */
  --geo-radius-sm: 8px;
  --geo-radius-md: 16px;
  --geo-radius-lg: 32px;
  --geo-radius-full: 9999px;

  /* Animations */
  --geo-hover-scale: 1.15;
  --geo-hover-duration: 0.4s;
  --geo-morph-duration: 0.8s;
  --geo-parallax-speed: 0.05;

  /* Typography */
  --geo-font-sans: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
  --geo-font-mono: "Courier New", monospace;

  /* Z-index layers */
  --z-background: 0;
  --z-decor: 1;
  --z-content: 10;
  --z-nav: 50;
  --z-modal: 100;
  --z-toast: 200;
}
```

**geometric-base.css:**
```css
.geometric-body {
  margin: 0;
  padding: 0;
  overflow: hidden;
  background-color: var(--geo-beige);
  font-family: var(--geo-font-sans);
}

.geometric-canvas {
  position: relative;
  width: 100vw;
  height: 100vh;
  overflow: hidden;
}

.geo-shape {
  transition: all var(--geo-hover-duration) ease-out;
  cursor: pointer;
}

.geo-shape:hover {
  transform: scale(var(--geo-hover-scale));
}

.geo-label {
  opacity: 0;
  transition: opacity 0.3s ease-out, transform 0.3s ease-out;
  transform: translateY(10px);
  font-family: var(--geo-font-mono);
  font-weight: 700;
  font-size: 14px;
  letter-spacing: 2px;
  pointer-events: none;
}

.geo-shape:hover + .geo-label {
  opacity: 1;
  transform: translateY(0);
}

.geo-clickable {
  cursor: pointer;
  user-select: none;
}

/* Formes décoratives avec parallax */
.geo-decor {
  position: absolute;
  pointer-events: none;
  opacity: 0.6;
  z-index: var(--z-decor);
  transition: transform 0.1s ease-out;
}
```

**geometric-animations.css:**
```css
@keyframes geo-float {
  0%, 100% { transform: translateY(0px); }
  50% { transform: translateY(-20px); }
}

@keyframes geo-pulse {
  0%, 100% { opacity: 0.6; }
  50% { opacity: 1; }
}

@keyframes geo-fade-up {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@keyframes geo-morph-out {
  0% { transform: scale(1) rotate(0deg); opacity: 1; }
  100% { transform: scale(0.5) rotate(180deg); opacity: 0; }
}

@keyframes geo-slide-in-right {
  from { transform: translateX(100%); }
  to { transform: translateX(0); }
}

@keyframes geo-toast-in {
  from {
    transform: translateY(100px);
    opacity: 0;
  }
  to {
    transform: translateY(0);
    opacity: 1;
  }
}

/* Support prefers-reduced-motion */
@media (prefers-reduced-motion: reduce) {
  *,
  *::before,
  *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
  }
}
```

**geometric-home.html:**
```html
{% extends "base-geometric.html" %}

{% block content %}
<div class="geometric-canvas" id="geometricCanvas">
  <svg viewBox="0 0 1920 1080" preserveAspectRatio="xMidYMid slice" class="geo-svg-main">

    <!-- Formes de navigation (6) -->

    <!-- FORME 1: LOGIN (circle rose, haut-gauche) -->
    <g class="geo-nav-item" data-href="/login" data-color="pink">
      <circle cx="200" cy="150" r="80" fill="var(--geo-pink)" class="geo-shape" />
      <text x="200" y="155" text-anchor="middle" class="geo-label">LOGIN</text>
    </g>

    <!-- FORME 2: REGISTER (arc jaune, haut-centre) -->
    <g class="geo-nav-item" data-href="/register" data-color="yellow">
      <path d="M 800 100 A 120 120 0 0 1 1040 100" fill="none" stroke="var(--geo-yellow)" stroke-width="40" class="geo-shape" />
      <text x="920" y="80" text-anchor="middle" class="geo-label">REGISTER</text>
    </g>

    <!-- FORME 3: LISTINGS (large arch bleu, centre) -->
    <g class="geo-nav-item" data-href="/listings" data-color="blue">
      <path d="M 600 400 Q 960 200 1320 400" fill="var(--geo-blue)" class="geo-shape" />
      <text x="960" y="350" text-anchor="middle" class="geo-label">LISTINGS</text>
    </g>

    <!-- FORME 4: ORDERS (circle rose, droite) -->
    <g class="geo-nav-item" data-href="/orders" data-color="pink">
      <circle cx="1600" cy="300" r="100" fill="var(--geo-pink)" class="geo-shape" />
      <text x="1600" y="310" text-anchor="middle" class="geo-label">ORDERS</text>
    </g>

    <!-- FORME 5: ESCROW (forme turquoise abstraite, bas-centre) -->
    <g class="geo-nav-item" data-href="/escrow" data-color="turquoise">
      <polygon points="960,700 1100,850 820,850" fill="var(--geo-turquoise)" class="geo-shape" />
      <text x="960" y="820" text-anchor="middle" class="geo-label">ESCROW</text>
    </g>

    <!-- FORME 6: SETTINGS (cactus turquoise, bas-gauche) -->
    <g class="geo-nav-item" data-href="/settings" data-color="turquoise">
      <rect x="150" y="700" width="60" height="120" rx="8" fill="var(--geo-turquoise)" class="geo-shape" />
      <circle cx="140" cy="780" r="20" fill="var(--geo-turquoise)" class="geo-shape" />
      <circle cx="220" cy="750" r="15" fill="var(--geo-turquoise)" class="geo-shape" />
      <text x="180" y="840" text-anchor="middle" class="geo-label">SETTINGS</text>
    </g>

    <!-- Formes décoratives (15-20) avec parallax -->
    <circle cx="1400" cy="600" r="40" fill="var(--geo-yellow)" class="geo-decor" data-speed="0.02" />
    <rect x="300" y="500" width="80" height="80" rx="16" fill="var(--geo-pink)" class="geo-decor" data-speed="0.05" />
    <circle cx="1700" cy="800" r="60" fill="var(--geo-turquoise)" class="geo-decor" data-speed="0.03" />
    <!-- ... 12-17 autres formes décoratives ... -->

  </svg>
</div>

<!-- Modal Onboarding -->
<div id="geoOnboardingModal" class="geo-modal" style="display: none;">
  <div class="geo-modal-content">
    <h2>Bienvenue sur NEXUS Geometric</h2>
    <p>Naviguez en cliquant sur les formes géométriques colorées.</p>
    <div class="geo-modal-buttons">
      <button id="skipTour" class="geo-btn-secondary">Passer</button>
      <button id="startTour" class="geo-btn-primary">Démarrer le tour</button>
    </div>
  </div>
</div>

<!-- Toast Quick Buy -->
<div id="geoQuickBuyToast" class="geo-toast" style="display: none;">
  <div class="geo-toast-content">
    <p>💡 Vous cherchez à acheter rapidement ?</p>
    <a href="/listings" class="geo-btn-primary-sm">Voir les Listings</a>
  </div>
  <button class="geo-toast-close" aria-label="Fermer">✕</button>
</div>

<!-- User Menu (si logged_in) -->
{% if logged_in %}
<div class="geo-user-menu">
  <button class="geo-user-avatar" id="geoUserAvatar" aria-label="Menu utilisateur">
    {{ username[0]|upper }}
  </button>
  <div class="geo-user-dropdown" id="geoUserDropdown" style="display: none;">
    <a href="/profile">Mon Profil</a>
    <a href="/orders">Mes Commandes</a>
    <a href="/settings">Paramètres</a>
    <hr>
    <a href="/logout">Déconnexion</a>
  </div>
</div>
{% endif %}

{% endblock %}
```

**base-geometric.html:**
```html
<!DOCTYPE html>
<html lang="fr">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{% block title %}NEXUS Geometric{% endblock %}</title>

  <!-- Geometric CSS -->
  <link rel="stylesheet" href="/static/css/geometric-variables.css">
  <link rel="stylesheet" href="/static/css/geometric-base.css">
  <link rel="stylesheet" href="/static/css/geometric-animations.css">
  <link rel="stylesheet" href="/static/css/geometric-components.css">

  <!-- HTMX (local) -->
  <script src="/static/js/htmx.min.js"></script>
  <script src="/static/js/json-enc.js"></script>

  {% block extra_css %}{% endblock %}
</head>
<body class="geometric-body">

  {% block content %}{% endblock %}

  <!-- Geometric JS -->
  <script src="/static/js/geometric-utils.js" defer></script>
  <script src="/static/js/geometric-nav.js" defer></script>
  <script src="/static/js/geometric-onboarding.js" defer></script>
  <script src="/static/js/geometric-toast.js" defer></script>
  <script src="/static/js/geometric-user-menu.js" defer></script>

  {% block extra_js %}{% endblock %}

  <!-- NEXUS_DEBUG from Phase 6.2 -->
  <script>
    window.NEXUS_DEBUG = {{ nexus_debug|default(value=false) }};
    if (window.NEXUS_DEBUG) {
      console.log('[NEXUS DEBUG] Geometric UI loaded');
    }
  </script>
</body>
</html>
```

**Checklist Jour 1-2:**
- [ ] Créer geometric-variables.css avec toutes les custom properties
- [ ] Créer geometric-base.css avec styles canvas et shapes
- [ ] Créer geometric-animations.css avec keyframes
- [ ] Créer geometric-home.html avec 6 formes navigation SVG
- [ ] Créer base-geometric.html minimal
- [ ] Tester affichage statique dans navigateur
- [ ] Vérifier responsive (mobile, tablet, desktop)

---

### **Jour 3: Components CSS (Modal, Toast, User Menu)**

**Objectifs:**
- ✅ Implémenter modal onboarding
- ✅ Implémenter toast quick buy
- ✅ Implémenter user menu dropdown
- ✅ Styliser tous les boutons géométriques

**Livrables:**
1. `geometric-components.css` (complet)
2. Styles pour modal, toast, user menu, buttons

**Code Clé:**

**geometric-components.css:**
```css
/* === MODAL === */
.geo-modal {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background-color: rgba(0, 0, 0, 0.8);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: var(--z-modal);
  animation: geo-fade-in 0.3s ease-out;
}

.geo-modal-content {
  background: linear-gradient(135deg, var(--geo-beige) 0%, #fff 100%);
  padding: 3rem;
  border-radius: var(--geo-radius-lg);
  max-width: 600px;
  text-align: center;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
  animation: geo-fade-up 0.4s ease-out;
}

.geo-modal h2 {
  font-size: 2rem;
  margin-bottom: 1rem;
  color: var(--geo-blue);
}

.geo-modal-buttons {
  display: flex;
  gap: 1rem;
  justify-content: center;
  margin-top: 2rem;
}

/* === TOAST === */
.geo-toast {
  position: fixed;
  bottom: 2rem;
  right: 2rem;
  background: white;
  padding: 1.5rem;
  border-radius: var(--geo-radius-md);
  box-shadow: 0 10px 40px rgba(0, 0, 0, 0.2);
  z-index: var(--z-toast);
  max-width: 400px;
  animation: geo-toast-in 0.5s ease-out;
  border-left: 4px solid var(--geo-pink);
}

.geo-toast-content {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.geo-toast-close {
  position: absolute;
  top: 0.5rem;
  right: 0.5rem;
  background: none;
  border: none;
  font-size: 1.2rem;
  cursor: pointer;
  color: #999;
}

.geo-toast-close:hover {
  color: #333;
}

/* === USER MENU === */
.geo-user-menu {
  position: fixed;
  top: 2rem;
  right: 2rem;
  z-index: var(--z-nav);
}

.geo-user-avatar {
  width: 60px;
  height: 60px;
  border-radius: var(--geo-radius-full);
  background: linear-gradient(135deg, var(--geo-pink), var(--geo-blue));
  color: white;
  font-size: 1.5rem;
  font-weight: 700;
  border: 3px solid white;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
  cursor: pointer;
  transition: transform 0.3s ease-out;
  display: flex;
  align-items: center;
  justify-content: center;
}

.geo-user-avatar:hover {
  transform: scale(1.1);
}

.geo-user-dropdown {
  position: absolute;
  top: 70px;
  right: 0;
  background: white;
  border-radius: var(--geo-radius-md);
  box-shadow: 0 10px 40px rgba(0, 0, 0, 0.2);
  min-width: 200px;
  overflow: hidden;
  animation: geo-fade-up 0.2s ease-out;
}

.geo-user-dropdown a {
  display: block;
  padding: 1rem 1.5rem;
  color: #333;
  text-decoration: none;
  transition: background-color 0.2s;
}

.geo-user-dropdown a:hover {
  background-color: var(--geo-beige);
}

.geo-user-dropdown hr {
  margin: 0;
  border: none;
  border-top: 1px solid #eee;
}

/* === BUTTONS === */
.geo-btn-primary {
  background: linear-gradient(135deg, var(--geo-pink), var(--geo-blue));
  color: white;
  padding: 0.75rem 2rem;
  border: none;
  border-radius: var(--geo-radius-md);
  font-weight: 600;
  cursor: pointer;
  transition: transform 0.2s, box-shadow 0.2s;
  box-shadow: 0 4px 12px rgba(233, 30, 140, 0.3);
}

.geo-btn-primary:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(233, 30, 140, 0.4);
}

.geo-btn-secondary {
  background: white;
  color: var(--geo-blue);
  padding: 0.75rem 2rem;
  border: 2px solid var(--geo-blue);
  border-radius: var(--geo-radius-md);
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.geo-btn-secondary:hover {
  background: var(--geo-blue);
  color: white;
}

.geo-btn-primary-sm {
  background: var(--geo-pink);
  color: white;
  padding: 0.5rem 1rem;
  border: none;
  border-radius: var(--geo-radius-sm);
  font-size: 0.9rem;
  font-weight: 600;
  cursor: pointer;
  text-decoration: none;
  display: inline-block;
  transition: background-color 0.2s;
}

.geo-btn-primary-sm:hover {
  background-color: var(--geo-pink-dark);
}
```

**Checklist Jour 3:**
- [ ] Créer geometric-components.css complet
- [ ] Tester modal onboarding (affichage + fermeture)
- [ ] Tester toast quick buy (apparition + dismiss)
- [ ] Tester user menu (toggle dropdown + click outside)
- [ ] Vérifier responsive de tous les composants
- [ ] Tester accessibilité (tabulation, Escape key)

---

### **Jour 4: JavaScript & Backend Integration**

**Objectifs:**
- ✅ Implémenter logique navigation interactive
- ✅ Implémenter parallax background
- ✅ Implémenter onboarding localStorage
- ✅ Implémenter toast timer
- ✅ Modifier routes backend

**Livrables:**
1. `geometric-nav.js` (complet)
2. `geometric-onboarding.js` (complet)
3. `geometric-toast.js` (complet)
4. `geometric-user-menu.js` (complet)
5. `geometric-utils.js` (complet)
6. Routes backend modifiées

**Code Clé:**

**geometric-nav.js:**
```javascript
class GeometricNav {
  constructor() {
    this.navItems = document.querySelectorAll('.geo-nav-item');
    this.decorShapes = document.querySelectorAll('.geo-decor');
    this.canvas = document.getElementById('geometricCanvas');
    this.init();
  }

  init() {
    this.navItems.forEach(item => {
      const shape = item.querySelector('.geo-shape');
      const label = item.querySelector('.geo-label');
      const href = item.dataset.href;
      const color = item.dataset.color;

      // Hover effects
      item.addEventListener('mouseenter', () => this.handleHover(shape, label, color, true));
      item.addEventListener('mouseleave', () => this.handleHover(shape, label, color, false));

      // Click navigation
      item.addEventListener('click', (e) => this.handleClick(e, shape, href));
    });

    // Parallax on mouse move
    if (!prefersReducedMotion()) {
      this.canvas.addEventListener('mousemove', (e) => this.handleParallax(e));
    }

    // Keyboard navigation
    this.navItems.forEach((item, index) => {
      item.setAttribute('tabindex', '0');
      item.addEventListener('keydown', (e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault();
          item.click();
        }
      });
    });
  }

  handleHover(shape, label, color, isEntering) {
    if (isEntering) {
      shape.style.transform = 'scale(1.15)';
      shape.style.fill = `var(--geo-${color}-dark)`;
      if (label) {
        label.style.opacity = '1';
        label.style.transform = 'translateY(0)';
      }
    } else {
      shape.style.transform = 'scale(1)';
      shape.style.fill = `var(--geo-${color})`;
      if (label) {
        label.style.opacity = '0';
        label.style.transform = 'translateY(10px)';
      }
    }
  }

  handleClick(e, shape, href) {
    e.preventDefault();

    if (window.NEXUS_DEBUG) {
      console.log('[NEXUS DEBUG] Navigating to:', href);
    }

    // Morphing animation
    shape.style.animation = 'geo-morph-out 0.8s cubic-bezier(0.68, -0.55, 0.265, 1.55)';

    setTimeout(() => {
      window.location.href = href;
    }, 800);
  }

  handleParallax(e) {
    const { clientX, clientY } = e;
    const { innerWidth, innerHeight } = window;
    const centerX = innerWidth / 2;
    const centerY = innerHeight / 2;

    this.decorShapes.forEach(decor => {
      const speed = parseFloat(decor.dataset.speed || 0.05);
      const deltaX = (clientX - centerX) * speed;
      const deltaY = (clientY - centerY) * speed;

      decor.style.transform = `translate(${deltaX}px, ${deltaY}px)`;
    });
  }
}

// Initialize on DOMContentLoaded
document.addEventListener('DOMContentLoaded', () => {
  new GeometricNav();
});
```

**geometric-onboarding.js:**
```javascript
class GeometricOnboarding {
  constructor() {
    this.modal = document.getElementById('geoOnboardingModal');
    this.skipBtn = document.getElementById('skipTour');
    this.startBtn = document.getElementById('startTour');
    this.storageKey = 'nexus_onboarding_seen';
    this.init();
  }

  init() {
    // Check if user has seen onboarding
    if (this.hasSeenOnboarding()) {
      return;
    }

    // Show modal after 1 second
    setTimeout(() => {
      this.showModal();
    }, 1000);

    // Event listeners
    this.skipBtn.addEventListener('click', () => this.skipTour());
    this.startBtn.addEventListener('click', () => this.startTour());
  }

  hasSeenOnboarding() {
    return localStorage.getItem(this.storageKey) === 'true';
  }

  showModal() {
    this.modal.style.display = 'flex';
    if (window.NEXUS_DEBUG) {
      console.log('[NEXUS DEBUG] Showing onboarding modal');
    }
  }

  hideModal() {
    this.modal.style.display = 'none';
    localStorage.setItem(this.storageKey, 'true');
  }

  skipTour() {
    this.hideModal();
    if (window.NEXUS_DEBUG) {
      console.log('[NEXUS DEBUG] User skipped onboarding');
    }
  }

  startTour() {
    this.hideModal();
    // Tour interactif (Phase 2 - post-MVP)
    // Pour MVP: juste fermer modal
    if (window.NEXUS_DEBUG) {
      console.log('[NEXUS DEBUG] User started tour');
    }
  }
}

document.addEventListener('DOMContentLoaded', () => {
  new GeometricOnboarding();
});
```

**geometric-toast.js:**
```javascript
class GeometricToast {
  constructor() {
    this.toast = document.getElementById('geoQuickBuyToast');
    this.closeBtn = this.toast.querySelector('.geo-toast-close');
    this.inactivityDelay = 5000; // 5 seconds
    this.autoDismissDelay = 8000; // 8 seconds
    this.timer = null;
    this.dismissTimer = null;
    this.hasShown = false;
    this.init();
  }

  init() {
    // Start inactivity timer
    this.startTimer();

    // Reset timer on user interaction
    ['mousemove', 'click', 'keydown'].forEach(event => {
      document.addEventListener(event, () => this.resetTimer());
    });

    // Close button
    this.closeBtn.addEventListener('click', () => this.hideToast());
  }

  startTimer() {
    if (this.hasShown) return;

    this.timer = setTimeout(() => {
      this.showToast();
    }, this.inactivityDelay);
  }

  resetTimer() {
    if (this.hasShown) return;

    clearTimeout(this.timer);
    this.startTimer();
  }

  showToast() {
    this.toast.style.display = 'block';
    this.hasShown = true;

    if (window.NEXUS_DEBUG) {
      console.log('[NEXUS DEBUG] Quick buy toast shown');
    }

    // Auto-dismiss after 8 seconds
    this.dismissTimer = setTimeout(() => {
      this.hideToast();
    }, this.autoDismissDelay);
  }

  hideToast() {
    this.toast.style.display = 'none';
    clearTimeout(this.dismissTimer);

    if (window.NEXUS_DEBUG) {
      console.log('[NEXUS DEBUG] Quick buy toast hidden');
    }
  }
}

document.addEventListener('DOMContentLoaded', () => {
  new GeometricToast();
});
```

**geometric-user-menu.js:**
```javascript
class GeometricUserMenu {
  constructor() {
    this.avatar = document.getElementById('geoUserAvatar');
    this.dropdown = document.getElementById('geoUserDropdown');
    if (!this.avatar || !this.dropdown) return;
    this.isOpen = false;
    this.init();
  }

  init() {
    // Toggle dropdown on click
    this.avatar.addEventListener('click', (e) => {
      e.stopPropagation();
      this.toggle();
    });

    // Close on click outside
    document.addEventListener('click', (e) => {
      if (!this.dropdown.contains(e.target)) {
        this.close();
      }
    });

    // Keyboard support
    this.avatar.addEventListener('keydown', (e) => {
      if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault();
        this.toggle();
      }
      if (e.key === 'Escape') {
        this.close();
      }
    });
  }

  toggle() {
    this.isOpen ? this.close() : this.open();
  }

  open() {
    this.dropdown.style.display = 'block';
    this.isOpen = true;
    if (window.NEXUS_DEBUG) {
      console.log('[NEXUS DEBUG] User menu opened');
    }
  }

  close() {
    this.dropdown.style.display = 'none';
    this.isOpen = false;
  }
}

document.addEventListener('DOMContentLoaded', () => {
  new GeometricUserMenu();
});
```

**geometric-utils.js:**
```javascript
// Utility functions for geometric UI

function prefersReducedMotion() {
  return window.matchMedia('(prefers-reduced-motion: reduce)').matches;
}

function smoothScroll(target, duration = 800) {
  const targetEl = document.querySelector(target);
  if (!targetEl) return;

  const targetPosition = targetEl.offsetTop;
  const startPosition = window.pageYOffset;
  const distance = targetPosition - startPosition;
  let startTime = null;

  function animation(currentTime) {
    if (startTime === null) startTime = currentTime;
    const timeElapsed = currentTime - startTime;
    const run = ease(timeElapsed, startPosition, distance, duration);
    window.scrollTo(0, run);
    if (timeElapsed < duration) requestAnimationFrame(animation);
  }

  function ease(t, b, c, d) {
    t /= d / 2;
    if (t < 1) return c / 2 * t * t + b;
    t--;
    return -c / 2 * (t * (t - 2) - 1) + b;
  }

  requestAnimationFrame(animation);
}

function generateId(prefix = 'geo') {
  return `${prefix}-${Math.random().toString(36).substr(2, 9)}`;
}

function debounce(func, wait) {
  let timeout;
  return function executedFunction(...args) {
    const later = () => {
      clearTimeout(timeout);
      func(...args);
    };
    clearTimeout(timeout);
    timeout = setTimeout(later, wait);
  };
}

// Export for use in other modules
window.GeometricUtils = {
  prefersReducedMotion,
  smoothScroll,
  generateId,
  debounce
};
```

**Backend Routes (server/src/main.rs ou routes/frontend.rs):**
```rust
use actix_web::{web, HttpResponse, Responder};
use actix_session::Session;
use tera::{Context, Tera};

pub async fn index(
    tera: web::Data<Tera>,
    session: Session,
) -> impl Responder {
    let mut ctx = Context::new();

    // Check if user is logged in
    let logged_in = session.get::<bool>("logged_in").unwrap_or(Some(false)).unwrap_or(false);
    ctx.insert("logged_in", &logged_in);

    if logged_in {
        if let Ok(Some(username)) = session.get::<String>("username") {
            ctx.insert("username", &username);
        }
    }

    // CSRF token (si utilisé)
    // ctx.insert("csrf_token", &generate_csrf_token());

    // NEXUS_DEBUG flag from Phase 6.2
    #[cfg(debug_assertions)]
    ctx.insert("nexus_debug", &true);
    #[cfg(not(debug_assertions))]
    ctx.insert("nexus_debug", &false);

    match tera.render("geometric-home.html", &ctx) {
        Ok(html) => HttpResponse::Ok().content_type("text/html").body(html),
        Err(e) => {
            tracing::error!("Template error: {}", e);
            HttpResponse::InternalServerError().body("Template rendering failed")
        }
    }
}

pub async fn listings(
    tera: web::Data<Tera>,
    session: Session,
) -> impl Responder {
    let mut ctx = Context::new();

    // Same auth context as index
    let logged_in = session.get::<bool>("logged_in").unwrap_or(Some(false)).unwrap_or(false);
    ctx.insert("logged_in", &logged_in);

    // TODO: Fetch actual listings from DB
    // ctx.insert("listings", &listings_vec);

    match tera.render("listings/geometric-listings.html", &ctx) {
        Ok(html) => HttpResponse::Ok().content_type("text/html").body(html),
        Err(e) => {
            tracing::error!("Template error: {}", e);
            HttpResponse::InternalServerError().body("Template rendering failed")
        }
    }
}
```

**Checklist Jour 4:**
- [ ] Créer tous les fichiers JS
- [ ] Tester navigation interactive (hover + click)
- [ ] Tester parallax (mouvement souris)
- [ ] Tester onboarding localStorage (première visite)
- [ ] Tester toast inactivité (5s + auto-dismiss 8s)
- [ ] Tester user menu (si logged_in)
- [ ] Modifier routes backend (index + listings)
- [ ] Compiler et tester backend
- [ ] Vérifier CSRF tokens préservés
- [ ] Vérifier auth session préservée

---

### **Jour 5-6: Listings Hybrid Page**

**Objectifs:**
- ✅ Créer page Listings hybride (branding géométrique + grid classique)
- ✅ Implémenter mini-nav back button
- ✅ Styliser cards avec accents géométriques
- ✅ Intégrer avec données réelles du backend

**Livrables:**
1. `geometric-listings.css` (complet)
2. `listings/geometric-listings.html` (complet)
3. Intégration backend avec DB

**Code Clé:**

**geometric-listings.css:**
```css
/* Geometric Listings - Hybrid Page */

.geo-listings-page {
  min-height: 100vh;
  background-color: var(--geo-beige);
}

/* Mini-nav back button */
.geo-mini-nav {
  position: fixed;
  top: 2rem;
  left: 2rem;
  z-index: var(--z-nav);
}

.geo-back-btn {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  background: white;
  padding: 0.75rem 1.5rem;
  border-radius: var(--geo-radius-full);
  border: 2px solid var(--geo-pink);
  color: var(--geo-pink);
  font-weight: 600;
  text-decoration: none;
  transition: all 0.3s;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.geo-back-btn:hover {
  background: var(--geo-pink);
  color: white;
  transform: translateX(-5px);
}

/* Header */
.geo-listings-header {
  padding: 6rem 2rem 3rem;
  text-align: center;
  position: relative;
}

.geo-listings-header h1 {
  font-size: 3rem;
  background: linear-gradient(135deg, var(--geo-pink), var(--geo-blue));
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  margin-bottom: 1rem;
}

.geo-listings-header p {
  font-size: 1.2rem;
  color: #666;
}

/* Decorative shapes in header */
.geo-listings-decor {
  position: absolute;
  top: 2rem;
  right: 5%;
  width: 100px;
  height: 100px;
  background: var(--geo-turquoise);
  border-radius: 50%;
  opacity: 0.3;
  animation: geo-float 6s ease-in-out infinite;
}

/* Listings grid */
.geo-listings-container {
  max-width: 1400px;
  margin: 0 auto;
  padding: 0 2rem 4rem;
}

.geo-listings-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 2rem;
  margin-top: 2rem;
}

/* Product card with geometric accents */
.geo-product-card {
  background: white;
  border-radius: var(--geo-radius-lg);
  overflow: hidden;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  transition: transform 0.3s, box-shadow 0.3s;
  position: relative;
}

.geo-product-card:hover {
  transform: translateY(-8px);
  box-shadow: 0 12px 24px rgba(0, 0, 0, 0.15);
}

/* Geometric accent corner */
.geo-product-card::before {
  content: '';
  position: absolute;
  top: 0;
  right: 0;
  width: 0;
  height: 0;
  border-style: solid;
  border-width: 0 60px 60px 0;
  border-color: transparent var(--geo-pink) transparent transparent;
  opacity: 0.8;
  z-index: 1;
}

.geo-product-image {
  width: 100%;
  height: 200px;
  object-fit: cover;
}

.geo-product-content {
  padding: 1.5rem;
}

.geo-product-title {
  font-size: 1.25rem;
  font-weight: 600;
  margin-bottom: 0.5rem;
  color: #333;
}

.geo-product-price {
  font-size: 1.5rem;
  font-weight: 700;
  color: var(--geo-pink);
  margin-bottom: 1rem;
}

.geo-product-description {
  font-size: 0.95rem;
  color: #666;
  margin-bottom: 1rem;
  line-height: 1.5;
}

.geo-product-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 1rem;
}

.geo-view-btn {
  flex: 1;
  padding: 0.75rem;
  background: linear-gradient(135deg, var(--geo-pink), var(--geo-blue));
  color: white;
  border: none;
  border-radius: var(--geo-radius-md);
  font-weight: 600;
  cursor: pointer;
  transition: transform 0.2s;
}

.geo-view-btn:hover {
  transform: scale(1.05);
}

/* Footer */
.geo-listings-footer {
  background: linear-gradient(135deg, var(--geo-pink) 0%, var(--geo-blue) 100%);
  padding: 3rem 2rem;
  text-align: center;
  color: white;
}

.geo-listings-footer a {
  color: white;
  text-decoration: none;
  margin: 0 1rem;
  font-weight: 600;
}

/* Responsive */
@media (max-width: 768px) {
  .geo-listings-grid {
    grid-template-columns: 1fr;
  }

  .geo-listings-header h1 {
    font-size: 2rem;
  }

  .geo-mini-nav {
    top: 1rem;
    left: 1rem;
  }
}
```

**listings/geometric-listings.html:**
```html
{% extends "base-geometric.html" %}

{% block extra_css %}
<link rel="stylesheet" href="/static/css/geometric-listings.css">
{% endblock %}

{% block content %}
<div class="geo-listings-page">

  <!-- Mini-nav back button -->
  <div class="geo-mini-nav">
    <a href="/" class="geo-back-btn">
      <span>←</span>
      <span>Accueil</span>
    </a>
  </div>

  <!-- Header avec décoration géométrique -->
  <header class="geo-listings-header">
    <div class="geo-listings-decor"></div>
    <h1>Marketplace Listings</h1>
    <p>Découvrez nos produits en toute confidentialité</p>
  </header>

  <!-- Container listings -->
  <div class="geo-listings-container">

    <!-- TODO: Filtres/Search (Phase 2) -->

    <!-- Grid de produits -->
    <div class="geo-listings-grid">

      {% if listings %}
        {% for listing in listings %}
        <article class="geo-product-card">
          {% if listing.image_url %}
          <img src="{{ listing.image_url }}" alt="{{ listing.title }}" class="geo-product-image">
          {% else %}
          <img src="/static/images/placeholder.png" alt="No image" class="geo-product-image">
          {% endif %}

          <div class="geo-product-content">
            <h3 class="geo-product-title">{{ listing.title }}</h3>
            <div class="geo-product-price">{{ listing.price_xmr }} XMR</div>
            <p class="geo-product-description">
              {{ listing.description | truncate(length=120) }}
            </p>

            <div class="geo-product-footer">
              <a href="/listings/{{ listing.id }}" class="geo-view-btn">
                Voir Détails
              </a>
            </div>
          </div>
        </article>
        {% endfor %}
      {% else %}
        <div style="grid-column: 1 / -1; text-align: center; padding: 4rem;">
          <p style="font-size: 1.5rem; color: #999;">Aucun listing disponible pour le moment.</p>
        </div>
      {% endif %}

    </div>
  </div>

  <!-- Footer géométrique -->
  <footer class="geo-listings-footer">
    <p>&copy; 2025 NEXUS Geometric Marketplace</p>
    <nav>
      <a href="/about">À Propos</a>
      <a href="/terms">Conditions</a>
      <a href="/privacy">Confidentialité</a>
    </nav>
  </footer>

</div>
{% endblock %}
```

**Checklist Jour 5-6:**
- [ ] Créer geometric-listings.css
- [ ] Créer listings/geometric-listings.html
- [ ] Modifier backend pour fetch listings DB
- [ ] Tester affichage avec données réelles
- [ ] Tester back button navigation
- [ ] Tester hover effects sur cards
- [ ] Vérifier responsive (mobile, tablet, desktop)
- [ ] Tester performance avec 50+ listings
- [ ] Vérifier accessibilité (keyboard navigation)

---

### **Jour 7: Tests & Polish**

**Objectifs:**
- ✅ Tests complets de tous les flows
- ✅ Corrections de bugs
- ✅ Polish animations
- ✅ Validation accessibilité
- ✅ Tests Tor Browser
- ✅ Documentation finale

**Checklist Tests:**

**Tests Fonctionnels:**
- [ ] Homepage charge correctement
- [ ] 6 formes navigation visibles et cliquables
- [ ] Hover effects fonctionnent (scale + darken + label)
- [ ] Click navigation redirige vers bonnes pages
- [ ] Parallax background fonctionne (sans prefers-reduced-motion)
- [ ] Modal onboarding s'affiche première visite
- [ ] localStorage persiste onboarding
- [ ] Toast quick buy apparaît après 5s inactivité
- [ ] Toast se ferme avec bouton close
- [ ] Toast s'auto-dismiss après 8s
- [ ] User menu toggle fonctionne (si logged_in)
- [ ] User menu dropdown affiche bonnes options
- [ ] Back button listings → homepage fonctionne
- [ ] Listings grid affiche produits
- [ ] Product cards hover effects fonctionnent
- [ ] Liens "Voir Détails" fonctionnent

**Tests Responsive:**
- [ ] Desktop (1920x1080): Toutes formes visibles, parallax OK
- [ ] Tablet (768px): Layout adapté, formes réorganisées
- [ ] Mobile (375px): Flex column, labels toujours visibles

**Tests Accessibilité:**
- [ ] Tabulation fonctionne sur toutes formes nav
- [ ] Enter/Space déclenche navigation
- [ ] Escape ferme modal et dropdown
- [ ] Labels ARIA présents sur éléments interactifs
- [ ] prefers-reduced-motion désactive animations
- [ ] Contraste couleurs WCAG AA minimum

**Tests Tor Browser:**
- [ ] Page charge via .onion (si configuré)
- [ ] Toutes ressources locales (pas de CDN)
- [ ] JS fonctionne sans erreurs console
- [ ] Animations fluides (ou désactivées si reduced-motion)
- [ ] localStorage fonctionne
- [ ] No privacy leaks (check DevTools Network)

**Tests Performance:**
- [ ] First Contentful Paint < 2s
- [ ] Largest Contentful Paint < 3s
- [ ] Total Blocking Time < 300ms
- [ ] Cumulative Layout Shift < 0.1
- [ ] SVG inline < 100KB

**Tests Backend:**
- [ ] Route `/` rend geometric-home.html
- [ ] Route `/listings` rend geometric-listings.html
- [ ] Auth session préservée
- [ ] CSRF tokens présents (si utilisés)
- [ ] NEXUS_DEBUG flag fonctionne
- [ ] Tera template errors loggés

**Polish Final:**
- [ ] Ajuster timings animations si besoin
- [ ] Vérifier typos dans labels
- [ ] Optimiser SVG (enlever paths inutiles)
- [ ] Minifier CSS/JS (si production)
- [ ] Ajouter comments dans code
- [ ] Documenter patterns réutilisables

**Documentation:**
- [ ] README avec captures écran
- [ ] Guide utilisateur pour navigation géométrique
- [ ] Vidéo démo (optionnel)
- [ ] Notes pour Phase 2 (amélioration post-MVP)

---

## 🎯 VALIDATION MVP

### Critères de Succès

**Must-Have (Bloquants):**
1. ✅ Homepage géométrique fonctionnelle avec 6 formes nav
2. ✅ Navigation fonctionne (click → redirect)
3. ✅ Listings page hybride affiche produits
4. ✅ Responsive mobile/tablet/desktop
5. ✅ Accessibilité basique (keyboard + ARIA)
6. ✅ Compatible Tor Browser
7. ✅ Aucune erreur console critique

**Nice-to-Have (Non-bloquants):**
1. ⭕ Onboarding modal avec tour interactif complet
2. ⭕ Parallax ultra-smooth (60fps)
3. ⭕ Animations complexes de morphing
4. ⭕ Search/Filtres listings (Phase 2)
5. ⭕ Product detail page géométrique (Phase 2)

### Décision Post-MVP

**Si Validation OK (≥7/7 Must-Have):**
- ✅ Continuer Phase 2 (3 semaines)
- ✅ Implémenter pages restantes (product detail, checkout, orders, settings)
- ✅ Améliorer onboarding tour
- ✅ Ajouter filtres/search listings
- ✅ Optimiser animations

**Si Validation Partielle (5-6/7):**
- ⚠️ Analyser points bloquants
- ⚠️ Ajuster approche (simplifier animations, changer layout)
- ⚠️ Itération MVP v2 (1 semaine)

**Si Validation Échec (<5/7):**
- ❌ Abandonner approche géométrique pure
- ❌ Revenir à NEXUS classique avec accents géométriques
- ❌ Leçons apprises documentées

---

## 🔧 TECHNICAL ARCHITECTURE

### Stack Technique

**Frontend:**
- HTML5 + SVG inline
- CSS3 (variables, animations, grid, flexbox)
- Vanilla JavaScript (ES6+)
- Tera templates (Rust templating engine)
- HTMX (préservé, local)

**Backend:**
- Rust (Actix-web framework)
- Tera (templating)
- Diesel ORM (PostgreSQL/SQLite)
- Actix-session (auth sessions)

**Infrastructure:**
- Tor hidden service (.onion)
- Monero RPC (localhost:18082)
- SQLite/PostgreSQL database

### Contraintes Techniques

**Tor Browser:**
- Toutes ressources doivent être locales (pas de CDN)
- JS doit être compatible avec NoScript si désactivé
- Animations doivent respecter prefers-reduced-motion
- Pas de tracking/analytics

**Performance:**
- SVG inline < 100KB
- First Paint < 2s sur Tor
- Animations 60fps (ou désactivées)
- localStorage pour persistence

**Accessibilité:**
- WCAG 2.1 Level AA minimum
- Keyboard navigation complète
- ARIA labels sur tous éléments interactifs
- Contrast ratio ≥4.5:1

**Security:**
- CSRF protection sur toutes formes
- Session cookies HttpOnly + Secure
- No inline JS (CSP-compatible)
- Input validation côté serveur

---

## 📊 RISK ASSESSMENT

### Risques Identifiés

| Risque | Probabilité | Impact | Mitigation |
|--------|-------------|--------|------------|
| **Learning curve trop steep** | Haute | Moyen | Onboarding modal obligatoire + toast quick buy |
| **Performance SVG/animations** | Moyenne | Moyen | prefers-reduced-motion + simplification formes |
| **Accessibilité non-conforme** | Faible | Haut | Tests WCAG + keyboard nav + ARIA |
| **Vendor pushback** | Haute | Faible | Dashboard vendor mode classique |
| **Tor Browser incompatibilité** | Faible | Haut | Tests dans Tor + no CDN |
| **Responsive complexité** | Moyenne | Moyen | Layouts alternatifs mobile/tablet |

### Stratégie Mitigation Globale

1. **MVP court (7 jours)** pour validation rapide
2. **Hybrid approach** (géométrique + classique) pour balance
3. **User testing** après MVP avant Phase 2
4. **Rollback plan** si validation échoue
5. **Documentation** exhaustive pour maintenance

---

## 🚀 PHASE 2 (Post-MVP - 3 Semaines)

### Scope Phase 2 (Si MVP validé)

**Semaine 1:**
- Product detail page géométrique
- Geometric checkout flow
- Vendor dashboard (mode classique avec branding)

**Semaine 2:**
- Orders page (user + vendor)
- Settings page
- Profile page
- Search/Filtres listings

**Semaine 3:**
- Escrow page géométrique
- Interactive tour complet onboarding
- Animations avancées (morphing paths)
- Polish final + tests E2E

**Total Effort Phase 1+2:** 4 semaines

---

## 📝 NOTES IMPORTANTES

### Décisions de Design

1. **Pourquoi SVG inline et pas CSS shapes?**
   - Plus de flexibilité pour formes complexes
   - Meilleure compatibilité cross-browser
   - Clickable areas plus précises

2. **Pourquoi localStorage pour onboarding?**
   - Pas besoin backend pour persister
   - Fonctionne sans auth
   - Privacy-friendly (local seulement)

3. **Pourquoi approche hybride listings?**
   - Balance entre innovation et usabilité
   - Grid classique = efficacité prouvée
   - Branding géométrique = identité visuelle

4. **Pourquoi toast quick buy et pas bouton permanent?**
   - Minimise clutter visuel
   - Cible users pressés seulement
   - Non-intrusif

### Leçons Apprises (Pré-Implémentation)

1. **Audace ≠ Folie** si compromis intelligents
2. **MVP = validation rapide** avant engagement complet
3. **Hybrid > Pure** pour équilibrer innovation et usabilité
4. **Accessibilité non-négociable** même avec design audacieux
5. **User testing critique** pour UX non-standard

---

## 🎨 COLOR PALETTE REFERENCE

```css
/* Couleurs principales (exactes de l'image) */
--geo-pink: #E91E8C;          /* Rose vif */
--geo-pink-dark: #C01A75;     /* Rose foncé (hover) */
--geo-yellow: #D4A44A;        /* Jaune/Or */
--geo-yellow-dark: #B8873D;   /* Jaune foncé (hover) */
--geo-turquoise: #4CA0A0;     /* Turquoise */
--geo-turquoise-dark: #3E8282;/* Turquoise foncé (hover) */
--geo-blue: #3E6DB5;          /* Bleu */
--geo-blue-dark: #325A96;     /* Bleu foncé (hover) */
--geo-beige: #EFE4D8;         /* Beige (background) */
```

**Usage Recommandé:**
- **Pink:** CTAs principaux, accents importants
- **Yellow:** Éléments secondaires, highlights
- **Turquoise:** Actions tertiaires, decorations
- **Blue:** Navigation principale, headers
- **Beige:** Background, surfaces calmes

---

## 📞 CONTACTS & RESOURCES

**Équipe:**
- Dev Frontend: (TBD)
- Dev Backend: (TBD)
- Designer: (TBD)

**Ressources:**
- Image référence: [Lien vers image géométrique]
- Moodboard: (TBD)
- Figma prototype: (TBD - Phase 2)

**Documentation:**
- CLAUDE.md (règles projet)
- base-nexus.html (template actuel NEXUS)
- partials/nexus-macros.html (composants NEXUS)

---

## ✅ CHECKLIST MASTER

### Pré-Implémentation
- [x] Plan MVP documenté
- [ ] Validation user sur maquettes/wireframes
- [ ] Environnement dev configuré
- [ ] Git branch `feature/geometric-ui-mvp` créée

### Implémentation (Jours 1-7)
- [ ] Jour 1-2: Design system CSS complet
- [ ] Jour 3: Components CSS (modal, toast, menu)
- [ ] Jour 4: JavaScript + Backend integration
- [ ] Jour 5-6: Listings hybrid page
- [ ] Jour 7: Tests & polish

### Post-MVP
- [ ] User testing (5+ utilisateurs)
- [ ] Feedback documenté
- [ ] Décision GO/NOGO Phase 2
- [ ] Plan Phase 2 ajusté si besoin

---

**FIN DU PLAN MVP - Version 1.0**

*Ce document est un guide vivant et sera mis à jour pendant l'implémentation.*
