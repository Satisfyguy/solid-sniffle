### Plan d'Implémentation : Messagerie de Litige Sécurisée (E2EE)

L'objectif est de permettre à l'acheteur, au vendeur et à l'arbitre d'un litige de communiquer de manière sécurisée. Le serveur ne doit agir que comme un simple relais pour les messages chiffrés.

---

#### **Principes Directeurs (Mis à jour)**

1.  **Authentification Forte :** L'identité de chaque participant est prouvée par une signature Ed25519 sur toutes les clés de session.
2.  **Confidentialité et Forward Secrecy :** Chiffrement E2EE avec rotation de clés (Double Ratchet) pour protéger les messages passés et futurs.
3.  **Intégrité :** Chaque message est signé pour garantir qu'il n'a pas été altéré.
4.  **Déni plausible :** L'architecture doit viser à ce qu'il soit difficile de prouver de manière irréfutable à un tiers qu'un utilisateur a bien envoyé un message spécifique (ceci est un objectif avancé du Double Ratchet).
5.  **Minimisation des données serveur :** Le serveur ne relaie que des blobs chiffrés et ne peut en déduire que le strict minimum de métadonnées.

---

### **Phase 1 : Structures de Données et Cryptographie (Module `common` et nouveau module `dispute-crypto`)**

1.  **Mettre à jour `common/src/types.rs` :**
    *   Définir les nouvelles structures `SignedSessionKey` et `EncryptedMessage` comme vous l'avez suggéré, pour qu'elles soient partagées entre le serveur et les clients.

2.  **Créer un nouveau Crate `dispute-crypto` (compilable en WASM) :**
    *   Ce module contiendra toute la logique cryptographique sensible.
    *   Implémenter la structure `DisputeCrypto` qui gérera l'état d'une session de chiffrement pour un litige.
    *   **Fonction `init_session()` :** Génère une paire de clés de session X25519 et retourne la clé publique **signée par la clé d'identité Ed25519**.
    *   **Fonction `establish_shared_secret()` :** Prend la clé de session signée d'un pair, **vérifie la signature Ed25519**, puis calcule le secret partagé via ECDH (X25519).
    *   **Fonction `encrypt_message()` :**
        *   Implémente le chiffrement pour multi-destinataires.
        *   Utilise HKDF pour dériver une clé de chiffrement à partir du secret partagé.
        *   Chiffre le message avec `ChaCha20-Poly1305`.
        *   Signe le message chiffré pour garantir l'intégrité.
    *   **Fonction `decrypt_message()` :**
        *   Vérifie la signature du message.
        *   Dérive la clé de la même manière et déchiffre avec `ChaCha20-Poly1305`.
    *   **Intégrer le Double Ratchet :** Ajouter la logique pour faire avancer l'état du "ratchet" après chaque échange de message, en supprimant les anciennes clés de déchiffrement pour assurer la forward secrecy.

### **Phase 2 : API Backend (Module `server`)**

1.  **Créer les nouveaux Handlers et Routes :**
    *   `POST /api/disputes/{dispute_id}/session-keys` : Reçoit une `SignedSessionKey`, vérifie la signature Ed25519, et la stocke si elle est valide.
    *   `GET /api/disputes/{dispute_id}/session-keys` : Retourne les clés de session signées pour les participants d'un litige.
    *   `POST /api/disputes/{dispute_id}/messages` : Reçoit un `EncryptedMessage`, vérifie la signature externe, et le stocke.
    *   `GET /api/disputes/{dispute_id}/messages` : Permet de récupérer les messages après un certain horodatage.

2.  **Mettre à jour la Base de Données :**
    *   Ajouter les tables `dispute_session_keys` et `dispute_messages` pour stocker les objets chiffrés.

### **Phase 3 : Intégration Frontend (WASM & JS)**

1.  **Compiler `dispute-crypto` en WASM.**
2.  **Développer la logique JavaScript (`DisputeChat`) :**
    *   Orchestrer le flux : `initialize` -> `publishSessionKey` -> `getSessionKeys` -> `establish_shared_secret`.
    *   Intégrer la logique d'envoi multi-destinataires.
    *   Mettre en place un polling (ou un WebSocket) pour appeler `receiveMessages` et afficher les messages déchiffrés.
