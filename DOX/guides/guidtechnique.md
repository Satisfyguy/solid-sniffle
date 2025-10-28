Guide Technique : Construire une Marketplace Non-Custodial avec Monero Multisig 2/3
1. Introduction : L'Architecture de la Confiance Décentralisée
Créer une marketplace en ligne sécurisée représente un défi fondamental : comment garantir qu'une transaction entre deux inconnus se déroulera sans encombre, sans qu'une entité centrale ne doive détenir les fonds en séquestre ? Ce modèle, dit "non-custodial", où la plateforme n'a jamais le contrôle direct des actifs de ses utilisateurs, est l'idéal en matière de sécurité et d'autonomie. Le protocole Monero (XMR), avec ses garanties de confidentialité natives et sa fonctionnalité de multi-signature (multisig), offre une solution technique particulièrement élégante pour construire une telle architecture de confiance décentralisée.
Le cœur de notre approche repose sur un schéma de signature multi-signature 2/3. Ce mécanisme distribue le contrôle des fonds entre trois parties, exigeant que deux d'entre elles approuvent toute transaction pour qu'elle soit valide. Dans le contexte d'une marketplace, les rôles sont clairement définis :
L'Acheteur (détenteur de la Clé 1) : Co-signataire de la transaction.
Le Vendeur (détenteur de la Clé 2) : L'autre co-signataire de la transaction.
L'Arbitre/Plateforme (détenteur de la Clé 3) : Un tiers de confiance qui n'intervient qu'en cas de désaccord.
Ce modèle permet deux scénarios principaux :
Cas Normal : La transaction se déroule comme prévu. L'Acheteur et le Vendeur signent conjointement (atteignant le seuil de 2/3) pour libérer les fonds de l'adresse séquestre vers le portefeuille du Vendeur. La plateforme n'a jamais besoin d'intervenir.
Cas de Litige : Un désaccord survient. L'Arbitre (la plateforme) intervient, examine la situation et utilise sa clé pour signer une transaction avec l'Acheteur (pour un remboursement) ou avec le Vendeur (pour finaliser le paiement). Dans les deux cas, le seuil de 2/3 signatures est atteint, et les fonds sont débloqués sans qu'une partie puisse les bloquer indéfiniment.
Ce guide technique décomposera le processus de développement d'une telle marketplace en cinq phases distinctes, de la conception de l'architecture système à la gestion sécurisée des transactions et des litiges.
2. Phase 1 : Architecture Système et Stack Technique
Le choix d'une architecture robuste et d'une stack technique adaptée est une décision stratégique qui conditionne l'ensemble du projet. Pour une application gérant des transactions financières décentralisées, cette fondation détermine non seulement la scalabilité et la maintenabilité, mais surtout la sécurité et la résilience du système. Une architecture bien pensée est la première ligne de défense dans un environnement où la confiance est distribuée.
Définition de la Stack Technique
Bien que flexible, une stack technique éprouvée est recommandée pour garantir la stabilité. Voici une suggestion de composants pour construire la marketplace :
Composant
Suggestion Technologique
Backend
Go ou Python (Django/Flask) pour leur robustesse et leurs bibliothèques matures.
Frontend
React ou Vue.js pour créer des interfaces utilisateur réactives et modulaires.
Base de Données
PostgreSQL (SQL) pour son intégrité transactionnelle ou MongoDB (NoSQL) pour sa flexibilité.

Architecture en Microservices
Une architecture en microservices est particulièrement pertinente ici. Elle permet d'isoler les responsabilités, de renforcer la sécurité en compartimentant les fonctions critiques, et d'améliorer la résilience globale. Si un service rencontre un problème, les autres peuvent continuer à fonctionner.
Les services essentiels pour notre marketplace seraient :
Service Utilisateurs : Gère les profils, l'authentification (sans jamais stocker de clés privées), et les informations de contact chiffrées. Ce service gère également les informations publiques nécessaires à la construction des portefeuilles multisig.
Service Annonces : Responsable de la création, de l'affichage, de la recherche et de la gestion des produits ou services listés sur la plateforme.
Service d'Arbitrage (Tiers de confiance) : Fournit l'interface pour la gestion des litiges. Il gère la communication sécurisée entre l'acheteur, le vendeur et l'arbitre, ainsi que le stockage des preuves (messages, photos) soumises hors chaîne.
Service d'Interaction Monero : C'est le cœur technique du système. Ce service isolé et hautement sécurisé doit être le seul point de contact avec le réseau Monero, agissant comme un "proxy sécurisé" pour l'ensemble de l'application. Il communique avec le démon Monero (monero-daemon) et le portefeuille RPC (monero-wallet-rpc), et est responsable de la construction des transactions, de la surveillance de la blockchain et de la diffusion des transactions finalisées. Il doit être conçu pour ne jamais avoir accès aux clés privées des utilisateurs.
Schéma de la Base de Données
La structure de la base de données doit refléter cette architecture modulaire. Voici les tables principales et leurs champs essentiels :
Utilisateurs
user_id (Clé primaire)
username
public_key_info_for_multisig (Informations publiques partagées pour initier le multisig)
encrypted_contact_info (Informations de contact chiffrées)
Annonces
listing_id (Clé primaire)
seller_id (Clé étrangère vers Utilisateurs)
title
description
price_xmr
Transactions
transaction_id (Clé primaire)
listing_id (Clé étrangère vers Annonces)
buyer_id (Clé étrangère vers Utilisateurs)
seller_id (Clé étrangère vers Utilisateurs)
multisig_address (L'adresse de dépôt 2/3)
status (ex: "en attente de dépôt", "fonds déposés", "libéré", "en litige")
PortefeuillesMultisig
multisig_address (Clé primaire)
related_transaction_id (Clé étrangère vers Transactions)
participant_keys_info (Stockage des informations publiques des 3 participants)
sync_data (Données de synchronisation sérialisées, telles que les images de clés partielles (partial key images) et les transactions non finalisées, nécessaires à la co-signature.)
Une fois l'architecture globale et le modèle de données définis, l'étape suivante consiste à plonger dans la mécanique spécifique à Monero pour la gestion des clés et la création sécurisée des portefeuilles multisig.
3. Phase 2 : Gestion des Clés et Création du Portefeuille Multisig
Cette phase constitue le cœur du modèle non-custodial. La génération et la gestion des clés doivent impérativement s'effectuer côté client pour garantir que la plateforme n'ait jamais le contrôle, même partiel, des fonds des utilisateurs. Contrairement aux plateformes basées sur des smart contracts (comme Gnosis Safe sur Ethereum), le multisig de Monero est une fonctionnalité au niveau du portefeuille (wallet-level) qui n'est pas visible sur la blockchain elle-même. Monero émule cette fonctionnalité via un schéma de "division de secret" (secret splitting), où la clé de dépense est partagée entre les participants. Ce processus collaboratif assure qu'aucun acteur unique ne peut disposer des fonds.
Avertissement : L'implémentation de la fonctionnalité multi-signature dans Monero est considérée comme expérimentale et peut contenir des bugs. Son utilisation comporte un risque élevé d'erreurs et est recommandée principalement pour des développeurs Monero expérimentés ou dans le cadre de solutions commerciales auditées.
Étapes de Création du Portefeuille Multisig 2/3 (via CLI)
Le processus de création d'un portefeuille multisig s'effectue via le portefeuille en ligne de commande (monero-wallet-cli) et nécessite un échange coordonné d'informations entre les trois participants (Acheteur, Vendeur, Arbitre).
Préparation Locale : Chaque participant doit générer ses propres clés Monero sur sa machine locale. Il est crucial que les clés privées de l'Acheteur et du Vendeur ne soient jamais transmises au serveur de la marketplace. Seules les informations publiques nécessaires seront échangées.
Activation du Mode Expérimental : Chaque participant doit activer le mode expérimental dans son CLI avec la commande suivante : set enable-multisig-experimental 1
Génération des Informations d'Initialisation : Chaque participant exécute la commande prepare_multisig. Cette commande génère une longue chaîne de caractères (ex: MultisigV1... ou MultisigxV2...). Cette chaîne contient des informations essentielles, y compris la clé de vue privée, et doit être partagée de manière sécurisée avec les deux autres participants. C'est via ce mécanisme que les trois parties pourront surveiller l'adresse commune.
Création de l'Adresse Multisig : Un des participants (par exemple, la plateforme agissant en tant qu'initiateur) collecte les informations d'initialisation des deux autres. Il exécute ensuite la commande make_multisig 2 <info_de_l_acheteur> <info_du_vendeur> pour créer l'adresse multisig. Le seuil '2' indique que deux signatures sont requises, et les deux chaînes d'information proviennent des deux autres participants. Cette commande génère une deuxième série de données qui doit être renvoyée aux autres participants.
Échange Final et Finalisation : Les autres participants reçoivent cette deuxième série de données et l'utilisent pour finaliser la création de leur portefeuille multisig local avec la commande exchange_multisig_keys. À l'issue de cette étape, les trois participants possèdent une copie du même portefeuille, partageant la même adresse publique mais chacun ne détenant qu'une partie de la clé de dépense.
Synchronisation et Surveillance du Portefeuille
Une fois le portefeuille créé, tous les participants partagent la même adresse publique et, surtout, la même clé de vue privée. Cela leur permet de voir le solde de l'adresse et de surveiller les transactions entrantes, mais pas de dépenser les fonds unilatéralement.
Pour maintenir les portefeuilles synchronisés et permettre la co-signature, un échange constant d'informations est nécessaire. Le Multisig Messaging System (MMS) est l'outil principal conçu pour cette tâche. Il utilise PyBitmessage comme unique canal de communication supporté pour transférer de manière sécurisée les données de synchronisation entre les portefeuilles des participants.
Une fois le portefeuille multisig créé et que l'acheteur y a déposé les fonds, le processus peut suivre le "happy path" de la transaction, où les deux parties sont en accord.
4. Phase 3 : Déroulement d'une Transaction Standard ("Happy Path")
Le flux idéal d'une transaction, ou "happy path", est celui où l'acheteur et le vendeur parviennent à un accord sans accroc. Ce processus collaboratif est la clé de voûte de la marketplace, illustrant le fonctionnement pratique du séquestre 2/3 : les fonds sont débloqués par la coopération directe des deux parties concernées, sans intervention de la plateforme.
Le processus se déroule en trois étapes clés :
1. Dépôt par l'Acheteur
Une fois que le portefeuille multisig 2/3 a été créé et que son adresse publique est connue de tous, l'acheteur initie la transaction en envoyant le montant convenu en XMR à cette adresse. Les trois participants peuvent vérifier la réception des fonds grâce à leur clé de vue privée partagée.
2. Construction et Signature Partielle
Lorsque l'acheteur confirme avoir reçu le bien ou le service, la phase de libération des fonds commence. L'un des deux participants (généralement le vendeur) initie la transaction de sortie.
Tout d'abord, les portefeuilles des deux signataires doivent être synchronisés. Pour cela, ils échangent des informations en utilisant les commandes export_multisig_info et import_multisig_info. Cette étape est cruciale pour obtenir une "image de clé partielle" (partial key image) valide, sans laquelle la transaction échouerait.
L'initiateur crée ensuite une transaction non signée, spécifiant l'adresse du vendeur comme destination. Cette transaction est enregistrée dans un fichier local (ex: multisig_monero_tx).
Ce fichier multisig_monero_tx est ensuite transmis par l'initiateur (le Vendeur) au second signataire (l'Acheteur) via un canal de communication chiffré.
3. Finalisation et Diffusion
Le deuxième signataire reçoit le fichier de transaction. Il peut le vérifier pour s'assurer que la destination et le montant sont corrects.
Il applique sa signature en utilisant la commande : sign_multisig multisig_monero_tx. La transaction est maintenant signée par deux des trois parties, atteignant le seuil requis de 2/3.
Le fichier de transaction, désormais complet et valide, peut être diffusé sur le réseau Monero par n'importe lequel des trois participants (acheteur, vendeur ou arbitre) à l'aide de la commande : submit_multisig multisig_monero_tx.
Les fonds sont alors transférés de l'adresse multisig au portefeuille du vendeur, concluant la transaction avec succès. Cependant, ce flux idéal n'est pas toujours garanti. Il est donc essentiel de prévoir un mécanisme robuste lorsque l'accord n'est pas trouvé, nécessitant l'intervention de l'arbitre.
5. Phase 4 : Gestion des Litiges et Rôle de l'Arbitre
La robustesse et la fiabilité d'une marketplace se mesurent à sa capacité à gérer efficacement les désaccords. Le mécanisme d'arbitrage est la soupape de sécurité qui garantit que les fonds ne sont jamais bloqués indéfiniment en cas de litige. L'arbitre, qui est la plateforme, agit comme un tiers de confiance impartial, utilisant sa clé de signature pour forcer la résolution de la transaction et préserver ainsi la confiance des utilisateurs dans le système.
Le processus de gestion des litiges se déroule comme suit :
Déclenchement du Litige Lorsqu'un problème survient (par exemple, un produit non reçu ou non conforme), l'acheteur ou le vendeur peut initier un litige via une interface dédiée sur la marketplace. Cette action signale formellement le désaccord et suspend la transaction standard, empêchant toute signature bilatérale.
Processus de Résolution L'arbitre intervient pour médier. Il recueille et examine les preuves fournies par les deux parties. Ces preuves (messages, photos du produit, confirmations de suivi, etc.) sont échangées et analysées hors chaîne. La confidentialité de Monero s'appliquant uniquement aux transactions, la gestion du litige lui-même repose sur les systèmes de communication de la plateforme. Sur la base des éléments fournis, l'arbitre prend une décision en faveur de l'une des deux parties :
Rembourser l'Acheteur si le vendeur est en faute.
Payer le Vendeur si l'acheteur est en faute ou si la réclamation est infondée.
Exécution de la Signature d'Arbitrage C'est ici que la troisième clé du schéma multisig 2/3 prend tout son sens. L'arbitre, détenteur de la Clé 3, collabore avec la partie en faveur de laquelle il a statué pour construire et co-signer la transaction finale.
En cas de remboursement : L'Arbitre (Clé 3) et l'Acheteur (Clé 1) collaborent pour créer et signer une transaction qui renvoie les fonds de l'adresse multisig vers le portefeuille de l'Acheteur.
En cas de paiement : L'Arbitre (Clé 3) et le Vendeur (Clé 2) collaborent pour créer et signer une transaction qui envoie les fonds de l'adresse multisig vers le portefeuille du Vendeur.
Dans les deux scénarios, le seuil de 2/3 signatures est atteint, ce qui permet de débloquer les fonds de manière décisive. Ce mécanisme garantit que même si l'une des parties refuse de coopérer, la transaction peut être finalisée, protégeant ainsi l'intégrité de la marketplace.
Il est crucial de comprendre que la réputation de l'arbitre (la plateforme) est le seul garant de l'impartialité de la résolution. Techniquement, l'arbitre pourrait s'entendre avec l'une des parties pour voler les fonds. La confiance n'est donc pas éliminée, mais déplacée de la garde des fonds à l'intégrité du processus d'arbitrage.
6. Phase 5 : Considérations de Sécurité Critiques
Dans un modèle non-custodial, la responsabilité de la sécurité est partagée entre la plateforme et ses utilisateurs. L'architecture doit être conçue pour minimiser la confiance que les utilisateurs doivent accorder à la plateforme elle-même, en particulier en ce qui concerne la gestion de leurs fonds. Une implémentation rigoureuse des principes de sécurité est non négociable pour garantir la viabilité et la réputation du projet.
Voici les points de sécurité les plus importants à considérer :
Stockage des Clés Privées Ceci est la règle non négociable d'une architecture non-custodial : les clés privées de dépense de l'Acheteur et du Vendeur ne doivent jamais, sous aucun prétexte, transiter par les serveurs de la plateforme, ni y être stockées sous quelque forme que ce soit, même chiffrée. Toutes les opérations cryptographiques sensibles, comme la génération des clés et la signature des transactions, doivent être exécutées exclusivement côté client, que ce soit dans le navigateur web via des bibliothèques JavaScript spécialisées ou, de manière plus sécurisée, dans une application de bureau dédiée.
Communication Sécurisée Le processus de création et de signature d'un portefeuille multisig nécessite l'échange d'informations sensibles (chaînes d'initialisation, transactions partiellement signées). Ces canaux de communication doivent être protégés contre les attaques de type "man-in-the-middle". Il est fortement recommandé d'utiliser des protocoles de communication pair-à-pair chiffrés de bout en bout et compatibles avec le réseau Tor pour préserver l'anonymat des participants. Des projets comme Haveno ou UnstoppableSwap, qui utilisent des protocoles comme libp2p sur Tor, offrent d'excellents modèles à suivre.
Interaction avec le Démon Monero Le "Service d'Interaction Monero" de la plateforme est un composant critique. Son accès au démon Monero (monero-daemon) ou au portefeuille RPC (monero-wallet-rpc) doit être rigoureusement contrôlé. Les bonnes pratiques incluent :
Isoler ce service dans un environnement réseau distinct.
Limiter les accès RPC à une liste blanche d'adresses IP.
Valider et assainir méticuleusement toutes les données provenant du réseau Monero pour prévenir toute forme de corruption ou d'attaque.
Conclusion
La construction d'une marketplace non-custodial basée sur le multisig Monero est un défi technique complexe mais tout à fait réalisable. Une telle plateforme offre un niveau de confidentialité, de sécurité et d'autonomie pour les utilisateurs qui reste inégalé par les solutions traditionnelles et la plupart des autres cryptomonnaies. Le succès d'un tel projet repose sur une adhésion sans compromis aux principes de la gestion de clés côté client et de la communication sécurisée. En acceptant la complexité inhérente à ce modèle, on bâtit une marketplace qui ne se contente pas de faciliter des échanges, mais qui offre à ses utilisateurs une véritable souveraineté numérique.

