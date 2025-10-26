# Guide des Couleurs de Statut - Monero Marketplace

## 🎨 Palette de Couleurs par Statut

### Statuts de Commande (Orders)

| Statut | Emoji | Couleur | Signification | Qui voit |
|--------|-------|---------|---------------|----------|
| **PENDING** | ⏳ | 🟠 Orange (#f59e0b) | En attente de paiement | Acheteur + Vendeur |
| **FUNDED** | 💰 | 🟢 Vert (#22c55e) | Paiement reçu, prêt à expédier | Acheteur + Vendeur |
| **SHIPPED** | 📦 | 🔵 Bleu (#3b82f6) | Colis en transit | Acheteur + Vendeur |
| **COMPLETED** | ✅ | 🟢 Vert clair (#10b981) | Transaction terminée avec succès | Acheteur + Vendeur |
| **CANCELLED** | ❌ | ⚪ Gris (#6b7280) | Commande annulée | Acheteur + Vendeur |
| **DISPUTED** | ⚠️ | 🔴 Rouge (#ef4444) | Litige en cours | Acheteur + Vendeur + Arbitre |
| **REFUNDED** | ↩️ | 🟣 Violet (#a855f7) | Fonds remboursés | Acheteur + Vendeur |

---

## 📊 Signification Visuelle

### 🟠 Orange (PENDING)
- **Signification:** Action requise
- **Pour l'acheteur:** "Vous devez payer"
- **Pour le vendeur:** "En attente du paiement"
- **Urgence:** Moyenne

### 🟢 Vert (FUNDED)
- **Signification:** Paiement confirmé, prêt pour la suite
- **Pour l'acheteur:** "Paiement effectué, en attente d'expédition"
- **Pour le vendeur:** "Vous pouvez expédier maintenant"
- **Urgence:** Action requise (vendeur)

### 🔵 Bleu (SHIPPED)
- **Signification:** En transit
- **Pour l'acheteur:** "Votre colis arrive"
- **Pour le vendeur:** "Colis expédié, en attente de confirmation"
- **Urgence:** Faible (attente)

### 🟢 Vert Clair (COMPLETED)
- **Signification:** Succès total!
- **Pour tous:** "Transaction terminée avec succès"
- **Urgence:** Aucune

### ⚪ Gris (CANCELLED)
- **Signification:** Neutre, annulé
- **Pour tous:** "Commande annulée"
- **Urgence:** Aucune

### 🔴 Rouge (DISPUTED)
- **Signification:** Problème! Attention requise
- **Pour tous:** "Un litige est en cours"
- **Urgence:** Haute

### 🟣 Violet (REFUNDED)
- **Signification:** Argent retourné
- **Pour l'acheteur:** "Vous avez été remboursé"
- **Pour le vendeur:** "Fonds retournés à l'acheteur"
- **Urgence:** Aucune

---

## 🎯 Flow Visuel Normal

```
⏳ PENDING (Orange)
    ↓ Acheteur paie
💰 FUNDED (Vert)
    ↓ Vendeur expédie
📦 SHIPPED (Bleu)
    ↓ Acheteur confirme
✅ COMPLETED (Vert clair)
```

## ⚠️ Flow avec Problème

```
💰 FUNDED (Vert)
    ↓ Problème détecté
⚠️ DISPUTED (Rouge)
    ↓ Arbitre décide
↩️ REFUNDED (Violet) ou ✅ COMPLETED (Vert)
```

## ❌ Flow d'Annulation

```
⏳ PENDING (Orange)
    ↓ Acheteur annule
❌ CANCELLED (Gris)
```

---

## 💡 Conseils UX

### Pour l'Acheteur:
- **Orange (PENDING):** Cliquez sur "Fund Escrow" pour payer
- **Vert (FUNDED):** Attendez que le vendeur expédie
- **Bleu (SHIPPED):** Attendez la livraison, puis confirmez
- **Vert clair (COMPLETED):** C'est terminé! Vous pouvez laisser un avis

### Pour le Vendeur:
- **Orange (PENDING):** Attendez le paiement
- **Vert (FUNDED):** Expédiez maintenant! Cliquez "Mark as Shipped"
- **Bleu (SHIPPED):** Attendez la confirmation de l'acheteur
- **Vert clair (COMPLETED):** Fonds reçus! Transaction terminée

---

## 🔧 Implémentation Technique

### CSS Classes
```css
.badge-pending    /* Orange avec fond transparent */
.badge-funded     /* Vert avec fond transparent */
.badge-shipped    /* Bleu avec fond transparent */
.badge-completed  /* Vert clair avec fond transparent */
.badge-cancelled  /* Gris avec fond transparent */
.badge-disputed   /* Rouge avec fond transparent */
.badge-refunded   /* Violet avec fond transparent */
```

### Structure HTML
```html
<span class="badge badge-funded">
    💰 FUNDED
</span>
```

### Caractéristiques Visuelles
- **Bordure:** 2px solid (couleur du statut)
- **Background:** rgba(couleur, 0.1) - fond semi-transparent
- **Padding:** 6px 12px
- **Font:** Bold, uppercase, 11px
- **Border-radius:** 3px

---

## 📱 Responsive

Les badges s'adaptent automatiquement:
- Desktop: Taille normale avec emoji + texte
- Mobile: Même style, mais peut être réduit si nécessaire

---

## ♿ Accessibilité

- ✅ Emoji + Texte = double indication visuelle
- ✅ Couleurs distinctives (pas seulement rouge/vert)
- ✅ Contraste élevé sur fond noir
- ✅ Bordures épaisses (2px) pour meilleure visibilité

---

## 🎨 Palette Complète (Hex)

```
Orange:     #f59e0b  (PENDING)
Vert:       #22c55e  (FUNDED)
Bleu:       #3b82f6  (SHIPPED)
Vert clair: #10b981  (COMPLETED)
Gris:       #6b7280  (CANCELLED)
Rouge:      #ef4444  (DISPUTED)
Violet:     #a855f7  (REFUNDED)
```

---

## 🚀 Améliorations Futures

- [ ] Animation de pulsation pour statuts actifs (PENDING, DISPUTED)
- [ ] Son différent par type de notification
- [ ] Historique des changements de statut avec timeline colorée
- [ ] Filtres par couleur dans la liste des commandes
