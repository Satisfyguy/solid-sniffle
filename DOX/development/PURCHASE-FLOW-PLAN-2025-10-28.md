# Plan d'Implémentation - Flow d'Achat/Vente
**Date:** 2025-10-28
**Status:** Planning Phase
**Objectif:** Implémenter l'interface utilisateur complète pour le flow d'achat/vente avec escrow multisig 2-of-3

---

## 🎯 Vue d'Ensemble

Le marketplace dispose d'un **backend complet et fonctionnel** avec:
- ✅ API REST pour orders/escrow
- ✅ Système multisig 2-of-3 (Buyer, Vendor, Arbiter)
- ✅ EscrowOrchestrator
- ✅ Tests E2E

**Ce qui manque:** L'interface utilisateur frontend pour permettre aux utilisateurs d'effectuer des achats.

---

## 📊 Machine d'État des Orders

```
┌─────────┐
│ PENDING │ ──────┐
└─────────┘       │
     │            ▼
     │       ┌───────────┐
     └──────▶│ CANCELLED │
             └───────────┘
     │
     ▼
┌────────┐
│ FUNDED │ ──────┐
└────────┘       │
     │           ▼
     │      ┌───────────┐
     ├─────▶│ DISPUTED  │────┐
     │      └───────────┘    │
     │                       │
     ▼                       ▼
┌─────────┐            ┌──────────┐
│ SHIPPED │            │ REFUNDED │
└─────────┘            └──────────┘
     │
     ▼
┌───────────┐
│ COMPLETED │
└───────────┘
```

### Transitions Valides

| De | Vers | Acteur | Action |
|---|---|---|---|
| Pending | Funded | Buyer | Initialise escrow + finance |
| Pending | Cancelled | Buyer/System | Timeout ou annulation |
| Funded | Shipped | Vendor | Marque comme expédié |
| Funded | Disputed | Buyer/Vendor | Ouvre dispute |
| Funded | Cancelled | Buyer | Annule avant expédition |
| Shipped | Completed | Buyer | Confirme réception |
| Shipped | Disputed | Buyer | Ouvre dispute |
| Disputed | Completed | Arbiter | Décision en faveur vendor |
| Disputed | Refunded | Arbiter | Décision en faveur buyer |

---

## 🔌 API Endpoints Existants

### Orders
- `POST /orders` - Créer une commande
- `GET /orders` - Lister mes commandes
- `GET /orders/{id}` - Détails d'une commande
- `GET /orders/pending-count` - Compteur notifications

### Escrow
- `POST /orders/{id}/init-escrow` - Initialiser multisig
- `POST /orders/{id}/ship` - Vendor marque shipped
- `POST /orders/{id}/complete` - Buyer confirme réception
- `PUT /orders/{id}/cancel` - Annuler commande
- `PUT /orders/{id}/dispute` - Ouvrir dispute

### Dev/Test
- `POST /orders/{id}/dev-simulate-payment` - Simuler paiement (dev only)

---

## 🎨 Interfaces à Créer

### 1. Page Détail Produit `/listings/{id}`
**Priorité:** HAUTE
**Fichier:** `templates/listings/detail.html`

#### Fonctionnalités
- [ ] Afficher toutes les informations du listing
- [ ] Galerie d'images IPFS (avec zoom)
- [ ] Quantité sélectionnable
- [ ] Calcul automatique du total (price × quantity)
- [ ] Bouton "BUY NOW" (style NEXUS)
- [ ] Vérification auth avant achat
- [ ] Modal de confirmation
- [ ] Infos vendor (reputation, sales count)

#### API Calls
- `GET /api/listings/{id}` - Récupérer listing
- `POST /orders` - Créer order

#### Design NEXUS
```html
<div class="nexus-product-detail">
  <div class="nexus-product-gallery">
    <!-- Images IPFS -->
  </div>
  <div class="nexus-product-info">
    <h1>{{ title }}</h1>
    <div class="nexus-price-large">{{ price_xmr }} XMR</div>
    <div class="nexus-quantity-selector">
      <input type="number" min="1" max="{{ stock }}" value="1">
    </div>
    <button class="nexus-btn-buy-large">
      🛒 BUY NOW
    </button>
  </div>
</div>
```

---

### 2. Page Checkout `/checkout/{order_id}`
**Priorité:** HAUTE
**Fichier:** `templates/checkout/index.html`

#### Fonctionnalités
- [ ] Résumé de la commande
- [ ] Afficher le total en XMR
- [ ] Instructions d'initialisation escrow
- [ ] Bouton "Initialize Escrow" avec loading state
- [ ] QR Code pour adresse multisig
- [ ] Adresse copyable
- [ ] Timer de timeout
- [ ] WebSocket pour mise à jour en temps réel

#### Flow
1. Afficher résumé order (status: pending)
2. User clique "Initialize Escrow"
3. Frontend appelle `POST /orders/{id}/init-escrow`
4. Backend crée wallet multisig 2-of-3
5. Retourne adresse multisig
6. Afficher adresse + QR code
7. User envoie XMR depuis son wallet
8. WebSocket notifie quand funded détecté
9. Redirect vers `/orders/{id}`

#### WebSocket Events
```javascript
{
  "event": "order_status_changed",
  "order_id": "...",
  "new_status": "funded"
}
```

---

### 3. Page My Orders `/orders`
**Priorité:** HAUTE
**Fichier:** `templates/orders/list.html`

#### Fonctionnalités
- [ ] Liste de toutes les commandes (buyer + vendor)
- [ ] Filtres par status
- [ ] Badges de couleur par status
- [ ] Actions selon rôle et status
- [ ] Compteur de notifications

#### Vue Buyer
```
┌─────────────────────────────────────┐
│ 📦 Order #ABC123                    │
│ Status: SHIPPED 🚚                  │
│ Vendor: CryptoVendor                │
│ Amount: 0.5000 XMR                  │
│ [Confirm Receipt] [Dispute]        │
└─────────────────────────────────────┘
```

#### Vue Vendor
```
┌─────────────────────────────────────┐
│ 📦 Order #ABC123                    │
│ Status: FUNDED ✅                   │
│ Buyer: Anonymous123                 │
│ Amount: 0.5000 XMR                  │
│ [Mark as Shipped]                   │
└─────────────────────────────────────┘
```

---

### 4. Page Détail Order `/orders/{id}`
**Priorité:** HAUTE
**Fichier:** `templates/orders/detail.html`

#### Fonctionnalités
- [ ] Timeline visuelle du statut
- [ ] Infos complètes de la commande
- [ ] Boutons d'action selon role + status
- [ ] Chat escrow (optionnel, phase 2)
- [ ] Historique des transitions
- [ ] Infos multisig (adresse, confirmations)

#### Template Structure
```html
<div class="nexus-order-detail">
  <!-- Order Timeline (réutiliser order-timeline.html) -->
  {% include "partials/nexus/organisms/order-timeline.html" %}

  <!-- Order Info Card -->
  <div class="nexus-order-info-card">
    <div class="nexus-order-header">
      <h1>Order #{{ order.id }}</h1>
      <span class="nexus-status-badge status-{{ order.status }}">
        {{ order.status | upper }}
      </span>
    </div>

    <!-- Product Details -->
    <div class="nexus-order-product">
      <img src="/ipfs/{{ listing.first_image }}" alt="{{ listing.title }}">
      <div>
        <h3>{{ listing.title }}</h3>
        <p>Quantity: {{ order.quantity }}</p>
        <p class="nexus-price">{{ order.total_xmr }} XMR</p>
      </div>
    </div>

    <!-- Escrow Info (si funded) -->
    {% if order.escrow_address %}
    <div class="nexus-escrow-info">
      <h3>🔐 Multisig Escrow</h3>
      <code>{{ order.escrow_address }}</code>
      <p>Status: {{ escrow.status }}</p>
    </div>
    {% endif %}

    <!-- Actions (dynamiques selon role + status) -->
    <div class="nexus-order-actions">
      {% if user_role == "vendor" and order.status == "funded" %}
        <button class="nexus-btn-primary" hx-post="/orders/{{ order.id }}/ship">
          📦 Mark as Shipped
        </button>
      {% endif %}

      {% if user_role == "buyer" and order.status == "shipped" %}
        <button class="nexus-btn-success" hx-post="/orders/{{ order.id }}/complete">
          ✅ Confirm Receipt
        </button>
        <button class="nexus-btn-danger" hx-put="/orders/{{ order.id }}/dispute">
          ⚠️ Open Dispute
        </button>
      {% endif %}

      {% if order.status in ["pending", "funded"] %}
        <button class="nexus-btn-secondary" hx-put="/orders/{{ order.id }}/cancel">
          ❌ Cancel Order
        </button>
      {% endif %}
    </div>
  </div>
</div>
```

---

### 5. Modal de Dispute
**Priorité:** MOYENNE
**Fichier:** `templates/partials/modals/dispute-modal.html`

#### Fonctionnalités
- [ ] Form pour raison de dispute
- [ ] Upload de preuves (IPFS)
- [ ] Sélection de l'arbiter
- [ ] Confirmation avant soumission

---

### 6. Handler Frontend Manquant
**Fichier:** `server/src/handlers/frontend.rs`

#### À ajouter
```rust
/// GET /listings/{id} - Product detail page
pub async fn listing_detail(
    tera: web::Data<Tera>,
    pool: web::Data<DbPool>,
    session: Session,
    id: web::Path<String>,
) -> impl Responder {
    // Load listing + vendor info + images
    // Render template
}

/// GET /checkout/{order_id} - Checkout page
pub async fn checkout_page(
    tera: web::Data<Tera>,
    pool: web::Data<DbPool>,
    session: Session,
    id: web::Path<String>,
) -> impl Responder {
    // Load order + listing
    // Render checkout
}

/// GET /orders - My Orders page
pub async fn orders_page(
    tera: web::Data<Tera>,
    pool: web::Data<DbPool>,
    session: Session,
) -> impl Responder {
    // Load user orders (buyer + vendor)
    // Render list
}

/// GET /orders/{id} - Order detail page
pub async fn order_detail_page(
    tera: web::Data<Tera>,
    pool: web::Data<DbPool>,
    session: Session,
    id: web::Path<String>,
) -> impl Responder {
    // Load order + listing + escrow
    // Render detail
}
```

---

## 🚀 Plan d'Implémentation (Ordre de Priorité)

### Phase 1: Flow d'Achat Basique (MVP)
1. ✅ **Backend API** (déjà fait)
2. **Page détail produit** + bouton Buy
3. **Création d'order** (POST /orders)
4. **Page My Orders** (liste simple)
5. **Page détail order** (infos basiques)

### Phase 2: Escrow Multisig
6. **Interface d'initialisation escrow**
7. **QR Code + copie d'adresse**
8. **WebSocket pour notifications en temps réel**
9. **Vendor: Mark as Shipped**
10. **Buyer: Confirm Receipt**

### Phase 3: Gestion Avancée
11. **Cancel order**
12. **Open dispute**
13. **Arbiter interface** (phase future)
14. **Chat escrow** (optionnel)

---

## 📱 Design System NEXUS

### Couleurs par Status
```css
.status-pending { background: hsl(45, 100%, 50%); color: #000; }
.status-funded { background: hsl(120, 60%, 50%); color: #fff; }
.status-shipped { background: hsl(200, 60%, 50%); color: #fff; }
.status-completed { background: hsl(120, 100%, 40%); color: #fff; }
.status-cancelled { background: hsl(0, 0%, 50%); color: #fff; }
.status-disputed { background: hsl(0, 100%, 50%); color: #fff; }
.status-refunded { background: hsl(280, 60%, 50%); color: #fff; }
```

### Boutons
- **Buy Now**: Grand, rouge NEXUS, avec effet glow
- **Actions Order**: Taille normale, couleurs sémantiques
- **Cancel**: Gris secondaire
- **Dispute**: Rouge danger

---

## 🔒 Sécurité

### CSRF Protection
Tous les forms POST/PUT/DELETE doivent inclure:
```html
<input type="hidden" name="csrf_token" value="{{ csrf_token }}">
```

### Authentication
Vérifier session avant toute action:
```rust
let user_id = get_user_id_from_session(&session)?;
```

### Authorization
- Buyer peut seulement confirm receipt / dispute sur SES orders
- Vendor peut seulement ship SES orders
- Arbiter peut seulement agir sur disputes

---

## 🧪 Tests à Faire

### Tests Frontend (manuels)
- [ ] Créer order en tant que buyer
- [ ] Initialiser escrow
- [ ] Vendor marque shipped
- [ ] Buyer confirme réception
- [ ] Test dispute flow
- [ ] Test cancel avant funded
- [ ] WebSocket notifications

### Tests E2E (automatisés)
- ✅ Déjà implémentés dans `server/tests/escrow_e2e.rs`

---

## 📝 Notes d'Implémentation

### HTMX
Utiliser `hx-boost="true"` pour navigation SPA-like:
```html
<a href="/listings/{{ id }}" hx-boost="true">View Product</a>
```

### WebSocket Client
```javascript
const ws = new WebSocket('ws://127.0.0.1:8080/ws');
ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  if (data.event === 'order_status_changed') {
    updateOrderStatus(data.order_id, data.new_status);
  }
};
```

### IPFS Images
Toutes les images doivent utiliser le gateway local:
```html
<img src="http://127.0.0.1:8081/ipfs/{{ cid }}" alt="...">
```

---

## ✅ Checklist de Complétion

### Phase 1 (MVP)
- [ ] Page détail produit créée
- [ ] Bouton Buy fonctionnel
- [ ] Order créé en DB
- [ ] Page My Orders affiche liste
- [ ] Page détail order affiche infos

### Phase 2 (Escrow)
- [ ] Init escrow fonctionne
- [ ] Adresse multisig affichée
- [ ] QR code généré
- [ ] WebSocket connecté
- [ ] Vendor peut ship
- [ ] Buyer peut confirm

### Phase 3 (Avancé)
- [ ] Cancel fonctionne
- [ ] Dispute peut être ouvert
- [ ] Timeline visuelle complète
- [ ] Tous les status badges corrects

---

## 🎯 Prochaine Étape

**COMMENCER PAR:** Page de détail produit (`/listings/{id}`)

Créer:
1. Handler `listing_detail()` dans `frontend.rs`
2. Template `templates/listings/detail.html`
3. Route dans `main.rs`
4. Test manuel avec un listing existant

---

**Document maintenu par:** Claude Code
**Dernière mise à jour:** 2025-10-28 16:15
