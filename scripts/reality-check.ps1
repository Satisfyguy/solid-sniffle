# Script: reality-check.ps1
# Genere un reality check pour une fonction
# Usage: .\scripts\reality-check.ps1 <function_name>

param(
    [Parameter(Mandatory=$true)]
    [string]$FunctionName
)

# Verifier que nous sommes dans le bon repertoire
if (-not (Test-Path ".cursorrules")) {
    Write-Host "ERREUR: Execute ce script depuis la racine du projet" -ForegroundColor Red
    exit 1
}

# Creer le repertoire reality-checks s'il n'existe pas
$realityChecksDir = "docs\reality-checks"
if (-not (Test-Path $realityChecksDir)) {
    New-Item -ItemType Directory -Path $realityChecksDir -Force | Out-Null
}

# Date actuelle
$date = Get-Date -Format "yyyy-MM-dd"
$timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"

# Nom du fichier reality check
$realityCheckFile = "$realityChecksDir\$FunctionName-$date.md"

# Verifier si le reality check existe deja
if (Test-Path $realityCheckFile) {
    Write-Host "ATTENTION: Le reality check $realityCheckFile existe deja" -ForegroundColor Yellow
    $overwrite = Read-Host "Voulez-vous le remplacer? (y/N)"
    if ($overwrite -ne "y" -and $overwrite -ne "Y") {
        Write-Host "Annule." -ForegroundColor Yellow
        exit 0
    }
}

# Verifier que la spec existe
$specFile = "docs\specs\$FunctionName.md"
if (-not (Test-Path $specFile)) {
    Write-Host "ERREUR: La spec $specFile n'existe pas" -ForegroundColor Red
    Write-Host "Creez d'abord la spec avec: .\scripts\new-spec.ps1 $FunctionName" -ForegroundColor Yellow
    exit 1
}

# Template de reality check
$template = @"
# Reality Check: $FunctionName
**Date:** $date  
**Heure:** $timestamp  
**Fonction:** $FunctionName

---

## Checklist de Validation

### Code Review
- [ ] **Spec respectee**: Le code implemente exactement ce qui est dans la spec
- [ ] **Error handling**: Tous les cas d'erreur sont geres avec .context() ou match
- [ ] **Pas d'unwrap**: Aucun .unwrap() ou .expect() sans message
- [ ] **Types corrects**: Les types d'entree/sortie correspondent a la spec
- [ ] **Documentation**: Commentaires clairs sur la logique complexe

### Tests
- [ ] **Tests unitaires**: Au moins un test par cas d'usage principal
- [ ] **Tests d'erreur**: Tests pour les cas d'erreur documentes
- [ ] **Tests d'integration**: Test avec Monero RPC reel (si applicable)
- [ ] **Tous les tests passent**: cargo test sans erreur

### Monero RPC
- [ ] **RPC accessible**: monero-wallet-rpc repond sur localhost:18082
- [ ] **Wallet deverrouille**: Le wallet est ouvert et accessible
- [ ] **Test manuel reussi**: Appel RPC manuel fonctionne
- [ ] **Gestion des timeouts**: Timeout configure (>30s)

### Performance & Securite
- [ ] **Pas de panics**: Aucun panic! dans le code
- [ ] **Pas de logs sensibles**: Aucun log de mots de passe/tokens
- [ ] **Taille des requetes**: Requetes RPC de taille raisonnable
- [ ] **Gestion memoire**: Pas de fuites evidentes

---

## Test Manuel

### Prerequis
``````powershell
# 1. Lancer Monero testnet
.\scripts\start-testnet.ps1

# 2. Verifier que RPC repond
Invoke-RestMethod -Uri "http://127.0.0.1:18082/json_rpc" 
  -Method Post -ContentType "application/json" 
  -Body '{"jsonrpc":"2.0","id":"0","method":"get_version"}'
``````

### Test de la fonction
``````powershell
# [A completer avec les commandes de test specifiques]
# Exemple:
# cargo test test_$FunctionName
# cargo run --bin cli -- $FunctionName --param1 value1
``````

### Resultat attendu
``````
# [A completer avec le resultat attendu]
# Exemple:
# Success: {"result": {...}}
# Error: {"error": {"code": -1, "message": "..."}}
``````

---

## Metriques

### Avant implementation
- Lines of Code: [A completer]
- Functions: [A completer]
- Unwraps: [A completer]

### Apres implementation
- Lines of Code: [A completer]
- Functions: [A completer] 
- Unwraps: [A completer]

---

## Validation Finale

### Criteres de succes
- [ ] **Fonctionne**: La fonction fait ce qu'elle doit faire
- [ ] **Robuste**: Geres tous les cas d'erreur
- [ ] **Testable**: Facile a tester et deboguer
- [ ] **Maintenable**: Code clair et documente
- [ ] **Securise**: Pas de vulnerabilites evidentes

### Points d'attention
- [ ] [A completer avec les points specifiques a surveiller]

---

## Notes & Observations

### Ce qui a bien marche
- [A completer]

### Difficultes rencontrees
- [A completer]

### Ameliorations futures
- [A completer]

---

## Validation

**Valide par:** [Nom]  
**Date de validation:** $date  
**Status:** [ ] VALIDE | [ ] REJETE | [ ] A CORRIGER

**Commentaires finaux:**
[A completer]
"@

# Ecrire le reality check
$template | Out-File -FilePath $realityCheckFile -Encoding UTF8

Write-Host "Reality check cree: $realityCheckFile" -ForegroundColor Green
Write-Host "Completez-le maintenant avec vos tests et observations" -ForegroundColor Cyan
Write-Host "Une fois valide, vous pourrez commiter en toute confiance" -ForegroundColor Cyan