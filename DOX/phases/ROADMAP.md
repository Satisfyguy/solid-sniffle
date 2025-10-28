# Monero Marketplace - Roadmap & Feature Planning

## 🎯 Phase 1: Expérience Utilisateur (UX) - PRIORITÉ HAUTE

### 1.1 Notifications en Temps Réel (Tor-Compatible)
**Status:** 🟡 Partiellement implémenté (WebSocket actif)

**À ajouter:**
- [ ] **Système de polling Tor-friendly**
  - Alternative au WebSocket pour utilisateurs Tor
  - Endpoint `/api/notifications/poll` avec long-polling
  - Fallback automatique si WebSocket échoue
  
- [ ] **Types de notifications supplémentaires:**
  - [x] ✅ Nouvelle commande (vendeur)
  - [x] ✅ Changement de statut
  - [ ] ⏰ Paiement confirmé (avec nombre de confirmations)
  - [ ] 📦 Colis expédié (avec numéro de suivi optionnel)
  - [ ] 💬 Nouveau message (système de messagerie à implémenter)
  - [ ] ⚠️ Délais qui expirent (ex: "Paiement attendu depuis 24h")
  - [ ] 🔔 Rappels automatiques

**Implémentation technique:**
```rust
// Nouveau système de polling pour Tor
#[get("/api/notifications/poll")]
pub async fn poll_notifications(
    session: Session,
    query: web::Query<PollQuery>,
) -> impl Responder {
    // Long-polling: attend jusqu'à 30s pour nouvelles notifs
    // Compatible Tor (pas de WebSocket)
}

// Notifications avec délais
struct NotificationScheduler {
    // Vérifie toutes les heures les délais expirés
    // Ex: "Commande non payée depuis 24h"
}
```

**Estimation:** 3-4 jours de développement

---

### 1.2 Tutoriel Interactif Multisig
**Status:** ❌ Non implémenté

**Fonctionnalités:**
- [ ] **Guide pas-à-pas pour première transaction**
  - Étape 1: Configuration du wallet Monero
  - Étape 2: Comprendre le multisig 2-of-3
  - Étape 3: Première commande guidée
  - Étape 4: Simulation de paiement (testnet)
  
- [ ] **Tooltips contextuels**
  - Bulles d'aide sur chaque action importante
  - Explications des termes techniques (escrow, multisig, etc.)
  
- [ ] **Mode démo/sandbox**
  - Testnet Monero intégré
  - Faux produits pour tester
  - Simulation complète du flow

**Design UI:**
```html
<!-- Overlay de tutoriel -->
<div class="tutorial-overlay">
  <div class="tutorial-step">
    <h3>🔐 Étape 1: Comprendre l'Escrow Multisig</h3>
    <p>Vos fonds sont sécurisés par 3 clés:</p>
    <ul>
      <li>✓ Votre clé (acheteur)</li>
      <li>✓ Clé du vendeur</li>
      <li>✓ Clé de l'arbitre</li>
    </ul>
    <p>2 signatures sur 3 sont nécessaires pour débloquer les fonds.</p>
    <button>Suivant →</button>
  </div>
</div>
```

**Estimation:** 5-6 jours de développement

---

### 1.3 Estimation des Frais Réseau
**Status:** ❌ Non implémenté

**Fonctionnalités:**
- [ ] **Calcul des frais Monero en temps réel**
  - Interroger le daemon Monero pour les frais actuels
  - Afficher avant confirmation de paiement
  - Mise à jour toutes les 5 minutes
  
- [ ] **Affichage transparent:**
  ```
  Prix du produit:     0.123456789012 XMR
  Frais réseau (est.): 0.000012345678 XMR
  ─────────────────────────────────────
  Total à payer:       0.123469134690 XMR
  ```

**Implémentation:**
```rust
// Récupérer les frais du daemon
async fn get_network_fees() -> Result<u64> {
    let daemon = MoneroDaemon::connect()?;
    let fee_estimate = daemon.get_fee_estimate()?;
    Ok(fee_estimate.fee)
}

// Afficher dans le template
ctx.insert("network_fee_xmr", &format_xmr(network_fee));
ctx.insert("total_with_fees", &format_xmr(price + network_fee));
```

**Estimation:** 2 jours de développement

---

## ⚖️ Phase 2: Arbitrage & Litiges - PRIORITÉ HAUTE

### 2.1 Critères d'Arbitrage Transparents
**Status:** ❌ Non implémenté

**Fonctionnalités:**
- [ ] **Page publique des règles d'arbitrage**
  - Critères de décision clairs
  - Exemples de cas résolus (anonymisés)
  - FAQ sur les litiges
  
- [ ] **Preuves requises:**
  - [ ] Photos du produit reçu (upload obligatoire)
  - [ ] Preuve d'expédition (numéro de suivi)
  - [ ] Conversation vendeur/acheteur (historique)
  - [ ] Délais de réponse (48h max par partie)

**Structure de données:**
```rust
struct DisputeEvidence {
    dispute_id: Uuid,
    submitted_by: Uuid,  // buyer ou vendor
    evidence_type: EvidenceType,  // Photo, TrackingNumber, Message
    file_ipfs_cid: Option<String>,  // Photo stockée sur IPFS
    description: String,
    submitted_at: DateTime<Utc>,
}

enum EvidenceType {
    ProductPhoto,
    ShippingProof,
    ConversationLog,
    Other,
}
```

**Page de règles:**
```markdown
# Règles d'Arbitrage

## Critères de Décision

1. **Preuve d'expédition**
   - Numéro de suivi valide = +50 points vendeur
   - Pas de preuve = -50 points vendeur

2. **Photos du produit**
   - Photos claires du défaut = +50 points acheteur
   - Pas de photos = -30 points acheteur

3. **Communication**
   - Réponse rapide (<48h) = +20 points
   - Pas de réponse = -50 points

4. **Historique**
   - Compte ancien (>6 mois) = +10 points
   - Réputation élevée = +30 points
```

**Estimation:** 4-5 jours de développement

---

### 2.2 Délai de Rétractation
**Status:** ❌ Non implémenté

**Fonctionnalités:**
- [ ] **Période de grâce de 24-48h**
  - Après "Confirmer réception", délai avant libération finale
  - Permet de détecter défauts cachés
  - Notification au vendeur du délai
  
- [ ] **États intermédiaires:**
  ```
  shipped → delivered → confirmed → [GRACE_PERIOD] → completed
                                          ↓
                                    (48h pour ouvrir litige)
  ```

**Implémentation:**
```rust
// Nouveau statut
enum OrderStatus {
    Pending,
    Funded,
    Shipped,
    Delivered,      // ← NOUVEAU: Acheteur a confirmé
    GracePeriod,    // ← NOUVEAU: 48h avant libération
    Completed,
    // ...
}

// Job scheduler
async fn check_grace_periods() {
    // Toutes les heures, vérifier les commandes en grace_period
    // Si >48h écoulées, passer à completed et libérer fonds
}
```

**Timeline:**
```
Jour 0: Acheteur confirme réception → status: grace_period
Jour 1: Notification "24h restantes pour ouvrir un litige"
Jour 2: Fin du délai → Libération automatique des fonds → status: completed
```

**Estimation:** 3 jours de développement

---

### 2.3 Pool d'Arbitres
**Status:** 🟡 Partiellement implémenté (1 arbitre système)

**Fonctionnalités:**
- [ ] **Système multi-arbitres**
  - Minimum 3 arbitres actifs
  - Rotation automatique (round-robin ou aléatoire)
  - Spécialisation par catégorie (électronique, vêtements, etc.)
  
- [ ] **Dashboard arbitre:**
  - [ ] Liste des litiges assignés
  - [ ] Outils de décision (voir preuves, historique)
  - [ ] Statistiques (taux de résolution, temps moyen)
  
- [ ] **Système de réputation des arbitres:**
  - [ ] Note par les utilisateurs après résolution
  - [ ] Transparence des décisions
  - [ ] Révocation si trop de plaintes

**Structure:**
```rust
struct Arbiter {
    user_id: Uuid,
    specializations: Vec<Category>,  // Catégories de produits
    active_disputes: u32,
    max_disputes: u32,  // Limite de charge
    reputation_score: f32,
    total_resolved: u32,
    avg_resolution_time_hours: f32,
}

// Sélection intelligente
fn select_arbiter(dispute: &Dispute) -> Result<Uuid> {
    // 1. Filtrer par spécialisation
    // 2. Exclure si surchargé (>max_disputes)
    // 3. Préférer meilleure réputation
    // 4. Round-robin parmi les éligibles
}
```

**Estimation:** 6-7 jours de développement

---

## 📊 Résumé des Priorités

| Fonctionnalité | Priorité | Complexité | Temps estimé | Impact UX |
|----------------|----------|------------|--------------|-----------|
| **Notifications Tor-compatible** | 🔴 Haute | Moyenne | 3-4 jours | ⭐⭐⭐⭐⭐ |
| **Estimation frais réseau** | 🟡 Moyenne | Faible | 2 jours | ⭐⭐⭐⭐ |
| **Délai de rétractation** | 🔴 Haute | Moyenne | 3 jours | ⭐⭐⭐⭐⭐ |
| **Critères arbitrage transparents** | 🔴 Haute | Moyenne | 4-5 jours | ⭐⭐⭐⭐ |
| **Pool d'arbitres** | 🟡 Moyenne | Haute | 6-7 jours | ⭐⭐⭐ |
| **Tutoriel interactif** | 🟢 Basse | Haute | 5-6 jours | ⭐⭐⭐⭐ |

**Total estimé:** ~25-30 jours de développement

---

## 🚀 Ordre de Développement Recommandé

### Sprint 1 (1 semaine)
1. ✅ Estimation des frais réseau (2j)
2. ✅ Délai de rétractation (3j)
3. ✅ Notifications Tor-compatible - base (2j)

### Sprint 2 (1 semaine)
4. ✅ Critères d'arbitrage + page de règles (5j)
5. ✅ Upload de preuves (photos IPFS) (2j)

### Sprint 3 (1 semaine)
6. ✅ Pool d'arbitres - base (4j)
7. ✅ Dashboard arbitre (3j)

### Sprint 4 (1 semaine)
8. ✅ Tutoriel interactif (5j)
9. ✅ Notifications complètes (2j)

---

## 📝 Notes Techniques

### Compatibilité Tor
- WebSocket peut ne pas fonctionner sur Tor
- Solution: Long-polling comme fallback
- Tester avec Tor Browser

### IPFS pour Preuves
- Photos de litiges stockées sur IPFS (décentralisé)
- Pas de serveur centralisé
- Chiffrement optionnel des preuves sensibles

### Sécurité Arbitres
- Multi-signature pour décisions importantes
- Logs immuables de toutes les décisions
- Révocation automatique si abus détecté

---

## 🎯 Objectif Final

**Marketplace Monero de confiance avec:**
- ✅ Escrow multisig sécurisé
- ✅ Notifications temps réel (WebSocket + polling)
- ✅ Arbitrage transparent et équitable
- ✅ Protection acheteur (délai de rétractation)
- ✅ UX simple malgré la complexité technique
- ✅ 100% décentralisé (IPFS + Monero)
- ✅ Compatible Tor

**Vision:** Amazon de Monero, mais décentralisé et privé! 🚀
