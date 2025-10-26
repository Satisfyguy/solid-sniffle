# 📦 NEXUS Design System - Inventaire Complet

**Date:** 2025-10-26
**Status:** Migration AMAZAWN → NEXUS en cours

---

## 🎨 CSS Variables & Design Tokens

### Couleurs
- **Primary (Pink/Red):** `--nexus-primary: hsl(349, 100%, 55%)` (#ff1a5c)
- **Secondary (Purple):** `--nexus-secondary: hsl(280, 60%, 50%)` (#9933cc)
- **Accent (Cyan):** `--nexus-accent: hsl(180, 100%, 50%)` (#00ffff)
- **Background:** `--nexus-bg: hsl(0, 0%, 5%)` (#0d0d0d)
- **Foreground:** `--nexus-fg: hsl(0, 0%, 98%)` (#fafafa)

### Effets Visuels
- **Glassmorphism:**
  - `--nexus-glass-bg: rgba(255, 255, 255, 0.05)`
  - `--nexus-glass-border: rgba(255, 255, 255, 0.1)`
  - `--nexus-glass-backdrop: blur(10px)`

- **Glows:**
  - `--nexus-glow-primary: 0 0 20px rgba(255, 26, 92, 0.5)`
  - `--nexus-glow-secondary: 0 0 20px rgba(153, 51, 204, 0.5)`
  - `--nexus-glow-accent: 0 0 20px rgba(0, 255, 255, 0.5)`

- **Shadows:**
  - `--nexus-shadow-sm` à `--nexus-shadow-2xl`

### Typographie
- **Tailles:** `--nexus-text-xs` (12px) à `--nexus-text-huge` (192px)
- **Poids:** `--nexus-weight-normal` (400) à `--nexus-weight-black` (900)
- **Fonts:**
  - Mono: `'Courier New', 'Courier', monospace`
  - Sans: `-apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto'`

### Espacement
- **Scale:** `--nexus-space-1` (4px) à `--nexus-space-24` (96px)

### Animations
- **Durées:** 150ms (fast) à 500ms (slower)
- **Easings:** ease-in, ease-out, ease-in-out

---

## 🧩 Composants Disponibles

### ⚛️ Atoms (Éléments de base)

| Composant | Fichier | Classes CSS | Status |
|-----------|---------|-------------|--------|
| **Badge** | `atoms/badge.html` | `.nexus-badge`, `.nexus-badge-{variant}` | ✅ Macro créée |
| **Button** | `atoms/button.html` | `.nexus-btn`, `.nexus-btn-{variant}`, `.nexus-btn-{size}` | ✅ Macro créée |
| **Checkbox** | `atoms/checkbox.html` | `.nexus-checkbox` | ⚠️ À intégrer |
| **Input** | `atoms/input.html` | `.nexus-input`, `.nexus-input-error` | ✅ Macro créée |
| **Label** | `atoms/label.html` | `.nexus-label` | ⚠️ À intégrer |
| **Radio** | `atoms/radio.html` | `.nexus-radio` | ⚠️ À intégrer |
| **Select** | `atoms/select.html` | `.nexus-select` | ✅ Macro créée |
| **Separator** | `atoms/separator.html` | `.nexus-separator` | ✅ Macro créée |
| **Switch** | `atoms/switch.html` | `.nexus-switch` | ⚠️ À intégrer |
| **Textarea** | `atoms/textarea.html` | `.nexus-textarea` | ✅ Macro créée |

### 🔬 Molecules (Composants composites)

| Composant | Fichier | Classes CSS | Status |
|-----------|---------|-------------|--------|
| **Accordion** | `molecules/accordion.html` | `.nexus-accordion` | ⚠️ À intégrer |
| **Alert** | `molecules/alert.html` | `.nexus-alert`, `.nexus-alert-{variant}` | ✅ Macro créée |
| **Avatar** | `molecules/avatar.html` | `.nexus-avatar` | ⚠️ À intégrer |
| **Breadcrumb** | `molecules/breadcrumb.html` | `.nexus-breadcrumb` | ✅ Utilisé (HTML inline) |
| **Card** | `molecules/card.html` | `.nexus-card`, `.nexus-card-elevated`, `.nexus-card-glass` | ✅ Macro créée + Utilisé |
| **Category Card** | `molecules/category-card.html` | `.nexus-category-card` | ⚠️ À intégrer |
| **Dialog** | `molecules/dialog.html` | `.nexus-dialog` | ⚠️ À intégrer |
| **Dropdown Menu** | `molecules/dropdown-menu.html` | `.nexus-dropdown` | ⚠️ À intégrer |
| **Popover** | `molecules/popover.html` | `.nexus-popover` | ⚠️ À intégrer |
| **Product Card** | `molecules/product-card.html` | `.nexus-product-card` | ✅ Utilisé (homepage) |
| **Progress** | `molecules/progress.html` | `.nexus-progress` | ⚠️ À intégrer |
| **Skeleton** | `molecules/skeleton.html` | `.nexus-skeleton` | ⚠️ À intégrer |
| **Tabs** | `molecules/tabs.html` | `.nexus-tabs` | ⚠️ À intégrer |
| **Toast** | `molecules/toast.html` | `.nexus-toast` | ⚠️ À intégrer |
| **Tooltip** | `molecules/tooltip.html` | `.nexus-tooltip` | ⚠️ À intégrer |

### 🏗️ Organisms (Sections complexes)

| Composant | Fichier | Classes CSS | Status |
|-----------|---------|-------------|--------|
| **Escrow Visualizer** | `organisms/escrow-visualizer.html` | `.nexus-escrow-visualizer` | ⚠️ À intégrer |
| **Footer** | `organisms/footer.html` | `.nexus-footer` | ✅ Inclus dans base-nexus.html |
| **Hero** | `organisms/hero.html` | `.nexus-hero`, `.nexus-hero-gradient`, `.nexus-hero-bg-orb` | ✅ Utilisé (homepage) |
| **Nav** | `organisms/nav.html` | `.nexus-nav`, `.nexus-nav-glass` | ✅ Inclus dans base-nexus.html |
| **Notification Center** | `organisms/notification-center.html` | `.nexus-notification-center` | ⚠️ À intégrer |
| **Order Timeline** | `organisms/order-timeline.html` | `.nexus-order-timeline` | ⚠️ À intégrer |
| **Search Bar** | `organisms/search-bar.html` | `.nexus-search-bar` | ⚠️ À intégrer |
| **Stats Banner** | `organisms/stats-banner.html` | `.nexus-stats-banner` | ⚠️ À intégrer |

---

## ✨ Effets Visuels Spéciaux

### Glassmorphism
```css
.nexus-card-glass {
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(20px) saturate(180%);
  border: 1px solid rgba(255, 255, 255, 0.1);
  box-shadow: 0 8px 32px 0 rgba(0, 0, 0, 0.37);
}
```

### Orbes Animés (Hero Background)
```html
<div class="nexus-hero-bg">
  <div class="nexus-hero-bg-orb nexus-hero-bg-orb-1"></div>
  <div class="nexus-hero-bg-orb nexus-hero-bg-orb-2"></div>
  <div class="nexus-hero-bg-orb nexus-hero-hero-bg-orb-3"></div>
</div>
```

### Hover Effects
- `.nexus-card-hover-lift` - Élévation au survol
- `.nexus-card-hover-shadow` - Ombre intensifiée
- `.nexus-card-hover-glow` - Glow effect

### Animations Keyframes
- `@keyframes jump` - Lettres animées (hero)
- `@keyframes wave` - Mouvement de vague
- `@keyframes float` - Formes géométriques flottantes
- `@keyframes pulse-slow` - Pulsation subtile
- `@keyframes accordion-down/up` - Accordéons
- `@keyframes fadeIn/Out` - Apparitions/disparitions
- `@keyframes slideIn/Out` - Glissements
- `@keyframes spin` - Rotation

---

## 📑 Pages Migrées vers NEXUS

| Page | Template | Status | Composants Utilisés |
|------|----------|--------|---------------------|
| **Homepage** | `listings/index.html` | ✅ Complète | Hero, Cards, Alerts, Buttons, Badges |
| **Listing Detail** | `listings/show.html` | ✅ Complète | Breadcrumb, Cards, Badges, Buttons |
| **Order Detail** | `orders/show.html` | ✅ Complète | Cards, Badges, Buttons, Timeline |
| **Orders List** | `orders/index.html` | ✅ Complète | Cards, Badges |
| **Login** | `auth/login.html` | ⚠️ AMAZAWN | À migrer vers Nexus |
| **Register** | `auth/register.html` | ⚠️ AMAZAWN | À migrer vers Nexus |
| **Settings** | `settings/index.html` | ⚠️ AMAZAWN | À migrer vers Nexus |
| **Wallet Settings** | `settings/wallet.html` | ⚠️ AMAZAWN | À migrer vers Nexus |
| **Create Listing** | `listings/create.html` | ⚠️ AMAZAWN | À migrer vers Nexus |
| **Edit Listing** | `listings/edit.html` | ⚠️ AMAZAWN | À migrer vers Nexus |

---

## 🎯 Prochaines Étapes

### Priorité 1 - Composants Critiques
- [ ] Intégrer **Search Bar** (organism) sur homepage
- [ ] Intégrer **Category Cards** (molecule) sur homepage
- [ ] Intégrer **Escrow Visualizer** (organism) sur order pages
- [ ] Intégrer **Order Timeline** (organism) sur order detail

### Priorité 2 - Pages Auth & Settings
- [ ] Migrer `/auth/login` vers Nexus
- [ ] Migrer `/auth/register` vers Nexus
- [ ] Migrer `/settings/*` vers Nexus

### Priorité 3 - Formulaires
- [ ] Migrer `/listings/create` vers Nexus (form inputs)
- [ ] Migrer `/listings/edit` vers Nexus (form inputs)
- [ ] Ajouter validation visuelle Nexus

### Priorité 4 - Interactions Avancées
- [ ] Intégrer **Tabs** component
- [ ] Intégrer **Dialog/Modal** component
- [ ] Intégrer **Toast** notifications
- [ ] Intégrer **Dropdown Menu** component

---

## 💡 Notes d'Implémentation

### Macros vs Includes
- **Macros** (dans `nexus-macros.html`) : Pour composants paramétrables
- **HTML Direct** : Pour layouts complexes (hero, grid)
- **Includes** : Pour composants statiques (nav, footer)

### Limitations Tera
- ❌ Pas de `{% include "file" with param=value %}`
- ❌ Pas de dictionnaires inline `{"key": "value"}`
- ❌ Pas d'opérateurs ternaires `a ? b : c`
- ❌ Pas de slicing `array[:n]`
- ✅ Utiliser des macros avec paramètres
- ✅ Utiliser `if/elif/else`
- ✅ Utiliser `truncate` filter

### Performance
- CSS déjà minifié : `nexus-bundle.min.css` disponible
- Variables CSS (pas de JavaScript pour les couleurs)
- Animations CSS pures (pas de jQuery)
- HTMX pour interactions (pas de SPA overhead)

---

**Légende:**
- ✅ Implémenté et fonctionnel
- ⚠️ Disponible mais non utilisé
- ❌ Manquant ou à créer
