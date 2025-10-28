# Guide de Test - Flow Complet de Commande

## 🎯 Objectif
Tester le flow complet d'une commande de A à Z avec simulation de paiement.

---

## 📋 Prérequis

1. **Serveur démarré:**
   ```bash
   DATABASE_URL=sqlite:marketplace.db \
   DB_ENCRYPTION_KEY=1507741993bdf8914031465a9dc63dd7e1f32a7bc2cd2b49e647042450503724 \
   cargo run -p server --bin server
   ```

2. **Deux navigateurs/onglets:**
   - Navigateur A: Vendeur
   - Navigateur B: Acheteur

---

## 🔧 Étape 0: Préparation

### Créer les comptes

**Vendeur (Navigateur A):**
1. Aller sur http://localhost:8080/register
2. Username: `vendor1`
3. Password: `password123`
4. Role: `vendor`
5. Cliquer "Register"

**Acheteur (Navigateur B):**
1. Aller sur http://localhost:8080/register
2. Username: `buyer1`
3. Password: `password123`
4. Role: `buyer`
5. Cliquer "Register"

### Créer un listing (Vendeur)

**Dans Navigateur A (vendor1):**
1. Cliquer sur "SELL" dans le menu
2. Remplir le formulaire:
   - Title: `Test Product`
   - Description: `This is a test product`
   - Price: `0.1` (XMR)
   - Stock: `10`
   - Category: `electronics`
3. Cliquer "Create Listing"
4. ✅ Listing créé!

---

## 🛒 Étape 1: Créer une Commande (Acheteur)

**Dans Navigateur B (buyer1):**
1. Aller sur la homepage: http://localhost:8080/
2. Voir le listing "Test Product"
3. Cliquer sur le listing pour voir les détails
4. Cliquer "🛒 Buy Now"
5. ✅ Commande créée avec statut **PENDING**

**Vérification:**
- URL change vers `/orders/{order_id}`
- Badge de statut: 🟡 **PENDING**
- Bouton visible: "💰 Fund Escrow"

---

## 💰 Étape 2: Initialiser l'Escrow (Acheteur)

**Dans Navigateur B (buyer1):**
1. Sur la page de la commande
2. Cliquer "💰 Fund Escrow"
3. Attendre 2-3 secondes
4. ✅ Instructions de paiement apparaissent:
   - Adresse escrow affichée
   - Montant à payer: `0.1 XMR`
   - Bouton "Copy Address"
   - Bouton orange "🧪 Simulate Payment (DEV)"

**Vérification:**
- Bouton "Fund Escrow" disparaît
- Instructions de paiement visibles
- Bouton de simulation visible

---

## 🧪 Étape 3: Simuler le Paiement (Acheteur)

**Dans Navigateur B (buyer1):**
1. Cliquer sur "🧪 Simulate Payment (DEV)"
2. Attendre 2 secondes
3. ✅ Message de succès: "Payment Simulated Successfully!"
4. Page se recharge automatiquement
5. Statut change: 🟡 PENDING → 🟢 **FUNDED**

**Vérification côté Acheteur:**
- Badge de statut: 🟢 **FUNDED**
- Bouton "Fund Escrow" n'est plus visible
- Message: "Waiting for vendor to ship..."

**Vérification côté Vendeur (Navigateur A):**
1. **IMPORTANT:** Ouvrir la console du navigateur (F12)
2. Vérifier les logs WebSocket:
   ```
   ✅ WebSocket connected
   Received notification: {OrderStatusChanged: {...}}
   ```
3. Toast notification apparaît: "💰 Order Update - Status: FUNDED - Refreshing..."
4. Page se recharge automatiquement après 2 secondes
5. Dans la liste des commandes, le statut est maintenant **FUNDED**

---

## 📦 Étape 4: Expédier (Vendeur)

**Dans Navigateur A (vendor1):**
1. Aller sur `/orders` ou cliquer sur "ORDERS" dans le menu
2. Voir la commande avec statut 🟢 **FUNDED**
3. Cliquer sur la commande pour voir les détails
4. Cliquer "📦 Mark as Shipped"
5. ✅ Statut change: 🟢 FUNDED → 🚚 **SHIPPED**

**Vérification côté Acheteur (Navigateur B):**
1. Toast notification: "📦 Order Update - Status: SHIPPED"
2. Page se recharge
3. Badge de statut: 🚚 **SHIPPED**
4. Bouton visible: "✅ Confirm Receipt"

---

## ✅ Étape 5: Confirmer Réception (Acheteur)

**Dans Navigateur B (buyer1):**
1. Sur la page de la commande
2. Cliquer "✅ Confirm Receipt"
3. ✅ Fonds libérés au vendeur
4. Statut change: 🚚 SHIPPED → ✅ **COMPLETED**

**Vérification côté Vendeur (Navigateur A):**
1. Toast notification: "✅ Order Update - Status: COMPLETED"
2. Page se recharge
3. Badge de statut: ✅ **COMPLETED**

---

## 🐛 Debugging: Si ça ne fonctionne pas

### Problème: Le vendeur ne voit pas le changement de statut

**Vérifications:**

1. **WebSocket connecté?**
   - Ouvrir console navigateur (F12)
   - Chercher: `✅ WebSocket connected`
   - Si absent, WebSocket ne fonctionne pas

2. **Notification reçue?**
   - Dans la console, chercher: `Received notification:`
   - Si absent, le serveur n'envoie pas la notification

3. **Logs serveur:**
   ```bash
   # Dans le terminal où le serveur tourne, chercher:
   INFO server: DEV: Simulated payment for order ...
   INFO server: Sent payment notification to vendor ...
   ```

4. **Recharger manuellement:**
   - Si WebSocket ne fonctionne pas, recharger la page manuellement (F5)
   - Le statut devrait être mis à jour

### Problème: Erreur lors de la simulation

**Erreurs possibles:**

1. **"Order has no escrow"**
   - Solution: Cliquer d'abord sur "Fund Escrow"

2. **"Only the buyer can simulate payment"**
   - Solution: Vous êtes connecté comme vendeur, connectez-vous comme acheteur

3. **"Order not found"**
   - Solution: L'ID de commande est invalide, créer une nouvelle commande

### Commandes utiles

**Voir les logs en temps réel:**
```bash
# Dans le terminal du serveur, les logs s'affichent automatiquement
```

**Vérifier la base de données:**
```bash
# Si vous avez sqlite3 installé:
sqlite3 marketplace.db "SELECT id, status FROM orders;"
sqlite3 marketplace.db "SELECT id, status FROM escrows;"
```

**Redémarrer le serveur:**
```bash
pkill -9 -f "target/debug/server"
DATABASE_URL=sqlite:marketplace.db \
DB_ENCRYPTION_KEY=1507741993bdf8914031465a9dc63dd7e1f32a7bc2cd2b49e647042450503724 \
cargo run -p server --bin server
```

---

## ✅ Checklist Complète

- [ ] Serveur démarré
- [ ] Compte vendeur créé
- [ ] Compte acheteur créé
- [ ] Listing créé par vendeur
- [ ] Commande créée par acheteur (statut: PENDING)
- [ ] Escrow initialisé (bouton Fund Escrow)
- [ ] Paiement simulé (bouton Simulate Payment)
- [ ] Statut change à FUNDED (acheteur + vendeur)
- [ ] Vendeur reçoit notification WebSocket
- [ ] Vendeur expédie (statut: SHIPPED)
- [ ] Acheteur reçoit notification
- [ ] Acheteur confirme réception (statut: COMPLETED)
- [ ] Flow complet terminé! 🎉

---

## 🎉 Succès!

Si toutes les étapes fonctionnent, le système de commandes est opérationnel!

**Prochaines étapes:**
- Tester avec de vrais XMR sur testnet
- Implémenter le système de litiges
- Ajouter le délai de rétractation (48h)
- Améliorer les notifications Tor-compatible
