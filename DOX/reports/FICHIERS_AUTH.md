# 🔐 Fichiers Modifiés - Flow d'Authentification UNIQUEMENT

## 🎯 Fichiers Critiques (Dans l'Ordre d'Importance)

### 1. **Navigation : Boutons LOGIN/SIGNUP**
**Fichier :** `templates/partials/nexus/organisms/nav.html`

**Lignes 78-98 :**
```html
{% else %}
  {# Guest User - Premium NEXUS Auth Buttons #}
  <a href="/login" class="nexus-btn nexus-btn-ghost nexus-btn-sm" hx-boost="true">
    <svg>...</svg>
    Login
  </a>
  <a href="/register" class="nexus-btn nexus-btn-primary nexus-btn-sm" hx-boost="true">
    <svg>...</svg>
    Sign Up
    <span class="nexus-btn-glow" style="animation: nexus-btn-shine 3s infinite;"></span>
  </a>
{% endif %}
```

**Ligne 65-75 : Formulaire de Logout**
```html
<form action="/logout" method="POST" style="margin: 0;">
  <input type="hidden" name="csrf_token" value="{{ csrf_token }}">
  <button type="submit" class="nexus-nav-dropdown-item">
    Logout
  </button>
</form>
```

---

### 2. **Page de Login**
**Fichier :** `templates/auth/login.html`

**Modifications principales :**

**Ligne 29 : Route API**
```html
<form hx-post="/api/auth/login" ...>
```

**Ligne 38 : CSRF Token**
```html
<input type="hidden" name="csrf_token" value="{{ csrf_token }}">
```

**Lignes 115-187 : Toast Notifications**
```javascript
// Handle HTMX response with premium toast notifications
document.body.addEventListener('htmx:afterRequest', function(event) {
  if (event.detail.pathInfo.requestPath === '/api/auth/login') {
    if (event.detail.successful) {
      window.notificationManager.showToast('✅ Login Successful', ...);
    }
  }
});
```

---

### 3. **Page de Register**
**Fichier :** `templates/auth/register.html`

**Modifications principales :**

**Ligne 29 : Route API**
```html
<form hx-post="/api/auth/register" ...>
```

**Ligne 38 : CSRF Token**
```html
<input type="hidden" name="csrf_token" value="{{ csrf_token }}">
```

**Ligne 89-110 : Sélection de Role**
```html
<select id="role" name="role" required>
  <option value="">— Select Role —</option>
  <option value="buyer">🛒 Buyer</option>
  <option value="vendor">🏪 Vendor</option>
</select>
```

**Lignes 147-219 : Toast Notifications**
```javascript
// Similar structure to login.html
window.notificationManager.showToast('🎉 Registration Successful', ...);
```

---

### 4. **Backend : Handlers Frontend**
**Fichier :** `server/src/handlers/frontend.rs`

**Modifications dans TOUTES les fonctions de rendu :**

**Pattern ajouté partout :**
```rust
// Check if user is logged in
if let Ok(Some(username)) = session.get::<String>("username") {
    ctx.insert("username", &username);
    ctx.insert("user_name", &username); // ← NOUVEAU (pour nav.html)
    ctx.insert("logged_in", &true);

    if let Ok(Some(role)) = session.get::<String>("role") {
        ctx.insert("role", &role);
    }
} else {
    ctx.insert("logged_in", &false);
}

// Add CSRF token for logout form
let csrf_token = get_csrf_token(&session); // ← NOUVEAU
ctx.insert("csrf_token", &csrf_token);     // ← NOUVEAU
```

**Fonctions modifiées :**
- `index()` - Ligne ~14-35
- `show_listings()` - Ligne ~135-150
- `show_escrow()` - Ligne ~733-741
- Et ~5 autres fonctions...

**Fonction show_login() - Ligne ~47-70 :**
```rust
pub async fn show_login(tera: web::Data<Tera>, session: Session) -> impl Responder {
    // Redirect if already logged in
    if let Ok(Some(_username)) = session.get::<String>("username") {
        return HttpResponse::Found()
            .append_header(("Location", "/"))
            .finish();
    }

    let mut ctx = Context::new();
    let csrf_token = get_csrf_token(&session);
    ctx.insert("csrf_token", &csrf_token);

    tera.render("auth/login.html", &ctx)
}
```

**Fonction logout() - Ligne ~98-106 :**
```rust
pub async fn logout(session: Session) -> impl Responder {
    session.purge(); // Clear all session data

    HttpResponse::Found()
        .append_header(("Location", "/login"))
        .finish()
}
```

---

### 5. **Backend : API Handlers (Déjà existants, pas modifiés)**
**Fichier :** `server/src/handlers/auth.rs`

Ces handlers étaient DÉJÀ là, je les ai juste vérifiés :
- `POST /api/auth/register` - Ligne 63-154
- `POST /api/auth/login` - Ligne 198-325
- `GET /api/auth/whoami` - Ligne 332-366
- `POST /api/auth/logout` - Ligne 369-387

**Pas de modifications nécessaires sur ce fichier !**

---

### 6. **CSS : Animation du bouton Sign Up**
**Fichier :** `static/css/nexus.css`

**Ajouté à la fin (après ligne 600) :**
```css
/* ===== BUTTON ANIMATIONS ===== */

@keyframes nexus-btn-shine {
  0% {
    transform: translateX(-100%);
  }
  100% {
    transform: translateX(100%);
  }
}

.nexus-btn-glow {
  pointer-events: none;
}
```

---

## 🔍 Commandes de Vérification Rapide

```bash
# 1. Vérifier les boutons dans la nav
grep -n "LOGIN\|SIGN UP" templates/partials/nexus/organisms/nav.html

# 2. Vérifier l'animation CSS
tail -20 static/css/nexus.css

# 3. Vérifier le formulaire de login
grep -n "csrf_token" templates/auth/login.html

# 4. Vérifier le handler frontend
grep -n "user_name" server/src/handlers/frontend.rs

# 5. Vérifier le logout form
grep -n "logout" templates/partials/nexus/organisms/nav.html
```

---

## ✅ Résumé : 4 Fichiers Principaux

Pour le **flow d'authentification complet** :

1. ✅ `templates/partials/nexus/organisms/nav.html` - Boutons + Logout form
2. ✅ `templates/auth/login.html` - Page de login + Toast
3. ✅ `templates/auth/register.html` - Page de register + Toast
4. ✅ `server/src/handlers/frontend.rs` - Context avec csrf_token et user_name
5. ✅ `static/css/nexus.css` - Animation (bonus)

**Le fichier `server/src/handlers/auth.rs` n'a PAS été modifié** - les API endpoints étaient déjà bons !

---

## 🧪 Flow de Test

### 1. Utilisateur non connecté
- Visite http://127.0.0.1:8080
- Voit : `[LOGIN] [SIGN UP]` dans le header
- Variable template : `logged_in = false`

### 2. Clique SIGN UP
- GET /register → `frontend::show_register()`
- Reçoit `csrf_token` dans le contexte
- Voit le formulaire avec roles

### 3. Remplit et soumet
- POST /api/auth/register → `auth::register()`
- Vérifie CSRF token
- Hash password avec Argon2id
- Crée user en DB
- Crée session (auto-login)
- Retourne `HX-Redirect: /`

### 4. Utilisateur connecté
- Visite http://127.0.0.1:8080
- Handler passe `logged_in = true`, `user_name = "alice"`
- Voit : `[ALICE ▼]` dans le header (dropdown)

### 5. Clique Logout
- POST /logout → `frontend::logout()`
- `session.purge()`
- Redirect vers /login
- Header redevient : `[LOGIN] [SIGN UP]`

---

## 🎯 Pour Vérifier Chez Toi

```bash
# Ouvre ces 4 fichiers dans ton éditeur :
nano templates/partials/nexus/organisms/nav.html
nano templates/auth/login.html
nano templates/auth/register.html
nano server/src/handlers/frontend.rs

# Cherche les mots-clés :
# - "LOGIN" et "Sign Up" dans nav.html
# - "csrf_token" dans login.html et register.html
# - "user_name" dans frontend.rs
```

**C'est tout ! Pas besoin de toucher 30 fichiers.** 🎯
