# PHASE 5: INTEGRATION & ADVANCED COMPONENTS

**Date de début:** 2025-10-26 19:30 UTC
**Statut:** 🟢 EN COURS
**Durée estimée:** 2-3 heures

---

## 📋 OBJECTIFS

Phase 5 se concentre sur l'intégration des fonctionnalités avancées avec le design Nexus et l'optimisation pour production.

### Objectifs Principaux

1. ✅ Vérifier que toutes les pages Nexus fonctionnent correctement
2. 🔲 Intégrer les WebSocket notifications avec les composants Nexus
3. 🔲 Optimiser les bundles CSS/JS pour Tor
4. 🔲 Tester le flux complet de commande avec le nouveau design
5. 🔲 Corriger les bugs d'intégration

---

## 🎯 TÂCHES DÉTAILLÉES

### 5.1 Test des Pages Migrées (30 min)

**Objectif:** S'assurer que toutes les 8 pages migrées s'affichent correctement

#### Pages à Tester:

**Listings (4 pages):**
- [ ] `/` - Homepage
  - Hero avec orbes animés
  - Stats banner
  - Search bar HTMX
  - Grille catégories (6 cartes)
  - Grille produits
  - Trust indicators

- [ ] `/listings/:id` - Listing detail
  - Two-column layout
  - Image gallery avec modal <dialog>
  - Order form avec calcul temps réel
  - Vendor actions (edit/delete)

- [ ] `/listings/new` - Create listing
  - Form avec listing-form.html partial
  - Convertisseur XMR ⇄ Atomic
  - Image upload avec preview

- [ ] `/listings/:id/edit` - Edit listing
  - Form pré-rempli
  - Images existantes affichées

**Orders (2 pages):**
- [ ] `/orders` - Orders list
  - Tabs de filtrage (All, Pending, Funded, Shipped, Completed, Disputed)
  - Order cards en grille
  - Empty state

- [ ] `/orders/:id` - Order detail
  - Timeline visuel
  - Escrow visualizer (2-of-3 multisig)
  - Action buttons contextuels
  - HTMX integration

**Auth (2 pages):**
- [ ] `/login` - Login page
  - Form centré avec NEXUS branding
  - HTMX submission
  - Success/error handling
  - Redirect après login

- [ ] `/register` - Register page
  - Form avec role selection
  - Validation helper text
  - HTMX submission
  - Redirect vers login après succès

**Settings (2 pages):**
- [ ] `/settings` - Settings menu
  - 3 cards (Wallet/Account/Security)
  - Hover effects

- [ ] `/settings/wallet` - Wallet setup
  - Form RPC avec validation localhost
  - Instructions sidebar sticky
  - Security features section

#### Checklist par Page:

Pour chaque page, vérifier:
- [ ] CSS Nexus chargé correctement
- [ ] Layout responsive (mobile/tablet/desktop)
- [ ] Animations CSS fonctionnent
- [ ] Glassmorphism visible
- [ ] HTMX endpoints répondent
- [ ] Forms soumettent correctement
- [ ] Error handling fonctionne
- [ ] Navigation fonctionne (hx-boost)
- [ ] Composants Nexus s'affichent
- [ ] Accessibilité (keyboard nav, ARIA)

---

### 5.2 Intégration WebSocket Notifications (1h)

**Objectif:** Connecter les notifications WebSocket au composant toast Nexus

#### État Actuel:

Le système utilise déjà:
- `server/src/websocket.rs` - WebSocket server
- `static/js/notifications.js` - Client WebSocket (si existe)
- Composant `organisms/notification-center.html` déjà créé
- Composant `molecules/toast.html` déjà créé

#### Tâches:

**5.2.1 Adapter le Client WebSocket**
- [ ] Localiser ou créer `static/js/websocket-client.js`
- [ ] Intégrer avec composant toast.html Nexus
- [ ] Mapper les types de notifications:
  ```javascript
  {
    'order_created': { variant: 'info', icon: '📦' },
    'order_funded': { variant: 'success', icon: '💰' },
    'order_shipped': { variant: 'info', icon: '🚚' },
    'order_completed': { variant: 'success', icon: '✅' },
    'dispute_raised': { variant: 'destructive', icon: '⚠️' },
    'message_received': { variant: 'info', icon: '💬' }
  }
  ```

**5.2.2 Affichage Toast**
- [ ] Fonction `showToast(message, variant, duration)`
- [ ] Position: top-right (var(--nexus-space-6))
- [ ] Auto-dismiss après 5s
- [ ] Empiler multiples toasts
- [ ] Animation entrée/sortie (fadeInUp/fadeOutDown)

**5.2.3 Notification Center**
- [ ] Intégrer notification-center.html dans nav
- [ ] Badge avec count non-lus
- [ ] Dropdown avec liste notifications
- [ ] Bouton "Mark all as read"
- [ ] Persistence en localStorage

**5.2.4 Tests WebSocket**
- [ ] Connexion au serveur WebSocket
- [ ] Réception notifications
- [ ] Affichage toast
- [ ] Update notification center
- [ ] Reconnexion automatique si déconnecté

---

### 5.3 Optimisation Bundles (30 min)

**Objectif:** Optimiser CSS/JS pour Tor (<25KB target)

#### État Actuel:
- `nexus-variables.css` (~18KB)
- `nexus-reset.css` (~10KB)
- `nexus-animations.css` (~8KB)
- `nexus.css` (~12KB)
- **Total: ~48KB (non minifié)**

#### Tâches:

**5.3.1 Minification CSS**
- [ ] Installer `csso-cli`: `npm install -g csso-cli`
- [ ] Créer script `scripts/minify-css.sh`:
  ```bash
  #!/bin/bash
  cd static/css
  csso nexus-variables.css -o nexus-variables.min.css
  csso nexus-reset.css -o nexus-reset.min.css
  csso nexus-animations.css -o nexus-animations.min.css
  csso nexus.css -o nexus.min.css
  ```
- [ ] Target: <25KB total minifié

**5.3.2 PurgeCSS (optionnel)**
- [ ] Analyser les classes CSS inutilisées
- [ ] Purger si gain significatif (>10KB)
- [ ] Tester après purge

**5.3.3 JavaScript**
- [ ] Minifier `static/js/htmx.min.js` (déjà minifié)
- [ ] Minifier `static/js/json-enc.js`
- [ ] Minifier websocket-client.js (à créer)
- [ ] Target: <40KB total JS

**5.3.4 Update base-nexus.html**
- [ ] Pointer vers .min.css en production
- [ ] Ajouter conditional loading:
  ```html
  {% if production %}
    <link rel="stylesheet" href="/static/css/nexus.min.css">
  {% else %}
    <link rel="stylesheet" href="/static/css/nexus-variables.css">
    <link rel="stylesheet" href="/static/css/nexus-reset.css">
    ...
  {% endif %}
  ```

---

### 5.4 Tests Flux Complet (45 min)

**Objectif:** Valider le flux de commande end-to-end avec design Nexus

#### Setup:
- [ ] Démarrer serveur: `./target/release/server`
- [ ] Créer 2 comptes: buyer + vendor
- [ ] Vendor crée un listing avec images

#### Flux à Tester:

**5.4.1 Création Commande**
```
1. Buyer browse homepage → voir listing
2. Buyer clique listing → page detail
3. Buyer entre quantité → calcul prix temps réel OK
4. Buyer clique "Create Order" → ordre créé
5. Vérifier: Ordre en status 'pending'
6. Vérifier: WebSocket notification reçue
7. Vérifier: Toast affiché (variant: info)
```

**5.4.2 Funding Escrow**
```
1. Buyer va sur /orders/:id
2. Voir timeline avec étape "Pending"
3. Voir escrow visualizer (buyer/vendor/arbiter)
4. Cliquer "Fund Escrow" → modal confirm
5. Confirmer → HTMX POST /api/orders/:id/fund
6. Vérifier: Status → 'funded'
7. Vérifier: Timeline update
8. Vérifier: Escrow visualizer update
9. Vérifier: WebSocket notif vendor
10. Vérifier: Toast success
```

**5.4.3 Shipping**
```
1. Vendor va sur /orders/:id
2. Voir bouton "Mark as Shipped"
3. Cliquer → form tracking number
4. Soumettre → HTMX POST /api/orders/:id/ship
5. Vérifier: Status → 'shipped'
6. Vérifier: Timeline update
7. Vérifier: WebSocket notif buyer
8. Vérifier: Toast info "Order shipped"
```

**5.4.4 Completion**
```
1. Buyer va sur /orders/:id
2. Voir bouton "Confirm Receipt"
3. Cliquer → HTMX POST /api/orders/:id/complete
4. Vérifier: Status → 'completed'
5. Vérifier: Timeline complète
6. Vérifier: Escrow released
7. Vérifier: WebSocket notif vendor
8. Vérifier: Toast success "Order completed"
```

**5.4.5 Dispute (optionnel)**
```
1. Créer nouvelle commande
2. Funder → Shipped
3. Buyer clique "Open Dispute"
4. Remplir formulaire dispute
5. Vérifier: Status → 'disputed'
6. Vérifier: WebSocket notif arbitre
7. Vérifier: Toast warning
```

#### Validation:
- [ ] Tous les statuts transitionnent correctement
- [ ] Toutes les notifications WebSocket fonctionnent
- [ ] Tous les toasts s'affichent
- [ ] Timeline se met à jour visuellement
- [ ] Escrow visualizer reflète l'état
- [ ] Aucune erreur console
- [ ] Aucune erreur serveur logs

---

### 5.5 Bug Fixes & Polish (30 min)

**Objectif:** Corriger les problèmes découverts lors des tests

#### Bugs Potentiels:

**UI/CSS:**
- [ ] Responsive breakpoints
- [ ] Animations qui ne jouent pas
- [ ] Glassmorphism pas visible
- [ ] Hover effects manquants
- [ ] Z-index issues (modals)

**HTMX:**
- [ ] Endpoints 404
- [ ] hx-swap ne fonctionne pas
- [ ] hx-indicator pas affiché
- [ ] Response parsing errors
- [ ] CSRF token manquant

**JavaScript:**
- [ ] Real-time calc ne fonctionne pas
- [ ] Modal dialog ne s'ouvre pas
- [ ] Image preview erreur
- [ ] Tab filtering cassé
- [ ] WebSocket déconnexion

**Accessibilité:**
- [ ] Focus invisible
- [ ] ARIA labels manquants
- [ ] Keyboard nav cassée
- [ ] Screen reader issues

---

## 📊 MÉTRIQUES DE SUCCÈS

### Performance (Tor-optimized)
- [ ] CSS bundle: <25KB (target)
- [ ] JS bundle: <40KB (target)
- [ ] Time to Interactive: <3s
- [ ] Lighthouse Score: >90
- [ ] Requests HTTP: <12

### Fonctionnalité
- [ ] 8/8 pages migrées fonctionnent
- [ ] 100% flux commande OK
- [ ] WebSocket notifications temps réel
- [ ] Aucune régression bugs

### Qualité
- [ ] Aucun warning CSS
- [ ] Aucune erreur console JS
- [ ] Aucune erreur serveur logs
- [ ] Accessibilité WCAG 2.1 AA

---

## 🚀 LIVRABLES PHASE 5

### Code:
- [ ] `static/js/websocket-client.js` (nouveau)
- [ ] `static/css/*.min.css` (minifiés)
- [ ] `scripts/minify-css.sh` (nouveau)
- [ ] `templates/base-nexus.html` (updated pour production)

### Documentation:
- [ ] `PHASE-5-RESULTS.md` (résultats tests)
- [ ] Screenshots Nexus UI
- [ ] Performance benchmarks

### Tests:
- [ ] Flux E2E validé manuellement
- [ ] WebSocket tests passent
- [ ] Responsive tests (3 breakpoints)

---

## 🎯 PROCHAINES ÉTAPES (Phase 6)

Après Phase 5:
- Phase 6: Security & Performance audit
- Phase 7: Documentation complète
- Phase 8: Testing & Deployment prep

---

**Créé le:** 2025-10-26 19:30 UTC
**Par:** Claude Code
**Status:** 🟢 Ready to execute
