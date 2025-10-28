# TÂCHES IMMÉDIATES - FRONTEND

**Date:** 2025-10-22
**Contexte:** Protocole Beta Terminal - Frontend v2.0
**Score Actuel:** 73/100
**Objectif:** 85/100 (Production-Ready)

---

## 🚨 BLOCKERS CRITIQUES (Must Fix - 30 min)

### BLOCKER 1: Module csrf.rs N'EXISTE PAS (15 min)

**Priorité:** P0 - CRITICAL
**Impact:** Compilation FAIL
**Fichiers affectés:**
- `server/src/handlers/frontend.rs:58,85`
- `server/src/handlers/auth.rs:73,208`

**Actions:**
1. Créer `server/src/middleware/csrf.rs`
2. Implémenter `get_csrf_token()` et `validate_csrf_token()`
3. Ajouter `mod csrf;` dans `server/src/middleware/mod.rs`

**Spec:**
```rust
// server/src/middleware/csrf.rs
use actix_session::Session;
use uuid::Uuid;

pub fn get_csrf_token(session: &Session) -> String {
    if let Ok(Some(token)) = session.get::<String>("csrf_token") {
        token
    } else {
        let new_token = Uuid::new_v4().to_string();
        session.insert("csrf_token", &new_token).ok();
        new_token
    }
}

pub fn validate_csrf_token(session: &Session, token: &str) -> bool {
    session.get::<String>("csrf_token")
        .ok()
        .flatten()
        .map(|stored| stored == token)
        .unwrap_or(false)
}
```

---

### BLOCKER 2: CSP Bloque HTMX CDN (10 min)

**Priorité:** P0 - CRITICAL
**Impact:** HTMX non fonctionnel en production
**Fichier:** `server/src/middleware/security_headers.rs:106`

**ACTUEL (BROKEN):**
```rust
script-src 'self';
```

**FIX REQUIS:**
```rust
script-src 'self' https://unpkg.com 'sha384-D1Kt99CQMDuVetoL1lrYwg5t+9QdHe7NLX/SoJYkXDFfX37iInKRy5xLSi8nO7UC';
```

**Action:** Remplacer ligne 106 dans `security_headers.rs`

---

### BLOCKER 3: XSS dans Inline Script (5 min)

**Priorité:** P0 - CRITICAL
**Impact:** Injection XSS possible
**Fichier:** `templates/listings/show.html:196`

**VULNÉRABLE:**
```javascript
const unitPrice = {{ listing.price_xmr }};
```

**FIX REQUIS:**
```javascript
const unitPrice = {{ listing.price_xmr | json_encode() | safe }};
```

**Action:** Remplacer ligne 196 dans `listings/show.html`

---

## ⚡ HIGH PRIORITY (Recommandé - 52 min)

### 4. Search Parameter Mismatch (2 min)

**Priorité:** P1 - HIGH
**Impact:** Search HTMX broken
**Fichier:** `templates/listings/index.html:27`

**ACTUEL (BROKEN):**
```html
<input name="search" hx-get="/api/listings/search">
```

**FIX:**
```html
<input name="q" hx-get="/api/listings/search">
```

---

### 5. Search Endpoint Dual-Mode (30 min)

**Priorité:** P1 - HIGH
**Impact:** HTMX incompatible (reçoit JSON au lieu de HTML)
**Fichier:** `server/src/handlers/listings.rs` (search handler)

**Spec:**
```rust
use actix_web::http::header;

async fn search_listings(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    query: web::Query<SearchQuery>,
) -> impl Responder {
    let is_htmx = req.headers()
        .get("HX-Request")
        .and_then(|h| h.to_str().ok())
        .map(|v| v == "true")
        .unwrap_or(false);

    let results = search_db(&pool, &query.q).await?;

    if is_htmx {
        // Return HTML fragment
        let html = render_listing_cards(&results);
        HttpResponse::Ok().content_type("text/html").body(html)
    } else {
        // Return JSON
        HttpResponse::Ok().json(results)
    }
}
```

---

### 6. Skip-Link CSS Styles (5 min)

**Priorité:** P1 - HIGH (Accessibility)
**Impact:** Skip link non visible au focus
**Fichier:** `static/css/main.css`

**MANQUANT:**
```css
.skip-link {
    position: absolute;
    top: -40px;
    left: 0;
    background: var(--bg-primary);
    color: var(--text-primary);
    padding: 0.5rem 1rem;
    z-index: 9999;
    text-decoration: none;
    border: 2px solid var(--highlight);
}

.skip-link:focus {
    top: 0;
}
```

**Action:** Ajouter après ligne 100 dans `main.css`

---

### 7. Prefers-Reduced-Motion (5 min)

**Priorité:** P1 - HIGH (Accessibility WCAG 2.1)
**Impact:** Animations forcées pour users avec sensibilité motion
**Fichier:** `static/css/main.css`

**MANQUANT:**
```css
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

**Action:** Ajouter à la fin de `main.css`

---

### 8. HTTP Caching/Compression (10 min)

**Priorité:** P2 - MEDIUM (Performance)
**Impact:** CSS/JS non cachés, gaspillage bande passante
**Fichier:** `server/src/main.rs` (static files config)

**Spec:**
```rust
use actix_files::Files;

Files::new("/static", "static")
    .use_etag(true)
    .use_last_modified(true)
    .prefer_utf8(true)
```

**Action:** Modifier config static files dans `main.rs`

---

## 📊 IMPACT SUR LE SCORE

| Tâche | Temps | Impact Score | Nouveau Score |
|-------|-------|--------------|---------------|
| **Blockers 1-3** | 30 min | +10 points | 83/100 |
| **Tasks 4-5** | 32 min | +3 points | 86/100 ✅ |
| **Tasks 6-7** | 10 min | +2 points | 88/100 |
| **Task 8** | 10 min | +1 point | 89/100 |
| **TOTAL** | **1h22min** | **+16 points** | **89/100** ✅ |

---

## ✅ VALIDATION

**Après fixes:**
```bash
# 1. Compilation
cargo build --package server

# 2. Security check
./scripts/check-security-theatre.sh

# 3. Start server
cargo run --package server

# 4. Test HTMX
curl -H "HX-Request: true" http://localhost:8080/api/listings/search?q=laptop

# 5. Test CSP (DevTools Console - aucune erreur)
# 6. Test skip-link (Tab → doit apparaître)
# 7. Lighthouse audit (Performance > 90, A11y > 90)
```

---

**Deadline:** 1h22min
**Score Cible:** 89/100 ✅ PRODUCTION-READY
