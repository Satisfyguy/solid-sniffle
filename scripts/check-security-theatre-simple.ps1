# scripts/check-security-theatre-simple.ps1
# Détection automatique du security theatre dans le code Rust

param(
    [string]$Path = ".",
    [string]$IgnoreFile = ".security-theatre-ignore",
    [switch]$Verbose = $false
)

Write-Host "Security Theatre Detection" -ForegroundColor Cyan
Write-Host "=============================" -ForegroundColor Cyan
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

# Compteurs d'issues
$totalIssues = 0
$issuesFound = @()

# Patterns de détection
$patterns = @(
    "assert!.*true",
    "assert!.*false", 
    "//.*Placeholder",
    "//.*TODO",
    "//.*FIXME",
    "//.*XXX",
    "//.*HACK",
    "should.*work",
    "probably.*works",
    "assume",
    "HYPOTHÈSES",
    "À.*VALIDER",
    "ERREUR.*POSSIBLE",
    "À.*IMPLÉMENTER",
    "unimplemented!",
    "todo!",
    "panic!",
    "password.*=",
    "secret.*=",
    "key.*=",
    "token.*=",
    "api_key.*=",
    "private_key.*=",
    "\.unwrap\s*\(\s*\)",
    "println!",
    "print!",
    "dbg!"
)

# Scanner les fichiers Rust
$rustFiles = Get-ChildItem -Path $Path -Recurse -Include "*.rs" | Where-Object {
    $_.FullName -notlike "*target*" -and 
    $_.FullName -notlike "*.git*"
}

Write-Host "Scanning $($rustFiles.Count) Rust files..." -ForegroundColor Yellow
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
        foreach ($pattern in $patterns) {
            if ($line -match $pattern) {
                $issue = @{
                    File = $relativePath
                    Line = $lineNumber
                    Pattern = $pattern
                    Content = $line.Trim()
                }
                $issuesFound += $issue
                $totalIssues++
                
                if ($Verbose) {
                    Write-Host "❌ Security theatre detected" -ForegroundColor Red
                    Write-Host "   ${relativePath}:${lineNumber}" -ForegroundColor Gray
                    Write-Host "   $($line.Trim())" -ForegroundColor Gray
                    Write-Host ""
                }
            }
        }
    }
}

# Afficher le rapport
Write-Host "Security Theatre Report" -ForegroundColor Cyan
Write-Host "=========================" -ForegroundColor Cyan
Write-Host ""

if ($totalIssues -eq 0) {
    Write-Host "✅ No security theatre detected!" -ForegroundColor Green
    Write-Host ""
    exit 0
}

Write-Host "❌ Security theatre detected: $totalIssues issues" -ForegroundColor Red
Write-Host ""

# Top 10 des issues les plus critiques
Write-Host "Top Issues:" -ForegroundColor Red
$topIssues = $issuesFound | Select-Object -First 10
foreach ($issue in $topIssues) {
    Write-Host "  $($issue.File):$($issue.Line)" -ForegroundColor Red
    Write-Host "    $($issue.Content)" -ForegroundColor Gray
}
Write-Host ""

# Recommandations
Write-Host "Recommendations:" -ForegroundColor Yellow
Write-Host "  1. Replace .unwrap() with proper error handling" -ForegroundColor White
Write-Host "  2. Remove placeholder comments and implement real code" -ForegroundColor White
Write-Host "  3. Replace assumptions with validated logic" -ForegroundColor White
Write-Host "  4. Use constants instead of magic numbers" -ForegroundColor White
Write-Host "  5. Remove hardcoded credentials" -ForegroundColor White
Write-Host ""

Write-Host "❌ COMMIT BLOCKED - Fix security theatre issues first" -ForegroundColor Red
exit 1
