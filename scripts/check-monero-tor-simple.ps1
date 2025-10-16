# scripts/check-monero-tor-simple.ps1
# Détection simple de patterns Monero/Tor avec exceptions

param(
    [string]$Path = ".",
    [string]$IgnoreFile = ".security-theatre-ignore",
    [switch]$Verbose = $false
)

Write-Host "Monero/Tor Security Check" -ForegroundColor Cyan
Write-Host "=========================" -ForegroundColor Cyan
Write-Host ""

# Charger les exceptions
$exceptions = @()
if (Test-Path $IgnoreFile) {
    $exceptions = Get-Content $IgnoreFile | Where-Object {
        $_ -notmatch "^\s*#" -and $_.Trim() -ne ""
    }
    if ($Verbose) {
        Write-Host "Loaded $($exceptions.Count) exceptions from $IgnoreFile" -ForegroundColor Yellow
    }
}

# Fonction pour vérifier si une ligne est dans les exceptions
function Test-Exception {
    param($filePath, $lineContent)
    
    foreach ($exception in $exceptions) {
        if ($exception -match "^([^:]+):(.+)$") {
            $pattern = $matches[1]
            $linePattern = $matches[2]
            
            # Vérifier si le fichier match le pattern
            if ($filePath -like $pattern) {
                # Vérifier si la ligne match le pattern
                if ($lineContent -match $linePattern) {
                    return $true
                }
            }
        }
    }
    return $false
}

$totalIssues = 0
$issuesFound = @()

# Patterns Monero/Tor critiques
$patterns = @{
    "RPC Exposé Publiquement" = @(
        "0\.0\.0\.0.*18082",
        "192\.168\..*18082",
        "10\..*18082"
    )
    "Connexions Directes" = @(
        "reqwest::get\(",
        "curl.*http://",
        "fetch.*http://"
    )
    "Adresses .onion Loggées" = @(
        "\.onion"
    )
    "IPs Réelles Loggées" = @(
        "\d+\.\d+\.\d+\.\d+"
    )
    "Credentials Hardcodés" = @(
        "password.*=.*[\"']",
        "secret.*=.*[\"']",
        "key.*=.*[\"']"
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
        
        # Vérifier les exceptions
        if (Test-Exception -filePath $relativePath -lineContent $line) {
            continue
        }
        
        # Tester chaque pattern
        foreach ($category in $patterns.Keys) {
            foreach ($pattern in $patterns[$category]) {
                if ($line -match $pattern) {
                    $issue = @{
                        File = $relativePath
                        Line = $lineNumber
                        Category = $category
                        Pattern = $pattern
                        Content = $line.Trim()
                    }
                    $issuesFound += $issue
                    $totalIssues++
                    
                    if ($Verbose) {
                        Write-Host "❌ Monero/Tor Security Issue" -ForegroundColor Red
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

# Recommandations
Write-Host "Recommendations:" -ForegroundColor Yellow
Write-Host "  1. Use localhost-only RPC binding (127.0.0.1)" -ForegroundColor White
Write-Host "  2. Use SOCKS5 proxy for external connections" -ForegroundColor White
Write-Host "  3. Never log .onion addresses or IPs" -ForegroundColor White
Write-Host "  4. Use environment variables for credentials" -ForegroundColor White
Write-Host ""

Write-Host "❌ COMMIT BLOCKED - Fix Monero/Tor security issues first" -ForegroundColor Red
exit 1
