# Script: auto-reality-check-tor.ps1
# Genere automatiquement un reality check Tor avec tests automatiques
# Usage: .\scripts\auto-reality-check-tor.ps1 <function_name>

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
$realityCheckFile = "$realityChecksDir\tor-$FunctionName-$date.md"

Write-Host "Generation Reality Check TOR pour: $FunctionName" -ForegroundColor Cyan
Write-Host "===============================================" -ForegroundColor Cyan

# ============================================
# TESTS AUTOMATIQUES TOR
# ============================================

Write-Host "`nExecution tests automatiques..." -ForegroundColor Yellow

# 1. Test Tor Daemon
Write-Host "1. Test Tor Daemon..." -ForegroundColor White
$torProcess = Get-Process -Name "tor" -ErrorAction SilentlyContinue
$torDaemonRunning = $torProcess -ne $null
if ($torDaemonRunning) {
    Write-Host "   Tor Daemon: Running" -ForegroundColor Green
} else {
    Write-Host "   Tor Daemon: NOT Running" -ForegroundColor Red
}

# 2. Test IP Leak
Write-Host "2. Test IP Leak..." -ForegroundColor White
$ipLeakTest = $false
$torIP = "N/A"
try {
    $torResponse = Invoke-RestMethod `
        -Uri "https://check.torproject.org/api/ip" `
        -Proxy "socks5://127.0.0.1:9050" `
        -TimeoutSec 10
    $torIP = $torResponse.IP
    $ipLeakTest = $torResponse.IsTor -eq $true
    if ($ipLeakTest) {
        Write-Host "   IP Leak Test: Using Tor ($torIP)" -ForegroundColor Green
    } else {
        Write-Host "   IP Leak Test: NOT using Tor ($torIP)" -ForegroundColor Red
    }
} catch {
    Write-Host "   IP Leak Test: FAILED - Tor not accessible" -ForegroundColor Red
}

# 3. Test Monero RPC
Write-Host "3. Test Monero RPC..." -ForegroundColor White
$moneroRPCAccessible = $false
try {
    $rpcResponse = Invoke-RestMethod `
        -Uri "http://127.0.0.1:18082/json_rpc" `
        -Method Post `
        -ContentType "application/json" `
        -Body '{"jsonrpc":"2.0","id":"0","method":"get_version"}' `
        -TimeoutSec 5
    $moneroRPCAccessible = $true
    Write-Host "   Monero RPC: Accessible on localhost" -ForegroundColor Green
} catch {
    Write-Host "   Monero RPC: NOT accessible" -ForegroundColor Red
}

# 4. Test Port Exposure
Write-Host "4. Test Port Exposure..." -ForegroundColor White
$rpcExposed = $false
try {
    $netstatOutput = netstat -an 2>&1
    if ($netstatOutput -match "0\.0\.0\.0:18082") {
        $rpcExposed = $true
        Write-Host "   Port Exposure: RPC exposed publicly (DANGER!)" -ForegroundColor Red
    } elseif ($netstatOutput -match "127\.0\.0\.1:18082") {
        Write-Host "   Port Exposure: RPC isolated on localhost" -ForegroundColor Green
    } else {
        Write-Host "   Port Exposure: RPC not running" -ForegroundColor Yellow
    }
} catch {
    Write-Host "   Port Exposure: Test failed" -ForegroundColor Red
}

# 5. Test Logs Audit
Write-Host "5. Test Logs Audit..." -ForegroundColor White
$logsClean = $true
$sensitivePatterns = @("\.onion", "view_key", "spend_key", "password", "secret")
$logFiles = Get-ChildItem -Path "logs" -Include "*.log" -ErrorAction SilentlyContinue

if ($logFiles) {
    foreach ($pattern in $sensitivePatterns) {
        $matches = Select-String -Path $logFiles.FullName -Pattern $pattern -ErrorAction SilentlyContinue
        if ($matches) {
            $logsClean = $false
            Write-Host "   Logs Audit: Sensitive data found ($pattern)" -ForegroundColor Red
            break
        }
    }
    if ($logsClean) {
        Write-Host "   Logs Audit: No sensitive data in logs" -ForegroundColor Green
    }
} else {
    Write-Host "   Logs Audit: No log files found" -ForegroundColor Yellow
}

# 6. Test Tor Version
Write-Host "6. Test Tor Version..." -ForegroundColor White
$torVersion = "Unknown"
try {
    $torVersionOutput = tor --version 2>&1
    if ($torVersionOutput -match "Tor version (\d+\.\d+\.\d+\.\d+)") {
        $torVersion = $matches[1]
        Write-Host "   Tor Version: $torVersion" -ForegroundColor Green
    } else {
        Write-Host "   Tor Version: Unknown" -ForegroundColor Yellow
    }
} catch {
    Write-Host "   Tor Version: Not accessible" -ForegroundColor Red
}

# Calculer les issues critiques
$criticalIssues = 0
if (-not $torDaemonRunning) { $criticalIssues++ }
if (-not $ipLeakTest) { $criticalIssues++ }
if ($rpcExposed) { $criticalIssues++ }
if (-not $logsClean) { $criticalIssues++ }

$autoTestsPassed = $criticalIssues -eq 0

# ============================================
# GENERATION DU REALITY CHECK
# ============================================

Write-Host "`nGeneration du Reality Check Tor..." -ForegroundColor Yellow

# Metadata JSON
$metadata = @{
    date = $date
    timestamp = $timestamp
    function_name = $FunctionName
    tor_daemon = if ($torDaemonRunning) { "Running" } else { "NOT Running" }
    ip_leak_test = if ($ipLeakTest) { "Using Tor ($torIP)" } else { "NOT using Tor" }
    monero_rpc = if ($moneroRPCAccessible) { "Accessible on localhost" } else { "NOT accessible" }
    port_exposure = if ($rpcExposed) { "RPC exposed publicly" } else { "RPC isolated on localhost" }
    logs_audit = if ($logsClean) { "No sensitive data in logs" } else { "Sensitive data found" }
    tor_version = $torVersion
    auto_tests_passed = $autoTestsPassed
    critical_issues = $criticalIssues
} | ConvertTo-Json -Depth 3

# Template Reality Check Tor
$template = @"
# Reality Check Tor: $FunctionName
**Date:** $date  
**Heure:** $timestamp  
**Fonction:** $FunctionName
**Threat Level:** HIGH (Network Code)

---

## 🧅 Tests Automatiques
``````json
$metadata
``````

## 📋 Résultats Tests Automatiques:
- **Tor Daemon:** $(if ($torDaemonRunning) { "Running" } else { "NOT Running" })
- **IP Leak Test:** $(if ($ipLeakTest) { "Using Tor ($torIP)" } else { "NOT using Tor" })
- **Monero RPC:** $(if ($moneroRPCAccessible) { "Accessible on localhost" } else { "NOT accessible" })
- **Port Exposure:** $(if ($rpcExposed) { "RPC exposed publicly (DANGER!)" } else { "RPC isolated on localhost" })
- **Logs Audit:** $(if ($logsClean) { "No sensitive data in logs" } else { "Sensitive data found" })
- **Tor Version:** $torVersion

**Issues Critiques:** $criticalIssues
**Tests Auto Passés:** $(if ($autoTestsPassed) { "OUI" } else { "NON" })

---

## ✅ Tests Manuels OPSEC

### Tests de Fuite
- [ ] **DNS Leak Test**
  - [ ] DNS via Tor uniquement
  - [ ] Pas de requetes DNS directes
  - [ ] Resolution .onion fonctionnelle

- [ ] **Fingerprinting Test**
  - [ ] Fingerprint anonyme
  - [ ] User-Agent generique
  - [ ] Pas de metadata unique

- [ ] **Hidden Service Test** (si applicable)
  - [ ] Acces .onion fonctionnel
  - [ ] Pas de fallback clearnet
  - [ ] Certificat valide

- [ ] **Traffic Analysis Test**
  - [ ] Pas de patterns temporels
  - [ ] Taille de paquets variable
  - [ ] Pas de correlation evidente

### Tests de Securite
- [ ] **RPC Isolation**
  - [ ] RPC NOT exposed publicly
  - [ ] Bind uniquement sur 127.0.0.1
  - [ ] Pas d'acces depuis l'exterieur

- [ ] **Logs Security**
  - [ ] Pas de .onion dans logs
  - [ ] Pas de credentials dans logs
  - [ ] Logs niveau approprie

- [ ] **Network Security**
  - [ ] Toutes requetes via Tor
  - [ ] Pas de connexions directes
  - [ ] Timeouts appropries

---

## Decision Finale

### Status des Tests
- [ ] **APPROUVE** - Pret pour production Tor
- [ ] **CONDITIONNEL** - Ameliorations requises
- [ ] **REJETE** - Recommencer

### Justification
[Expliquer la decision basee sur les tests]

### Actions Requises (si conditionnel/rejete)
- [ ] [Action 1]
- [ ] [Action 2]
- [ ] [Action 3]

---

## Notes OPSEC

### Observations
[Notes sur le comportement Tor, anomalies, etc.]

### Recommandations
[Suggestions d'amelioration OPSEC]

### Limitations Identifiees
[Limitations de securite connues]

---

## Checklist Finale

- [ ] Tous les tests auto passent
- [ ] Tests manuels completes
- [ ] Aucune fuite detectee
- [ ] RPC correctement isole
- [ ] Logs propres
- [ ] Decision prise et justifiee

---

## Validation

**Teste par:** [Nom] **[Signature]**  
**Date de validation:** $date  
**Status:** [ ] Valide pour production

**Commentaires finaux:**
[Commentaires sur la validation Tor]
"@

# Ecrire le reality check
$template | Out-File -FilePath $realityCheckFile -Encoding UTF8

Write-Host "`nReality Check TOR genere: $realityCheckFile" -ForegroundColor Green
Write-Host "Tests automatiques passes - complete les tests manuels" -ForegroundColor Cyan

# Afficher le resume
Write-Host "`nRESUME TESTS AUTOMATIQUES:" -ForegroundColor Cyan
Write-Host "=========================" -ForegroundColor Cyan
Write-Host "Tor Daemon: $(if ($torDaemonRunning) { '✓' } else { '✗' })" -ForegroundColor $(if ($torDaemonRunning) { "Green" } else { "Red" })
Write-Host "IP Leak Test: $(if ($ipLeakTest) { '✓' } else { '✗' })" -ForegroundColor $(if ($ipLeakTest) { "Green" } else { "Red" })
Write-Host "Monero RPC: $(if ($moneroRPCAccessible) { '✓' } else { '✗' })" -ForegroundColor $(if ($moneroRPCAccessible) { "Green" } else { "Red" })
Write-Host "Port Exposure: $(if ($rpcExposed) { '✗' } else { '✓' })" -ForegroundColor $(if ($rpcExposed) { "Red" } else { "Green" })
Write-Host "Logs Audit: $(if ($logsClean) { '✓' } else { '✗' })" -ForegroundColor $(if ($logsClean) { "Green" } else { "Red" })
Write-Host "Tor Version: $torVersion" -ForegroundColor White

Write-Host "`nIssues Critiques: $criticalIssues" -ForegroundColor $(if ($criticalIssues -eq 0) { "Green" } else { "Red" })
Write-Host "Tests Auto Passés: $(if ($autoTestsPassed) { '✅ OUI' } else { '❌ NON' })" -ForegroundColor $(if ($autoTestsPassed) { "Green" } else { "Red" })

if (-not $autoTestsPassed) {
    Write-Host "`n⚠️ ATTENTION: Issues critiques détectées!" -ForegroundColor Red
    Write-Host "Corrigez les problèmes avant de continuer." -ForegroundColor Yellow
    exit 1
} else {
    Write-Host "`n✅ Tests automatiques passés - complète les tests manuels" -ForegroundColor Green
    exit 0
}
