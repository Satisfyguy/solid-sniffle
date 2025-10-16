# Script: setup-monero.ps1
# Setup Monero pour le developpement
# Usage: .\scripts\setup-monero.ps1

Write-Host "SETUP MONERO" -ForegroundColor Cyan
Write-Host "=============" -ForegroundColor Cyan

# Verifier que nous sommes dans le bon repertoire
if (-not (Test-Path ".cursorrules")) {
    Write-Host "ERREUR: Execute ce script depuis la racine du projet" -ForegroundColor Red
    exit 1
}

# Verifier si Monero est deja installe
$moneroPath = "C:\monero"
if (Test-Path $moneroPath) {
    Write-Host "Monero semble deja installe dans $moneroPath" -ForegroundColor Green
    $continue = Read-Host "Voulez-vous continuer quand meme? (y/N)"
    if ($continue -ne "y" -and $continue -ne "Y") {
        Write-Host "Annule." -ForegroundColor Yellow
        exit 0
    }
}

Write-Host "`n1. Telechargement de Monero..." -ForegroundColor Yellow

# URL de telechargement Monero (version testnet)
$moneroUrl = "https://downloads.getmonero.org/cli/windows64"
$moneroZip = "monero-windows64.zip"

try {
    Write-Host "Telechargement depuis $moneroUrl..." -ForegroundColor White
    Invoke-WebRequest -Uri $moneroUrl -OutFile $moneroZip -UseBasicParsing
    Write-Host "Telechargement termine" -ForegroundColor Green
} catch {
    Write-Host "Erreur lors du telechargement: $_" -ForegroundColor Red
    Write-Host "Telechargez manuellement Monero depuis https://www.getmonero.org/downloads/" -ForegroundColor Yellow
    exit 1
}

Write-Host "`n2. Extraction de Monero..." -ForegroundColor Yellow

try {
    # Creer le repertoire Monero
    if (-not (Test-Path $moneroPath)) {
        New-Item -ItemType Directory -Path $moneroPath -Force | Out-Null
    }
    
    # Extraire l'archive
    Expand-Archive -Path $moneroZip -DestinationPath $moneroPath -Force
    Write-Host "Extraction terminee dans $moneroPath" -ForegroundColor Green
    
    # Nettoyer l'archive
    Remove-Item $moneroZip -Force
    Write-Host "Archive nettoyee" -ForegroundColor Green
} catch {
    Write-Host "Erreur lors de l'extraction: $_" -ForegroundColor Red
    exit 1
}

Write-Host "`n3. Configuration des variables d'environnement..." -ForegroundColor Yellow

# Ajouter Monero au PATH
$currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($currentPath -notlike "*$moneroPath*") {
    $newPath = "$currentPath;$moneroPath"
    [Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
    Write-Host "Monero ajoute au PATH utilisateur" -ForegroundColor Green
} else {
    Write-Host "Monero deja dans le PATH" -ForegroundColor Green
}

Write-Host "`n4. Verification de l'installation..." -ForegroundColor Yellow

# Tester monerod
try {
    $version = & "$moneroPath\monerod.exe" --version 2>&1
    Write-Host "Monerod version: $($version[0])" -ForegroundColor Green
} catch {
    Write-Host "Erreur lors du test de monerod: $_" -ForegroundColor Red
}

# Tester monero-wallet-cli
try {
    $version = & "$moneroPath\monero-wallet-cli.exe" --version 2>&1
    Write-Host "Monero-wallet-cli version: $($version[0])" -ForegroundColor Green
} catch {
    Write-Host "Erreur lors du test de monero-wallet-cli: $_" -ForegroundColor Red
}

Write-Host "`n5. Creation des repertoires de travail..." -ForegroundColor Yellow

# Creer les repertoires pour testnet
$testnetDir = ".\testnet"
$daemonDir = "$testnetDir\daemon"
$walletDir = "$testnetDir\wallet"

New-Item -ItemType Directory -Path $daemonDir -Force | Out-Null
New-Item -ItemType Directory -Path $walletDir -Force | Out-Null

Write-Host "Repertoires crees:" -ForegroundColor Green
Write-Host "  Daemon: $daemonDir" -ForegroundColor White
Write-Host "  Wallet: $walletDir" -ForegroundColor White

Write-Host "`n6. Configuration des fichiers de config..." -ForegroundColor Yellow

# Fichier de config daemon
$daemonConfig = @"
# Monero daemon config for testnet
testnet=1
rpc-bind-ip=127.0.0.1
rpc-bind-port=18081
p2p-bind-ip=127.0.0.1
p2p-bind-port=18080
data-dir=$daemonDir
log-level=1
"@

$daemonConfig | Out-File -FilePath "$daemonDir\monerod.conf" -Encoding UTF8

# Fichier de config wallet RPC
$walletConfig = @"
# Monero wallet RPC config for testnet
testnet=1
rpc-bind-ip=127.0.0.1
rpc-bind-port=18082
wallet-dir=$walletDir
daemon-address=127.0.0.1:18081
"@

$walletConfig | Out-File -FilePath "$walletDir\monero-wallet-rpc.conf" -Encoding UTF8

Write-Host "Fichiers de config crees" -ForegroundColor Green

Write-Host "`nSETUP TERMINE!" -ForegroundColor Green
Write-Host "===============" -ForegroundColor Green

Write-Host "`nProchaines etapes:" -ForegroundColor Cyan
Write-Host "1. Redemarrer votre terminal pour que le PATH soit mis a jour" -ForegroundColor White
Write-Host "2. Lancer: .\scripts\start-testnet.ps1" -ForegroundColor White
Write-Host "3. Tester: .\scripts\test-rpc.ps1" -ForegroundColor White

Write-Host "`nFichiers crees:" -ForegroundColor Cyan
Write-Host "- $moneroPath\monerod.exe" -ForegroundColor White
Write-Host "- $moneroPath\monero-wallet-cli.exe" -ForegroundColor White
Write-Host "- $daemonDir\monerod.conf" -ForegroundColor White
Write-Host "- $walletDir\monero-wallet-rpc.conf" -ForegroundColor White
