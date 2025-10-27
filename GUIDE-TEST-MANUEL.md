# Guide de Test Manuel - Marketplace NEXUS

**Date:** 2025-10-27
**Version:** v0.2.6 Alpha (Testnet)
**Durée estimée:** 30-45 minutes

---

## 🎯 Objectif

Tester le **flow complet** de bout en bout :
- Création d'utilisateurs (buyer, vendor, arbiter)
- Création de listing par le vendor
- Achat par le buyer
- Initialisation escrow multisig 2-of-3
- (Configuration wallets RPC si disponible)
- Vérifier chaque page et chaque bouton

---

## 📋 Prérequis

### 1. Serveur démarré
```bash
cd ~/Desktop/monero.marketplace
cargo build --release --package server
./target/release/server
```

✅ Le serveur devrait afficher :
```
Server running on http://127.0.0.1:8080
WebSocket server started
```

### 2. Ouvrir 3 navigateurs/sessions

**Option A:** 3 navigateurs différents
- Firefox (pour Alice - buyer)
- Chrome (pour Bob - vendor)
- Brave/Edge (pour Charlie - arbiter)

**Option B:** 3 fenêtres incognito du même navigateur
- Fenêtre incognito #1 (Alice)
- Fenêtre incognito #2 (Bob)
- Fenêtre incognito #3 (Charlie)

### 3. Préparer un fichier de notes
Crée un fichier texte pour noter :
- Les URLs des listings créés
- Les IDs des orders
- Les IDs des escrows
- Les problèmes rencontrés

---

## 📝 Scénario de Test

### PHASE 1: Création des 3 Utilisateurs

#### 🔵 Session 1 - Alice (Buyer)

**Navigateur:** Firefox (ou Incognito #1)

**1.1 - Aller sur la page d'accueil**
```
URL: http://127.0.0.1:8080
```

**Ce que tu dois voir:**
- [ ] Header NEXUS avec shield logo 🛡️
- [ ] Boutons "LOGIN" et "SIGN UP" en haut à droite
- [ ] Hero section avec "NEXUS" en grandes lettres
- [ ] Footer NEXUS avec liens

**1.2 - Cliquer sur "SIGN UP"**
```
URL: http://127.0.0.1:8080/register
```

**Remplir le formulaire:**
- Username: `alice`
- Password: `alice123`
- Confirm Password: `alice123`
- Role: **Buyer** (sélectionner dans le dropdown)

**1.3 - Soumettre le formulaire**

**Ce que tu dois voir:**
- [ ] Toast notification "🎉 Registration Successful"
- [ ] Redirection vers homepage `/`
- [ ] Header montre maintenant "👤 ALICE" avec dropdown
- [ ] Plus de boutons LOGIN/SIGN UP (remplacés par menu user)

**1.4 - Vérifier le menu utilisateur**

Clique sur "👤 ALICE" dans le header

**Ce que tu dois voir:**
- [ ] Dropdown menu s'ouvre
- [ ] "📦 MY ORDERS"
- [ ] "⚙️ SETTINGS"
- [ ] "🚪 LOGOUT"

✅ **Session Alice prête !** Ne ferme pas ce navigateur.

---

#### 🟢 Session 2 - Bob (Vendor)

**Navigateur:** Chrome (ou Incognito #2)

**2.1 - Aller sur http://127.0.0.1:8080**

**2.2 - Cliquer sur "SIGN UP"**

**Remplir le formulaire:**
- Username: `bob`
- Password: `bob123`
- Confirm Password: `bob123`
- Role: **Vendor** (⚠️ IMPORTANT - pas Buyer!)

**2.3 - Soumettre**

**Ce que tu dois voir:**
- [ ] Toast notification success
- [ ] Header montre "👤 BOB"
- [ ] Dropdown avec MY ORDERS, SETTINGS, LOGOUT

✅ **Session Bob prête !** Ne ferme pas ce navigateur.

---

#### 🟡 Session 3 - Charlie (Arbiter)

**Navigateur:** Brave/Edge (ou Incognito #3)

**3.1 - Aller sur http://127.0.0.1:8080**

**3.2 - Cliquer sur "SIGN UP"**

**Remplir le formulaire:**
- Username: `charlie`
- Password: `charlie123`
- Confirm Password: `charlie123`
- Role: **Arbiter** (⚠️ IMPORTANT!)

**3.3 - Soumettre**

**Ce que tu dois voir:**
- [ ] Toast notification success
- [ ] Header montre "👤 CHARLIE"

✅ **Session Charlie prête !** Ne ferme pas ce navigateur.

---

### PHASE 2: Bob Crée un Listing

**⚠️ Reste dans la session Bob (navigateur Chrome/Incognito #2)**

**4.1 - Aller sur la page Listings**
```
URL: http://127.0.0.1:8080/listings
```

**Ce que tu dois voir:**
- [ ] Page "NEXUS — Anonymous Marketplace"
- [ ] Liste des listings (peut être vide)
- [ ] **Chercher un bouton "Create Listing" ou "New Listing"**

**❓ Question de diagnostic:**
- **Y a-t-il un bouton pour créer un listing visible sur cette page ?**
  - ✅ OUI → Noter sa position et continuer
  - ❌ NON → Noter "MANQUE: Bouton Create Listing sur /listings"

**4.2 - Aller directement à /listings/create**
```
URL: http://127.0.0.1:8080/listings/create
```

**Ce que tu dois voir:**
- [ ] Formulaire "Create New Listing"
- [ ] Champs: Title, Description, Price (XMR), Stock
- [ ] Design NEXUS (fond dark, glassmorphism)

**4.3 - Remplir le formulaire**

**Données de test:**
- **Title:** `Premium VPN Subscription - 1 Year`
- **Description:** `High-speed VPN service with no logs policy. Supports WireGuard and OpenVPN. Access to 50+ countries. 1 year subscription.`
- **Price (XMR):** `0.5` (ou `500000000000` atomic units si le champ demande ça)
- **Stock:** `10`

**4.4 - Soumettre le formulaire**

**Ce que tu dois voir:**
- [ ] Toast notification "Listing created successfully" (ou similaire)
- [ ] Redirection vers la page du listing `/listings/{id}`
- [ ] Détails du listing affichés correctement

**4.5 - Noter l'ID du listing**

Dans la barre d'URL, copie l'ID du listing :
```
http://127.0.0.1:8080/listings/{ID-DU-LISTING}
```

📝 **Note:** ID du listing de Bob = `_________________`

**4.6 - Vérifier la page du listing**

**Ce que tu dois voir:**
- [ ] Titre: "PREMIUM VPN SUBSCRIPTION - 1 YEAR"
- [ ] Description complète
- [ ] Prix: "0.500000000000 XMR"
- [ ] Stock: 10
- [ ] Vendor: "BOB"
- [ ] Badge "ACTIVE"

**❓ Question de diagnostic:**
- **Y a-t-il un bouton "BUY NOW" ou "ADD TO CART" visible ?**
  - ✅ OUI → Noter sa position
  - ❌ NON → Noter "MANQUE: Bouton Buy sur listing page"

✅ **Listing créé !**

---

### PHASE 3: Alice Achète le Listing

**⚠️ Retourne dans la session Alice (navigateur Firefox/Incognito #1)**

**5.1 - Aller sur la page du listing de Bob**
```
URL: http://127.0.0.1:8080/listings/{ID-DU-LISTING-DE-BOB}
```

*Remplace {ID-DU-LISTING-DE-BOB} par l'ID noté à l'étape 4.5*

**Ce que tu dois voir:**
- [ ] Le listing de Bob s'affiche
- [ ] Titre: "PREMIUM VPN SUBSCRIPTION - 1 YEAR"
- [ ] Prix: 0.5 XMR
- [ ] **Chercher un bouton "BUY NOW" ou similaire**

**5.2 - Cliquer sur "BUY NOW"**

**⚠️ SI LE BOUTON N'EXISTE PAS:**

Tu vas devoir créer l'order manuellement via API. Ouvre la console du navigateur (F12) et exécute :

```javascript
fetch('http://127.0.0.1:8080/api/orders', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
  },
  credentials: 'include',
  body: JSON.stringify({
    listing_id: 'ID-DU-LISTING-ICI', // Remplace par l'ID
    quantity: 1
  })
})
.then(r => r.json())
.then(data => {
  console.log('Order created:', data);
  alert('Order ID: ' + data.id);
  window.location.href = '/orders/' + data.id;
})
.catch(err => console.error(err));
```

**5.3 - Vérifier la création de l'order**

**Ce que tu dois voir:**
- [ ] Redirection vers `/orders/{order-id}`
- [ ] Page "Order #XXXXXXXX"
- [ ] Badge "⏳ PENDING" ou "💰 FUNDED"
- [ ] Détails de la commande affichés

**5.4 - Noter l'ID de l'order**

📝 **Note:** ID de l'order d'Alice = `_________________`

**5.5 - Vérifier les informations**

**Sur la page de l'order, tu dois voir:**
- [ ] Order Details (ID, status, date)
- [ ] Listing info (titre, prix)
- [ ] Buyer: ALICE
- [ ] Vendor: BOB
- [ ] Total: 0.5 XMR

**❓ Question de diagnostic:**
- **Y a-t-il un bouton "INITIALIZE ESCROW" visible ?**
  - ✅ OUI → Noter sa position et continuer
  - ❌ NON → Noter "MANQUE: Bouton Initialize Escrow"

✅ **Order créé !**

---

### PHASE 4: Initialisation de l'Escrow Multisig

**⚠️ Toujours dans la session Alice**

**6.1 - Sur la page de l'order, chercher "Initialize Escrow"**

**6.2 - Cliquer sur "Initialize Escrow"**

**Possibilités:**
- **Cas A:** Modal s'ouvre pour sélectionner un arbiter
- **Cas B:** L'escrow s'initialise directement (arbiter auto-sélectionné)
- **Cas C:** Rien ne se passe (bouton non connecté)

**Si Modal pour sélectionner arbiter:**
- [ ] Liste des arbiters disponibles s'affiche
- [ ] Charlie devrait être dans la liste
- [ ] Sélectionner "CHARLIE"
- [ ] Cliquer "Confirm"

**⚠️ SI LE BOUTON N'EXISTE PAS:**

Initialiser via API (console F12) :

```javascript
// Trouve l'order ID dans l'URL ou dans le code HTML
const orderId = window.location.pathname.split('/').pop();

fetch(`http://127.0.0.1:8080/api/orders/${orderId}/init-escrow`, {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
  },
  credentials: 'include',
  body: JSON.stringify({
    arbiter_id: 'charlie' // Ou l'ID de Charlie si tu l'as
  })
})
.then(r => r.json())
.then(data => {
  console.log('Escrow initialized:', data);
  alert('Escrow ID: ' + data.escrow_id);
  location.reload();
})
.catch(err => console.error(err));
```

**6.3 - Vérifier la création de l'escrow**

**Ce que tu dois voir:**
- [ ] Order status change vers "ESCROW_INITIATED" ou similaire
- [ ] Un lien ou bouton vers "View Escrow" apparaît
- [ ] Ou redirection automatique vers `/escrow/{escrow-id}`

**6.4 - Aller sur la page de l'escrow**
```
URL: http://127.0.0.1:8080/escrow/{escrow-id}
```

**Ce que tu dois voir:**
- [ ] Page "Escrow #XXXXXXXX"
- [ ] Badge avec le status (PENDING ou AWAITING_FUNDING)
- [ ] Escrow Amount: 0.5 XMR
- [ ] Timeline avec les étapes du multisig
- [ ] Buyer: ALICE
- [ ] Vendor: BOB
- [ ] Arbiter: CHARLIE

**6.5 - Noter l'ID de l'escrow**

📝 **Note:** ID de l'escrow = `_________________`

✅ **Escrow initialisé !**

---

### PHASE 5: Configuration des Wallets RPC

**Cette phase teste si l'UI pour configurer les wallets RPC existe.**

**7.1 - Dans la session Alice, aller sur Settings**
```
URL: http://127.0.0.1:8080/settings/wallet
```

**Ce que tu dois voir:**
- [ ] Page "Wallet Settings" ou similaire
- [ ] Formulaire pour entrer RPC URL
- [ ] Champs: RPC URL, Username (optional), Password (optional)
- [ ] Bouton "Save" ou "Connect Wallet"

**❓ Question de diagnostic:**
- **La page /settings/wallet existe-t-elle ?**
  - ✅ OUI → Noter ce qui est affiché
  - ❌ NON (404) → Noter "MANQUE: Page wallet settings"

**Si la page existe, NE PAS remplir le formulaire pour l'instant**
(On n'a pas de vrais wallets RPC qui tournent)

**7.2 - Vérifier dans les sessions Bob et Charlie**

Répète pour Bob et Charlie :
- Session Bob → http://127.0.0.1:8080/settings/wallet
- Session Charlie → http://127.0.0.1:8080/settings/wallet

---

### PHASE 6: Vérification de la Page Escrow

**8.1 - Retourner sur la page escrow (session Alice)**
```
URL: http://127.0.0.1:8080/escrow/{escrow-id}
```

**8.2 - Vérifier la timeline des étapes**

**Tu devrais voir des étapes comme:**
- [ ] Step 1: Escrow Initiated ✓
- [ ] Step 2: Multisig Setup Complete (en attente ou actif)
- [ ] Step 3: Funds Deposited (en attente)
- [ ] Step 4: Order Shipped (en attente)
- [ ] Step 5: Funds Released (en attente)

**8.3 - Chercher des boutons d'action**

**Boutons possibles:**
- [ ] "Prepare Multisig"
- [ ] "Fund Escrow"
- [ ] "Release Funds"
- [ ] "Dispute"
- [ ] "View Multisig Address"

**Noter quels boutons sont présents et leur état (enabled/disabled)**

**8.4 - Vérifier l'affichage de l'adresse multisig**

**Si l'adresse multisig est affichée:**
- [ ] Format: 4... (adresse Monero commençant par 4)
- [ ] Bouton "Copy" à côté
- [ ] Warning "Send EXACTLY 0.5 XMR to this address"

**Si pas d'adresse multisig:**
- [ ] Message "Multisig address will be generated after setup"

---

### PHASE 7: Vérification dans les 3 Sessions

**9.1 - Session Bob (Vendor)**

Aller sur :
```
URL: http://127.0.0.1:8080/orders
```

**Ce que tu dois voir:**
- [ ] Liste des orders
- [ ] Order d'Alice visible dans la liste
- [ ] Status: PENDING ou ESCROW_INITIATED

Cliquer sur l'order pour voir les détails :
```
URL: http://127.0.0.1:8080/orders/{order-id}
```

**Depuis la vue vendor, vérifier:**
- [ ] Détails de l'order visibles
- [ ] Buyer: ALICE
- [ ] Status badge correct
- [ ] **Boutons d'action pour vendor (Ship Order, etc.)**

**9.2 - Session Charlie (Arbiter)**

Aller sur :
```
URL: http://127.0.0.1:8080/escrow/{escrow-id}
```

**Ce que tu dois voir:**
- [ ] Page escrow accessible (même pour arbiter)
- [ ] Détails de l'escrow visibles
- [ ] Role: ARBITER affiché
- [ ] **Boutons d'action pour arbiter (si dispute)**

---

### PHASE 8: Test de Navigation Générale

**10.1 - Tester le menu de navigation**

Dans chaque session (Alice, Bob, Charlie), tester ces liens :

**Header Navigation:**
- [ ] HOME → http://127.0.0.1:8080/
- [ ] LISTINGS → http://127.0.0.1:8080/listings
- [ ] VENDORS → http://127.0.0.1:8080/vendors (vérifie si ça existe)
- [ ] CATEGORIES → Scroll ou ancre #categories

**User Dropdown Menu:**
- [ ] MY ORDERS → http://127.0.0.1:8080/orders
- [ ] SETTINGS → http://127.0.0.1:8080/settings

**10.2 - Tester le bouton Search**

Cliquer sur 🔍 dans le header :
- [ ] Modal de recherche s'ouvre
- [ ] Champ de recherche fonctionnel
- ❌ Rien ne se passe → Noter "MANQUE: Search functionality"

**10.3 - Tester le Footer**

Cliquer sur les liens du footer :
- [ ] Tous les liens fonctionnent
- [ ] Pas de 404
- ❌ Certains liens 404 → Noter lesquels

---

### PHASE 9: Test de Logout

**11.1 - Dans la session Alice**

Cliquer sur "👤 ALICE" → "🚪 LOGOUT"

**Ce que tu dois voir:**
- [ ] Redirection vers homepage "/"
- [ ] Header affiche à nouveau "LOGIN" et "SIGN UP"
- [ ] Plus de menu utilisateur
- [ ] Session détruite

**11.2 - Vérifier la protection des pages**

Essaye d'aller sur :
```
URL: http://127.0.0.1:8080/orders
```

**Ce que tu dois voir:**
- [ ] Redirection vers /login
- [ ] Message "You must be logged in"
- ❌ Page s'affiche quand même → Bug de sécurité !

**11.3 - Te reconnecter**

Aller sur /login :
- Username: `alice`
- Password: `alice123`

**Ce que tu dois voir:**
- [ ] Login réussi
- [ ] Redirection vers homepage
- [ ] Session restaurée avec "👤 ALICE"

---

## 📊 Résumé des Tests

### ✅ Checklist Complète

**Frontend UI:**
- [ ] Header NEXUS avec logo et boutons auth
- [ ] Footer NEXUS avec liens
- [ ] Page registration fonctionnelle
- [ ] Page login fonctionnelle
- [ ] Logout fonctionnel
- [ ] Dropdown menu utilisateur
- [ ] Page listings existe
- [ ] Page listing detail existe
- [ ] Formulaire create listing existe
- [ ] Page order detail existe
- [ ] Page escrow detail existe
- [ ] Page settings/wallet existe

**Fonctionnalités:**
- [ ] Création de 3 users (buyer, vendor, arbiter) ✅
- [ ] Vendor peut créer un listing
- [ ] Buyer peut voir le listing
- [ ] Buyer peut créer un order (via bouton ou API)
- [ ] Escrow peut être initialisé (via bouton ou API)
- [ ] Page escrow affiche les infos correctement
- [ ] Timeline des étapes multisig visible

**Problèmes Trouvés:**
```
1. _________________________________________________
2. _________________________________________________
3. _________________________________________________
4. _________________________________________________
5. _________________________________________________
```

**Fonctionnalités Manquantes:**
```
1. _________________________________________________
2. _________________________________________________
3. _________________________________________________
```

---

## 🐛 Que Faire Ensuite ?

### Si tout fonctionne (90%+)
✅ Le flow est prêt ! Il reste juste à :
- Connecter de vrais wallets Monero RPC pour tester le multisig complet
- Tester le funding et le release

### Si des bugs sont trouvés
Je vais les corriger un par un :
1. Boutons manquants → Ajouter les boutons
2. Pages 404 → Créer les pages manquantes
3. Formulaires non connectés → Brancher les endpoints
4. Design cassé → Corriger le CSS

### Si des fonctionnalités majeures manquent
On les implémente ensemble, dans l'ordre de priorité :
1. Bouton "Buy Now" sur listing page
2. Bouton "Initialize Escrow" sur order page
3. Configuration wallet RPC
4. Boutons multisig (prepare, make, exchange, finalize)

---

## 📝 Notes pour Toi

**Pendant les tests, note:**
- Les URLs qui donnent 404
- Les boutons qui ne font rien quand tu cliques
- Les erreurs dans la console (F12)
- Les messages d'erreur du serveur (dans le terminal)
- Les choses qui manquent visuellement

**Après les tests, partage-moi:**
- La checklist complète (ce qui marche et ce qui ne marche pas)
- La liste des bugs trouvés
- Les screenshots si possible (ou descriptions textuelles)

Et je corrigerai tout ! 🚀

---

**Prêt à commencer ?** Démarre le serveur et commence par la **PHASE 1** ! 🎮
