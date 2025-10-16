# Script: start-testnet.ps1
# Lance Monero testnet (daemon + wallet RPC)
# Usage: .\scripts\start-testnet.ps1

Write-Host "DEMARRAGE MONERO TESTNET" -ForegroundColor Cyan
Write-Host "=========================" -ForegroundColor Cyan

# Verifier que nous sommes dans le bon repertoire
if (-not (Test-Path ".cursorrules")) {
    Write-Host "ERREUR: Execute ce script depuis la racine du projet" -ForegroundColor Red
    exit 1
}

# Verifier que Monero est installe
$moneroPath = "C:\monero"
if (-not (Test-Path "$moneroPath\monerod.exe")) {
    Write-Host "ERREUR: Monero non installe" -ForegroundColor Red
    Write-Host "Lancez d'abord: .\scripts\setup-monero.ps1" -ForegroundColor Yellow
    exit 1
}

# Repertoires de travail
$testnetDir = ".\testnet"
$daemonDir = "$testnetDir\daemon"
$walletDir = "$testnetDir\wallet"

# Creer les repertoires s'ils n'existent pas
if (-not (Test-Path $daemonDir)) {
    New-Item -ItemType Directory -Path $daemonDir -Force | Out-Null
}
if (-not (Test-Path $walletDir)) {
    New-Item -ItemType Directory -Path $walletDir -Force | Out-Null
}

# Verifier si Monero tourne deja
$monerodProcess = Get-Process -Name "monerod" -ErrorAction SilentlyContinue
$walletRpcProcess = Get-Process -Name "monero-wallet-rpc" -ErrorAction SilentlyContinue

if ($monerodProcess -or $walletRpcProcess) {
    Write-Host "Monero semble deja tourner" -ForegroundColor Yellow
    $restart = Read-Host "Voulez-vous redemarrer? (y/N)"
    if ($restart -eq "y" -or $restart -eq "Y") {
        Write-Host "Arret des processus existants..." -ForegroundColor Yellow
        if ($monerodProcess) { $monerodProcess | Stop-Process -Force }
        if ($walletRpcProcess) { $walletRpcProcess | Stop-Process -Force }
        Start-Sleep -Seconds 2
    } else {
        Write-Host "Utilisation des processus existants" -ForegroundColor Green
        exit 0
    }
}

Write-Host "`n1. Demarrage du daemon Monero..." -ForegroundColor Yellow

try {
    # Lancer monerod en arriere-plan
    $daemonArgs = @(
        "--testnet",
        "--rpc-bind-ip=127.0.0.1",
        "--rpc-bind-port=18081",
        "--p2p-bind-ip=127.0.0.1",
        "--p2p-bind-port=18080",
        "--data-dir=$daemonDir",
        "--log-level=1",
        "--non-interactive"
    )
    
    Start-Process -FilePath "$moneroPath\monerod.exe" -ArgumentList $daemonArgs -WindowStyle Hidden
    Write-Host "Daemon Monero lance" -ForegroundColor Green
    
    # Attendre que le daemon soit pret
    Write-Host "Attente du daemon (30 secondes)..." -ForegroundColor White
    Start-Sleep -Seconds 30
    
} catch {
    Write-Host "Erreur lors du demarrage du daemon: $_" -ForegroundColor Red
    exit 1
}

Write-Host "`n2. Verification du daemon..." -ForegroundColor Yellow

# Tester la connexion au daemon
try {
    $response = Invoke-RestMethod -Uri "http://127.0.0.1:18081/json_rpc" -Method Post -ContentType "application/json" -Body '{"jsonrpc":"2.0","id":"0","method":"get_version"}' -TimeoutSec 10
    Write-Host "Daemon accessible (version: $($response.result.version))" -ForegroundColor Green
} catch {
    Write-Host "Daemon non accessible, continuons quand meme..." -ForegroundColor Yellow
}

Write-Host "`n3. Demarrage du wallet RPC..." -ForegroundColor Yellow

try {
    # Lancer monero-wallet-rpc en arriere-plan
    $walletArgs = @(
        "--testnet",
        "--rpc-bind-ip=127.0.0.1",
        "--rpc-bind-port=18082",
        "--wallet-dir=$walletDir",
        "--daemon-address=127.0.0.1:18081",
        "--non-interactive"
    )
    
    Start-Process -FilePath "$moneroPath\monero-wallet-rpc.exe" -ArgumentList $walletArgs -WindowStyle Hidden
    Write-Host "Wallet RPC lance" -ForegroundColor Green
    
    # Attendre que le wallet RPC soit pret
    Write-Host "Attente du wallet RPC (10 secondes)..." -ForegroundColor White
    Start-Sleep -Seconds 10
    
} catch {
    Write-Host "Erreur lors du demarrage du wallet RPC: $_" -ForegroundColor Red
    Write-Host "Le daemon tourne, mais le wallet RPC a echoue" -ForegroundColor Yellow
}

Write-Host "`n4. Verification du wallet RPC..." -ForegroundColor Yellow

# Tester la connexion au wallet RPC
try {
    $response = Invoke-RestMethod -Uri "http://127.0.0.1:18082/json_rpc" -Method Post -ContentType "application/json" -Body '{"jsonrpc":"2.0","id":"0","method":"get_version"}' -TimeoutSec 10
    Write-Host "Wallet RPC accessible (version: $($response.result.version))" -ForegroundColor Green
} catch {
    Write-Host "Wallet RPC non accessible" -ForegroundColor Red
    Write-Host "Vous devrez peut-etre creer un wallet d'abord" -ForegroundColor Yellow
}

Write-Host "`nTESTNET DEMARRE!" -ForegroundColor Green
Write-Host "=================" -ForegroundColor Green

Write-Host "`nServices actifs:" -ForegroundColor Cyan
Write-Host "- Daemon: http://127.0.0.1:18081" -ForegroundColor White
Write-Host "- Wallet RPC: http://127.0.0.1:18082" -ForegroundColor White

Write-Host "`nProchaines etapes:" -ForegroundColor Cyan
Write-Host "1. Tester: .\scripts\test-rpc.ps1" -ForegroundColor White
Write-Host "2. Creer un wallet si necessaire" -ForegroundColor White
Write-Host "3. Developper avec Cursor" -ForegroundColor White

Write-Host "`nPour arreter:" -ForegroundColor Cyan
Write-Host "Get-Process monero* | Stop-Process -Force" -ForegroundColor White
