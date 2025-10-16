# Script: new-spec.ps1
# Cree une nouvelle spec depuis le template
# Usage: .\scripts\new-spec.ps1 <function_name>

param(
    [Parameter(Mandatory=$true)]
    [string]$FunctionName
)

# Verifier que nous sommes dans le bon repertoire
if (-not (Test-Path ".cursorrules")) {
    Write-Host "ERREUR: Execute ce script depuis la racine du projet" -ForegroundColor Red
    exit 1
}

# Creer le repertoire specs s'il n'existe pas
$specsDir = "docs\specs"
if (-not (Test-Path $specsDir)) {
    New-Item -ItemType Directory -Path $specsDir -Force | Out-Null
}

# Nom du fichier spec
$specFile = "$specsDir\$FunctionName.md"

# Verifier si la spec existe deja
if (Test-Path $specFile) {
    Write-Host "ATTENTION: La spec $specFile existe deja" -ForegroundColor Yellow
    $overwrite = Read-Host "Voulez-vous la remplacer? (y/N)"
    if ($overwrite -ne "y" -and $overwrite -ne "Y") {
        Write-Host "Annule." -ForegroundColor Yellow
        exit 0
    }
}

# Template de spec
$template = @"
## Spec: $FunctionName

### Objectif
[Decrire en 1 ligne ce que fait cette fonction]

### Preconditions
- [ ] monero-wallet-rpc tourne sur localhost:18082
- [ ] Wallet ouvert et deverrouille
- [ ] [Autres preconditions specifiques]

### Input
``````rust
// Types exacts des parametres
param1: Type1,
param2: Type2,
``````

### Output
``````rust
Result<ReturnType, ErrorType>
``````

### Erreurs Possibles
- ErrorType::Variant1 - [Quand ca arrive]
- ErrorType::Variant2 - [Quand ca arrive]

### Dependances
``````toml
[dependencies]
dep1 = "version"
``````

### Test de Validation (PowerShell)
``````powershell
# Setup
.\scripts\start-testnet.ps1

# Test manuel
Invoke-RestMethod -Uri "http://127.0.0.1:18082/json_rpc" 
  -Method Post -ContentType "application/json" 
  -Body '{"jsonrpc":"2.0","id":"0","method":"{rpc_method}"}'

# Expected output:
# result : @{...}
``````

### Estimation
- Code: XX min
- Test: XX min
- Total: XX min

### Status
- [ ] Spec validee
- [ ] Code ecrit
- [ ] Tests passent
- [ ] Reality check fait
"@

# Ecrire la spec
$template | Out-File -FilePath $specFile -Encoding UTF8

Write-Host "Spec creee: $specFile" -ForegroundColor Green
Write-Host "Editez-la maintenant avec Cursor" -ForegroundColor Cyan
Write-Host "Puis demandez a Cursor de generer le code" -ForegroundColor Cyan