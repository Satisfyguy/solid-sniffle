# scripts/setup-monero-testnet.ps1
# Setup complet Monero testnet pour tests

param(
    [string]$WalletName = "buyer",
    [string]$MoneroPath = "C:\monero-dev"
)

Write-Host "🔧 Setup Monero Testnet" -ForegroundColor Cyan
Write-Host ""

# Trouver binaires Monero
$monerodPath = Get-ChildItem "$MoneroPath\monero-*\monerod.exe" | Select-Object -First 1
$walletCliPath = Get-ChildItem "$MoneroPath\monero-*\monero-wallet-cli.exe" | Select-Object -First 1
$walletRpcPath = Get-ChildItem "$MoneroPath\monero-*\monero-wallet-rpc.exe" | Select-Object -First 1

if (-not $monerodPath) {
    Write-Host "❌ monerod.exe non trouvé dans $MoneroPath" -ForegroundColor Red
    Write-Host "Lance d'abord: .\scripts\setup-monero.ps1" -ForegroundColor Yellow
    exit 1
}

$binDir = Split-Path $monerodPath

Write-Host "Binaires Monero: $binDir" -ForegroundColor Green
Write-Host ""

# 1. Lancer daemon testnet (si pas déjà lancé)
$monerodRunning = Get-Process monerod -ErrorAction SilentlyContinue
if (-not $monerodRunning) {
    Write-Host "1️⃣ Lancement daemon testnet..." -ForegroundColor Yellow
    Start-Process -FilePath $monerodPath.FullName `
        -ArgumentList "--testnet","--detach" `
        -WorkingDirectory $binDir `
        -NoNewWindow
    
    Write-Host "   Attente synchronisation (10s)..." -ForegroundColor Cyan
    Start-Sleep 10
    Write-Host "   ✅ Daemon lancé" -ForegroundColor Green
} else {
    Write-Host "1️⃣ Daemon déjà lancé ✅" -ForegroundColor Green
}
Write-Host ""

# 2. Créer wallet si pas déjà fait
$walletPath = Join-Path $binDir $WalletName
if (-not (Test-Path $walletPath)) {
    Write-Host "2️⃣ Création wallet testnet: $WalletName" -ForegroundColor Yellow
    Write-Host "   (Password vide pour tests)" -ForegroundColor Cyan
    
    # Alternative: utiliser --generate-from-json
    $walletConfig = @{
        "version" = 1
        "filename" = $WalletName
        "password" = ""
    } | ConvertTo-Json
    
    $configPath = Join-Path $env:TEMP "wallet-config.json"
    $walletConfig | Set-Content $configPath
    
    try {
        & $walletCliPath.FullName --testnet --generate-from-json $configPath --log-file $null
        Write-Host "   ✅ Wallet créé" -ForegroundColor Green
    } catch {
        Write-Host "   ⚠️ Erreur création wallet (peut-être déjà existant)" -ForegroundColor Yellow
    }
    
    Remove-Item $configPath -ErrorAction SilentlyContinue
} else {
    Write-Host "2️⃣ Wallet existe déjà ✅" -ForegroundColor Green
}
Write-Host ""

# 3. Lancer wallet RPC (si pas déjà lancé)
$walletRpcRunning = Get-Process monero-wallet-rpc -ErrorAction SilentlyContinue
if ($walletRpcRunning) {
    Write-Host "3️⃣ Wallet RPC déjà lancé" -ForegroundColor Yellow
    Write-Host "   Fermeture pour relancer proprement..." -ForegroundColor Cyan
    Stop-Process -Name monero-wallet-rpc -Force
    Start-Sleep 2
}

Write-Host "3️⃣ Lancement wallet RPC: $WalletName" -ForegroundColor Yellow
Start-Process -FilePath $walletRpcPath.FullName `
    -ArgumentList `
        "--testnet", `
        "--wallet-file","$WalletName", `
        "--password","""", `
        "--rpc-bind-ip","127.0.0.1", `
        "--rpc-bind-port","18082", `
        "--disable-rpc-login", `
        "--daemon-address","127.0.0.1:28081" `
    -WorkingDirectory $binDir `
    -WindowStyle Hidden

Write-Host "   Attente démarrage RPC (5s)..." -ForegroundColor Cyan
Start-Sleep 5

# 4. Tester connexion RPC
Write-Host "4️⃣ Test connexion RPC..." -ForegroundColor Yellow
try {
    $response = Invoke-RestMethod `
        -Uri "http://127.0.0.1:18082/json_rpc" `
        -Method Post `
        -ContentType "application/json" `
        -Body '{"jsonrpc":"2.0","id":"0","method":"get_version"}' `
        -TimeoutSec 5
    
    Write-Host "   ✅ RPC accessible" -ForegroundColor Green
    Write-Host "   Version: $($response.result.version)" -ForegroundColor Cyan
} catch {
    Write-Host "   ❌ RPC non accessible" -ForegroundColor Red
    Write-Host "   Erreur: $_" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "✅ Setup Monero Testnet complet!" -ForegroundColor Green
Write-Host ""
Write-Host "📋 Résumé:" -ForegroundColor Cyan
Write-Host "  Daemon: testnet @ 127.0.0.1:28081"
Write-Host "  Wallet: $WalletName (password vide)"
Write-Host "  RPC: http://127.0.0.1:18082"
Write-Host ""
Write-Host "🧪 Prochaine étape:" -ForegroundColor Cyan
Write-Host "  cargo test --package wallet" -ForegroundColor Yellow
