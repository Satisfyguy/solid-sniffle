# scripts/metrics-dashboard.ps1
# Dashboard métriques complet pour Monero Marketplace Tor v2.0

Write-Host "📊 Monero Marketplace - Metrics Dashboard" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# 1. Code Metrics
Write-Host "📝 Code Metrics" -ForegroundColor Yellow
$rustFiles = Get-ChildItem -Recurse -Include "*.rs" | Get-Content | Measure-Object -Line
$loc = $rustFiles.Lines

$functions = (Select-String -Path "*/src/*.rs" -Pattern "pub (async )?fn \w+" | Measure-Object).Count
$specs = (Get-ChildItem "docs/specs/*.md" -ErrorAction SilentlyContinue | Measure-Object).Count
$functionsWithoutSpec = $functions - $specs

$unwraps = (Select-String -Path "*/src/*.rs" -Pattern "\.unwrap\(\)" | Measure-Object).Count
$todos = (Select-String -Path "*/src/*.rs" -Pattern "TODO|FIXME" | Measure-Object).Count

Write-Host "  Lines of Code: $loc"
Write-Host "  Functions: $functions"
Write-Host "  Specs: $specs"
Write-Host "  Functions sans spec: $functionsWithoutSpec"
Write-Host "  Unwraps: $unwraps $(if ($unwraps -eq 0) { '✅' } else { '⚠️' })"
Write-Host "  TODOs: $todos"
Write-Host ""

# 2. Tor Metrics
Write-Host "🧅 Tor Metrics" -ForegroundColor Yellow
$torProcess = Get-Process -Name "tor" -ErrorAction SilentlyContinue
$torRunning = ($torProcess -ne $null)

if ($torRunning) {
    Write-Host "  Tor Daemon: ✅ Running"
    
    # Test connexion Tor
    try {
        $torCheck = Invoke-RestMethod -Uri "https://check.torproject.org/api/ip" -Proxy "socks5://127.0.0.1:9050" -TimeoutSec 10
        if ($torCheck.IsTor) {
            Write-Host "  Tor Connected: ✅"
            Write-Host "  Exit Node: $($torCheck.IP)"
        } else {
            Write-Host "  Tor Connected: ❌"
        }
    } catch {
        Write-Host "  Tor Connected: ❌ (test failed)"
    }
    
    # Compter reality checks Tor
    $torRc = (Get-ChildItem "docs/reality-checks/tor-*.md" -ErrorAction SilentlyContinue | Measure-Object).Count
    Write-Host "  Tor Reality Checks: $torRc"
} else {
    Write-Host "  Tor Daemon: ❌ Not Running"
    Write-Host "  Tor Connected: ❌"
    Write-Host "  Tor Reality Checks: 0"
}
Write-Host ""

# 3. Monero Metrics
Write-Host "💰 Monero Metrics" -ForegroundColor Yellow
$moneroProcess = Get-Process -Name "monerod" -ErrorAction SilentlyContinue
$moneroRunning = ($moneroProcess -ne $null)

if ($moneroRunning) {
    Write-Host "  Daemon Running: ✅"
} else {
    Write-Host "  Daemon Running: ❌"
}

$walletRpcProcess = Get-Process -Name "monero-wallet-rpc" -ErrorAction SilentlyContinue
$walletRpcRunning = ($walletRpcProcess -ne $null)

Write-Host "  Wallet RPC Running: $(if ($walletRpcRunning) { '✅' } else { '❌' })"

if ($walletRpcRunning) {
    try {
        $rpcResponse = Invoke-RestMethod -Uri "http://127.0.0.1:18082/json_rpc" -Method Post -ContentType "application/json" -Body '{"jsonrpc":"2.0","id":"0","method":"get_version"}' -TimeoutSec 5
        Write-Host "  RPC Version: $($rpcResponse.result.version)"
        Write-Host "  RPC Accessible: ✅"
    } catch {
        Write-Host "  RPC Accessible: ❌"
    }
    
    # Vérifier isolation RPC
    $rpcPorts = netstat -an | Select-String "18082" | Select-String "LISTENING"
    if ($rpcPorts -match "0.0.0.0:18082") {
        Write-Host "  RPC Isolation: ⚠️ EXPOSED PUBLICLY!" -ForegroundColor Red
    } elseif ($rpcPorts -match "127.0.0.1:18082") {
        Write-Host "  RPC Isolation: ✅ Localhost only"
    }
}
Write-Host ""

# 4. Test Results
Write-Host "🧪 Test Results" -ForegroundColor Yellow
$testOutput = cargo test --workspace 2>&1 | Out-String
if ($testOutput -match "(\d+) passed") {
    $testsPassed = $matches[1]
    Write-Host "  Tests Passed: $testsPassed ✅"
} else {
    Write-Host "  Tests: ❌ Not run or failed"
}
Write-Host ""

# 5. Reality Checks
Write-Host "📋 Reality Checks" -ForegroundColor Yellow
$totalRc = (Get-ChildItem "docs/reality-checks/*.md" -ErrorAction SilentlyContinue | Measure-Object).Count
$torRc = (Get-ChildItem "docs/reality-checks/tor-*.md" -ErrorAction SilentlyContinue | Measure-Object).Count
$validatedRc = (Select-String -Path "docs/reality-checks/*.md" -Pattern "\[x\] ✅ Validé" -ErrorAction SilentlyContinue | Measure-Object).Count

Write-Host "  Total: $totalRc"
Write-Host "  Tor-specific: $torRc"
Write-Host "  Validés: $validatedRc"
Write-Host ""

# 6. Security Status
Write-Host "🔒 Security Status" -ForegroundColor Yellow
$securityScore = 100

# Pénalités
if ($unwraps -gt 0) { $securityScore -= 20 }
if ($rpcPorts -match "0.0.0.0:18082") { $securityScore -= 50 }
if (-not $torRunning) { $securityScore -= 10 }
if ($functions -gt $specs) { $securityScore -= 10 }

$scoreColor = if ($securityScore -ge 90) { "Green" } elseif ($securityScore -ge 70) { "Yellow" } else { "Red" }
Write-Host "  Security Score: $securityScore/100" -ForegroundColor $scoreColor

if ($securityScore -lt 100) {
    Write-Host "`n  Issues détectées:" -ForegroundColor Yellow
    if ($unwraps -gt 0) { Write-Host "    - $unwraps unwrap() trouvés ⚠️" }
    if ($rpcPorts -match "0.0.0.0") { Write-Host "    - RPC exposé publiquement 🚨" }
    if (-not $torRunning) { Write-Host "    - Tor pas lancé ⚠️" }
    if ($functions -gt $specs) { Write-Host "    - $($functions - $specs) fonctions sans spec ⚠️" }
}
Write-Host ""

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Dernière mise à jour: $(Get-Date -Format 'yyyy-MM-dd HH:mm')" -ForegroundColor Cyan
