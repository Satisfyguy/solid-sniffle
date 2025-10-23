# Installation Rust - Monero Marketplace

## ✅ Installation Réussie

Rust a été installé avec succès sur la machine Ubuntu et le projet Monero Marketplace compile correctement.

## 🔧 Configuration OpenSSL

Le projet nécessite OpenSSL pour la compilation. Les variables d'environnement suivantes ont été configurées :

```bash
export OPENSSL_DIR=/usr
export OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu
export OPENSSL_INCLUDE_DIR=/usr/include/openssl
```

## 🚀 Script de Compilation

Un script `build.sh` a été créé pour faciliter la compilation :

```bash
./build.sh          # Compilation debug
./build.sh --release # Compilation release
```

## 📦 Binaires Compilés

Les binaires suivants ont été générés dans `target/release/` :

- `monero-marketplace` - CLI principal
- `server` - Serveur web
- `init_db` - Outil d'initialisation de base de données
- `test-tool` - Outil de test

## 🛠️ Versions Installées

- **Rust**: 1.90.0 (1159e78c4 2025-09-14)
- **Cargo**: 1.90.0 (840b83a10 2025-07-30)
- **OpenSSL**: 3.0.13-0ubuntu3.6 (déjà installé)

## 📝 Notes Importantes

1. **Variables d'environnement** : Les variables OpenSSL doivent être définies à chaque session ou ajoutées au `.bashrc`
2. **Compilation** : Utiliser le script `build.sh` pour éviter les problèmes d'OpenSSL
3. **Dépendances** : Toutes les dépendances système sont installées (OpenSSL, libssl-dev)

## 🎯 Prochaines Étapes

Le projet est maintenant prêt pour :
- Développement
- Tests
- Déploiement
- Utilisation des outils CLI

---
*Installation terminée le 23 octobre 2025*
