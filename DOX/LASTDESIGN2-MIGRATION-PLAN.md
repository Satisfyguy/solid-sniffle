# LASTDESIGN2 → NEXUS Backend Migration Plan

**Objectif:** Recréer l'interface React de LASTDESIGN2 (Lovable.dev) en Tera templates + HTMX, branchée au backend NEXUS existant (Actix-Web + Diesel + Monero Escrow).

**Date:** 2025-10-30

---

## Table des Matières

1. [Vue d'ensemble](#vue-densemble)
2. [Stack Technique](#stack-technique)
3. [Phase 0: Audit Backend NEXUS](#phase-0-audit-backend-nexus)
4. [Phase 1: Mapping LASTDESIGN2 → NEXUS](#phase-1-mapping-lastdesign2--nexus)
5. [Phase 2: Infrastructure Templates](#phase-2-infrastructure-templates)
6. [Phase 3: Composants Principaux](#phase-3-composants-principaux)
7. [Phase 4: Composants UI (Macros Tera)](#phase-4-composants-ui-macros-tera)
8. [Phase 5: Gestion État → Backend](#phase-5-gestion-état--backend)
9. [Phase 6: Interactions HTMX](#phase-6-interactions-htmx)
10. [Phase 7: Intégration Escrow](#phase-7-intégration-escrow)
11. [Phase 8: Design System](#phase-8-design-system)
12. [Phase 9: Animations Tor-Safe](#phase-9-animations-tor-safe)
13. [Phase 10: Testing](#phase-10-testing)
14. [Checklist Complète](#checklist-complète)
15. [Estimation Temps](#estimation-temps)

---

## Vue d'ensemble

### LASTDESIGN2 (Source)
- **Framework:** React 18 + Vite + TypeScript
- **UI:** Shadcn/ui (Radix UI primitives)
- **Styling:** Tailwind CSS + tailwindcss-animate
- **Router:** React Router v6
- **State:** TanStack Query
- **Source:** Lovable.dev (no-code platform)

### NEXUS Backend (Destination)
- **Framework:** Actix-Web 4.4
- **Templates:** Tera 1.19
- **Database:** Diesel + SQLite (SQLCipher)
- **Blockchain:** Monero RPC + 2/3 Multisig Escrow
- **Features:** Auth, Listings, Orders, Cart, Disputes, Reputation

### Objectif Final
Interface LASTDESIGN2 identique (couleurs, animations, layout) avec données réelles du backend NEXUS.

---

## Stack Technique

### Frontend
```
HTML5 (Tera templates)
+ Tailwind CSS (design system LASTDESIGN2)
+ HTMX 1.9 (interactivité SPA-like)
+ Alpine.js (dropdowns, modals)
+ Vanilla JS (animations stagger, theme toggle)
```

### Backend
```
Actix-Web (routes + handlers)
+ Diesel ORM (queries DB)
+ Tera (render templates)
+ Monero Wallet RPC (escrow)
```

### Design System
```css
Palette: Coral (#FF6B6B), Mint (#4ECDC4), Sky (#45B7D1), Sunshine (#FFD93D)
Font: Inter (system-ui fallback)
Animations: fade-in (0.5s), slide-up (0.5s), hover-scale (0.3s)
Radius: 0.75rem
Shadows: subtle (Tor-safe)
```

---

## Phase 0: Audit Backend NEXUS

### Handlers Existants
```
server/src/handlers/
├── auth.rs          → Login, register, logout, session
├── listings.rs      → CRUD produits, featured, search
├── orders.rs        → Create order, status, history
├── escrow.rs        → 2/3 multisig, fund, release, dispute
├── cart.rs          → Add/remove items, show cart
├── search.rs        → Full-text search listings
├── vendor.rs        → Become vendor, dashboard
├── frontend.rs      → Routes templates Tera
```

### Schéma Base de Données
```sql
users (id, username, email, password_hash, is_vendor, created_at)
listings (id, title, description, price_xmr, category, images_ipfs_cids, seller_id, featured, status)
orders (id, buyer_id, seller_id, total_xmr, status, created_at)
escrow_wallets (id, order_id, multisig_address, buyer_key, seller_key, arbiter_key)
cart_items (id, user_id, listing_id, quantity)
reviews (id, order_id, rating, comment)
disputes (id, order_id, reason, status, arbiter_id)
```

### Routes Actix-Web Actuelles
```rust
App::new()
  .route("/", web::get().to(handlers::frontend::index))
  .route("/listings", web::get().to(handlers::listings::list))
  .route("/listings/{id}", web::get().to(handlers::listings::show))
  .route("/orders", web::get().to(handlers::orders::index))
  .route("/orders/{id}", web::get().to(handlers::orders::show))
  .route("/cart", web::get().to(handlers::cart::show))
  .route("/login", web::get().to(handlers::auth::login_form))
  .route("/register", web::get().to(handlers::auth::register_form))
  // API endpoints
  .route("/api/auth/login", web::post().to(handlers::auth::login))
  .route("/api/cart/add", web::post().to(handlers::cart::add))
  .route("/api/orders/create", web::post().to(handlers::orders::create))
```

---

## Phase 1: Mapping LASTDESIGN2 → NEXUS

### Pages React → Handlers NEXUS

| Page LASTDESIGN2 | Route | Handler NEXUS | Données Context |
|------------------|-------|---------------|-----------------|
| `Index.tsx` | `/` | `frontend::index()` | `featured_products`, `categories`, `stats`, `logged_in`, `username` |
| `Categories.tsx` | `/categories` | `listings::list()` | `listings` (filtré par catégorie), `categories` |
| `Search.tsx` | `/search` | `search::index()` | `query`, `results`, `filters` |
| `Cart.tsx` | `/cart` | `cart::show()` | `cart_items`, `total_xmr`, `cart_count` |
| `BecomeVendor.tsx` | `/vendors/become` | `vendor::become()` | `user`, `is_vendor`, `form_data` |
| `Auth.tsx` | `/login` + `/register` | `auth::login_form()` + `auth::register_form()` | `csrf_token`, `errors` |
| `NotFound.tsx` | `*` | `frontend::not_found()` | - |

### Composants React → Templates Tera

| Composant React | Template Tera | Description |
|-----------------|---------------|-------------|
| `Header.tsx` | `components/header.html` | Logo NEXUS + Nav + Search + Cart + User menu |
| `Hero.tsx` | `components/hero.html` | Hero 2-col avec CTA + stats dynamiques |
| `TrustBadges.tsx` | `components/trust-badges.html` | 100% Private, 2/3 Multisig, Non-Custodial |
| `Categories.tsx` | `components/categories.html` | Grid catégories avec icônes + count |
| `FeaturedProducts.tsx` | `components/featured-products.html` | Grid 4 produits featured de la DB |
| `HowItWorks.tsx` | `components/how-it-works.html` | Étapes 1-2-3 du workflow |
| `Footer.tsx` | `components/footer.html` | Links + Copyright |

### Composants UI Shadcn → Macros Tera

| Shadcn Component | Macro Tera | Path |
|------------------|------------|------|
| `Button` | `{% macro button() %}` | `components/ui/button.html` |
| `Card` | `{% macro card() %}` | `components/ui/card.html` |
| `Input` | `{% macro input() %}` | `components/ui/input.html` |
| `Badge` | `{% macro badge() %}` | `components/ui/badge.html` |
| `Alert` | `{% macro alert() %}` | `components/ui/alert.html` |
| `Dialog` | Alpine.js modal | `components/ui/dialog.html` |

---

## Phase 2: Infrastructure Templates

### Structure Répertoire
```
templates/
├── base.html                    # Layout principal (head, scripts, header, footer)
├── pages/
│   ├── index.html               # Homepage (Hero + Featured + Categories)
│   ├── categories.html          # Liste par catégorie
│   ├── search.html              # Recherche + filtres
│   ├── cart.html                # Panier + checkout CTA
│   ├── become-vendor.html       # Formulaire devenir vendeur
│   ├── auth/
│   │   ├── login.html           # Formulaire login
│   │   └── register.html        # Formulaire register
│   └── errors/
│       └── 404.html             # Page not found
├── components/
│   ├── header.html              # Header avec nav + user menu
│   ├── footer.html              # Footer global
│   ├── hero.html                # Section hero homepage
│   ├── trust-badges.html        # Badges sécurité
│   ├── categories.html          # Grid catégories
│   ├── featured-products.html   # Grid produits featured
│   ├── how-it-works.html        # Workflow 3 étapes
│   ├── search-modal.html        # Modal search (Ctrl+K)
│   ├── search-results.html      # Fragment HTMX résultats
│   └── ui/                      # Macros Tera pour composants UI
│       ├── button.html
│       ├── card.html
│       ├── input.html
│       ├── badge.html
│       ├── alert.html
│       └── dialog.html
```

### base.html Template
```html
<!DOCTYPE html>
<html lang="en" class="{% if theme == 'dark' %}dark{% endif %}">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{% block title %}NEXUS - Secure Monero Marketplace{% endblock %}</title>

    <!-- CSS -->
    <link rel="stylesheet" href="/static/css/lastdesign2-variables.css">
    <link rel="stylesheet" href="/static/css/lastdesign2-animations.css">
    <link rel="stylesheet" href="/static/css/lastdesign2-components.css">

    <!-- Fonts -->
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800&display=swap" rel="stylesheet">

    <!-- HTMX -->
    <script src="/static/js/htmx.min.js" defer></script>

    <!-- Alpine.js (dropdowns, modals) -->
    <script defer src="https://cdn.jsdelivr.net/npm/alpinejs@3.x.x/dist/cdn.min.js"></script>

    {% block extra_head %}{% endblock %}
</head>
<body class="min-h-screen bg-background text-foreground">

    {% include "components/header.html" %}

    <main id="main-content">
        {% block content %}{% endblock %}
    </main>

    {% include "components/footer.html" %}

    <!-- Scripts -->
    <script src="/static/js/lastdesign2-animations.js"></script>
    <script src="/static/js/theme-toggle.js"></script>
    <script src="/static/js/nexus-letters-animation.js"></script>

    {% block extra_scripts %}{% endblock %}
</body>
</html>
```

---

## Phase 3: Composants Principaux

### 3.1 Hero Component

**Fichier:** `templates/components/hero.html`

**Context Tera requis:**
```rust
context.insert("hero", &HeroData {
    title: "Your Market.",
    subtitle: "Your Keys.",
    description: "Welcome to NEXUS. Commerce with complete confidentiality...",
});
context.insert("stats", &Stats {
    total_listings: 234,
    active_escrows: 42,
    satisfied_users: 1580,
});
context.insert("logged_in", &true);
```

**Template:**
```html
<section class="relative overflow-hidden bg-gradient-to-br from-coral/5 via-background to-sky/5">
  <div class="container mx-auto px-4 py-20 md:py-28">
    <div class="grid md:grid-cols-2 gap-12 items-center">

      <!-- Left Column: Text -->
      <div class="space-y-8 animate-fade-in">
        <div class="inline-flex items-center gap-2 bg-coral/10 text-coral px-4 py-2 rounded-full text-sm font-medium">
          <svg class="h-4 w-4 lucide" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10"/>
          </svg>
          100% Private & Secure
        </div>

        <h1 class="text-5xl md:text-6xl font-bold leading-tight">
          {{ hero.title }}
          <br>
          <span class="text-coral">{{ hero.subtitle }}</span>
          <br>
          Your Privacy.
        </h1>

        <p class="text-lg text-muted-foreground max-w-lg">
          {{ hero.description }}
        </p>

        <!-- CTA Buttons -->
        <div class="flex flex-wrap gap-4">
          <a href="/listings" class="inline-flex items-center justify-center gap-2 h-12 px-8 text-base rounded-md font-medium bg-coral text-white hover:bg-coral/90 shadow-lg transition-colors group">
            Start Shopping
            <svg class="h-5 w-5 group-hover:translate-x-1 transition-transform" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M5 12h14M12 5l7 7-7 7"/>
            </svg>
          </a>
          {% if not logged_in %}
          <a href="/login" class="inline-flex items-center justify-center h-12 px-8 text-base rounded-md font-medium border border-input bg-background hover:bg-accent transition-colors">
            Get Started
          </a>
          {% endif %}
        </div>

        <!-- Stats Dynamiques -->
        <div class="grid grid-cols-3 gap-8 pt-8 border-t border-white/10">
          <div>
            <div class="text-3xl font-bold text-coral">{{ stats.total_listings }}</div>
            <div class="text-sm text-muted-foreground">Active Listings</div>
          </div>
          <div>
            <div class="text-3xl font-bold text-coral">{{ stats.active_escrows }}</div>
            <div class="text-sm text-muted-foreground">Secure Escrows</div>
          </div>
          <div>
            <div class="text-3xl font-bold text-coral">{{ stats.satisfied_users }}</div>
            <div class="text-sm text-muted-foreground">Trusted Users</div>
          </div>
        </div>
      </div>

      <!-- Right Column: Image -->
      <div class="relative animate-slide-up">
        <div class="aspect-square rounded-3xl overflow-hidden shadow-2xl">
          <img
            src="/static/images/hero-monero-secure.png"
            alt="Secure Monero Marketplace"
            class="w-full h-full object-cover"
          />
        </div>
        <!-- Decorative blurs (Tor-safe, no Canvas) -->
        <div class="absolute -bottom-6 -right-6 w-48 h-48 bg-mint/20 rounded-full blur-3xl"></div>
        <div class="absolute -top-6 -left-6 w-48 h-48 bg-sky/20 rounded-full blur-3xl"></div>
      </div>

    </div>
  </div>

  <!-- Background orbs -->
  <div class="absolute top-0 left-0 w-96 h-96 bg-primary/10 rounded-full blur-3xl animate-pulse-slow"></div>
  <div class="absolute bottom-0 right-0 w-96 h-96 bg-accent/10 rounded-full blur-3xl animate-pulse-slow" style="animation-delay: 1.5s;"></div>
</section>
```

**Handler Rust:**
```rust
// server/src/handlers/frontend.rs
pub async fn index(
    tmpl: web::Data<tera::Tera>,
    pool: web::Data<DbPool>,
    session: Session,
) -> Result<HttpResponse, AppError> {
    let conn = pool.get()?;

    // Stats dynamiques
    let total_listings = listings::table
        .filter(listings::status.eq("active"))
        .count()
        .get_result::<i64>(&conn)?;

    let active_escrows = orders::table
        .filter(orders::status.eq("in_escrow"))
        .count()
        .get_result::<i64>(&conn)?;

    let satisfied_users = users::table
        .count()
        .get_result::<i64>(&conn)?;

    let stats = Stats {
        total_listings,
        active_escrows,
        satisfied_users,
    };

    // Featured products
    let featured = listings::table
        .filter(listings::featured.eq(true))
        .filter(listings::status.eq("active"))
        .order(listings::created_at.desc())
        .limit(4)
        .load::<Listing>(&conn)?;

    // Session
    let logged_in = session.get::<String>("user_id")?.is_some();
    let username = session.get::<String>("username")?;

    let mut context = tera::Context::new();
    context.insert("stats", &stats);
    context.insert("featured_products", &featured);
    context.insert("logged_in", &logged_in);
    context.insert("username", &username.unwrap_or_default());

    let html = tmpl.render("pages/index.html", &context)?;
    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}
```

---

### 3.2 Featured Products Component

**Fichier:** `templates/components/featured-products.html`

**Context requis:**
```rust
context.insert("featured_products", &vec![
    Listing {
        id: "uuid",
        title: "Premium VPN",
        price_xmr: "0.05",
        category: "Software",
        images_ipfs_cids: "[\"QmXxx\"]",
        ...
    },
    // ...
]);
```

**Template:**
```html
<section class="py-20 bg-secondary/30">
  <div class="container mx-auto px-4">
    <div class="text-center mb-12">
      <h2 class="text-4xl font-bold mb-4">Featured Products</h2>
      <p class="text-muted-foreground text-lg">
        Top-rated items from trusted vendors
      </p>
    </div>

    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
      {% for product in featured_products %}
      <div class="stagger-item group overflow-hidden border-none shadow-md hover:shadow-xl transition-all duration-300 hover:scale-105 animate-fade-in cursor-pointer rounded-xl bg-card"
           style="animation-delay: {{ loop.index0 * 100 }}ms">

        <!-- Image -->
        <div class="relative aspect-[4/3] overflow-hidden bg-muted">
          {% set images = product.images_ipfs_cids | json_decode %}
          {% if images and images | length > 0 %}
            <img src="/ipfs/{{ images[0] }}"
                 alt="{{ product.title }}"
                 class="w-full h-full object-cover group-hover:scale-110 transition-transform duration-500"
                 loading="lazy">
          {% else %}
            <img src="/static/images/placeholder-product.jpg"
                 alt="{{ product.title }}"
                 class="w-full h-full object-cover group-hover:scale-110 transition-transform duration-500"
                 loading="lazy">
          {% endif %}

          <div class="absolute top-3 right-3 bg-coral text-white px-3 py-1 rounded-full text-xs font-bold">
            Featured
          </div>
        </div>

        <!-- Content -->
        <div class="p-5">
          <p class="text-xs text-muted-foreground mb-2">{{ product.category }}</p>
          <h3 class="font-bold text-lg mb-2 line-clamp-1">{{ product.title }}</h3>

          <!-- Reviews (si disponible) -->
          <div class="flex items-center gap-2 mb-3">
            <div class="flex items-center gap-1">
              <svg class="h-4 w-4 fill-sunshine text-sunshine" viewBox="0 0 24 24">
                <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/>
              </svg>
              <span class="text-sm font-medium">4.8</span>
            </div>
            <span class="text-xs text-muted-foreground">(42 reviews)</span>
          </div>

          <div class="flex items-center justify-between">
            <span class="text-xl font-bold text-coral">{{ product.price_xmr }} XMR</span>
            <a href="/listings/{{ product.id }}"
               class="inline-flex items-center justify-center h-8 px-3 text-sm rounded-md font-medium border border-input bg-background hover:bg-accent transition-colors">
              View
            </a>
          </div>
        </div>
      </div>
      {% endfor %}
    </div>

    <div class="text-center mt-10">
      <a href="/listings"
         class="inline-flex items-center justify-center h-12 px-8 text-base rounded-md font-medium bg-coral text-white hover:bg-coral/90 shadow-lg transition-colors">
        View All Products
      </a>
    </div>
  </div>
</section>
```

---

### 3.3 Header Component

**Fichier:** `templates/components/header.html`

**Context requis:**
```rust
context.insert("logged_in", &true);
context.insert("username", &"alice");
context.insert("cart_count", &3);
context.insert("user", &User { is_vendor: true, ... });
```

**Template:**
```html
<header class="sticky top-0 z-50 w-full border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
  <div class="container mx-auto px-4">
    <div class="flex h-16 items-center justify-between">

      <!-- Logo + Nav -->
      <div class="flex items-center gap-8">
        <!-- Logo NEXUS (conserver lettres animées) -->
        <a href="/" class="flex items-center gap-2">
          <span class="text-2xl font-bold">
            {% for letter in "NEXUS" %}
            <span class="nexus-animated-letter">{{ letter }}</span>
            {% endfor %}
          </span>
        </a>

        <!-- Navigation -->
        <nav class="hidden md:flex items-center gap-6">
          <a href="/listings" class="text-sm font-medium hover:text-coral transition-colors">
            Browse
          </a>
          <a href="/categories" class="text-sm font-medium hover:text-coral transition-colors">
            Categories
          </a>
          {% if logged_in and user.is_vendor %}
          <a href="/vendors/dashboard" class="text-sm font-medium hover:text-coral transition-colors">
            My Shop
          </a>
          {% elif logged_in %}
          <a href="/vendors/become" class="text-sm font-medium hover:text-coral transition-colors">
            Become Vendor
          </a>
          {% endif %}
        </nav>
      </div>

      <!-- Actions Right -->
      <div class="flex items-center gap-4">

        <!-- Search Button (ouvre modal) -->
        <button
          @click="$store.searchModal.open = true"
          class="inline-flex items-center gap-2 h-9 px-3 rounded-md hover:bg-accent transition-colors">
          <svg class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="11" cy="11" r="8"/>
            <path d="m21 21-4.35-4.35"/>
          </svg>
          <span class="hidden md:inline text-sm">Search</span>
          <kbd class="hidden md:inline ml-2 px-2 py-1 bg-muted rounded text-xs">Ctrl+K</kbd>
        </button>

        <!-- Cart Icon -->
        {% if logged_in %}
        <a href="/cart" class="relative inline-flex items-center justify-center h-9 w-9 rounded-md hover:bg-accent transition-colors">
          <svg class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="8" cy="21" r="1"/>
            <circle cx="19" cy="21" r="1"/>
            <path d="M2.05 2.05h2l2.66 12.42a2 2 0 0 0 2 1.58h9.78a2 2 0 0 0 1.95-1.57l1.65-7.43H5.12"/>
          </svg>
          {% if cart_count > 0 %}
          <span class="absolute -top-1 -right-1 h-5 w-5 rounded-full bg-coral text-white text-xs flex items-center justify-center font-bold">
            {{ cart_count }}
          </span>
          {% endif %}
        </a>
        {% endif %}

        <!-- Theme Toggle -->
        <button id="theme-toggle" class="inline-flex items-center justify-center h-9 w-9 rounded-md hover:bg-accent transition-colors">
          <svg class="h-5 w-5 dark:hidden" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="4"/>
            <path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M6.34 17.66l-1.41 1.41M19.07 4.93l-1.41 1.41"/>
          </svg>
          <svg class="h-5 w-5 hidden dark:block" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/>
          </svg>
        </button>

        <!-- User Menu -->
        {% if logged_in %}
        <div class="relative" x-data="{ open: false }">
          <button @click="open = !open" class="flex items-center gap-2">
            <div class="h-8 w-8 rounded-full bg-gradient-to-br from-coral to-sky flex items-center justify-center text-white font-bold text-sm">
              {{ username[0] | upper }}
            </div>
          </button>

          <!-- Dropdown -->
          <div x-show="open"
               @click.away="open = false"
               x-transition:enter="transition ease-out duration-100"
               x-transition:enter-start="opacity-0 scale-95"
               x-transition:enter-end="opacity-100 scale-100"
               x-transition:leave="transition ease-in duration-75"
               x-transition:leave-start="opacity-100 scale-100"
               x-transition:leave-end="opacity-0 scale-95"
               class="absolute right-0 mt-2 w-48 rounded-xl bg-popover border shadow-lg py-1">
            <div class="px-4 py-2 border-b">
              <p class="text-sm font-medium">{{ username }}</p>
              <p class="text-xs text-muted-foreground">{{ user.email }}</p>
            </div>
            <a href="/orders" class="block px-4 py-2 text-sm hover:bg-accent transition-colors">
              My Orders
            </a>
            <a href="/settings" class="block px-4 py-2 text-sm hover:bg-accent transition-colors">
              Settings
            </a>
            {% if user.is_vendor %}
            <a href="/vendors/dashboard" class="block px-4 py-2 text-sm hover:bg-accent transition-colors">
              Vendor Dashboard
            </a>
            {% endif %}
            <div class="border-t mt-1 pt-1">
              <form method="POST" action="/logout">
                <button type="submit" class="w-full text-left px-4 py-2 text-sm text-destructive hover:bg-accent transition-colors">
                  Logout
                </button>
              </form>
            </div>
          </div>
        </div>
        {% else %}
        <a href="/login" class="inline-flex items-center justify-center h-9 px-4 text-sm rounded-md font-medium bg-coral text-white hover:bg-coral/90 transition-colors">
          Sign In
        </a>
        {% endif %}

      </div>
    </div>
  </div>
</header>

<!-- Search Modal (Alpine.js) -->
{% include "components/search-modal.html" %}
```

---

## Phase 4: Composants UI (Macros Tera)

### 4.1 Button Macro

**Fichier:** `templates/components/ui/button.html`

```html
{% macro button(variant="default", size="default", class="", type="button", href="", hx_get="", hx_post="", hx_target="") %}
  {% if href %}
    <a href="{{ href }}"
       {% if hx_get %}hx-get="{{ hx_get }}"{% endif %}
       {% if hx_post %}hx-post="{{ hx_post }}"{% endif %}
       {% if hx_target %}hx-target="{{ hx_target }}"{% endif %}
       class="inline-flex items-center justify-center gap-2 rounded-md font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 disabled:pointer-events-none disabled:opacity-50
         {% if variant == 'hero' %}bg-coral text-white hover:bg-coral/90 shadow-lg
         {% elif variant == 'outline' %}border border-input bg-background hover:bg-accent
         {% elif variant == 'ghost' %}hover:bg-accent
         {% elif variant == 'destructive' %}bg-destructive text-destructive-foreground hover:bg-destructive/90
         {% else %}bg-primary text-primary-foreground hover:bg-primary/90{% endif %}
         {% if size == 'lg' %}h-12 px-8 text-base
         {% elif size == 'sm' %}h-8 px-3 text-sm
         {% else %}h-10 px-4{% endif %}
         {{ class }}">
      {{ caller() }}
    </a>
  {% else %}
    <button type="{{ type }}"
            {% if hx_get %}hx-get="{{ hx_get }}"{% endif %}
            {% if hx_post %}hx-post="{{ hx_post }}"{% endif %}
            {% if hx_target %}hx-target="{{ hx_target }}"{% endif %}
            class="inline-flex items-center justify-center gap-2 rounded-md font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 disabled:pointer-events-none disabled:opacity-50
              {% if variant == 'hero' %}bg-coral text-white hover:bg-coral/90 shadow-lg
              {% elif variant == 'outline' %}border border-input bg-background hover:bg-accent
              {% elif variant == 'ghost' %}hover:bg-accent
              {% elif variant == 'destructive' %}bg-destructive text-destructive-foreground hover:bg-destructive/90
              {% else %}bg-primary text-primary-foreground hover:bg-primary/90{% endif %}
              {% if size == 'lg' %}h-12 px-8 text-base
              {% elif size == 'sm' %}h-8 px-3 text-sm
              {% else %}h-10 px-4{% endif %}
              {{ class }}">
      {{ caller() }}
    </button>
  {% endif %}
{% endmacro %}
```

**Usage:**
```html
{% from "components/ui/button.html" import button %}

{% call button(variant="hero", size="lg", href="/listings") %}
  Start Shopping
  <svg class="h-5 w-5">...</svg>
{% endcall %}

{% call button(variant="outline", hx_post="/api/cart/add", hx_target="#cart-count") %}
  Add to Cart
{% endcall %}
```

---

### 4.2 Card Macro

**Fichier:** `templates/components/ui/card.html`

```html
{% macro card(class="", hover=false) %}
<div class="rounded-xl border bg-card text-card-foreground shadow
  {% if hover %}hover:shadow-xl hover:scale-105 transition-all duration-300{% endif %}
  {{ class }}">
  {{ caller() }}
</div>
{% endmacro %}

{% macro card_header(class="") %}
<div class="flex flex-col space-y-1.5 p-6 {{ class }}">
  {{ caller() }}
</div>
{% endmacro %}

{% macro card_title(class="") %}
<h3 class="text-2xl font-semibold leading-none tracking-tight {{ class }}">
  {{ caller() }}
</h3>
{% endmacro %}

{% macro card_description(class="") %}
<p class="text-sm text-muted-foreground {{ class }}">
  {{ caller() }}
</p>
{% endmacro %}

{% macro card_content(class="") %}
<div class="p-6 pt-0 {{ class }}">
  {{ caller() }}
</div>
{% endmacro %}

{% macro card_footer(class="") %}
<div class="flex items-center p-6 pt-0 {{ class }}">
  {{ caller() }}
</div>
{% endmacro %}
```

**Usage:**
```html
{% from "components/ui/card.html" import card, card_header, card_title, card_content %}

{% call card(hover=true) %}
  {% call card_header() %}
    {% call card_title() %}Featured Product{% endcall %}
  {% endcall %}
  {% call card_content() %}
    <p>Product description...</p>
  {% endcall %}
{% endcall %}
```

---

### 4.3 Input Macro

**Fichier:** `templates/components/ui/input.html`

```html
{% macro input(type="text", name="", id="", placeholder="", value="", required=false, class="", error="") %}
<div class="space-y-2">
  <input
    type="{{ type }}"
    name="{{ name }}"
    id="{{ id or name }}"
    placeholder="{{ placeholder }}"
    value="{{ value }}"
    {% if required %}required{% endif %}
    class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 {{ class }}"
  >
  {% if error %}
  <p class="text-sm text-destructive">{{ error }}</p>
  {% endif %}
</div>
{% endmacro %}

{% macro label(for="", class="") %}
<label for="{{ for }}" class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70 {{ class }}">
  {{ caller() }}
</label>
{% endmacro %}
```

**Usage:**
```html
{% from "components/ui/input.html" import input, label %}

{% call label(for="email") %}Email{% endcall %}
{{ input(type="email", name="email", id="email", placeholder="you@example.com", required=true) }}
```

---

## Phase 5: Gestion État → Backend

### React State → Rust Queries

**React (client-side):**
```tsx
const [products, setProducts] = useState([]);
useEffect(() => {
  fetch('/api/products')
    .then(res => res.json())
    .then(data => setProducts(data));
}, []);
```

**Rust (server-side):**
```rust
pub async fn index(
    tmpl: web::Data<tera::Tera>,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, AppError> {
    let conn = pool.get()?;

    let products = listings::table
        .filter(listings::featured.eq(true))
        .limit(4)
        .load::<Listing>(&conn)?;

    let mut context = tera::Context::new();
    context.insert("products", &products);

    let html = tmpl.render("pages/index.html", &context)?;
    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}
```

---

## Phase 6: Interactions HTMX

### 6.1 Live Search

**Template:**
```html
<input
  type="text"
  name="q"
  placeholder="Search products..."
  hx-get="/api/search"
  hx-trigger="keyup changed delay:300ms"
  hx-target="#search-results"
  hx-indicator="#search-loading"
  class="input"
>

<div id="search-loading" class="htmx-indicator">
  <div class="spinner"></div>
</div>

<div id="search-results">
  <!-- Résultats injectés ici -->
</div>
```

**Handler:**
```rust
pub async fn api_search(
    query: web::Query<SearchQuery>,
    tmpl: web::Data<tera::Tera>,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, AppError> {
    let conn = pool.get()?;

    let results = listings::table
        .filter(listings::title.ilike(format!("%{}%", query.q)))
        .or_filter(listings::description.ilike(format!("%{}%", query.q)))
        .limit(10)
        .load::<Listing>(&conn)?;

    let mut context = tera::Context::new();
    context.insert("results", &results);

    // Retourner FRAGMENT (pas page complète)
    let html = tmpl.render("components/search-results.html", &context)?;
    Ok(HttpResponse::Ok().body(html))
}
```

---

### 6.2 Add to Cart

**Template:**
```html
<button
  hx-post="/api/cart/add"
  hx-vals='{"listing_id": "{{ product.id }}", "quantity": 1}'
  hx-target="#cart-count"
  hx-swap="outerHTML"
  class="btn btn-primary">
  Add to Cart
</button>

<span id="cart-count" class="badge">{{ cart_count }}</span>
```

**Handler:**
```rust
pub async fn add_to_cart(
    form: web::Json<AddToCartForm>,
    pool: web::Data<DbPool>,
    session: Session,
) -> Result<HttpResponse, AppError> {
    let user_id = session.get::<String>("user_id")?
        .ok_or_else(|| AppError::Unauthorized)?;

    let conn = pool.get()?;

    diesel::insert_into(cart_items::table)
        .values(NewCartItem {
            user_id: &user_id,
            listing_id: &form.listing_id,
            quantity: form.quantity,
        })
        .execute(&conn)?;

    let cart_count = cart_items::table
        .filter(cart_items::user_id.eq(&user_id))
        .count()
        .get_result::<i64>(&conn)?;

    // Retourner nouveau badge count
    Ok(HttpResponse::Ok().body(format!(
        r#"<span id="cart-count" class="badge">{}</span>"#,
        cart_count
    )))
}
```

---

### 6.3 Remove from Cart

**Template:**
```html
<div id="cart-item-{{ item.id }}" class="cart-item">
  <img src="..." />
  <h3>{{ item.listing.title }}</h3>
  <button
    hx-delete="/api/cart/{{ item.id }}"
    hx-target="#cart-item-{{ item.id }}"
    hx-swap="outerHTML"
    hx-confirm="Remove this item?"
    class="btn btn-ghost text-destructive">
    Remove
  </button>
</div>
```

**Handler:**
```rust
pub async fn remove_from_cart(
    item_id: web::Path<String>,
    pool: web::Data<DbPool>,
    session: Session,
) -> Result<HttpResponse, AppError> {
    let user_id = session.get::<String>("user_id")?
        .ok_or_else(|| AppError::Unauthorized)?;

    let conn = pool.get()?;

    diesel::delete(
        cart_items::table
            .filter(cart_items::id.eq(item_id.as_str()))
            .filter(cart_items::user_id.eq(&user_id))
    ).execute(&conn)?;

    // HTMX swap outerHTML avec vide = supprime élément
    Ok(HttpResponse::Ok().body(""))
}
```

---

## Phase 7: Intégration Escrow

### 7.1 Checkout → Create Order + Escrow

**Handler:**
```rust
// server/src/handlers/orders.rs
pub async fn create_from_cart(
    pool: web::Data<DbPool>,
    session: Session,
    monero_client: web::Data<Arc<MoneroClient>>,
) -> Result<HttpResponse, AppError> {
    let user_id = session.get::<String>("user_id")?
        .ok_or_else(|| AppError::Unauthorized)?;

    let conn = pool.get()?;

    // 1. Get cart items
    let cart_items = cart_items::table
        .inner_join(listings::table)
        .filter(cart_items::user_id.eq(&user_id))
        .load::<(CartItem, Listing)>(&conn)?;

    if cart_items.is_empty() {
        return Err(AppError::BadRequest("Cart is empty".to_string()));
    }

    // 2. Calculate total
    let total_xmr: f64 = cart_items.iter()
        .map(|(item, listing)| {
            listing.price_xmr.parse::<f64>().unwrap_or(0.0) * (item.quantity as f64)
        })
        .sum();

    // 3. Create order
    let order_id = Uuid::new_v4().to_string();
    let seller_id = &cart_items[0].1.seller_id;

    diesel::insert_into(orders::table)
        .values(NewOrder {
            id: &order_id,
            buyer_id: &user_id,
            seller_id,
            total_xmr: &total_xmr.to_string(),
            status: "pending_escrow",
            created_at: Utc::now().naive_utc(),
        })
        .execute(&conn)?;

    // 4. Initialize 2/3 multisig escrow
    let arbiter_id = get_random_arbiter(&conn)?;

    let escrow = create_escrow_wallet(
        &order_id,
        &user_id,
        seller_id,
        &arbiter_id,
        &monero_client,
    ).await?;

    diesel::insert_into(escrow_wallets::table)
        .values(NewEscrowWallet {
            id: &Uuid::new_v4().to_string(),
            order_id: &order_id,
            multisig_address: &escrow.multisig_address,
            buyer_key: &escrow.buyer_key,
            seller_key: &escrow.seller_key,
            arbiter_key: &escrow.arbiter_key,
            status: "awaiting_funding",
        })
        .execute(&conn)?;

    // 5. Clear cart
    diesel::delete(
        cart_items::table.filter(cart_items::user_id.eq(&user_id))
    ).execute(&conn)?;

    // 6. Redirect to order page
    Ok(HttpResponse::SeeOther()
        .append_header(("Location", format!("/orders/{}", order_id)))
        .finish())
}
```

---

### 7.2 Order Page with Escrow Status

**Template:** `templates/pages/order-show.html`

```html
{% extends "base.html" %}

{% block content %}
<div class="container mx-auto px-4 py-12">
  <div class="max-w-4xl mx-auto">

    <!-- Order Header -->
    <div class="mb-8">
      <h1 class="text-3xl font-bold mb-2">Order #{{ order.id | truncate(length=8) }}</h1>
      <div class="flex items-center gap-4">
        <span class="px-3 py-1 rounded-full text-sm font-medium
          {% if order.status == 'completed' %}bg-mint/20 text-mint
          {% elif order.status == 'disputed' %}bg-destructive/20 text-destructive
          {% elif order.status == 'in_escrow' %}bg-sky/20 text-sky
          {% else %}bg-muted text-muted-foreground{% endif %}">
          {{ order.status | replace(from="_", to=" ") | title }}
        </span>
        <span class="text-muted-foreground">
          {{ order.created_at | date(format="%B %d, %Y") }}
        </span>
      </div>
    </div>

    <!-- Escrow Card -->
    {% if order.escrow %}
    <div class="bg-card border rounded-xl p-6 mb-8">
      <h2 class="text-xl font-bold mb-4 flex items-center gap-2">
        <svg class="h-5 w-5 text-coral" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect width="18" height="11" x="3" y="11" rx="2" ry="2"/>
          <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
        </svg>
        2/3 Multisig Escrow
      </h2>

      <div class="space-y-3">
        <div class="flex justify-between items-center">
          <span class="text-muted-foreground">Escrow Address</span>
          <code class="text-sm bg-muted px-3 py-1 rounded font-mono">
            {{ order.escrow.multisig_address | truncate(length=16) }}...
          </code>
        </div>
        <div class="flex justify-between items-center">
          <span class="text-muted-foreground">Amount Locked</span>
          <span class="font-bold text-coral text-lg">{{ order.total_xmr }} XMR</span>
        </div>
        <div class="flex justify-between items-center">
          <span class="text-muted-foreground">Confirmations</span>
          <div class="flex items-center gap-2">
            <span class="font-medium">{{ order.escrow.confirmations }}/10</span>
            <div class="w-32 h-2 bg-muted rounded-full overflow-hidden">
              <div class="h-full bg-coral transition-all"
                   style="width: {{ (order.escrow.confirmations / 10 * 100) | round }}%"></div>
            </div>
          </div>
        </div>
        <div class="flex justify-between items-center">
          <span class="text-muted-foreground">Escrow Status</span>
          <span class="font-medium text-sky">{{ order.escrow.status | replace(from="_", to=" ") | title }}</span>
        </div>
      </div>

      <!-- Actions (buyer/seller specific) -->
      <div class="mt-6 flex gap-3">
        {% if user_id == order.buyer_id %}
          {% if order.status == "shipped" %}
            <button
              hx-post="/api/orders/{{ order.id }}/release"
              hx-confirm="Release funds to seller? This action cannot be undone."
              class="btn btn-hero">
              Release Funds
            </button>
            <button
              @click="$store.disputeModal.open = true; $store.disputeModal.orderId = '{{ order.id }}'"
              class="btn btn-outline">
              Open Dispute
            </button>
          {% elif order.status == "in_escrow" %}
            <p class="text-sm text-muted-foreground">Waiting for seller to ship the order...</p>
          {% endif %}
        {% elif user_id == order.seller_id %}
          {% if order.status == "in_escrow" %}
            <button
              hx-post="/api/orders/{{ order.id }}/mark-shipped"
              hx-confirm="Mark this order as shipped?"
              class="btn btn-hero">
              Mark as Shipped
            </button>
          {% elif order.status == "shipped" %}
            <p class="text-sm text-muted-foreground">Waiting for buyer to confirm delivery...</p>
          {% endif %}
        {% endif %}
      </div>
    </div>
    {% endif %}

    <!-- Order Items -->
    <div class="bg-card border rounded-xl p-6">
      <h2 class="text-xl font-bold mb-4">Order Items</h2>
      <div class="space-y-4">
        {% for item in order.items %}
        <div class="flex gap-4 py-4 border-b last:border-0">
          <img
            src="/static/images/placeholder-product.jpg"
            alt="{{ item.listing.title }}"
            class="h-20 w-20 rounded-lg object-cover"
          />
          <div class="flex-1">
            <h3 class="font-semibold text-lg">{{ item.listing.title }}</h3>
            <p class="text-sm text-muted-foreground">{{ item.listing.category }}</p>
            <p class="text-sm text-muted-foreground">Qty: {{ item.quantity }}</p>
          </div>
          <div class="text-right">
            <p class="font-bold text-lg">{{ item.price_xmr }} XMR</p>
            <p class="text-sm text-muted-foreground">{{ item.quantity }} × {{ item.listing.price_xmr }} XMR</p>
          </div>
        </div>
        {% endfor %}
      </div>

      <!-- Total -->
      <div class="border-t pt-4 mt-4">
        <div class="flex justify-between items-center text-xl font-bold">
          <span>Total</span>
          <span class="text-coral">{{ order.total_xmr }} XMR</span>
        </div>
      </div>
    </div>

    <!-- Shipping Info (if shipped) -->
    {% if order.tracking_number %}
    <div class="bg-card border rounded-xl p-6 mt-8">
      <h2 class="text-xl font-bold mb-4">Shipping Information</h2>
      <div class="space-y-2">
        <div class="flex justify-between">
          <span class="text-muted-foreground">Tracking Number</span>
          <code class="text-sm bg-muted px-2 py-1 rounded">{{ order.tracking_number }}</code>
        </div>
        <div class="flex justify-between">
          <span class="text-muted-foreground">Shipped Date</span>
          <span>{{ order.shipped_at | date(format="%B %d, %Y") }}</span>
        </div>
      </div>
    </div>
    {% endif %}

  </div>
</div>
{% endblock %}
```

---

## Phase 8: Design System

### 8.1 CSS Variables (LASTDESIGN2)

**Fichier:** `static/css/lastdesign2-variables.css`

```css
@tailwind base;
@tailwind components;
@tailwind utilities;

@layer base {
  :root {
    --background: 0 0% 100%;
    --foreground: 222 47% 11%;

    --card: 0 0% 100%;
    --card-foreground: 222 47% 11%;

    --popover: 0 0% 100%;
    --popover-foreground: 222 47% 11%;

    --primary: 222 47% 11%;
    --primary-foreground: 0 0% 100%;

    --secondary: 0 0% 96%;
    --secondary-foreground: 222 47% 11%;

    --muted: 0 0% 96%;
    --muted-foreground: 215 16% 47%;

    --accent: 222 47% 11%;
    --accent-foreground: 0 0% 100%;

    --destructive: 0 84% 60%;
    --destructive-foreground: 0 0% 100%;

    --border: 214 32% 91%;
    --input: 214 32% 91%;
    --ring: 222 47% 11%;

    --radius: 0.75rem;

    /* LASTDESIGN2 Colors */
    --coral: 6 93% 71%;
    --coral-foreground: 0 0% 100%;

    --sunshine: 45 100% 62%;
    --sunshine-foreground: 222 47% 11%;

    --mint: 142 52% 60%;
    --mint-foreground: 0 0% 100%;

    --sky: 215 96% 62%;
    --sky-foreground: 0 0% 100%;
  }

  .dark {
    --background: 222 47% 11%;
    --foreground: 0 0% 100%;

    --card: 222 47% 11%;
    --card-foreground: 0 0% 100%;

    --popover: 222 47% 11%;
    --popover-foreground: 0 0% 100%;

    --primary: 0 0% 100%;
    --primary-foreground: 222 47% 11%;

    --secondary: 217 33% 18%;
    --secondary-foreground: 0 0% 100%;

    --muted: 217 33% 18%;
    --muted-foreground: 215 20% 65%;

    --accent: 0 0% 100%;
    --accent-foreground: 222 47% 11%;

    --destructive: 0 63% 31%;
    --destructive-foreground: 0 0% 100%;

    --border: 217 33% 18%;
    --input: 217 33% 18%;
    --ring: 213 27% 84%;
  }
}

@layer base {
  * {
    @apply border-border;
  }

  body {
    @apply bg-background text-foreground;
    font-family: 'Inter', system-ui, -apple-system, sans-serif;
  }

  /* Utility classes */
  .container {
    @apply mx-auto px-4 max-w-7xl;
  }

  .btn {
    @apply inline-flex items-center justify-center gap-2 rounded-md font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 disabled:pointer-events-none disabled:opacity-50;
  }

  .btn-primary {
    @apply bg-primary text-primary-foreground hover:bg-primary/90;
  }

  .btn-hero {
    @apply bg-coral text-white hover:bg-coral/90 shadow-lg;
  }

  .btn-outline {
    @apply border border-input bg-background hover:bg-accent;
  }

  .btn-ghost {
    @apply hover:bg-accent;
  }

  .btn-lg {
    @apply h-12 px-8 text-base;
  }

  .btn-sm {
    @apply h-8 px-3 text-sm;
  }

  .input {
    @apply flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50;
  }

  .badge {
    @apply inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-semibold;
  }

  /* Line clamp utilities */
  .line-clamp-1 {
    display: -webkit-box;
    -webkit-line-clamp: 1;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .line-clamp-2 {
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
}

/* Tailwind Gradients */
.bg-gradient-to-br {
  background-image: linear-gradient(to bottom right, var(--tw-gradient-stops));
}

.from-coral\/5 {
  --tw-gradient-from: hsl(var(--coral) / 0.05);
  --tw-gradient-to: hsl(var(--coral) / 0);
  --tw-gradient-stops: var(--tw-gradient-from), var(--tw-gradient-to);
}

.via-background {
  --tw-gradient-to: hsl(var(--background) / 0);
  --tw-gradient-stops: var(--tw-gradient-from), hsl(var(--background)), var(--tw-gradient-to);
}

.to-sky\/5 {
  --tw-gradient-to: hsl(var(--sky) / 0.05);
}
```

---

### 8.2 Animations CSS

**Fichier:** `static/css/lastdesign2-animations.css`

```css
/* ==========================================
   LASTDESIGN2 - Animations Tor-Safe
   Pure CSS - No Canvas, No WebGL
   ========================================== */

@keyframes fade-in {
  0% {
    opacity: 0;
    transform: translateY(10px);
  }
  100% {
    opacity: 1;
    transform: translateY(0);
  }
}

@keyframes slide-up {
  0% {
    transform: translateY(20px);
    opacity: 0;
  }
  100% {
    transform: translateY(0);
    opacity: 1;
  }
}

@keyframes pulse-slow {
  0%, 100% {
    opacity: 0.3;
  }
  50% {
    opacity: 0.6;
  }
}

@keyframes accordion-down {
  from {
    height: 0;
    opacity: 0;
  }
  to {
    height: var(--radix-accordion-content-height, auto);
    opacity: 1;
  }
}

@keyframes accordion-up {
  from {
    height: var(--radix-accordion-content-height, auto);
    opacity: 1;
  }
  to {
    height: 0;
    opacity: 0;
  }
}

/* Animation Classes */
.animate-fade-in {
  animation: fade-in 0.5s ease-out;
}

.animate-slide-up {
  animation: slide-up 0.5s ease-out;
}

.animate-pulse-slow {
  animation: pulse-slow 4s ease-in-out infinite;
}

.animate-accordion-down {
  animation: accordion-down 0.2s ease-out;
}

.animate-accordion-up {
  animation: accordion-up 0.2s ease-out;
}

/* Hover Effects (Tor-safe) */
.hover\:scale-105:hover {
  transform: scale(1.05);
}

.hover\:scale-110:hover {
  transform: scale(1.1);
}

.group:hover .group-hover\:scale-110 {
  transform: scale(1.1);
}

.group:hover .group-hover\:translate-x-1 {
  transform: translateX(0.25rem);
}

/* Transitions */
.transition-all {
  transition-property: all;
  transition-timing-function: cubic-bezier(0.4, 0, 0.2, 1);
  transition-duration: 300ms;
}

.transition-colors {
  transition-property: color, background-color, border-color;
  transition-timing-function: cubic-bezier(0.4, 0, 0.2, 1);
  transition-duration: 150ms;
}

.transition-transform {
  transition-property: transform;
  transition-timing-function: cubic-bezier(0.4, 0, 0.2, 1);
  transition-duration: 500ms;
}

.duration-300 {
  transition-duration: 300ms;
}

.duration-500 {
  transition-duration: 500ms;
}

/* HTMX Indicators */
.htmx-indicator {
  opacity: 0;
  transition: opacity 200ms ease-in;
}

.htmx-request .htmx-indicator {
  opacity: 1;
}

.htmx-request.htmx-indicator {
  opacity: 1;
}

/* Spinner */
@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.spinner {
  width: 20px;
  height: 20px;
  border: 2px solid hsl(var(--border));
  border-top-color: hsl(var(--coral));
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}

/* Accessibility: Respect prefers-reduced-motion */
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

---

### 8.3 JavaScript Animations

**Fichier:** `static/js/lastdesign2-animations.js`

```javascript
/**
 * LASTDESIGN2 Animations
 * Stagger effects + Intersection Observer
 */
(function() {
  'use strict';

  // Respect prefers-reduced-motion
  const prefersReducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;

  if (prefersReducedMotion) {
    console.log('⏸️ Animations disabled (prefers-reduced-motion)');
    return;
  }

  /**
   * Stagger animation delays
   * Ajoute un delay séquentiel aux éléments .stagger-item
   */
  function initStaggerAnimations() {
    const staggerItems = document.querySelectorAll('.stagger-item');

    staggerItems.forEach((el, index) => {
      el.style.animationDelay = `${index * 100}ms`;
    });

    console.log(`✅ Stagger animation initialized (${staggerItems.length} items)`);
  }

  /**
   * Intersection Observer pour scroll reveals
   * Ajoute classe .visible quand élément entre dans viewport
   */
  function initScrollReveal() {
    const revealElements = document.querySelectorAll('.scroll-reveal');

    if (revealElements.length === 0) return;

    const observer = new IntersectionObserver((entries) => {
      entries.forEach(entry => {
        if (entry.isIntersecting) {
          entry.target.classList.add('visible');
          // Disconnect après reveal (performance)
          observer.unobserve(entry.target);
        }
      });
    }, {
      threshold: 0.1,
      rootMargin: '0px 0px -50px 0px'
    });

    revealElements.forEach(el => observer.observe(el));

    console.log(`✅ Scroll reveal initialized (${revealElements.length} elements)`);
  }

  /**
   * Init on DOM ready
   */
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
      initStaggerAnimations();
      initScrollReveal();
    });
  } else {
    initStaggerAnimations();
    initScrollReveal();
  }

})();
```

**Fichier:** `static/js/theme-toggle.js`

```javascript
/**
 * Dark Mode Toggle (localStorage)
 */
(function() {
  'use strict';

  const STORAGE_KEY = 'nexus-theme';
  const themeToggle = document.getElementById('theme-toggle');

  if (!themeToggle) return;

  // Load saved theme
  const savedTheme = localStorage.getItem(STORAGE_KEY) || 'light';
  document.documentElement.classList.toggle('dark', savedTheme === 'dark');

  // Toggle theme
  themeToggle.addEventListener('click', () => {
    const isDark = document.documentElement.classList.toggle('dark');
    localStorage.setItem(STORAGE_KEY, isDark ? 'dark' : 'light');

    console.log(`🎨 Theme switched to ${isDark ? 'dark' : 'light'} mode`);
  });

  console.log(`✅ Theme toggle initialized (current: ${savedTheme})`);
})();
```

---

## Phase 9: Animations Tor-Safe

### Problèmes OPSEC Évités

❌ **Interdits (Fingerprinting):**
- Canvas API (getContext('2d'), WebGL)
- Three.js, GSAP (heavy libraries)
- Performance.now() timing
- Device detection (GPU, screen size unique)

✅ **Autorisés (Tor-safe):**
- CSS transitions/animations
- Intersection Observer
- Stagger delays (fixed timing)
- SVG animations
- Alpine.js dropdowns

### Checklist Animations

- [x] Fade-in (0.5s ease-out)
- [x] Slide-up (0.5s ease-out)
- [x] Hover scale (0.3s)
- [x] Group hover (image zoom)
- [x] Stagger delays (100ms increment)
- [x] Pulse slow (4s infinite)
- [x] Accordion (0.2s)
- [x] Spinner (0.6s linear)
- [x] prefers-reduced-motion support
- [x] No Canvas/WebGL
- [x] No performance.now()
- [x] Timing constants (pas de random)

---

## Phase 10: Testing

### 10.1 Visual Comparison

```bash
# Terminal 1: LASTDESIGN2 (React)
cd LASTDESIGN2
npm run dev  # Port 5173

# Terminal 2: NEXUS (Rust)
cd server
cargo run    # Port 8080

# Browser: Ouvrir côte-à-côte
http://localhost:5173  (React original)
http://localhost:8080  (Rust migration)
```

### 10.2 Checklist Fonctionnel

**Pages:**
- [ ] Homepage (Hero + Featured + Categories)
- [ ] Categories (liste filtrable)
- [ ] Search (modal Ctrl+K + live results)
- [ ] Cart (add/remove HTMX)
- [ ] Checkout → Order + Escrow
- [ ] Order Detail (escrow status)
- [ ] Login/Register (forms)
- [ ] Become Vendor
- [ ] 404 Not Found

**Components:**
- [ ] Header (nav + search + cart + user menu)
- [ ] Footer (links)
- [ ] Product Cards (hover effects)
- [ ] Button variants (hero, outline, ghost)
- [ ] Input fields (validation)
- [ ] Modals (Alpine.js)
- [ ] Dropdowns (Alpine.js)
- [ ] Badges (status)

**Interactions:**
- [ ] Add to cart (HTMX)
- [ ] Remove from cart (HTMX)
- [ ] Live search (HTMX)
- [ ] Theme toggle (dark/light)
- [ ] User menu dropdown
- [ ] Search modal (Ctrl+K)
- [ ] Form validation
- [ ] HTMX indicators

**Escrow:**
- [ ] Create order → Initialize escrow
- [ ] Display multisig address
- [ ] Show confirmations (x/10)
- [ ] Buyer release funds
- [ ] Seller mark shipped
- [ ] Open dispute

---

### 10.3 Testing Tor

```bash
# Démarrer Tor
sudo systemctl start tor

# Configurer .onion (si hidden service)
# Tester via Tor Browser

# Vérifier:
- [ ] Pas de Canvas fingerprinting
- [ ] Animations fluides sur Tor (lent)
- [ ] Pas de leaks IP
- [ ] Images chargent (IPFS/static)
- [ ] HTMX fonctionne
- [ ] Forms submit OK
```

---

## Checklist Complète

### Fichiers à Créer

**Templates:**
```
✅ templates/base.html
✅ templates/pages/index.html
✅ templates/pages/categories.html
✅ templates/pages/search.html
✅ templates/pages/cart.html
✅ templates/pages/become-vendor.html
✅ templates/pages/auth/login.html
✅ templates/pages/auth/register.html
✅ templates/pages/errors/404.html
✅ templates/components/header.html
✅ templates/components/footer.html
✅ templates/components/hero.html
✅ templates/components/trust-badges.html
✅ templates/components/categories.html
✅ templates/components/featured-products.html
✅ templates/components/how-it-works.html
✅ templates/components/search-modal.html
✅ templates/components/search-results.html
✅ templates/components/ui/button.html
✅ templates/components/ui/card.html
✅ templates/components/ui/input.html
✅ templates/components/ui/badge.html
✅ templates/components/ui/alert.html
✅ templates/components/ui/dialog.html
```

**CSS:**
```
✅ static/css/lastdesign2-variables.css
✅ static/css/lastdesign2-animations.css
✅ static/css/lastdesign2-components.css (optionnel)
```

**JavaScript:**
```
✅ static/js/lastdesign2-animations.js
✅ static/js/theme-toggle.js
✅ static/js/htmx.min.js (si pas déjà présent)
```

**Rust Handlers:**
```
✅ server/src/handlers/frontend.rs (adapter routes)
✅ server/src/handlers/listings.rs (adapter contexts)
✅ server/src/handlers/cart.rs (HTMX endpoints)
✅ server/src/handlers/search.rs (live search)
✅ server/src/handlers/orders.rs (checkout + escrow)
✅ server/src/handlers/vendor.rs (become vendor)
```

**Routes Actix-Web:**
```rust
// server/src/main.rs
App::new()
  // Pages
  .route("/", web::get().to(frontend::index))
  .route("/categories", web::get().to(listings::categories))
  .route("/search", web::get().to(search::index))
  .route("/cart", web::get().to(cart::show))
  .route("/vendors/become", web::get().to(vendor::become))
  .route("/login", web::get().to(auth::login_form))
  .route("/register", web::get().to(auth::register_form))

  // API HTMX
  .route("/api/search", web::get().to(search::api_search))
  .route("/api/cart/add", web::post().to(cart::add))
  .route("/api/cart/{id}", web::delete().to(cart::remove))
  .route("/api/cart/{id}/increase", web::post().to(cart::increase))
  .route("/api/cart/{id}/decrease", web::post().to(cart::decrease))
  .route("/api/orders/create", web::post().to(orders::create_from_cart))
  .route("/api/orders/{id}/release", web::post().to(orders::release_escrow))
  .route("/api/orders/{id}/mark-shipped", web::post().to(orders::mark_shipped))
```

---

## Estimation Temps

| Phase | Tâche | Durée |
|-------|-------|-------|
| 0 | Audit backend NEXUS | 2h |
| 1 | Mapping données | 2h |
| 2 | Setup infrastructure | 2h |
| 3 | Hero + Header + Footer | 4h |
| 3 | Featured Products | 2h |
| 3 | Categories + How It Works | 2h |
| 4 | Macros UI (Button, Card, Input...) | 3h |
| 5 | Handlers frontend (contexts) | 3h |
| 6 | HTMX interactions (cart, search) | 4h |
| 7 | Intégration escrow (checkout, orders) | 5h |
| 8 | Design system CSS | 2h |
| 9 | Animations JS | 2h |
| 10 | Testing + debug | 4h |
| 11 | Pages secondaires (Auth, Vendor, 404) | 3h |
| 12 | Polish + responsive | 2h |
| **TOTAL** | | **~42h** |

---

## Notes Importantes

### Différences React vs Tera

1. **State Management:**
   - React: useState, useEffect
   - Tera: Backend queries → Context

2. **Routing:**
   - React: React Router (client-side)
   - Tera: Actix-Web routes (server-side)

3. **Forms:**
   - React: React Hook Form + Zod
   - Tera: HTML5 validation + Rust validator

4. **Interactions:**
   - React: onClick handlers
   - Tera: HTMX attributes (hx-get, hx-post)

5. **Modals:**
   - React: Radix Dialog
   - Tera: Alpine.js x-data

---

## Commandes Utiles

```bash
# Development
cargo watch -x "run --package server"

# Build release
cargo build --release --package server

# Run tests
cargo test --workspace

# Check security
./scripts/audit-pragmatic.sh

# Database migration
diesel migration run

# Format
cargo fmt --workspace

# Lint
cargo clippy --workspace -- -D warnings
```

---

## Ressources

- **LASTDESIGN2 Source:** `LASTDESIGN2/src/`
- **Shadcn/ui Docs:** https://ui.shadcn.com/
- **HTMX Docs:** https://htmx.org/
- **Alpine.js Docs:** https://alpinejs.dev/
- **Tailwind CSS:** https://tailwindcss.com/
- **Tera Templates:** https://keats.github.io/tera/

---

**Créé le:** 2025-10-30
**Dernière mise à jour:** 2025-10-30
**Status:** Ready for Implementation

---

**Prochaines Étapes:**

1. Créer structure `templates/` et fichiers de base
2. Copier CSS LASTDESIGN2 dans `static/css/`
3. Créer macros UI (Button, Card, Input)
4. Implémenter Hero + FeaturedProducts
5. Tester avec données DB réelles
6. Continuer page par page...

**Bon courage pour l'implémentation ! 🚀**
