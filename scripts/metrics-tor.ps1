# Script: metrics-tor.ps1
# Collecte metriques specifiques Tor
# Usage: .\scripts\metrics-tor.ps1

# Verifier que nous sommes dans le bon repertoire
if (-not (Test-Path ".cursorrules")) {
    Write-Host "ERREUR: Execute ce script depuis la racine du projet" -ForegroundColor Red
    exit 1
}

# Creer le repertoire metrics s'il n'existe pas
$metricsDir = "docs\metrics"
if (-not (Test-Path $metricsDir)) {
    New-Item -ItemType Directory -Path $metricsDir -Force | Out-Null
}

$date = Get-Date -Format "yyyy-MM-dd"
$timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
$output = "$metricsDir\tor-$date.json"

Write-Host "Collecte metriques Tor..." -ForegroundColor Cyan

# Test Tor connectivity
$torConnected = $false
$torExitNode = "N/A"
try {
    $torCheck = Invoke-RestMethod `
        -Uri "https://check.torproject.org/api/ip" `
        -Proxy "socks5://127.0.0.1:9050" `
        -TimeoutSec 10
    $torConnected = $torCheck.IsTor
    $torExitNode = $torCheck.IP
} catch {
    $torConnected = $false
    $torExitNode = "N/A"
}

# Count .onion references in code (should be 0 in logs, OK in code)
$onionInLogs = 0
$logFiles = Get-ChildItem -Path "logs" -Include "*.log" -ErrorAction SilentlyContinue
if ($logFiles) {
    $onionInLogs = (Select-String -Path $logFiles.FullName -Pattern "\.onion" -ErrorAction SilentlyContinue | Measure-Object).Count
}

$onionInCode = (Select-String -Path "*/src/*.rs" -Pattern "\.onion" -ErrorAction SilentlyContinue | Measure-Object).Count

# Count Tor-related functions
$torFunctions = (Select-String -Path "*/src/*.rs" -Pattern "socks5|proxy.*tor|via_tor" -ErrorAction SilentlyContinue | Measure-Object).Count

# RPC exposure check
$rpcExposed = $false
try {
    $netstatOutput = netstat -an 2>&1
    $rpcExposed = $netstatOutput -match "0\.0\.0\.0:18082"
} catch {
    $rpcExposed = $false
}

# Reality checks Tor count
$torRealityChecks = (Get-ChildItem "docs/reality-checks/tor-*.md" -ErrorAction SilentlyContinue | Measure-Object).Count

# Count functions with network code
$networkFunctions = (Select-String -Path "*/src/*.rs" -Pattern "reqwest::|curl|http://|https://" -ErrorAction SilentlyContinue | Measure-Object).Count

# Count functions without Tor reality checks
$functionsWithoutTorCheck = $networkFunctions - $torRealityChecks

# Security violations count
$securityViolations = 0
$violationPatterns = @(
    "0\.0\.0\.0:18082",  # RPC exposed publicly
    "log.*\.onion",      # .onion in logs
    "log.*view_key",     # View key in logs
    "log.*spend_key",    # Spend key in logs
    "log.*password"      # Password in logs
)

foreach ($pattern in $violationPatterns) {
    $matches = (Select-String -Path "*/src/*.rs" -Pattern $pattern -ErrorAction SilentlyContinue | Measure-Object).Count
    $securityViolations += $matches
}

# Tor daemon status
$torDaemonRunning = (Get-Process -Name "tor" -ErrorAction SilentlyContinue) -ne $null

# Monero RPC status
$moneroRPCRunning = $false
try {
    $rpcResponse = Invoke-RestMethod `
        -Uri "http://127.0.0.1:18082/json_rpc" `
        -Method Post `
        -ContentType "application/json" `
        -Body '{"jsonrpc":"2.0","id":"0","method":"get_version"}' `
        -TimeoutSec 5
    $moneroRPCRunning = $true
} catch {
    $moneroRPCRunning = $false
}

$metrics = @{
    date = $date
    timestamp = $timestamp
    tor_connected = $torConnected
    tor_exit_node = $torExitNode
    tor_daemon_running = $torDaemonRunning
    onion_refs_in_logs = $onionInLogs
    onion_refs_in_code = $onionInCode
    tor_functions = $torFunctions
    network_functions = $networkFunctions
    functions_without_tor_check = $functionsWithoutTorCheck
    rpc_exposed_publicly = $rpcExposed
    monero_rpc_running = $moneroRPCRunning
    tor_reality_checks = $torRealityChecks
    security_violations = $securityViolations
    tor_coverage = if ($networkFunctions -gt 0) { [math]::Round(($torRealityChecks / $networkFunctions) * 100, 1) } else { 0 }
}

$metrics | ConvertTo-Json -Depth 3 | Set-Content $output

Write-Host "Metriques Tor sauvegardees" -ForegroundColor Green
Write-Host ""
Write-Host "Resume:" -ForegroundColor Cyan
Write-Host "  Tor Connected: $(if ($torConnected) { '✓' } else { '✗' })" -ForegroundColor $(if ($torConnected) { "Green" } else { "Red" })
Write-Host "  Exit Node: $torExitNode" -ForegroundColor White
Write-Host "  Tor Daemon: $(if ($torDaemonRunning) { '✓' } else { '✗' })" -ForegroundColor $(if ($torDaemonRunning) { "Green" } else { "Red" })
Write-Host "  .onion in logs: $onionInLogs $(if ($onionInLogs -eq 0) { '✅' } else { '⚠️' })" -ForegroundColor $(if ($onionInLogs -eq 0) { "Green" } else { "Red" })
Write-Host "  RPC Exposed: $(if ($rpcExposed) { '⚠️ YES' } else { '✅ NO' })" -ForegroundColor $(if ($rpcExposed) { "Red" } else { "Green" })
Write-Host "  Tor Functions: $torFunctions" -ForegroundColor White
Write-Host "  Network Functions: $networkFunctions" -ForegroundColor White
Write-Host "  Tor Reality Checks: $torRealityChecks" -ForegroundColor White
Write-Host "  Tor Coverage: $($metrics.tor_coverage)%" -ForegroundColor $(if ($metrics.tor_coverage -ge 80) { "Green" } elseif ($metrics.tor_coverage -ge 50) { "Yellow" } else { "Red" })
Write-Host "  Security Violations: $securityViolations" -ForegroundColor $(if ($securityViolations -eq 0) { "Green" } else { "Red" })

# Afficher les warnings
$warnings = @()
if (-not $torConnected) { $warnings += "Tor non connecte" }
if ($onionInLogs -gt 0) { $warnings += ".onion dans logs" }
if ($rpcExposed) { $warnings += "RPC expose publiquement" }
if ($functionsWithoutTorCheck -gt 0) { $warnings += "$functionsWithoutTorCheck fonction(s) sans reality check Tor" }
if ($securityViolations -gt 0) { $warnings += "$securityViolations violation(s) de securite" }

if ($warnings.Count -gt 0) {
    Write-Host ""
    Write-Host "WARNINGS:" -ForegroundColor Yellow
    foreach ($warning in $warnings) {
        Write-Host "  - $warning" -ForegroundColor Yellow
    }
} else {
    Write-Host ""
    Write-Host "Aucun warning Tor detecte ✅" -ForegroundColor Green
}

Write-Host ""
Write-Host "Fichier sauvegarde: $output" -ForegroundColor Cyan
