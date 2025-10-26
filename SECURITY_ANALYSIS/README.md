# Security Analysis Toolkit

Ce dépôt contient des outils d'analyse de sécurité pour votre projet Rust, utilisant DeepSeek-V3 pour une analyse approfondie du code.

## Prérequis

- Python 3.8+
- Clé API DeepSeek (obtenez-la sur [DeepSeek Platform](https://platform.deepseek.com/))

## Installation

1. Créez un environnement virtuel :
   ```bash
   python -m venv venv
   source venv/bin/activate  # Sur Linux/Mac
   # OU
   .\venv\Scripts\activate  # Sur Windows
   ```

2. Installez les dépendances :
   ```bash
   pip install -r requirements.txt
   ```

3. Configurez votre clé API :
   ```bash
   echo "DEEPSEEK_API_KEY=votre_cle_api" > .env
   ```

## Utilisation

### Analyse Complète

Pour une analyse détaillée d'un fichier avec DeepSeek-V3 :
```bash
python deepseek_analyzer.py
```

### Scan Rapide

Pour un scan rapide des vulnérabilités courantes :
```bash
python quick_scan.py
```

## Fonctionnalités

- **Analyse approfondie** avec DeepSeek-V3 (128K tokens de contexte)
- **Détection de vulnérabilités** courantes
- **Rapports détaillés** avec niveaux de sévérité
- **Recommandations** pour corriger les problèmes

## Structure du Projet

- `deepseek_analyzer.py` - Analyseur principal utilisant DeepSeek-V3
- `quick_scan.py` - Scanner rapide basé sur des motifs
- `config.py` - Configuration de l'analyseur
- `requirements.txt` - Dépendances Python

## Personnalisation

Vous pouvez modifier les modèles et paramètres dans `config.py` :
- Modèle DeepSeek utilisé
- Niveau de détail des rapports
- Délais d'attente
- Répertoires à analyser

## Licence

MIT
