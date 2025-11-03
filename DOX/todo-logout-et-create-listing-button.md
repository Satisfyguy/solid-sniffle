# TODO: Logout + Create Listing Button

## ✅ CORRECTIONS DÉJÀ TERMINÉES

1. **Login Case-Insensitive** ✅
   - frank = Frank = FRANK
   - Fichier: server/src/handlers/auth.rs (lignes 130, 156, 262)

2. **CSP Hash Ajouté** ✅
   - Plus d'erreur inline script
   - Fichier: server/src/middleware/security_headers.rs (ligne 106)

3. **SESSION_SECRET_KEY** ✅
   - Ajouté dans .env
   - Serveur démarre correctement

---

## 🔨 PROCHAINES TÂCHES

### Task 1: Logout → Redirect Homepage

**Demande utilisateur:** "quand on logout on doit revenir à la page d'acceuil"

**Fichiers identifiés:**
- `server/src/handlers/auth.rs:407` - API endpoint (retourne JSON)
- `server/src/handlers/frontend.rs:134` - Frontend endpoint (retourne redirect)

**Frontend logout actuel** (frontend.rs:138-139):
```rust
HttpResponse::Found()
    .append_header(("Location", "/login"))
```

**Modification requise:**
Changer `/login` → `/`

**Attention:** Il existe probablement un JS qui appelle l'API logout. Vérifier quel endpoint est utilisé.

---

### Task 2: Bouton "Create Listing" Mis en Évidence (Vendors)

**Demande utilisateur:** "un boutton pour publier un listing doit etre mis en evidence quand on est connecté en tant que vendeur"

**Fichier à modifier:** `templates/header.html` (probablement)

**Requirements:**
1. Afficher le bouton uniquement si `role == "vendor"`
2. Style prominent (CTA - Call To Action)
3. Texte: "Create Listing" ou "Publish Listing"
4. Lien: `/listings/new` ou `/listings/create`

**Style suggéré:**
```css
background: linear-gradient(135deg, #f59e0b 0%, #ef4444 100%); /* Or/Rouge */
padding: 0.75rem 1.5rem;
border-radius: 8px;
font-weight: 700;
text-transform: uppercase;
letter-spacing: 0.05em;
box-shadow: 0 4px 14px 0 rgba(251, 146, 60, 0.4);
```

---

## 📋 PLAN D'EXÉCUTION

### Étape 1: Logout Redirect

1. Lire `templates/header.html` pour voir quel endpoint logout est utilisé
2. Modifier le bon endpoint pour rediriger vers `/` au lieu de `/login`
3. Tester en logout

### Étape 2: Create Listing Button

1. Lire `templates/header.html` pour comprendre la structure
2. Ajouter un bouton conditionnel:
   ```html
   {% if user_role == "vendor" %}
   <a href="/listings/new" class="btn-create-listing">
       📦 Create Listing
   </a>
   {% endif %}
   ```
3. Ajouter les styles CSS inline ou dans `static/css/main.css`

### Étape 3: Test

1. Login en tant que vendor → Voir le bouton "Create Listing"
2. Login en tant que buyer → Ne PAS voir le bouton
3. Logout → Redirect vers homepage `/`

---

## ⏰ ESTIMATION

- Task 1 (Logout): 2 minutes
- Task 2 (Create Listing Button): 5-10 minutes
- **Total**: 12 minutes maximum

---

**Status:** Prêt pour implémentation
**Serveur:** http://127.0.0.1:8080 (actif)
