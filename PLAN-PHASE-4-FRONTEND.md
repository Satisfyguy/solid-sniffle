# PLAN PHASE 4 : FRONTEND HTMX (15 jours)

**Projet:** Monero Marketplace
**Mission:** Frontend HTMX avec design premium dark glassmorphism
**Durée:** 15 jours (5 milestones)
**Score Cible:** 95/100 (Production-Ready)

---

## 🎯 VUE D'ENSEMBLE

**Objectif:** Implémenter interface utilisateur complète avec :
- **Design Premium:** Dark theme, glassmorphism, gradients (2025 aesthetic)
- **HTMX:** Zero full page reload, lazy loading, live search
- **Sécurité:** CSRF tokens, input validation, CSP headers, XSS protection
- **Simplicité:** Structure simple, un seul fichier CSS, pas de frameworks JS

**Architecture:**
- **Backend:** Actix-Web (Rust) - déjà configuré
- **Templates:** Tera (déjà initialisé dans `server/src/main.rs:103`)
- **HTMX:** v1.9.10 CDN (déjà dans `templates/base.html:10`)
- **Session:** Cookies HttpOnly + SameSite=Strict (déjà configuré)
- **WebSocket:** Actix-actors (déjà running)

---

## 📊 STATUT ACTUEL (Mise à jour: 2025-10-22)

**Infrastructure Existante ✅**
- [x] Tera template engine configuré
- [x] Session middleware (24h TTL, HttpOnly, SameSite=Strict)
- [x] Security headers (CSP, X-Frame-Options, X-XSS-Protection)
- [x] Rate limiting (global 100/min, auth 5/15min, protected 60/min)
- [x] Static files serving (`/static/`)
- [x] WebSocket server running
- [x] Frontend handlers (`server/src/handlers/frontend.rs`)
- [x] Templates basiques (base.html, login.html, register.html, listings/index.html)

**API Endpoints Disponibles ✅**
- Auth: `/api/auth/register`, `/api/auth/login`, `/api/auth/logout`, `/api/auth/whoami`
- Listings: `/api/listings` (GET/POST), `/api/listings/{id}` (GET/PUT/DELETE), `/api/listings/search`
- Orders: `/api/orders` (GET/POST), `/api/orders/{id}` (GET/PUT), `/api/orders/{id}/ship|complete|cancel|dispute`
- Escrow: `/api/escrow/{id}` (GET), `/api/escrow/{id}/prepare|release|refund|dispute|resolve`

**Ce qui est Complété ✅ (Milestones 4.1-4.5)**
- [x] Design premium CSS (thème clair minimaliste - demande utilisateur)
- [x] Templates complets (11 fichiers HTML)
- [x] HTMX interactivité (9 attributs, live search avec debounce)
- [x] WebSocket integration pour escrow real-time
- [x] Accessibility (WCAG 2.1 Level AA: skip link, 8 ARIA labels, 4 roles)
- [x] Animations CSS (5 types: fadeIn, slideIn, slideUp, scaleIn, shimmer)

**Métriques Frontend Actuelles:**
- Templates HTML: 11 fichiers
- CSS: 1014 lignes (21KB)
- HTMX Attributes: 9 types
- ARIA Labels: 8
- ARIA Roles: 4
- Formulaires: 5
- Animations CSS: 5

**Ce qui Manque AVANT PRODUCTION ❌**
- [ ] **BLOCKER:** Module csrf.rs (non créé) - 15 min
- [ ] **CRITICAL:** CSP allow HTMX CDN - 10 min
- [ ] **CRITICAL:** Fix XSS listings/show.html:196 - 5 min
- [ ] Fix search dual-mode (HTML + JSON) - 30 min
- [ ] Skip-link CSS styles - 5 min
- [ ] prefers-reduced-motion media query - 5 min
- [ ] HTTP caching/compression - 10 min

**Score Production-Ready:** 73/100 (Cible: 85+)
**Temps estimé pour production-ready:** 1h22min

---

## 📋 MILESTONE 4.1 : SETUP FRONTEND (3 jours)

### Objectif
Configuration Tera + HTMX + Structure + Premium CSS

### ✅ TÂCHES

#### Tâche 1.1 : Vérifier Dépendances
**Fichier:** `server/Cargo.toml`

**Vérifier présence :**
```toml
[dependencies]
# Template engine
tera = "1.19"

# Static file serving
actix-files = "0.6"

# Session cookies (déjà présent)
actix-session = { version = "0.9", features = ["cookie-session"] }
```

**Si manquant, ajouter.**

#### Tâche 1.2 : Structure Dossiers
**Créer structure simple :**
```
templates/
├── base.html (✅ existe - à mettre à jour)
├── partials/
│   ├── header.html (✅ existe - à mettre à jour)
│   └── footer.html (✅ existe)
├── auth/
│   ├── login.html (✅ existe - à styler)
│   └── register.html (✅ existe - à styler)
├── listings/
│   ├── index.html (✅ existe - à améliorer)
│   ├── show.html ⭕ NOUVEAU
│   └── create.html ⭕ NOUVEAU
├── orders/
│   ├── index.html ⭕ NOUVEAU
│   └── show.html ⭕ NOUVEAU
└── escrow/
    └── show.html ⭕ NOUVEAU

static/
├── css/
│   └── main.css ⭕ NOUVEAU (design premium)
└── js/
    └── (vide - HTMX via CDN)
```

**Pas de dossier `fragments/`, pas de `htmx.rs` séparé - TOUT dans `frontend.rs`**

#### Tâche 1.3 : Premium CSS - Design System
**Fichier:** `static/css/main.css`

**Créer avec design premium dark glassmorphism :**

```css
/* ========================================
   MONERO MARKETPLACE - PREMIUM DARK THEME
   Design: 2025 Glassmorphism Aesthetic
   ======================================== */

/* ===== VARIABLES (Design Tokens) ===== */
:root {
  /* Primary Colors */
  --primary-dark: #0a0e27;
  --primary-accent: #6366f1;
  --primary-accent-hover: #4f46e5;

  /* Neutral Colors */
  --surface: #1a1d35;
  --surface-light: #2a2d45;
  --text-primary: #ffffff;
  --text-secondary: #94a3b8;
  --border: rgba(255, 255, 255, 0.1);

  /* Semantic Colors */
  --success: #10b981;
  --warning: #f59e0b;
  --error: #ef4444;
  --info: #3b82f6;

  /* Monero Brand */
  --monero-orange: #ff6600;

  /* Typography */
  --font-sans: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto,
               "Helvetica Neue", Arial, sans-serif;
  --font-mono: "SF Mono", Monaco, "Cascadia Code", Consolas, monospace;

  /* Spacing */
  --spacing-1: 0.25rem;
  --spacing-2: 0.5rem;
  --spacing-3: 0.75rem;
  --spacing-4: 1rem;
  --spacing-5: 1.5rem;
  --spacing-6: 2rem;
  --spacing-8: 3rem;

  /* Shadows */
  --shadow-sm: 0 2px 8px rgba(0, 0, 0, 0.15);
  --shadow-md: 0 4px 12px rgba(99, 102, 241, 0.2);
  --shadow-lg: 0 8px 24px rgba(99, 102, 241, 0.3);
}

/* ===== RESET & BASE ===== */
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: var(--font-sans);
  line-height: 1.6;
  color: var(--text-primary);
  background: var(--primary-dark);
  background-image: radial-gradient(circle at 20% 50%, rgba(99, 102, 241, 0.1) 0%, transparent 50%),
                    radial-gradient(circle at 80% 80%, rgba(255, 102, 0, 0.05) 0%, transparent 50%);
  background-attachment: fixed;
}

.container {
  max-width: 1200px;
  margin: 0 auto;
  padding: var(--spacing-6);
}

/* ===== HEADER (Sticky with Blur) ===== */
header {
  position: sticky;
  top: 0;
  z-index: 1000;
  background: rgba(10, 14, 39, 0.95);
  backdrop-filter: blur(20px);
  border-bottom: 1px solid var(--border);
  padding: var(--spacing-4) 0;
}

header nav {
  display: flex;
  justify-content: space-between;
  align-items: center;
  max-width: 1200px;
  margin: 0 auto;
  padding: 0 var(--spacing-6);
}

header .logo {
  font-size: 1.5rem;
  font-weight: 700;
  color: var(--monero-orange);
  text-decoration: none;
}

header .nav-links {
  display: flex;
  gap: var(--spacing-5);
  list-style: none;
}

header .nav-links a {
  color: var(--text-secondary);
  text-decoration: none;
  transition: color 0.2s ease;
  font-weight: 500;
}

header .nav-links a:hover {
  color: var(--text-primary);
}

/* ===== GLASSMORPHISM CARDS ===== */
.card {
  background: rgba(26, 29, 53, 0.6);
  backdrop-filter: blur(10px);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: var(--spacing-5);
  transition: all 0.3s ease;
}

.card:hover {
  background: rgba(42, 45, 69, 0.8);
  border-color: var(--primary-accent);
  transform: translateY(-2px);
  box-shadow: var(--shadow-md);
}

/* ===== BUTTONS (Premium Gradients) ===== */
.btn {
  display: inline-block;
  padding: var(--spacing-3) var(--spacing-6);
  border-radius: 8px;
  font-weight: 600;
  text-decoration: none;
  border: none;
  cursor: pointer;
  transition: all 0.2s ease;
  font-size: 1rem;
}

.btn-primary {
  background: linear-gradient(135deg, var(--primary-accent), var(--primary-accent-hover));
  color: var(--text-primary);
}

.btn-primary:hover {
  transform: translateY(-1px);
  box-shadow: var(--shadow-md);
}

.btn-success {
  background: linear-gradient(135deg, var(--success), #059669);
  color: white;
}

.btn-warning {
  background: linear-gradient(135deg, var(--warning), #d97706);
  color: white;
}

.btn-danger {
  background: linear-gradient(135deg, var(--error), #dc2626);
  color: white;
}

/* ===== FORMS ===== */
.form-group {
  margin-bottom: var(--spacing-4);
}

.form-group label {
  display: block;
  margin-bottom: var(--spacing-2);
  font-weight: 500;
  color: var(--text-secondary);
}

.form-group input,
.form-group textarea,
.form-group select {
  width: 100%;
  padding: var(--spacing-3);
  background: rgba(26, 29, 53, 0.8);
  border: 1px solid var(--border);
  border-radius: 8px;
  color: var(--text-primary);
  font-size: 1rem;
  font-family: var(--font-sans);
  transition: all 0.2s ease;
}

.form-group input:focus,
.form-group textarea:focus,
.form-group select:focus {
  outline: none;
  border-color: var(--primary-accent);
  box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.1);
}

/* ===== LISTINGS GRID ===== */
.listings-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: var(--spacing-5);
  margin-top: var(--spacing-6);
}

.listing-card {
  background: rgba(26, 29, 53, 0.6);
  backdrop-filter: blur(10px);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: var(--spacing-5);
  transition: all 0.3s ease;
}

.listing-card:hover {
  background: rgba(42, 45, 69, 0.8);
  border-color: var(--primary-accent);
  transform: translateY(-4px);
  box-shadow: var(--shadow-lg);
}

.listing-card h3 {
  color: var(--text-primary);
  margin-bottom: var(--spacing-3);
  font-size: 1.25rem;
}

.listing-card .description {
  color: var(--text-secondary);
  margin-bottom: var(--spacing-4);
  line-height: 1.5;
}

.listing-card .price {
  font-size: 1.5rem;
  color: var(--monero-orange);
  font-weight: 700;
  margin-bottom: var(--spacing-2);
  font-family: var(--font-mono);
}

.listing-card .stock {
  color: var(--text-secondary);
  font-size: 0.875rem;
  margin-bottom: var(--spacing-3);
}

/* ===== ESCROW TIMELINE ===== */
.timeline {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin: var(--spacing-8) 0;
  position: relative;
  padding: var(--spacing-4) 0;
}

.timeline::before {
  content: '';
  position: absolute;
  top: 50%;
  left: 0;
  right: 0;
  height: 2px;
  background: linear-gradient(90deg, var(--border) 0%, var(--primary-accent) 50%, var(--border) 100%);
  z-index: 0;
}

.step {
  background: rgba(26, 29, 53, 0.9);
  backdrop-filter: blur(10px);
  padding: var(--spacing-3) var(--spacing-4);
  border-radius: 20px;
  border: 2px solid var(--border);
  position: relative;
  z-index: 1;
  font-weight: 600;
  font-size: 0.875rem;
  transition: all 0.3s ease;
}

.step.active {
  border-color: var(--primary-accent);
  background: linear-gradient(135deg, var(--primary-accent), var(--primary-accent-hover));
  color: white;
  box-shadow: var(--shadow-md);
}

/* ===== STATUS BADGES ===== */
.status-badge {
  display: inline-block;
  padding: var(--spacing-2) var(--spacing-4);
  border-radius: 20px;
  font-weight: 600;
  font-size: 0.875rem;
  margin: var(--spacing-2) 0;
}

.status-created { background: var(--info); color: white; }
.status-ready { background: #9b59b6; color: white; }
.status-funded { background: var(--warning); color: white; }
.status-releasing { background: #e67e22; color: white; }
.status-completed { background: var(--success); color: white; }
.status-disputed { background: var(--error); color: white; }
.status-refunded { background: #6366f1; color: white; }

/* ===== SEARCH BAR ===== */
.search-bar {
  margin: var(--spacing-6) 0;
}

.search-bar input {
  width: 100%;
  max-width: 600px;
  padding: var(--spacing-4);
  background: rgba(26, 29, 53, 0.8);
  border: 1px solid var(--border);
  border-radius: 12px;
  color: var(--text-primary);
  font-size: 1rem;
  transition: all 0.2s ease;
}

.search-bar input:focus {
  outline: none;
  border-color: var(--primary-accent);
  box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.1);
}

/* ===== LOADING INDICATOR ===== */
.htmx-indicator {
  display: inline-block;
  width: 20px;
  height: 20px;
  border: 3px solid var(--border);
  border-top-color: var(--primary-accent);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* ===== RESPONSIVE (Mobile First) ===== */
@media (max-width: 768px) {
  .container {
    padding: var(--spacing-4);
  }

  header nav {
    flex-direction: column;
    gap: var(--spacing-4);
    padding: var(--spacing-4);
  }

  .listings-grid {
    grid-template-columns: 1fr;
  }

  .timeline {
    flex-direction: column;
    gap: var(--spacing-3);
  }

  .timeline::before {
    display: none;
  }
}

/* ===== FOOTER ===== */
footer {
  margin-top: var(--spacing-8);
  padding: var(--spacing-6);
  text-align: center;
  color: var(--text-secondary);
  border-top: 1px solid var(--border);
}

footer a {
  color: var(--primary-accent);
  text-decoration: none;
}

footer a:hover {
  text-decoration: underline;
}
```

#### Tâche 1.4 : Mettre à Jour base.html avec Premium Design
**Fichier:** `templates/base.html`

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta http-equiv="X-UA-Compatible" content="ie=edge">
    <title>{% block title %}Monero Marketplace{% endblock %}</title>

    <!-- Premium CSS -->
    <link rel="stylesheet" href="/static/css/main.css">

    <!-- HTMX for dynamic interactions -->
    <script src="https://unpkg.com/htmx.org@1.9.10"
            integrity="sha384-D1Kt99CQMDuVetoL1lrYwg5t+9QdHe7NLX/SoJYkXDFfX37iInKRy5xLSi8nO7UC"
            crossorigin="anonymous"></script>
</head>
<body>
    {% include "partials/header.html" %}

    <main class="container">
        {% block content %}{% endblock %}
    </main>

    {% include "partials/footer.html" %}
</body>
</html>
```

#### Tâche 1.5 : Mettre à Jour Header avec Premium Styling
**Fichier:** `templates/partials/header.html`

```html
<header>
    <nav>
        <a href="/" class="logo">⟠ Monero Marketplace</a>

        <ul class="nav-links">
            <li><a href="/">Listings</a></li>
            {% if logged_in %}
                <li><a href="/orders">My Orders</a></li>
                {% if role == "vendor" %}
                    <li><a href="/listings/new">Create Listing</a></li>
                {% endif %}
                <li><a href="/profile">{{ username }}</a></li>
                <li>
                    <form action="/logout" method="POST" style="display: inline;">
                        <button type="submit" class="btn btn-primary">Logout</button>
                    </form>
                </li>
            {% else %}
                <li><a href="/login">Login</a></li>
                <li><a href="/register">Register</a></li>
            {% endif %}
        </ul>
    </nav>
</header>
```

#### Tâche 1.6 : **SÉCURITÉ** - Mettre à Jour CSP pour HTMX
**Fichier:** `server/src/middleware/security_headers.rs`

**Modifier ligne 106 :**
```rust
headers.insert(
    actix_web::http::header::HeaderName::from_static("content-security-policy"),
    actix_web::http::header::HeaderValue::from_static(
        "default-src 'self'; \
         script-src 'self' https://unpkg.com/htmx.org@1.9.10; \
         style-src 'self' 'unsafe-inline'; \
         img-src 'self' data:; \
         font-src 'self'; \
         connect-src 'self' ws://127.0.0.1:8080; \
         frame-ancestors 'none'; \
         base-uri 'self'; \
         form-action 'self'",
    ),
);
```

**CRITICAL:** Autoriser HTMX CDN + WebSocket dans CSP

### ✅ VALIDATION MILESTONE 1

**Tests compilation :**
```bash
cargo build --package server
```

**Tests visuels :**
```bash
cargo run --package server
# Visiter http://localhost:8080/
# Vérifier premium design (dark theme, glassmorphism)
```

**Critères d'acceptance :**
- [ ] Tera configuré et compile sans erreur
- [ ] Structure `templates/` + `static/` créée
- [ ] `static/css/main.css` créé avec premium design
- [ ] base.html mis à jour
- [ ] Header premium avec navigation
- [ ] CSP mis à jour pour HTMX + WebSocket
- [ ] Page d'accueil affiche dark theme avec glassmorphism
- [ ] Aucun .unwrap() dans le code

**Durée:** 3 jours
**Score:** 30/100

---

## 📋 MILESTONE 4.2 : AUTHENTICATION FRONTEND (3 jours)

### Objectif
Pages login/register avec premium styling + HTMX + **SÉCURITÉ**

### ✅ TÂCHES

#### Tâche 2.1 : Template Login Premium
**Fichier:** `templates/auth/login.html`

```html
{% extends "base.html" %}

{% block title %}Login - Monero Marketplace{% endblock %}

{% block content %}
<div class="auth-container">
    <div class="card" style="max-width: 500px; margin: 4rem auto;">
        <h1 style="margin-bottom: 2rem; text-align: center;">Login</h1>

        <form
            hx-post="/api/auth/login"
            hx-target="#auth-result"
            hx-swap="innerHTML"
            hx-indicator=".htmx-indicator"
        >
            <div class="form-group">
                <label for="username">Username</label>
                <input type="text" id="username" name="username" required minlength="3">
            </div>

            <div class="form-group">
                <label for="password">Password</label>
                <input type="password" id="password" name="password" required minlength="8">
            </div>

            <button type="submit" class="btn btn-primary" style="width: 100%;">
                Login <span class="htmx-indicator" style="margin-left: 0.5rem;"></span>
            </button>
        </form>

        <div id="auth-result" style="margin-top: 1rem;"></div>

        <p style="margin-top: 2rem; text-align: center; color: var(--text-secondary);">
            Don't have an account? <a href="/register" style="color: var(--primary-accent);">Register here</a>
        </p>
    </div>
</div>
{% endblock %}
```

#### Tâche 2.2 : Template Register Premium
**Fichier:** `templates/auth/register.html`

```html
{% extends "base.html" %}

{% block title %}Register - Monero Marketplace{% endblock %}

{% block content %}
<div class="auth-container">
    <div class="card" style="max-width: 500px; margin: 4rem auto;">
        <h1 style="margin-bottom: 2rem; text-align: center;">Create Account</h1>

        <form
            hx-post="/api/auth/register"
            hx-target="#auth-result"
            hx-swap="innerHTML"
            hx-indicator=".htmx-indicator"
        >
            <div class="form-group">
                <label for="username">Username</label>
                <input type="text" id="username" name="username" required minlength="3" maxlength="50">
                <small style="color: var(--text-secondary);">3-50 characters</small>
            </div>

            <div class="form-group">
                <label for="password">Password</label>
                <input type="password" id="password" name="password" required minlength="8">
                <small style="color: var(--text-secondary);">Minimum 8 characters</small>
            </div>

            <div class="form-group">
                <label for="role">Role</label>
                <select id="role" name="role" required>
                    <option value="">-- Select Role --</option>
                    <option value="buyer">Buyer</option>
                    <option value="vendor">Vendor</option>
                </select>
            </div>

            <button type="submit" class="btn btn-primary" style="width: 100%;">
                Register <span class="htmx-indicator" style="margin-left: 0.5rem;"></span>
            </button>
        </form>

        <div id="auth-result" style="margin-top: 1rem;"></div>

        <p style="margin-top: 2rem; text-align: center; color: var(--text-secondary);">
            Already have an account? <a href="/login" style="color: var(--primary-accent);">Login here</a>
        </p>
    </div>
</div>
{% endblock %}
```

#### Tâche 2.3 : **SÉCURITÉ** - Modifier Handlers Auth pour HTMX Response
**Fichier:** `server/src/handlers/auth.rs`

**Ajouter fonction helper en haut du fichier :**
```rust
use actix_web::http::header;

/// Détecte si la requête vient de HTMX
fn is_htmx_request(req: &HttpRequest) -> bool {
    req.headers()
        .get("HX-Request")
        .and_then(|h| h.to_str().ok())
        .map(|v| v == "true")
        .unwrap_or(false)
}
```

**Modifier handler `login` (chercher `#[post("/login")]`) :**
```rust
#[post("/login")]
pub async fn login(
    req: HttpRequest,  // Ajouter ce paramètre
    pool: web::Data<DbPool>,
    session: Session,
    credentials: web::Json<LoginRequest>,
) -> impl Responder {
    // ... validation existante ...

    // Après succès du login:
    session.insert("user_id", &user.id.clone()).ok();
    session.insert("username", &user.username).ok();
    session.insert("role", &user.role).ok();

    // Détecter si HTMX
    if is_htmx_request(&req) {
        // Response HTML pour HTMX
        return HttpResponse::Ok()
            .content_type("text/html")
            .insert_header(("HX-Redirect", "/"))
            .body(format!(
                "<div class='success' style='color: var(--success);'>✓ Welcome back, {}!</div>",
                user.username
            ));
    }

    // Response JSON classique
    HttpResponse::Ok().json(LoginResponse {
        message: "Login successful".to_string(),
        user: UserInfo {
            id: user.id,
            username: user.username,
            role: user.role,
        },
    })
}
```

**Faire pareil pour `register` handler.**

#### Tâche 2.4 : **SÉCURITÉ** - Input Validation
**Vérifier que `validator` crate est utilisé dans handlers auth.**

**Dans `server/src/handlers/auth.rs`, vérifier présence de :**
```rust
#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: String,

    #[validate(length(min = 8))]
    pub password: String,
}
```

**Si absent, ajouter.**

### ✅ VALIDATION MILESTONE 2

**Tests manuels :**
```bash
cargo run --package server

# Visiter http://localhost:8080/login
# Vérifier:
# - Premium dark design
# - Glassmorphism card
# - Form validation frontend (minlength)
# - HTMX login (pas de page reload)
# - Redirect vers / après login
# - Message success affiché
```

**Tests sécurité :**
```bash
# 1. XSS Test
# Essayer login avec username: <script>alert('xss')</script>
# → Doit être échappé automatiquement par Tera

# 2. Rate Limiting
# Essayer 6 logins en 15min
# → Doit bloquer après 5 tentatives

# 3. SQL Injection
# Essayer username: ' OR '1'='1
# → Diesel ORM protège automatiquement
```

**Critères d'acceptance :**
- [ ] Page /login avec premium design
- [ ] Page /register avec premium design
- [ ] HTMX login fonctionne (pas de reload)
- [ ] HTMX register fonctionne
- [ ] Session cookies persistés (HttpOnly + SameSite=Strict)
- [ ] Input validation côté serveur (`validator` crate)
- [ ] XSS protection (Tera auto-escape)
- [ ] Rate limiting actif (5 req/15min)
- [ ] Aucun .unwrap() dans le code

**Durée:** 3 jours
**Score:** 45/100

---

## 📋 MILESTONE 4.3 : LISTINGS FRONTEND (3 jours)

### Objectif
Interface browse/search/create listings avec HTMX + **SÉCURITÉ**

### ✅ TÂCHES

#### Tâche 3.1 : Listings Index Premium avec HTMX Search
**Fichier:** `templates/listings/index.html`

```html
{% extends "base.html" %}

{% block title %}Browse Listings - Monero Marketplace{% endblock %}

{% block content %}
<div class="listings-container">
    <h1 style="margin-bottom: 2rem;">Browse Listings</h1>

    <!-- Search bar avec HTMX debounce -->
    <div class="search-bar">
        <input
            type="search"
            name="query"
            placeholder="Search listings..."
            hx-get="/api/listings/search"
            hx-trigger="keyup changed delay:500ms"
            hx-target="#listings-results"
            hx-swap="innerHTML"
            hx-indicator=".search-spinner"
        >
        <span class="htmx-indicator search-spinner" style="margin-left: 1rem;"></span>
    </div>

    {% if logged_in and role == "vendor" %}
    <div style="margin: 2rem 0;">
        <a href="/listings/new" class="btn btn-success">+ Create New Listing</a>
    </div>
    {% endif %}

    <!-- Listings grid -->
    <div id="listings-results" class="listings-grid">
        {% if listings %}
            {% for listing in listings %}
            <div class="listing-card">
                <h3>{{ listing.title }}</h3>
                <p class="description">{{ listing.description | truncate(length=150) }}</p>
                <p class="price">{{ listing.price_display }}</p>
                <p class="stock">Stock: {{ listing.stock }}</p>
                <span class="status-badge status-{{ listing.status }}">{{ listing.status | upper }}</span>
                <div style="margin-top: 1rem;">
                    <a href="/listings/{{ listing.id }}" class="btn btn-primary">View Details</a>
                </div>
            </div>
            {% endfor %}
        {% else %}
            <p style="color: var(--text-secondary); text-align: center; padding: 4rem;">
                No listings available.
                {% if logged_in and role == "vendor" %}
                    <a href="/listings/new" style="color: var(--primary-accent);">Create the first one!</a>
                {% endif %}
            </p>
        {% endif %}
    </div>
</div>
{% endblock %}
```

#### Tâche 3.2 : Listing Detail Page
**Fichier:** `templates/listings/show.html`

```html
{% extends "base.html" %}

{% block title %}{{ listing.title }} - Monero Marketplace{% endblock %}

{% block content %}
<div class="listing-detail">
    <div class="card" style="max-width: 800px; margin: 2rem auto;">
        <h1 style="margin-bottom: 1.5rem;">{{ listing.title }}</h1>

        <span class="status-badge status-{{ listing.status }}">{{ listing.status | upper }}</span>

        <p class="price" style="margin: 1.5rem 0;">{{ listing.price_display }}</p>

        <div style="margin: 2rem 0;">
            <h3 style="color: var(--text-secondary); margin-bottom: 1rem;">Description</h3>
            <p style="line-height: 1.8; color: var(--text-secondary);">{{ listing.description }}</p>
        </div>

        <div style="display: flex; gap: 2rem; margin: 2rem 0;">
            <div>
                <strong style="color: var(--text-secondary);">Stock:</strong> {{ listing.stock }}
            </div>
            <div>
                <strong style="color: var(--text-secondary);">Vendor:</strong> {{ vendor.username }}
            </div>
        </div>

        {% if logged_in and role == "buyer" and listing.status == "active" and listing.stock > 0 %}
        <div style="margin-top: 2rem; padding-top: 2rem; border-top: 1px solid var(--border);">
            <h3 style="margin-bottom: 1rem;">Place Order</h3>
            <form
                hx-post="/api/orders"
                hx-target="#order-result"
                hx-swap="innerHTML"
            >
                <input type="hidden" name="listing_id" value="{{ listing.id }}">

                <div class="form-group">
                    <label for="quantity">Quantity</label>
                    <input type="number" id="quantity" name="quantity" min="1" max="{{ listing.stock }}" value="1" required>
                </div>

                <button type="submit" class="btn btn-success" style="width: 100%;">Create Order</button>
            </form>

            <div id="order-result" style="margin-top: 1rem;"></div>
        </div>
        {% endif %}

        {% if logged_in and user_id == listing.vendor_id %}
        <div style="margin-top: 2rem; display: flex; gap: 1rem;">
            <a href="/listings/{{ listing.id }}/edit" class="btn btn-primary">Edit Listing</a>
            <button
                hx-delete="/api/listings/{{ listing.id }}"
                hx-confirm="Are you sure you want to delete this listing?"
                hx-target="body"
                class="btn btn-danger"
            >
                Delete
            </button>
        </div>
        {% endif %}
    </div>
</div>
{% endblock %}
```

#### Tâche 3.3 : Create Listing Form (Vendor Only)
**Fichier:** `templates/listings/create.html`

```html
{% extends "base.html" %}

{% block title %}Create Listing - Monero Marketplace{% endblock %}

{% block content %}
<div class="listing-form-container">
    <div class="card" style="max-width: 700px; margin: 2rem auto;">
        <h1 style="margin-bottom: 2rem;">Create New Listing</h1>

        <form
            hx-post="/api/listings"
            hx-target="#listing-result"
            hx-swap="innerHTML"
        >
            <div class="form-group">
                <label for="title">Title</label>
                <input type="text" id="title" name="title" required minlength="3" maxlength="200">
                <small style="color: var(--text-secondary);">3-200 characters</small>
            </div>

            <div class="form-group">
                <label for="description">Description</label>
                <textarea id="description" name="description" required minlength="10" maxlength="5000" rows="6"></textarea>
                <small style="color: var(--text-secondary);">10-5000 characters</small>
            </div>

            <div class="form-group">
                <label for="price_xmr">Price (atomic units)</label>
                <input type="number" id="price_xmr" name="price_xmr" min="1" required>
                <small style="color: var(--text-secondary);">1 XMR = 1,000,000,000,000 atomic units</small>
            </div>

            <div class="form-group">
                <label for="stock">Stock</label>
                <input type="number" id="stock" name="stock" min="0" required>
            </div>

            <button type="submit" class="btn btn-success" style="width: 100%;">Create Listing</button>
        </form>

        <div id="listing-result" style="margin-top: 1.5rem;"></div>
    </div>
</div>
{% endblock %}
```

#### Tâche 3.4 : **SÉCURITÉ** - Frontend Handlers avec Authorization
**Fichier:** `server/src/handlers/frontend.rs`

**Ajouter handlers :**
```rust
use actix_session::Session;
use actix_web::{web, HttpResponse, Responder};
use tera::{Context, Tera};
use tracing::{error, info};
use uuid::Uuid;

use crate::db::DbPool;
use crate::models::listing::Listing;
use crate::models::user::User;

/// GET /listings/{id} - Listing detail page
pub async fn show_listing(
    tera: web::Data<Tera>,
    pool: web::Data<DbPool>,
    session: Session,
    path: web::Path<String>,
) -> impl Responder {
    let listing_id_str = path.into_inner();

    // Load listing from DB
    let listing = match db_get_listing(&pool, &listing_id_str).await {
        Ok(l) => l,
        Err(e) => {
            error!("Failed to load listing {}: {}", listing_id_str, e);
            return HttpResponse::NotFound().body("Listing not found");
        }
    };

    // Load vendor info
    let vendor = match db_get_user(&pool, &listing.vendor_id).await {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to load vendor: {}", e);
            return HttpResponse::InternalServerError().body("Error loading vendor");
        }
    };

    let mut ctx = Context::new();
    ctx.insert("listing", &listing);
    ctx.insert("vendor", &vendor);

    // Check auth
    if let Ok(Some(username)) = session.get::<String>("username") {
        ctx.insert("username", &username);
        ctx.insert("logged_in", &true);

        if let Ok(Some(role)) = session.get::<String>("role") {
            ctx.insert("role", &role);
        }

        if let Ok(Some(user_id)) = session.get::<String>("user_id") {
            ctx.insert("user_id", &user_id);
        }
    } else {
        ctx.insert("logged_in", &false);
    }

    match tera.render("listings/show.html", &ctx) {
        Ok(html) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(html),
        Err(e) => {
            error!("Template error: {}", e);
            HttpResponse::InternalServerError().body(format!("Template error: {}", e))
        }
    }
}

/// GET /listings/new - Create listing form (VENDOR ONLY)
pub async fn show_create_listing(
    tera: web::Data<Tera>,
    session: Session,
) -> impl Responder {
    // SECURITY: Check auth + role
    let role = match session.get::<String>("role") {
        Ok(Some(r)) => r,
        _ => {
            return HttpResponse::Unauthorized()
                .body("You must be logged in as a vendor to create listings");
        }
    };

    if role != "vendor" {
        return HttpResponse::Forbidden()
            .body("Only vendors can create listings");
    }

    let ctx = Context::new();

    match tera.render("listings/create.html", &ctx) {
        Ok(html) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(html),
        Err(e) => {
            error!("Template error: {}", e);
            HttpResponse::InternalServerError().body(format!("Template error: {}", e))
        }
    }
}
```

#### Tâche 3.5 : Ajouter Routes dans main.rs
**Fichier:** `server/src/main.rs`

**Chercher la section frontend routes et ajouter :**
```rust
// Frontend routes (HTML pages)
.route("/", web::get().to(frontend::index))
.route("/login", web::get().to(frontend::show_login))
.route("/register", web::get().to(frontend::show_register))
.route("/logout", web::post().to(frontend::logout))
.route("/listings", web::get().to(frontend::show_listings))
.route("/listings/new", web::get().to(frontend::show_create_listing))  // NOUVEAU
.route("/listings/{id}", web::get().to(frontend::show_listing))        // NOUVEAU
```

### ✅ VALIDATION MILESTONE 3

**Tests manuels :**
```bash
cargo run --package server

# 1. Browse listings
http://localhost:8080/

# 2. Search HTMX (taper "laptop")
# → Vérifier debounce 500ms
# → Vérifier pas de page reload

# 3. View listing detail
http://localhost:8080/listings/{id}

# 4. Create listing (en tant que vendor)
http://localhost:8080/listings/new
# → Buyers doivent recevoir 403 Forbidden
```

**Tests sécurité :**
```bash
# 1. Authorization check
# Essayer /listings/new en tant que buyer
# → Doit retourner 403 Forbidden

# 2. XSS Test
# Créer listing avec title: <script>alert('xss')</script>
# → Doit être échappé dans HTML

# 3. Input validation
# Créer listing avec title: "ab" (trop court)
# → Doit être rejeté (validator crate)
```

**Critères d'acceptance :**
- [ ] Page /listings avec premium grid
- [ ] HTMX search avec debounce 500ms fonctionne
- [ ] Page /listings/{id} affiche détails
- [ ] Page /listings/new (vendor only)
- [ ] Authorization checks (vendor role pour create)
- [ ] Input validation côté serveur
- [ ] XSS protection (Tera auto-escape)
- [ ] Aucun .unwrap() dans le code

**Durée:** 3 jours
**Score:** 65/100

---

## 📋 MILESTONE 4.4 : ORDERS & ESCROW FRONTEND (4 jours)

### Objectif
Interface orders + escrow avec timeline + WebSocket + **SÉCURITÉ**

### ✅ TÂCHES

#### Tâche 4.1 : Orders Index
**Fichier:** `templates/orders/index.html`

```html
{% extends "base.html" %}

{% block title %}My Orders - Monero Marketplace{% endblock %}

{% block content %}
<div class="orders-container">
    <h1 style="margin-bottom: 2rem;">My Orders</h1>

    {% if orders %}
    <div style="display: flex; flex-direction: column; gap: 1.5rem;">
        {% for order in orders %}
        <div class="card">
            <div style="display: flex; justify-content: space-between; align-items: start;">
                <div>
                    <h3 style="margin-bottom: 0.5rem;">Order #{{ order.id | truncate(length=8) }}</h3>
                    <p style="color: var(--text-secondary);">{{ order.listing_title }}</p>
                </div>
                <span class="status-badge status-{{ order.status }}">{{ order.status | upper }}</span>
            </div>

            <div style="display: flex; gap: 2rem; margin: 1.5rem 0;">
                <div>
                    <strong style="color: var(--text-secondary);">Quantity:</strong> {{ order.quantity }}
                </div>
                <div>
                    <strong style="color: var(--text-secondary);">Total:</strong>
                    <span style="color: var(--monero-orange); font-family: var(--font-mono);">
                        {{ order.total_price_display }}
                    </span>
                </div>
            </div>

            <div style="margin-top: 1rem;">
                <a href="/orders/{{ order.id }}" class="btn btn-primary">View Details</a>
            </div>
        </div>
        {% endfor %}
    </div>
    {% else %}
    <p style="color: var(--text-secondary); text-align: center; padding: 4rem;">
        No orders yet. <a href="/" style="color: var(--primary-accent);">Browse listings</a>
    </p>
    {% endif %}
</div>
{% endblock %}
```

#### Tâche 4.2 : Order Detail avec Escrow Link
**Fichier:** `templates/orders/show.html`

```html
{% extends "base.html" %}

{% block title %}Order #{{ order.id | truncate(length=8) }} - Monero Marketplace{% endblock %}

{% block content %}
<div class="order-detail">
    <div class="card" style="max-width: 800px; margin: 2rem auto;">
        <h1 style="margin-bottom: 1rem;">Order #{{ order.id | truncate(length=8) }}</h1>

        <span class="status-badge status-{{ order.status }}">{{ order.status | upper }}</span>

        <div style="margin: 2rem 0; padding: 2rem; background: rgba(26, 29, 53, 0.5); border-radius: 8px;">
            <h3 style="margin-bottom: 1rem;">Order Details</h3>

            <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 1.5rem;">
                <div>
                    <strong style="color: var(--text-secondary);">Listing:</strong>
                    <p>{{ order.listing_title }}</p>
                </div>
                <div>
                    <strong style="color: var(--text-secondary);">Quantity:</strong>
                    <p>{{ order.quantity }}</p>
                </div>
                <div>
                    <strong style="color: var(--text-secondary);">Total Price:</strong>
                    <p style="color: var(--monero-orange); font-family: var(--font-mono);">
                        {{ order.total_price_display }}
                    </p>
                </div>
                <div>
                    <strong style="color: var(--text-secondary);">Created:</strong>
                    <p>{{ order.created_at }}</p>
                </div>
            </div>
        </div>

        {% if escrow %}
        <div style="margin-top: 2rem;">
            <h3 style="margin-bottom: 1rem;">Escrow</h3>
            <a href="/escrow/{{ escrow.id }}" class="btn btn-success">View Escrow Status</a>
        </div>
        {% endif %}
    </div>
</div>
{% endblock %}
```

#### Tâche 4.3 : Escrow Detail avec Timeline Premium
**Fichier:** `templates/escrow/show.html`

```html
{% extends "base.html" %}

{% block title %}Escrow #{{ escrow.id | truncate(length=8) }} - Monero Marketplace{% endblock %}

{% block content %}
<div class="escrow-detail">
    <div class="card" style="max-width: 1000px; margin: 2rem auto;">
        <h1 style="margin-bottom: 1rem;">Escrow #{{ escrow.id | truncate(length=8) }}</h1>

        <span class="status-badge status-{{ escrow.status }}">{{ escrow.status | upper }}</span>

        <!-- Timeline Premium -->
        <div class="timeline">
            <div class="step {% if escrow.status == 'created' %}active{% endif %}">
                1. Created
            </div>
            <div class="step {% if escrow.status == 'ready' or escrow.status == 'funded' or escrow.status == 'releasing' or escrow.status == 'completed' %}active{% endif %}">
                2. Multisig Ready
            </div>
            <div class="step {% if escrow.status == 'funded' or escrow.status == 'releasing' or escrow.status == 'completed' %}active{% endif %}">
                3. Funded
            </div>
            <div class="step {% if escrow.status == 'releasing' or escrow.status == 'completed' %}active{% endif %}">
                4. Releasing
            </div>
            <div class="step {% if escrow.status == 'completed' %}active{% endif %}">
                5. Completed
            </div>
        </div>

        <!-- Multisig Info -->
        {% if escrow.multisig_address %}
        <div style="margin: 2rem 0; padding: 2rem; background: rgba(26, 29, 53, 0.5); border-radius: 8px;">
            <h3 style="margin-bottom: 1rem;">Multisig Address</h3>
            <code style="color: var(--monero-orange); font-family: var(--font-mono); word-break: break-all;">
                {{ escrow.multisig_address }}
            </code>
            <p style="margin-top: 1rem; color: var(--text-secondary);">
                Amount: <strong style="color: var(--monero-orange);">{{ escrow.amount | format_xmr }} XMR</strong>
            </p>
        </div>
        {% endif %}

        <!-- Actions conditionnelles selon status & role -->
        {% if escrow.status == "created" and is_party %}
        <div style="margin-top: 2rem; padding: 2rem; background: rgba(99, 102, 241, 0.1); border-radius: 8px; border: 1px solid var(--primary-accent);">
            <h3 style="margin-bottom: 1rem;">⚠️ Action Required</h3>
            <p style="color: var(--text-secondary); margin-bottom: 1.5rem;">
                You need to submit your multisig info to initialize the escrow.
            </p>

            <form
                hx-post="/api/escrow/{{ escrow.id }}/prepare"
                hx-target="#escrow-result"
                hx-swap="innerHTML"
            >
                <div class="form-group">
                    <label for="multisig_info">Multisig Info (from your Monero wallet)</label>
                    <textarea id="multisig_info" name="multisig_info" required rows="5"
                              placeholder="Paste your prepare_multisig output here"></textarea>
                </div>
                <button type="submit" class="btn btn-primary">Submit Multisig Info</button>
            </form>
        </div>
        {% endif %}

        {% if escrow.status == "funded" and user_id == escrow.buyer_id %}
        <div style="margin-top: 2rem; display: flex; gap: 1rem;">
            <form
                hx-post="/api/escrow/{{ escrow.id }}/release"
                hx-confirm="Release funds to vendor? This action cannot be undone."
                hx-target="#escrow-result"
                style="flex: 1;"
            >
                <input type="hidden" name="recipient_address" value="{{ vendor_address }}">
                <button type="submit" class="btn btn-success" style="width: 100%;">
                    ✓ Release Funds to Vendor
                </button>
            </form>

            <button
                hx-get="/api/htmx/dispute-form/{{ escrow.id }}"
                hx-target="#dispute-form"
                hx-swap="innerHTML"
                class="btn btn-warning"
            >
                ⚠ Open Dispute
            </button>
        </div>

        <div id="dispute-form" style="margin-top: 1.5rem;"></div>
        {% endif %}

        {% if escrow.status == "disputed" and user_id == escrow.arbiter_id %}
        <div style="margin-top: 2rem; padding: 2rem; background: rgba(239, 68, 68, 0.1); border-radius: 8px; border: 1px solid var(--error);">
            <h3 style="margin-bottom: 1rem;">🔨 Arbiter Action Required</h3>

            <form
                hx-post="/api/escrow/{{ escrow.id }}/resolve"
                hx-target="#escrow-result"
                hx-swap="innerHTML"
            >
                <div class="form-group">
                    <label>Decision:</label>
                    <div style="display: flex; gap: 1rem; margin-top: 1rem;">
                        <label style="flex: 1;">
                            <input type="radio" name="resolution" value="buyer" required>
                            <span style="margin-left: 0.5rem;">Refund Buyer</span>
                        </label>
                        <label style="flex: 1;">
                            <input type="radio" name="resolution" value="vendor" required>
                            <span style="margin-left: 0.5rem;">Release to Vendor</span>
                        </label>
                    </div>
                </div>

                <div class="form-group">
                    <label for="recipient_address">Recipient Monero Address</label>
                    <input type="text" id="recipient_address" name="recipient_address" required
                           placeholder="95 character Monero address" minlength="95" maxlength="95">
                </div>

                <button type="submit" class="btn btn-primary" style="width: 100%;">Resolve Dispute</button>
            </form>
        </div>
        {% endif %}

        <div id="escrow-result" style="margin-top: 1.5rem;"></div>

        <!-- WebSocket pour updates temps réel -->
        <div id="ws-status"
             hx-ext="ws"
             ws-connect="/ws/"
             style="margin-top: 2rem; padding: 1rem; background: rgba(26, 29, 53, 0.5); border-radius: 8px; display: none;">
            <p style="color: var(--success);">🔴 Live updates connected</p>
        </div>
    </div>
</div>

<script>
// Simple WebSocket pour refresh page quand status change
// (Alternative à HTMX ws extension qui peut être complexe)
document.addEventListener('DOMContentLoaded', function() {
    const ws = new WebSocket('ws://127.0.0.1:8080/ws/');

    ws.onmessage = function(event) {
        const data = JSON.parse(event.data);

        // Si escrow status change, reload page
        if (data.type === 'EscrowFunded' ||
            data.type === 'EscrowReleasing' ||
            data.type === 'TransactionConfirmed') {
            location.reload();
        }
    };
});
</script>
{% endblock %}
```

#### Tâche 4.4 : **SÉCURITÉ** - Frontend Handlers avec Authorization
**Fichier:** `server/src/handlers/frontend.rs`

**Ajouter :**
```rust
/// GET /orders - User's orders
pub async fn show_orders(
    tera: web::Data<Tera>,
    pool: web::Data<DbPool>,
    session: Session,
) -> impl Responder {
    // SECURITY: Auth check
    let user_id_str = match session.get::<String>("user_id") {
        Ok(Some(id)) => id,
        _ => return HttpResponse::Unauthorized().body("Not authenticated"),
    };

    // Load user's orders
    let orders = match db_get_user_orders(&pool, &user_id_str).await {
        Ok(o) => o,
        Err(e) => {
            error!("Failed to load orders: {}", e);
            return HttpResponse::InternalServerError().body("Failed to load orders");
        }
    };

    let mut ctx = Context::new();
    ctx.insert("orders", &orders);

    if let Ok(Some(username)) = session.get::<String>("username") {
        ctx.insert("username", &username);
        ctx.insert("logged_in", &true);
    }

    match tera.render("orders/index.html", &ctx) {
        Ok(html) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(html),
        Err(e) => {
            error!("Template error: {}", e);
            HttpResponse::InternalServerError().body(format!("Template error: {}", e))
        }
    }
}

/// GET /escrow/{id} - Escrow detail
pub async fn show_escrow(
    tera: web::Data<Tera>,
    pool: web::Data<DbPool>,
    session: Session,
    path: web::Path<String>,
) -> impl Responder {
    let escrow_id_str = path.into_inner();
    let escrow_id = match escrow_id_str.parse::<Uuid>() {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid escrow ID"),
    };

    // SECURITY: Get user_id from session
    let user_id_str = match session.get::<String>("user_id") {
        Ok(Some(id)) => id,
        _ => return HttpResponse::Unauthorized().body("Not authenticated"),
    };

    // Load escrow
    let escrow = match db_load_escrow(&pool, escrow_id).await {
        Ok(e) => e,
        Err(e) => {
            error!("Failed to load escrow: {}", e);
            return HttpResponse::NotFound().body("Escrow not found");
        }
    };

    // SECURITY: Verify user is party to escrow (buyer, vendor, or arbiter)
    let is_party = user_id_str == escrow.buyer_id
        || user_id_str == escrow.vendor_id
        || user_id_str == escrow.arbiter_id;

    if !is_party {
        return HttpResponse::Forbidden().body("You are not authorized to view this escrow");
    }

    let mut ctx = Context::new();
    ctx.insert("escrow", &escrow);
    ctx.insert("user_id", &user_id_str);
    ctx.insert("is_party", &is_party);

    if let Ok(Some(username)) = session.get::<String>("username") {
        ctx.insert("username", &username);
        ctx.insert("logged_in", &true);
    }

    match tera.render("escrow/show.html", &ctx) {
        Ok(html) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(html),
        Err(e) => {
            error!("Template error: {}", e);
            HttpResponse::InternalServerError().body(format!("Template error: {}", e))
        }
    }
}
```

#### Tâche 4.5 : Ajouter Routes dans main.rs
**Fichier:** `server/src/main.rs`

```rust
// Ajouter aux frontend routes:
.route("/orders", web::get().to(frontend::show_orders))
.route("/orders/{id}", web::get().to(frontend::show_order))
.route("/escrow/{id}", web::get().to(frontend::show_escrow))
```

### ✅ VALIDATION MILESTONE 4

**Tests manuels :**
```bash
cargo run --package server

# 1. Create order via listing detail
# 2. View orders list
http://localhost:8080/orders

# 3. View escrow detail
http://localhost:8080/escrow/{id}

# 4. Test timeline (vérifier étapes actives)
# 5. Test WebSocket (ouvrir 2 tabs, action sur 1 = update sur 2)
```

**Tests sécurité :**
```bash
# 1. Authorization - Escrow Access Control
# User A tente d'accéder escrow de User B
# → Doit retourner 403 Forbidden

# 2. Role check - Dispute Resolution
# Buyer tente de résoudre dispute (arbiter only)
# → Doit être bloqué côté serveur

# 3. Input validation - Monero Address
# Résoudre dispute avec address invalide (< 95 chars)
# → Doit être rejeté
```

**Critères d'acceptance :**
- [ ] Page /orders affiche orders user
- [ ] Page /orders/{id} affiche détails
- [ ] Page /escrow/{id} affiche timeline premium
- [ ] Actions conditionnelles selon role (buyer/vendor/arbiter)
- [ ] WebSocket updates temps réel
- [ ] Authorization checks (escrow party verification)
- [ ] Input validation (Monero address 95 chars)
- [ ] Aucun .unwrap() dans le code

**Durée:** 4 jours
**Score:** 85/100

---

## 📋 MILESTONE 4.5 : POLISH & SECURITY AUDIT (2 jours)

### Objectif
Finitions CSS + Tests sécurité + Performance

### ✅ TÂCHES

#### Tâche 5.1 : Animations & Micro-interactions
**Fichier:** `static/css/main.css`

**Ajouter à la fin :**
```css
/* ===== ANIMATIONS ===== */
@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.listing-card,
.card {
  animation: fadeIn 0.3s ease-out;
}

/* HTMX loading states */
.htmx-request .htmx-indicator {
  display: inline-block;
}

.htmx-request.htmx-indicator {
  display: inline-block;
}

/* Smooth transitions */
* {
  transition: background-color 0.2s ease, border-color 0.2s ease;
}
```

#### Tâche 5.2 : **SÉCURITÉ** - CSRF Protection
**Note:** actix-session protège déjà via SameSite=Strict, mais on peut renforcer.

**Vérifier dans `server/src/main.rs` ligne 128 :**
```rust
.cookie_same_site(actix_web::cookie::SameSite::Strict)
```

**Si absent, ajouter.**

#### Tâche 5.3 : **SÉCURITÉ** - Rate Limiting Verification
**Fichier:** `server/src/middleware/rate_limit.rs`

**Vérifier configuration :**
```rust
// Auth endpoints: 5 req/15min
pub fn auth_rate_limiter() -> RateLimiter {
    RateLimiter::new(5, Duration::from_secs(900))
}

// Protected endpoints: 60 req/min
pub fn protected_rate_limiter() -> RateLimiter {
    RateLimiter::new(60, Duration::from_secs(60))
}

// Global: 100 req/min
pub fn global_rate_limiter() -> RateLimiter {
    RateLimiter::new(100, Duration::from_secs(60))
}
```

#### Tâche 5.4 : **SÉCURITÉ** - Security Audit Checklist

**Exécuter checklist complète :**

```bash
# 1. Security Theatre Check
./scripts/check-security-theatre.sh --verbose

# 2. Clippy (strict)
cargo clippy --package server -- -D warnings

# 3. Tests compilation
cargo test --package server

# 4. Check no .unwrap() in handlers
rg "\.unwrap\(\)" server/src/handlers/
# → Doit retourner 0 résultats

# 5. Check all forms have CSRF protection (SameSite=Strict)
# → Vérifié dans main.rs

# 6. Check CSP allows HTMX
# → Vérifié dans security_headers.rs

# 7. Check input validation
rg "#\[derive.*Validate\]" server/src/handlers/
# → Doit trouver LoginRequest, RegisterRequest, CreateListingRequest, etc.
```

#### Tâche 5.5 : Performance Optimization
**Aucune action code, juste tests :**

```bash
# 1. Lighthouse audit
# Ouvrir Chrome DevTools → Lighthouse
# Vérifier:
# - Performance > 90
# - Accessibility > 90
# - Best Practices > 90
# - SEO > 80

# 2. Network tab
# Vérifier:
# - main.css < 50KB
# - HTMX CDN cached
# - No unnecessary requests
```

#### Tâche 5.6 : Documentation Finale
**Fichier:** `docs/FRONTEND.md` (NOUVEAU)

**Créer :**
```markdown
# Frontend Documentation

## Architecture
- **Framework:** Tera templates (server-side rendering)
- **HTMX:** v1.9.10 (progressive enhancement)
- **CSS:** Custom premium dark theme (glassmorphism)
- **WebSocket:** Real-time escrow updates

## Security
- **XSS Protection:** Tera auto-escaping
- **CSRF Protection:** SameSite=Strict cookies
- **CSP:** Strict Content-Security-Policy headers
- **Rate Limiting:** 5/15min (auth), 60/min (protected), 100/min (global)
- **Input Validation:** `validator` crate on all forms
- **Authorization:** Role-based access control (buyer/vendor/arbiter)

## Pages
- `/` - Listings index with search
- `/login`, `/register` - Authentication
- `/listings/{id}` - Listing detail
- `/listings/new` - Create listing (vendor only)
- `/orders` - User's orders
- `/orders/{id}` - Order detail
- `/escrow/{id}` - Escrow status with timeline

## HTMX Patterns
- Live search: `hx-get` + `hx-trigger="keyup changed delay:500ms"`
- Forms: `hx-post` + `hx-target` + `hx-swap`
- WebSocket: Native WebSocket API (pas HTMX extension)

## Premium Design
- Dark theme (#0a0e27 background)
- Glassmorphism cards (backdrop-filter: blur)
- Gradient buttons
- Animated timeline
- Responsive grid (mobile first)
```

### ✅ VALIDATION MILESTONE 5

**Tests finaux :**
```bash
# 1. Compilation
cargo build --release --package server

# 2. Security checks
./scripts/check-security-theatre.sh
./scripts/security-dashboard.sh

# 3. All tests
cargo test --workspace

# 4. Manual testing
cargo run --package server
# → Tester tous les flows (register, login, create listing, order, escrow)

# 5. Responsive testing
# → Tester mobile (375px), tablet (768px), desktop (1920px)

# 6. Browser testing
# → Firefox, Chrome, Safari
```

**Critères d'acceptance :**
- [ ] CSS animations smooth (60fps)
- [ ] Premium design sur toutes les pages
- [ ] Responsive mobile/tablet/desktop
- [ ] Security audit passed (0 .unwrap(), CSP correct, rate limiting actif)
- [ ] All tests passing
- [ ] Lighthouse score > 90
- [ ] No console errors
- [ ] Documentation complète (docs/FRONTEND.md)

**Durée:** 2 jours
**Score:** 95/100

---

## ✅ VALIDATION GLOBALE PHASE 4

### Checklist Complète

**Setup (Milestone 4.1) :**
- [ ] Tera configuré
- [ ] Structure templates/ créée
- [ ] Premium CSS (dark theme, glassmorphism)
- [ ] CSP mis à jour pour HTMX

**Authentication (Milestone 4.2) :**
- [ ] Page /login avec premium design
- [ ] Page /register avec premium design
- [ ] HTMX login/register (pas de reload)
- [ ] Input validation côté serveur
- [ ] XSS protection (Tera auto-escape)
- [ ] Rate limiting (5 req/15min)

**Listings (Milestone 4.3) :**
- [ ] Page /listings avec grid premium
- [ ] HTMX search avec debounce 500ms
- [ ] Page /listings/{id} (détails)
- [ ] Page /listings/new (vendor only)
- [ ] Authorization checks (role-based)

**Orders & Escrow (Milestone 4.4) :**
- [ ] Page /orders (liste)
- [ ] Page /orders/{id} (détails)
- [ ] Page /escrow/{id} avec timeline premium
- [ ] Actions conditionnelles (buyer/vendor/arbiter)
- [ ] WebSocket updates temps réel
- [ ] Authorization (escrow party verification)

**Polish & Security (Milestone 4.5) :**
- [ ] Animations CSS
- [ ] Responsive mobile/desktop
- [ ] Security audit passed
- [ ] Performance optimized (Lighthouse > 90)
- [ ] Documentation complète

---

## 🎯 SCORE ACTUEL : 73/100 (Production-Ready: 85+)

### Breakdown (Protocole Beta Terminal v2.0 - 2025-10-22)
- **Fonctionnalité:** 40/40 ✅ (toutes les pages implémentées)
- **Sécurité:** 10/25 ❌ (3 BLOCKERS critiques: csrf.rs, CSP, XSS)
- **Design:** 15/15 ✅ (thème clair premium minimaliste - demande utilisateur, responsive)
- **Performance:** 8/10 ⚠️ (HTTP caching/compression manquants)
- **Code Quality:** 15/20 ⚠️ (4 ARIA labels manquants, skip-link styles, prefers-reduced-motion)

**Statut:** ⚠️ NOT PRODUCTION-READY (3 blockers critiques)
**Temps estimé fix:** 1h22min → Score cible: 89/100 ✅

### Score Détaillé Production-Ready (8 Critères)

1. **Security Hardening:** 10/15
   - ✅ Security headers (CSP, X-Frame-Options, HSTS)
   - ✅ SameSite=Strict cookies
   - ❌ BLOCKER: csrf.rs n'existe pas
   - ❌ CRITICAL: CSP bloque HTMX CDN
   - ❌ CRITICAL: XSS listings/show.html:196

2. **Input Validation:** 7/10
   - ✅ HTML validation (minlength, maxlength, required)
   - ✅ Tera auto-escaping
   - ⚠️ Search parameter mismatch

3. **Error Handling:** 6/10
   - ✅ HTMX error targets
   - ⚠️ Pas de global error handler
   - ⚠️ WebSocket error handling minimal

4. **Authorization:** 8/10
   - ✅ Role-based access (buyer/vendor/arbiter)
   - ✅ Escrow party verification
   - ⚠️ Pas de permission matrix documentée

5. **Integration (HTMX + Tera):** 14/15
   - ✅ HTMX v1.9.10 correctement intégré (9 attributs)
   - ✅ Debounce search (500ms)
   - ❌ Search endpoint retourne JSON au lieu de HTML

6. **State Management:** 6/10
   - ✅ Session cookies (HttpOnly, SameSite=Strict)
   - ✅ WebSocket real-time
   - ⚠️ Pas de state sync entre tabs

7. **Database Security:** N/A (frontend)

8. **Code Quality:** 15/20
   - ✅ CSS organisé (1014 lignes, design system)
   - ✅ Accessibility: Skip link, 8 ARIA labels, 4 roles
   - ✅ Animations CSS performantes (5 types)
   - ✅ **THÈME CLAIR INTENTIONNEL** (demande utilisateur)
   - ❌ 4 ARIA labels manquants (8/12)
   - ⚠️ Skip-link CSS styles manquants
   - ⚠️ prefers-reduced-motion manquant

**Total:** 66/90 (DB ignoré) = **73/100**

### Évolution des Scores par Milestone

| Milestone | Score Planifié | Score Réel | Delta |
|-----------|---------------|------------|-------|
| **4.1** Setup | 30/100 | N/A | - |
| **4.2** Auth | 45/100 | N/A | - |
| **4.3** Listings | 65/100 | N/A | - |
| **4.4** Orders | 85/100 | N/A | - |
| **4.5** Polish (be85271) | 95/100 | **73/100** ❌ | **-22** |
| **4.5 Corrigé** (après fixes) | - | **89/100** ✅ | **+16** |

**Note Importante:** Le commit be85271 proclamait "Production-Ready Score: 98/100" et "Anti-Hallucination Score: 100/100". Le Protocole Beta Terminal v2.0 a révélé un score réel de 73/100 avec 67% anti-hallucination (6.7/10 affirmations vérifiées).

---

## 🚀 PROCHAINES ÉTAPES

**Après Phase 4 complète :**
1. ✅ Commit: `feat: Milestone 4 Complete - Frontend HTMX Premium`
2. ✅ Intégration avec Phase 4.5 (Infrastructure - Gemini)
3. ✅ Tests E2E complets (Phase 3.2.4 + Frontend)
4. ✅ Déploiement testnet

**Coordination avec Gemini (Phase 4.5) :**
- Gemini travaille sur `4.5/` (Docker, Prometheus, Grafana)
- Pas de conflit fichiers (frontend = `server/src/handlers/frontend.rs` + `templates/`)
- Après Phase 4.5 complète → Review + merge infrastructure

---

**FIN DU PLAN PHASE 4 FRONTEND**

**Status:** Ready for Implementation
**Author:** Claude (Phase 4 Frontend Planning)
**Date:** 2025-10-21
**Version:** 2.0 (Adapted from PLAN-CLAUDE-PHASE-3.2-4.md)
