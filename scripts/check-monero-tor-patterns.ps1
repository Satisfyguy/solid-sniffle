# scripts/check-monero-tor-patterns.ps1
# Détection de patterns spécifiques Monero/Tor

param(
    [string]$Path = ".",
    [switch]$Verbose = $false
)

Write-Host "Monero/Tor Patterns Detection" -ForegroundColor Cyan
Write-Host "==============================" -ForegroundColor Cyan
Write-Host ""

$totalIssues = 0
$issuesFound = @()

# Patterns spécifiques Monero
$moneroPatterns = @{
    "RPC Exposé Publiquement" = @(
        "--rpc-bind-ip 0\.0\.0\.0",
        "0\.0\.0\.0:18082",
        "rpc-bind-ip.*0\.0\.0\.0"
    )
    "Credentials Monero Hardcodés" = @(
        "wallet-password.*=.*[\"']",
        "rpc-password.*=.*[\"']",
        "daemon-password.*=.*[\"']"
    )
    "View/Spend Keys Loggés" = @(
        "log.*view_key",
        "log.*spend_key",
        "println.*view_key",
        "println.*spend_key",
        "tracing.*view_key",
        "tracing.*spend_key"
    )
    "Multisig Info Non Sécurisé" = @(
        "multisig_info.*println",
        "multisig_info.*log",
        "export_multisig.*println"
    )
    "Monero URLs Clearnet" = @(
        "https://.*monero",
        "http://.*monero",
        "monero.*\.com",
        "monero.*\.org"
    )
}

# Patterns spécifiques Tor
$torPatterns = @{
    "Connexions Directes" = @(
        "reqwest::get\(",
        "curl.*http://",
        "fetch.*http://",
        "TcpStream::connect"
    )
    "Adresses .onion Loggées" = @(
        "log.*\.onion",
        "println.*\.onion",
        "tracing.*\.onion"
    )
    "IPs Réelles Loggées" = @(
        "log.*\d+\.\d+\.\d+\.\d+",
        "println.*\d+\.\d+\.\d+\.\d+",
        "tracing.*\d+\.\d+\.\d+\.\d+"
    )
    "User-Agent Identifiant" = @(
        "User-Agent.*Monero",
        "User-Agent.*Tor",
        "User-Agent.*Marketplace"
    )
    "Tor Bypass" = @(
        "bypass.*tor",
        "skip.*tor",
        "disable.*tor"
    )
}

# Patterns de sécurité générale
$securityPatterns = @{
    "Timing Attacks" = @(
        "sleep\(0\)",
        "delay\(0\)",
        "wait\(0\)"
    )
    "Random Faible" = @(
        "rand::",
        "random\(\)",
        "Math\.random"
    )
    "Secrets en Mémoire" = @(
        "String::from.*password",
        "String::from.*secret",
        "String::from.*key"
    )
}

# Scanner les fichiers Rust
$rustFiles = Get-ChildItem -Path $Path -Recurse -Include "*.rs" | Where-Object {
    $_.FullName -notlike "*target*" -and 
    $_.FullName -notlike "*.git*"
}

Write-Host "Scanning $($rustFiles.Count) Rust files for Monero/Tor patterns..." -ForegroundColor Yellow
Write-Host ""

foreach ($file in $rustFiles) {
    $relativePath = $file.FullName.Replace((Get-Location).Path + "\", "").Replace("\", "/")
    $lines = Get-Content $file.FullName -ErrorAction SilentlyContinue
    
    if (-not $lines) { continue }
    
    for ($i = 0; $i -lt $lines.Count; $i++) {
        $lineNumber = $i + 1
        $line = $lines[$i]
        
        # Vérifier les patterns Monero
        foreach ($category in $moneroPatterns.Keys) {
            foreach ($pattern in $moneroPatterns[$category]) {
                if ($line -match $pattern) {
                    $issue = @{
                        File = $relativePath
                        Line = $lineNumber
                        Category = "Monero: $category"
                        Pattern = $pattern
                        Content = $line.Trim()
                        Severity = "Critical"
                    }
                    $issuesFound += $issue
                    $totalIssues++
                    
                    if ($Verbose) {
                        Write-Host "❌ Monero Security Issue" -ForegroundColor Red
                        Write-Host "   ${relativePath}:${lineNumber}" -ForegroundColor Gray
                        Write-Host "   $($line.Trim())" -ForegroundColor Gray
                        Write-Host ""
                    }
                }
            }
        }
        
        # Vérifier les patterns Tor
        foreach ($category in $torPatterns.Keys) {
            foreach ($pattern in $torPatterns[$category]) {
                if ($line -match $pattern) {
                    $issue = @{
                        File = $relativePath
                        Line = $lineNumber
                        Category = "Tor: $category"
                        Pattern = $pattern
                        Content = $line.Trim()
                        Severity = "Critical"
                    }
                    $issuesFound += $issue
                    $totalIssues++
                    
                    if ($Verbose) {
                        Write-Host "❌ Tor Security Issue" -ForegroundColor Red
                        Write-Host "   ${relativePath}:${lineNumber}" -ForegroundColor Gray
                        Write-Host "   $($line.Trim())" -ForegroundColor Gray
                        Write-Host ""
                    }
                }
            }
        }
        
        # Vérifier les patterns de sécurité générale
        foreach ($category in $securityPatterns.Keys) {
            foreach ($pattern in $securityPatterns[$category]) {
                if ($line -match $pattern) {
                    $issue = @{
                        File = $relativePath
                        Line = $lineNumber
                        Category = "Security: $category"
                        Pattern = $pattern
                        Content = $line.Trim()
                        Severity = "High"
                    }
                    $issuesFound += $issue
                    $totalIssues++
                    
                    if ($Verbose) {
                        Write-Host "⚠️ Security Issue" -ForegroundColor Yellow
                        Write-Host "   ${relativePath}:${lineNumber}" -ForegroundColor Gray
                        Write-Host "   $($line.Trim())" -ForegroundColor Gray
                        Write-Host ""
                    }
                }
            }
        }
    }
}

# Afficher le rapport
Write-Host "Monero/Tor Security Report" -ForegroundColor Cyan
Write-Host "===========================" -ForegroundColor Cyan
Write-Host ""

if ($totalIssues -eq 0) {
    Write-Host "✅ No Monero/Tor security issues detected!" -ForegroundColor Green
    Write-Host ""
    exit 0
}

Write-Host "❌ Monero/Tor security issues detected: $totalIssues issues" -ForegroundColor Red
Write-Host ""

# Grouper par catégorie
$groupedIssues = $issuesFound | Group-Object Category

foreach ($group in $groupedIssues) {
    Write-Host "$($group.Name): $($group.Count) issues" -ForegroundColor Red
    foreach ($issue in $group.Group | Select-Object -First 3) {
        Write-Host "  $($issue.File):$($issue.Line) - $($issue.Content)" -ForegroundColor Gray
    }
    if ($group.Count -gt 3) {
        Write-Host "  ... and $($group.Count - 3) more" -ForegroundColor Gray
    }
    Write-Host ""
}

# Recommandations spécifiques
Write-Host "Recommendations:" -ForegroundColor Yellow
Write-Host "  1. Use localhost-only RPC binding (127.0.0.1)" -ForegroundColor White
Write-Host "  2. Never log view/spend keys or .onion addresses" -ForegroundColor White
Write-Host "  3. Use SOCKS5 proxy for all external connections" -ForegroundColor White
Write-Host "  4. Use cryptographically secure random number generation" -ForegroundColor White
Write-Host "  5. Implement proper timing attack protection" -ForegroundColor White
Write-Host ""

Write-Host "❌ COMMIT BLOCKED - Fix Monero/Tor security issues first" -ForegroundColor Red
exit 1
