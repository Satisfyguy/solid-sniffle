# UI Improvements Implementation Plan

## 🎯 Objectif
Améliorer l'expérience utilisateur des flux d'achat et de vente en respectant strictement la charte graphique existante (#C9A445 accent, #1A1A1A background, Inter font).

## ✅ Phase 1 : Quick Wins
### 1. Convertisseur XMR ↔ Atomic Units
- **Fichiers à créer :**
  - `static/js/xmr-converter.js`
- **Fonctionnalités :**
  - Conversion bidirectionnelle en temps réel.
  - Précision 12 décimales (BigInt).
  - Validation complète (min, max, négatifs).
  - Messages d'erreur clairs.
- **Intégration :**
  - Modifier `templates/listings/create.html` pour ajouter le widget.
  - Le widget aura 2 inputs : XMR (humain) ↔ Atomic (blockchain).
  - L'input "atomic" sera en lecture seule.
  - Ajouter une bannière d'information : "1 XMR = 1,000,000,000,000 piconeros".

### 2. Checkout Stepper Visuel
- **Fichiers à créer :**
  - `static/css/checkout-stepper.css`
  - `static/js/checkout-stepper.js`
- **Fonctionnalités :**
  - 4 étapes : Shipping Info → Escrow Setup → Payment → Confirmation.
  - États : pending, active, completed, error, loading.
  - Indicateurs circulaires avec numéros et icônes.
  - Connecteurs animés.
- **Intégration :**
  - Modifier `templates/checkout/index.html` pour ajouter le stepper.
  - Synchroniser le stepper avec les sections de la page.

## ✅ Phase 2 : UX Majeure
### 3. Galerie d'Images Produit avec Lightbox
- **Fichiers à créer :**
  - `static/css/product-gallery.css`
  - `static/js/product-gallery.js`
- **Fonctionnalités :**
  - Image principale et miniatures.
  - Lightbox plein écran avec navigation clavier.
  - Compteur d'images.
  - Zoom au survol.
- **Intégration :**
  - Modifier `templates/listings/show.html` pour ajouter la galerie.
  - Passer les images via un élément de données JSON.
  - Prévoir un fallback si aucune image n'est disponible.

## 🚧 Phase 3 : Features Avancées
### 4. Système de Disputes
- **Fichiers à créer :**
  - `static/css/dispute-system.css`
  - `static/js/dispute-system.js`
- **Fonctionnalités :**
  - Modale pour le formulaire de litige.
  - Formulaire avec raison, description et upload de preuves.
  - Prévisualisation des images.
  - Compteur de caractères.
- **Intégration :**
  - Ajouter un bouton "Dispute" sur la page de commande.
  - Gérer la soumission du formulaire au backend.

## 📋 Plan d'implémentation étape par étape

1.  **Créer les fichiers CSS et JS vides :**
    - `touch static/css/checkout-stepper.css static/css/dispute-system.css static/css/product-gallery.css`
    - `touch static/js/checkout-stepper.js static/js/product-gallery.js static/js/xmr-converter.js static/js/dispute-system.js`

2.  **Implémenter le convertisseur XMR :**
    - Remplir `static/js/xmr-converter.js` avec la logique de conversion.
    - Modifier `templates/listings/create.html` pour ajouter le widget et inclure le script.

3.  **Implémenter le stepper de checkout :**
    - Remplir `static/css/checkout-stepper.css` avec les styles du stepper.
    - Remplir `static/js/checkout-stepper.js` avec la logique du stepper.
    - Modifier `templates/checkout/index.html` pour ajouter le stepper et inclure les fichiers CSS et JS.

4.  **Implémenter la galerie d'images :**
    - Remplir `static/css/product-gallery.css` avec les styles de la galerie.
    - Remplir `static/js/product-gallery.js` avec la logique de la galerie et de la lightbox.
    - Modifier `templates/listings/show.html` pour ajouter la galerie et inclure les fichiers CSS et JS.

5.  **Implémenter le système de litiges :**
    - Remplir `static/css/dispute-system.css` avec les styles du formulaire de litige.
    - Remplir `static/js/dispute-system.js` avec la logique de la modale et de l'upload de fichiers.
    - Ajouter le bouton "Dispute" sur la page de commande et intégrer le système.

## 🧪 Plan de test
### Tests manuels
1.  **Créer un listing :**
    - Aller sur `/listings/new`.
    - Vérifier que le convertisseur XMR ↔ atomic fonctionne.
    - Tester les cas limites (0, max, négatifs).
2.  **Voir un produit :**
    - Aller sur `/listings/{id}`.
    - Vérifier que la galerie d'images s'affiche avec les miniatures.
    - Tester la lightbox et la navigation au clavier.
3.  **Checkout :**
    - Aller sur `/checkout`.
    - Vérifier que le stepper visuel à 4 étapes s'affiche.
    - Tester les transitions entre les étapes.
### Tests automatisés (à créer)
- `cargo test --package server test_xmr_conversion`
- `cargo test --package server test_dispute_submission`
