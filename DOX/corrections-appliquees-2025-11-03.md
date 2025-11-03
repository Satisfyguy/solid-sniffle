# Corrections Appliquées - 2025-11-03

## ✅ CORRECTIONS TERMINÉES

### 1. Login Case-Insensitive
**Fichier:** `server/src/handlers/auth.rs`

**Lignes modifiées:**
- Ligne 130: `req.username.to_lowercase()` lors de la vérification
- Ligne 156: `username: req.username.to_lowercase()` lors de la création
- Ligne 262: `let username = req.username.to_lowercase()` lors du login

**Résultat:** `frank`, `Frank`, `FRANK` → Tous connectent le même utilisateur

---

### 2. CSP Hash Ajouté
**Fichier:** `server/src/middleware/security_headers.rs`

**Ligne 106:** Ajouté le hash `'sha256-lolxUSgQkT0uB/gvibkkv3ggZX11uDt1lpP/XLCtLTs='`

**Résultat:** Plus d'erreur CSP pour les scripts inline

---

### 3. SESSION_SECRET_KEY
**Fichier:** `.env`

**Ajout:** `SESSION_SECRET_KEY=e93835ce35734c0e427d91d8b95781be7410e5cb0a32231b662693fd83a76e275b2b511f4d1658672976683cb9cd1de595be5c167ccb53d606ae2f488a0d1ff9`

**Résultat:** Serveur démarre correctement sans panic

---

### 4. HTMX Header Detection Fix (Précédemment)
**Fichier:** `server/src/handlers/auth.rs`

**Ligne 32:** Changé de `"HX-Request"` → `"hx-request"` (lowercase)

**Résultat:** Backend détecte correctement les requêtes HTMX

---

## 🔨 PROCHAINES TÂCHES

### 5. Logout → Redirection Homepage
**À faire:**
- Modifier `server/src/handlers/auth.rs` fonction `logout`
- Changer redirect de `/login` vers `/`

### 6. Bouton "Create Listing" Mis en Évidence (Vendors)
**À faire:**
- Modifier `templates/header.html`
- Ajouter bouton CTA prominent pour role="vendor"
- Style: bouton doré/vert mis en avant

---

**Status:** Serveur opérationnel sur http://127.0.0.1:8080
