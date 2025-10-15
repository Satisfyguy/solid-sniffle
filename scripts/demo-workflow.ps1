# Script: demo-workflow.ps1
# Demonstration complete du workflow Cursor Rules v2.0
# Usage: .\scripts\demo-workflow.ps1

Write-Host "DEMO WORKFLOW CURSOR RULES v2.0" -ForegroundColor Cyan
Write-Host "=================================" -ForegroundColor Cyan

# Verifier que nous sommes dans le bon repertoire
if (-not (Test-Path ".cursorrules")) {
    Write-Host "ERREUR: Execute ce script depuis la racine du projet" -ForegroundColor Red
    exit 1
}

Write-Host "`nCe script demontre le workflow complet:" -ForegroundColor White
Write-Host "1. Creation d'une spec" -ForegroundColor White
Write-Host "2. Generation de code avec Cursor" -ForegroundColor White
Write-Host "3. Reality check" -ForegroundColor White
Write-Host "4. Pre-commit checks" -ForegroundColor White
Write-Host "5. Metriques" -ForegroundColor White

$continue = Read-Host "`nContinuer? (y/N)"
if ($continue -ne "y" -and $continue -ne "Y") {
    Write-Host "Demo annulee." -ForegroundColor Yellow
    exit 0
}

Write-Host "`n=== ETAPE 1: CREATION D'UNE SPEC ===" -ForegroundColor Green

# Creer une spec pour une fonction demo
$functionName = "get_transaction_info"
Write-Host "Creation de la spec pour: $functionName" -ForegroundColor Yellow

& ".\scripts\new-spec.ps1" $functionName

if (Test-Path "docs\specs\$functionName.md") {
    Write-Host "Spec creee avec succes!" -ForegroundColor Green
} else {
    Write-Host "Erreur lors de la creation de la spec" -ForegroundColor Red
    exit 1
}

Write-Host "`n=== ETAPE 2: EDITION DE LA SPEC ===" -ForegroundColor Green

# Editer la spec automatiquement
$specContent = @"
## Spec: $functionName

### Objectif
Recupere les informations d'une transaction Monero par son hash

### Preconditions
- [ ] monero-wallet-rpc tourne sur localhost:18082
- [ ] Wallet ouvert et deverrouille
- [ ] Transaction existe dans le wallet

### Input
``````rust
// Hash de la transaction
tx_hash: String,
``````

### Output
``````rust
Result<TransactionInfo, Error>
``````

### Erreurs Possibles
- Error::MoneroRpc - Erreur de communication avec le wallet RPC
- Error::InvalidInput - Hash de transaction invalide
- Error::Network - Erreur de reseau ou timeout

### Dependances
``````toml
[dependencies]
reqwest = "0.11"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
``````

### Test de Validation (PowerShell)
``````powershell
# Setup
.\scripts\start-testnet.ps1

# Test manuel
Invoke-RestMethod -Uri "http://127.0.0.1:18082/json_rpc" 
  -Method Post -ContentType "application/json" 
  -Body '{"jsonrpc":"2.0","id":"0","method":"get_transfer_by_txid","params":{"txid":"hash_ici"}}'

# Expected output:
# result : @{...}
``````

### Estimation
- Code: 20 min
- Test: 15 min
- Total: 35 min

### Status
- [x] Spec validee
- [ ] Code ecrit
- [ ] Tests passent
- [ ] Reality check fait
"@

$specContent | Out-File -FilePath "docs\specs\$functionName.md" -Encoding UTF8
Write-Host "Spec editee automatiquement" -ForegroundColor Green

Write-Host "`n=== ETAPE 3: GENERATION DE CODE ===" -ForegroundColor Green

Write-Host "A ce stade, vous demanderiez a Cursor:" -ForegroundColor Yellow
Write-Host "  'Genere le code pour $functionName selon la spec dans docs/specs/$functionName.md'" -ForegroundColor White

Write-Host "`nCursor effectuerait automatiquement:" -ForegroundColor Cyan
Write-Host "  ✓ Verification que la spec existe" -ForegroundColor Green
Write-Host "  ✓ Verification que le projet compile" -ForegroundColor Green
Write-Host "  ✓ Generation du code + tests" -ForegroundColor Green
Write-Host "  ✓ Auto-format du code" -ForegroundColor Green
Write-Host "  ✓ Clippy check" -ForegroundColor Green
Write-Host "  ✓ Mise a jour des metriques" -ForegroundColor Green

Write-Host "`n=== ETAPE 4: REALITY CHECK ===" -ForegroundColor Green

& ".\scripts\reality-check.ps1" $functionName

if (Test-Path "docs\reality-checks\$functionName-$(Get-Date -Format 'yyyy-MM-dd').md") {
    Write-Host "Reality check cree avec succes!" -ForegroundColor Green
} else {
    Write-Host "Erreur lors de la creation du reality check" -ForegroundColor Red
}

Write-Host "`n=== ETAPE 5: PRE-COMMIT CHECKS ===" -ForegroundColor Green

Write-Host "Execution des verifications pre-commit..." -ForegroundColor Yellow
& ".\scripts\pre-commit.ps1"

Write-Host "`n=== ETAPE 6: METRIQUES ===" -ForegroundColor Green

Write-Host "Mise a jour des metriques..." -ForegroundColor Yellow
& ".\scripts\update-metrics.ps1"

Write-Host "`n=== DEMO TERMINEE ===" -ForegroundColor Green
Write-Host "=====================" -ForegroundColor Green

Write-Host "`nFichiers crees:" -ForegroundColor Cyan
Write-Host "- docs\specs\$functionName.md" -ForegroundColor White
Write-Host "- docs\reality-checks\$functionName-$(Get-Date -Format 'yyyy-MM-dd').md" -ForegroundColor White
Write-Host "- docs\metrics\daily-$(Get-Date -Format 'yyyy-MM-dd').json" -ForegroundColor White

Write-Host "`nProchaines etapes:" -ForegroundColor Cyan
Write-Host "1. Editer la spec si necessaire" -ForegroundColor White
Write-Host "2. Demander a Cursor de generer le code" -ForegroundColor White
Write-Host "3. Completer le reality check" -ForegroundColor White
Write-Host "4. Commiter avec: git commit -m '[CODE] Implement $functionName'" -ForegroundColor White

Write-Host "`nLe workflow Cursor Rules v2.0 est maintenant operationnel!" -ForegroundColor Green
