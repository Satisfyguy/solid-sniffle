# scripts/check-monero-tor-basic.ps1
# Détection basique de patterns Monero/Tor

param(
    [switch]$Verbose = $false
)

Write-Host "Monero/Tor Security Check" -ForegroundColor Cyan
Write-Host "=========================" -ForegroundColor Cyan
Write-Host ""

$issues = @()

# 1. Vérifier RPC exposé publiquement (hors tests)
$rpcExposedLines = Select-String -Path "*/src/*.rs" -Pattern "0\.0\.0\.0.*18082" -ErrorAction SilentlyContinue
$rpcExposed = 0
foreach ($line in $rpcExposedLines) {
    # Vérifier si c'est dans une fonction de test
    $isInTest = $false
    $fileContent = Get-Content $line.Filename -ErrorAction SilentlyContinue
    if ($fileContent) {
        for ($i = [Math]::Max(0, $line.LineNumber - 10); $i -lt [Math]::Min($fileContent.Length, $line.LineNumber + 2); $i++) {
            if ($fileContent[$i] -match "#\[.*test.*\]|fn.*test|async fn.*test") {
                $isInTest = $true
                break
            }
        }
    }
    if (-not $isInTest) {
        $rpcExposed++
    }
}
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

# 4. Vérifier les IPs loggées (hors constantes et tests)
$ipsLogged = (Select-String -Path "*/src/*.rs" -Pattern "\d+\.\d+\.\d+\.\d+" -ErrorAction SilentlyContinue | Where-Object { 
        $_.Line -notlike "*const*" -and 
        $_.Line -notlike "*pub const*" -and 
        $_.Line -notlike "*test*" -and 
        $_.Line -notlike "*assert*" -and
        $_.Line -notlike "*//*" -and
        $_.Line -notlike "*localhost*"
    } | Measure-Object).Count
if ($ipsLogged -gt 0) {
    $issues += "IP addresses logged: $ipsLogged occurrences"
}

# 5. Vérifier les credentials hardcodés
$hardcodedCreds = (Select-String -Path "*/src/*.rs" -Pattern "password.*=" -ErrorAction SilentlyContinue | Measure-Object).Count
if ($hardcodedCreds -gt 0) {
    $issues += "Hardcoded credentials: $hardcodedCreds occurrences"
}

# Afficher les résultats
if ($issues.Count -eq 0) {
    Write-Host "✅ No Monero/Tor security issues detected" -ForegroundColor Green
}
else {
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
    exit 1
}

Write-Host ""
Write-Host "Security check completed" -ForegroundColor Green
