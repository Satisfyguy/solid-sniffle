# NEXUS Typography System

## Système de Polices (Inspiré de wodniack.dev)

### 🎯 Hiérarchie Typographique

#### 1. **Titre Principal / Hero**
- **Police cible**: PP Monument Extended (UltraBold/Black)
- **Alternative gratuite**: Space Grotesk Black / Archivo Black
- **Usage**: Titres hero, "NEXUS", grands titres de section
- **Caractéristiques**: Très large, impact visuel immédiat, ultra-bold

**Variable CSS**: `--nexus-font-hero`

#### 2. **Texte Technique / Monospace**
- **Police cible**: PP Fraktion Mono
- **Alternative gratuite**: JetBrains Mono / IBM Plex Mono
- **Usage**: Informations techniques, métadonnées, données, code
- **Caractéristiques**: Monospace, style "terminal", lisibilité technique

**Variable CSS**: `--nexus-font-mono`

#### 3. **Navigation / Identité**
- **Police cible**: PP Fraktion Grotesk
- **Alternative gratuite**: Inter / Space Grotesk
- **Usage**: Logo, navigation, UI générale
- **Caractéristiques**: Sans-serif épurée, claire, moderne

**Variable CSS**: `--nexus-font-sans`

---

## 📦 Implémentation Actuelle

### Polices Utilisées (Alternatives Gratuites)

```css
/* Hero / Titres principaux */
--nexus-font-hero: 'Space Grotesk', 'Archivo Black', sans-serif;

/* Technique / Monospace */
--nexus-font-mono: 'JetBrains Mono', 'IBM Plex Mono', 'Courier New', monospace;

/* Navigation / UI */
--nexus-font-sans: 'Inter', 'Space Grotesk', -apple-system, BlinkMacSystemFont, sans-serif;
```

---

## 🎨 Mapping des Usages

| Élément | Police | Variable CSS | Poids |
|---------|--------|--------------|-------|
| "NEXUS" Hero | Hero | `--nexus-font-hero` | 900 (Black) |
| Section Titles | Hero | `--nexus-font-hero` | 900 |
| Logo "NX" | Sans | `--nexus-font-sans` | 700 (Bold) |
| Navigation Menu | Sans | `--nexus-font-sans` | 600 (SemiBold) |
| Typewriter Header | Mono | `--nexus-font-mono` | 700 (Bold) |
| Category Titles | Mono | `--nexus-font-mono` | 900 (Black) |
| Metadata / Stats | Mono | `--nexus-font-mono` | 400-700 |
| Body Text | Sans | `--nexus-font-sans` | 400 (Regular) |
| Boutons | Sans | `--nexus-font-sans` | 600-700 |

---

## 📥 Installation des Polices Commerciales (Optionnel)

Si tu achètes les polices PP de Pangram Pangram Foundry:

### 1. Télécharger les Polices
- [PP Monument Extended](https://pangrampangram.com/products/monument-extended) (UltraBold/Black)
- [PP Fraktion Mono](https://pangrampangram.com/products/fraktion-mono)
- [PP Fraktion Grotesk](https://pangrampangram.com/products/fraktion)

### 2. Placer les Fichiers
```
static/fonts/
├── PPMonumentExtended-Black.woff2
├── PPMonumentExtended-UltraBold.woff2
├── PPFraktionMono-Regular.woff2
├── PPFraktionMono-Bold.woff2
├── PPFraktionGrotesk-Regular.woff2
├── PPFraktionGrotesk-SemiBold.woff2
└── PPFraktionGrotesk-Bold.woff2
```

### 3. Mettre à Jour `nexus-variables.css`
```css
/* Charger les polices */
@font-face {
  font-family: 'PP Monument Extended';
  src: url('/static/fonts/PPMonumentExtended-Black.woff2') format('woff2');
  font-weight: 900;
  font-style: normal;
  font-display: swap;
}

@font-face {
  font-family: 'PP Fraktion Mono';
  src: url('/static/fonts/PPFraktionMono-Bold.woff2') format('woff2');
  font-weight: 700;
  font-style: normal;
  font-display: swap;
}

@font-face {
  font-family: 'PP Fraktion Grotesk';
  src: url('/static/fonts/PPFraktionGrotesk-Bold.woff2') format('woff2');
  font-weight: 700;
  font-style: normal;
  font-display: swap;
}

/* Mettre à jour les variables */
:root {
  --nexus-font-hero: 'PP Monument Extended', 'Space Grotesk', sans-serif;
  --nexus-font-mono: 'PP Fraktion Mono', 'JetBrains Mono', monospace;
  --nexus-font-sans: 'PP Fraktion Grotesk', 'Inter', sans-serif;
}
```

---

## 🆓 Installer les Alternatives Gratuites

### Option 1: Google Fonts (CDN)
```html
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?family=Space+Grotesk:wght@700;900&family=JetBrains+Mono:wght@400;700&family=Inter:wght@400;600;700&display=swap" rel="stylesheet">
```

**⚠️ OPSEC**: Ne pas utiliser Google Fonts CDN pour Tor! Télécharger les fonts localement.

### Option 2: Self-Hosted (Recommandé pour Tor)
1. Télécharger depuis [Google Fonts](https://fonts.google.com/)
2. Placer dans `static/fonts/`
3. Utiliser `@font-face` dans CSS

---

## 🔧 Règles d'Application

### Hero / Titres Principaux
```css
.nexus-hero-title {
  font-family: var(--nexus-font-hero);
  font-weight: 900;
  letter-spacing: -0.02em;
  text-transform: uppercase;
}
```

### Navigation / UI
```css
.nexus-nav {
  font-family: var(--nexus-font-sans);
  font-weight: 600;
  letter-spacing: 0.05em;
}
```

### Technique / Mono
```css
.nexus-technical {
  font-family: var(--nexus-font-mono);
  font-weight: 700;
  letter-spacing: 0.02em;
}
```

---

## 📊 Comparaison Polices

| Aspect | PP (Commercial) | Alternatives (Gratuit) |
|--------|----------------|------------------------|
| **Coût** | ~$300-500 | Gratuit |
| **Qualité** | Premium | Excellent |
| **Licensing** | Payant | Open Source |
| **Similarité** | Original | 85-90% similaire |
| **OPSEC** | Self-hosted OK | Self-hosted OK |

---

## ✅ Checklist Migration

- [ ] Télécharger polices gratuites (Space Grotesk, JetBrains Mono, Inter)
- [ ] Placer dans `static/fonts/`
- [ ] Mettre à jour `nexus-variables.css` avec `@font-face`
- [ ] Remplacer toutes les `font-family: 'Courier New'`
- [ ] Tester tous les composants (header, hero, categories, etc.)
- [ ] Vérifier light mode
- [ ] Vérifier responsive
- [ ] (Optionnel) Acheter et intégrer polices PP commerciales

---

**Note**: Les polices actuelles (Courier New monospace) seront remplacées progressivement par ce système typographique.
