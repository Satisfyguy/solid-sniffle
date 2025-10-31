# Plan d'implémentation détaillé pour la page /home2

L'objectif est de créer un environnement isolé pour cette nouvelle page afin de garantir qu'elle ne soit pas affectée par le style et la configuration du projet principal.

**Plan d'implémentation détaillé et mis à jour**

1.  **Gestion des Polices en Local :**
    *   **Téléchargement**: La police "Inter" (graisses 300, 400 et 500) a été téléchargée depuis le dépôt GitHub officiel (`rsms/inter`).
    *   **Stockage**: Les fichiers de police (`.woff2`) ont été sauvegardés dans le dossier `static/fonts/inter/`.
    *   **Intégration CSS**: Le fichier `static/css/home2.css` a été modifié pour inclure les règles `@font-face` chargeant les polices depuis `static/fonts/inter/`.
    *   **Nettoyage HTML**: La balise `<link>` faisant appel à Google Fonts a été supprimée du fichier `templates/home2.html`.

2.  **Isolation des Fichiers Statiques et des Templates :**
    *   **HTML**: Le fichier `templates/home2.html` a été créé avec le code HTML extrait de `newdes.md`.
    *   **CSS**: La feuille de style `static/css/home2.css` a été créée avec le code CSS extrait de `newdes.md`.
    *   **Liaison**: Le template `home2.html` est lié uniquement à la feuille de style locale `home2.css`.

3.  **Création d'un Serveur Dédié pour `/home2` :**
    *   Un nouveau binaire `server/src/bin/home2_server.rs` a été créé, contenant un serveur Actix-Web minimaliste et indépendant.

4.  **Configuration de la Compilation :**
    *   Une nouvelle entrée `[[bin]]` a été ajoutée dans `server/Cargo.toml` pour définir `home2_server` comme un exécutable distinct.

5.  **Implémentation du Serveur `home2_server.rs` :**
    *   Le serveur est configuré pour :
        *   Utiliser Tera pour rendre le template `home2.html`.
        *   Servir les fichiers statiques (y compris les nouvelles polices).
        *   Définir une unique route `/home2`.
        *   Écouter sur le port `8082` (initialement 8081, mais modifié en raison d'un conflit de port).

6.  **Lancement et Vérification :**
    *   Démarrage du serveur isolé avec `cargo run --bin home2_server`.
    *   Vérification de la page sur `http://127.0.0.1:8082/home2` pour s'assurer de la fidélité visuelle et de l'absence de requêtes externes.

**Détails des dossiers et fichiers créés/modifiés :**

*   `nexushero.md`: Ce document de plan.
*   `templates/home2.html`: Nouveau fichier HTML pour la page indépendante.
*   `static/css/home2.css`: Nouvelle feuille de style CSS pour la page indépendante, incluant les règles `@font-face`.
*   `static/fonts/inter/`: Nouveau dossier contenant les fichiers de police Inter (`.woff2`).
*   `server/src/bin/home2_server.rs`: Nouveau fichier source Rust pour le serveur dédié.
*   `server/Cargo.toml`: Modifié pour inclure la section `[[bin]]` pour `home2_server`.

---

# Plan de Migration : De la Page Statique à la Page d'Accueil Dynamique

1.  **Phase 1 : Préparation et Intégration du Design de Base**
    *   **Nettoyage :** Je vais d'abord supprimer le serveur de test (`home2_server.rs`) que nous avons créé, car nous allons maintenant intégrer le design dans le serveur principal du projet. Je nettoierai également le fichier `server/Cargo.toml`.
    *   **Sauvegarde :** Je vais sauvegarder votre page d'accueil actuelle (`templates/index.html`) sous le nom `templates/index.html.bak` pour pouvoir s'y référer au besoin.
    *   **Intégration :** Je remplacerai le contenu de `templates/index.html` par le code de notre nouvelle page. Le fichier `static/css/home2.css` sera conservé et utilisé comme base stylistique principale pour le projet.

2.  **Phase 2 : Analyse et Cartographie des Fonctionnalités Existantes**
    *   Je vais analyser en profondeur le code de votre serveur principal (`server/src/main.rs` et les handlers dans `server/src/handlers/`) ainsi que les anciens templates.
    *   L'objectif est de dresser une liste exhaustive de toutes les fonctionnalités et données dynamiques qui doivent être transférées. Cela inclut :
        *   L'authentification des utilisateurs (affichage conditionnel de "Login" ou "Profile").
        *   L'affichage des "listings" (produits).
        *   La fonctionnalité de recherche.
        *   Le système de panier (`cart`).
        *   Les notifications en temps réel (WebSockets).
        *   Tout autre élément dynamique.

3.  **Phase 3 : Migration Incrémentale des Fonctionnalités**
    *   Je vais aborder la migration fonctionnalité par fonctionnalité pour garantir un processus maîtrisé.
    *   **Navigation Dynamique :** J'intégrerai la logique pour afficher les boutons "Login/Register" pour les visiteurs, et "Profile/Logout" pour les utilisateurs connectés, en lieu et place des boutons statiques actuels.
    *   **Intégration des Données :** Je modifierai les "handlers" du serveur pour qu'ils passent les données nécessaires (comme la liste des produits, les informations de l'utilisateur, etc.) au nouveau template de la page d'accueil.
    *   **Portage des Composants :** Je vais recréer les éléments d'interface des fonctionnalités existantes (comme la grille des produits, la barre de recherche) à l'intérieur du nouveau design, en adaptant leur style pour qu'il soit cohérent avec la nouvelle direction artistique.
    *   **Connexion des Interactions :** Je m'assurerai que toutes les interactions (clics sur les boutons, soumission de formulaires) déclenchent les bonnes actions, en réutilisant la logique HTMX et JavaScript existante.

4.  **Phase 4 : Vérification et Finalisation**
    *   Une fois toutes les fonctionnalités migrées, nous effectuerons une passe de vérification complète pour s'assurer que tout fonctionne comme avant, mais dans le nouveau design.
    *   Je nettoierai le code, supprimerai les fichiers devenus inutiles et m'assurerai que la performance et la sécurité sont maintenues.
