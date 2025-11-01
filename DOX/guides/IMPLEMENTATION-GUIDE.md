# Guide d'Implémentation : Ajouter une Nouvelle Page Frontend

Ce guide décrit la procédure standard pour ajouter une nouvelle page au frontend du projet Monero Marketplace. Il est crucial de suivre ces étapes pour maintenir la cohérence, la sécurité (CSP) et la maintenabilité du code.

## Philosophie

Les nouvelles pages sont conçues comme des **templates autonomes**. Elles **n'héritent pas** d'un template de base (comme `base-marketplace.html`). Chaque fichier de template doit être un document HTML complet.

Cette approche garantit que chaque page charge uniquement les ressources dont elle a besoin et évite les dépendances complexes des templates parents.

---

## Étapes d'Implémentation

### 1. Créer le Fichier de Template

Créez un nouveau fichier `.html` dans le répertoire `templates/`. Il est recommandé d'utiliser un sous-dossier pour l'organisation.

**Exemple :** `templates/profile/index.html`

Le fichier doit avoir la structure suivante :

```html
<!DOCTYPE html>
<html lang="fr">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Titre de votre page</title>
    
    <!-- Feuille de style principale (obligatoire) -->
    <link rel="stylesheet" href="/static/css/main.css">
    
    <!-- CSS additionnel spécifique à la page (optionnel) -->
    <style>
        /* Vos styles ici */
    </style>
</head>
<body>
    <!-- En-tête partagé (obligatoire) -->
    {% include "header.html" %}

    <!-- Contenu principal de la page -->
    <main class="container mx-auto px-6 py-12">
        <h1>Contenu de votre page</h1>
        <!-- ... -->
    </main>

    <!-- Scripts de base (obligatoire, à la fin du body) -->
    <script src="/static/js/lucide.min.js"></script>
    <script src="/static/js/base.js"></script>
</body>
</html>
```

**Points Clés :**
- **Structure Complète :** Le fichier doit être un document HTML valide et complet.
- **CSS Principal :** Toujours inclure `/static/css/main.css`.
- **En-tête :** Toujours inclure `{% include "header.html" %}` au début du `<body>`.
- **Scripts :** Toujours inclure `lucide.min.js` et `base.js` à la fin du `<body>` pour initialiser les icônes et les interactions de base.

### 2. Créer le Handler Backend

Ouvrez `server/src/handlers/frontend.rs` et ajoutez une nouvelle fonction asynchrone pour votre page.

**Exemple :**

```rust
/// GET /ma-nouvelle-page - Description de la page
pub async fn show_my_new_page(tera: web::Data<Tera>, session: Session) -> impl Responder {
    // 1. (Optionnel) Garde d'authentification
    let user_id = match session.get::<String>("user_id") {
        Ok(Some(uid)) => uid,
        _ => {
            // Redirige vers la page de connexion si non authentifié
            return HttpResponse::Found()
                .append_header(("Location", "/login"))
                .finish();
        }
    };

    // 2. Créer le contexte Tera
    let mut ctx = Context::new();

    // 3. Insérer les données de session (si nécessaire)
    if let Ok(Some(username)) = session.get::<String>("username") {
        ctx.insert("username", &username);
        ctx.insert("logged_in", &true);
        if let Ok(Some(role)) = session.get::<String>("role") {
            ctx.insert("role", &role);
        }
    } else {
        // Logique pour les utilisateurs non connectés
        ctx.insert("logged_in", &false);
    }

    // 4. Rendre le template
    match tera.render("my_new_page/index.html", &ctx) {
        Ok(html) => {
            info!("Affichage de ma-nouvelle-page pour l'utilisateur {}", user_id);
            HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .body(html)
        }
        Err(e) => {
            error!("Erreur de template pour ma-nouvelle-page: {}", e);
            HttpResponse::InternalServerError().body(format!("Erreur de template: {}", e))
        }
    }
}
```

### 3. Enregistrer la Nouvelle Route

Ouvrez `server/src/main.rs` et ajoutez la nouvelle route dans la configuration du `HttpServer`.

**Exemple :**

```rust
// ... dans la configuration de l'App
.service(
    web::scope("")
        // ... autres routes
        .route("/listings/{id}/edit", web::get().to(frontend::show_edit_listing))
        
        // AJOUTER VOTRE ROUTE ICI
        .route("/ma-nouvelle-page", web::get().to(frontend::show_my_new_page))
        
        .route("/vendor/listings", web::get().to(frontend::show_vendor_listings))
        // ... autres routes
)
// ...
```

---

Une fois ces trois étapes terminées, votre nouvelle page sera intégrée au projet, accessible via son URL et suivra les conventions de développement actuelles du frontend.

---

## Exemple Concret : Page d'Authentification

Pour illustrer le processus, voici les fichiers qui ont été créés et modifiés pour la nouvelle page d'authentification unifiée (`/auth`) :

1.  **Fichier de Template (`templates/auth/index.html`) :**
    *   Un nouveau fichier a été créé pour contenir le HTML, le CSS inline (utilisant les variables du projet) et le JavaScript pour la page.
    *   Chemin : `templates/auth/index.html`

2.  **Handler Backend (`server/src/handlers/frontend.rs`) :**
    *   Une nouvelle fonction `show_auth_page` a été ajoutée pour gérer la logique de la page.
    *   Les anciennes fonctions `show_login` et `show_register` ont été supprimées.
    *   Les redirections d'authentification dans tout le fichier ont été mises à jour pour pointer vers `/auth`.
    *   Chemin : `server/src/handlers/frontend.rs`

3.  **Fichier de Routes (`server/src/main.rs`) :**
    *   La nouvelle route `web::get().to(frontend::show_auth_page)` a été ajoutée.
    *   Les anciennes routes `/login` et `/register` ont été supprimées.
    *   Chemin : `server/src/main.rs`

---

## Fichiers de Base du Frontend Actuel

Pour clarifier la structure actuelle du frontend et éviter toute confusion, voici les fichiers qui constituent la base de notre design unifié. Pour tout nouveau développement, veuillez vous référer à ces fichiers.

1.  **Feuille de Style Principale :**
    *   Définit l'identité visuelle du site, y compris les variables de couleur, les polices et les styles de base.
    *   Chemin : `static/css/main.css`

2.  **Template de l'En-tête :**
    *   C'est l'en-tête **unique et partagé** à inclure dans toutes les nouvelles pages. Ignorez les autres fichiers `header.html` qui pourraient exister dans d'autres répertoires (héritage d'anciennes versions).
    *   Chemin : `templates/header.html`

3.  **Templates de Page Principaux :**
    *   Ces fichiers servent de référence pour la structure des nouvelles pages.
    *   Page d'accueil : `templates/index.html`
    *   Page des annonces : `templates/listings/index.html`
    *   Page d'authentification : `templates/auth/index.html`

4.  **Scripts JavaScript de Base :**
    *   Scripts à inclure pour les fonctionnalités de base et les icônes.
    *   Icônes : `static/js/lucide.min.js`
    *   Logique de base : `static/js/base.js`

