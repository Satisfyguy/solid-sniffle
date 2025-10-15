# Script: test-rpc.ps1
# Teste la connexion Monero RPC
# Usage: .\scripts\test-rpc.ps1

Write-Host "TEST CONNEXION MONERO RPC" -ForegroundColor Cyan
Write-Host "==========================" -ForegroundColor Cyan

# Verifier que nous sommes dans le bon repertoire
if (-not (Test-Path ".cursorrules")) {
    Write-Host "ERREUR: Execute ce script depuis la racine du projet" -ForegroundColor Red
    exit 1
}

$errors = 0

# 1. Test connexion daemon
Write-Host "`n1. Test connexion daemon (port 18081)..." -ForegroundColor Yellow

try {
    $response = Invoke-RestMethod -Uri "http://127.0.0.1:18081/json_rpc" -Method Post -ContentType "application/json" -Body '{"jsonrpc":"2.0","id":"0","method":"get_version"}' -TimeoutSec 5
    Write-Host "   Daemon accessible" -ForegroundColor Green
    Write-Host "   Version: $($response.result.version)" -ForegroundColor White
} catch {
    Write-Host "   Daemon non accessible: $_" -ForegroundColor Red
    $errors++
}

# 2. Test connexion wallet RPC
Write-Host "`n2. Test connexion wallet RPC (port 18082)..." -ForegroundColor Yellow

try {
    $response = Invoke-RestMethod -Uri "http://127.0.0.1:18082/json_rpc" -Method Post -ContentType "application/json" -Body '{"jsonrpc":"2.0","id":"0","method":"get_version"}' -TimeoutSec 5
    Write-Host "   Wallet RPC accessible" -ForegroundColor Green
    Write-Host "   Version: $($response.result.version)" -ForegroundColor White
} catch {
    Write-Host "   Wallet RPC non accessible: $_" -ForegroundColor Red
    $errors++
}

# 3. Test get_balance
Write-Host "`n3. Test get_balance..." -ForegroundColor Yellow

try {
    $response = Invoke-RestMethod -Uri "http://127.0.0.1:18082/json_rpc" -Method Post -ContentType "application/json" -Body '{"jsonrpc":"2.0","id":"0","method":"get_balance"}' -TimeoutSec 5
    Write-Host "   get_balance fonctionne" -ForegroundColor Green
    Write-Host "   Balance: $($response.result.balance) atomic units" -ForegroundColor White
    Write-Host "   Unlocked: $($response.result.unlocked_balance) atomic units" -ForegroundColor White
} catch {
    Write-Host "   get_balance echoue: $_" -ForegroundColor Red
    $errors++
}

# 4. Test is_multisig
Write-Host "`n4. Test is_multisig..." -ForegroundColor Yellow

try {
    $response = Invoke-RestMethod -Uri "http://127.0.0.1:18082/json_rpc" -Method Post -ContentType "application/json" -Body '{"jsonrpc":"2.0","id":"0","method":"is_multisig"}' -TimeoutSec 5
    Write-Host "   is_multisig fonctionne" -ForegroundColor Green
    Write-Host "   Multisig: $($response.result.multisig)" -ForegroundColor White
} catch {
    Write-Host "   is_multisig echoue: $_" -ForegroundColor Red
    $errors++
}

# 5. Test avec notre CLI
Write-Host "`n5. Test avec notre CLI..." -ForegroundColor Yellow

try {
    # Tester si cargo est disponible
    $cargoTest = cargo --version 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "   Cargo disponible" -ForegroundColor Green
        
        # Tester notre CLI
        $cliTest = cargo run --bin monero-marketplace -- test 2>&1
        if ($LASTEXITCODE -eq 0) {
            Write-Host "   CLI fonctionne" -ForegroundColor Green
        } else {
            Write-Host "   CLI echoue: $cliTest" -ForegroundColor Red
            $errors++
        }
    } else {
        Write-Host "   Cargo non disponible, test CLI ignore" -ForegroundColor Yellow
    }
} catch {
    Write-Host "   Erreur lors du test CLI: $_" -ForegroundColor Red
    $errors++
}

# Resume
Write-Host "`nRESUME DES TESTS" -ForegroundColor Cyan
Write-Host "=================" -ForegroundColor Cyan

if ($errors -eq 0) {
    Write-Host "TOUS LES TESTS PASSENT!" -ForegroundColor Green
    Write-Host "Monero RPC est pret pour le developpement" -ForegroundColor Green
    exit 0
} else {
    Write-Host "$errors test(s) echoue(s)" -ForegroundColor Red
    Write-Host "`nSolutions possibles:" -ForegroundColor Yellow
    Write-Host "1. Lancer: .\scripts\start-testnet.ps1" -ForegroundColor White
    Write-Host "2. Verifier que Monero est installe: .\scripts\setup-monero.ps1" -ForegroundColor White
    Write-Host "3. Creer un wallet si necessaire" -ForegroundColor White
    exit 1
}
