# scripts/check-monero-tor-final.ps1
# Détection finale de patterns Monero/Tor

param(
    [switch]$Verbose = $false
)

Write-Host "Monero/Tor Security Check" -ForegroundColor Cyan
Write-Host "=========================" -ForegroundColor Cyan
Write-Host ""

$issues = @()

# 1. Vérifier RPC exposé publiquement (patterns simples)
$rpcExposed = (Select-String -Path "*/src/*.rs" -Pattern "0\.0\.0\.0" -ErrorAction SilentlyContinue | Where-Object { $_.Line -notlike "*test*" -and $_.Line -notlike "*assert*" } | Measure-Object).Count
if ($rpcExposed -gt 0) {
    $issues += "RPC exposed publicly: $rpcExposed occurrences"
}

# 2. Vérifier les connexions directes
$directConnections = (Select-String -Path "*/src/*.rs" -Pattern "reqwest::get" -ErrorAction SilentlyContinue | Measure-Object).Count
if ($directConnections -gt 0) {
    $issues += "Direct HTTP connections: $directConnections occurrences"
}

# 3. Vérifier les adresses .onion loggées
$onionLogged = (Select-String -Path "*/src/*.rs" -Pattern "\.onion" -ErrorAction SilentlyContinue | Measure-Object).Count
if ($onionLogged -gt 0) {
    $issues += "Onion addresses logged: $onionLogged occurrences"
}

# 4. Vérifier les IPs non-localhost (hors constantes)
$ipsLogged = (Select-String -Path "*/src/*.rs" -Pattern "192\.168\.|10\.|172\." -ErrorAction SilentlyContinue | Where-Object { $_.Line -notlike "*const*" -and $_.Line -notlike "*test*" } | Measure-Object).Count
if ($ipsLogged -gt 0) {
    $issues += "Non-localhost IPs: $ipsLogged occurrences"
}

# 5. Vérifier les credentials hardcodés (pattern simple)
$hardcodedCreds = (Select-String -Path "*/src/*.rs" -Pattern "password.*=" -ErrorAction SilentlyContinue | Where-Object { $_.Line -notlike "*test*" } | Measure-Object).Count
if ($hardcodedCreds -gt 0) {
    $issues += "Hardcoded credentials: $hardcodedCreds occurrences"
}

# Afficher les résultats
if ($issues.Count -eq 0) {
    Write-Host "✅ No Monero/Tor security issues detected" -ForegroundColor Green
    Write-Host ""
    exit 0
}

Write-Host "❌ Monero/Tor security issues detected:" -ForegroundColor Red
foreach ($issue in $issues) {
    Write-Host "  - $issue" -ForegroundColor Red
}
Write-Host ""
Write-Host "Recommendations:" -ForegroundColor Yellow
Write-Host "  1. Use localhost-only RPC binding (127.0.0.1)" -ForegroundColor White
Write-Host "  2. Use SOCKS5 proxy for external connections" -ForegroundColor White
Write-Host "  3. Never log .onion addresses or IPs" -ForegroundColor White
Write-Host "  4. Use environment variables for credentials" -ForegroundColor White
Write-Host ""
Write-Host "❌ COMMIT BLOCKED - Fix Monero/Tor security issues first" -ForegroundColor Red
exit 1
