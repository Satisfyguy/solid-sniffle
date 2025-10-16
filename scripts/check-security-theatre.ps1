# scripts/check-security-theatre.ps1
# Détection automatique du security theatre dans le code Rust

param(
    [string]$Path = ".",
    [switch]$Verbose = $false,
    [string]$IgnoreFile = ".security-theatre-ignore"
)

Write-Host "Security Theatre Detection" -ForegroundColor Cyan
Write-Host "=============================" -ForegroundColor Cyan
Write-Host ""

# Compteurs d'issues
$totalIssues = 0
$issuesByCategory = @{
    "Asserts inutiles" = 0
    "Placeholders" = 0
    "Suppositions" = 0
    "Hypothèses non validées" = 0
    "Commentaires vagues" = 0
    "Code mort" = 0
    "Credentials hardcodés" = 0
    "Magic numbers" = 0
    "Patterns interdits" = 0
}

# Patterns de détection
$patterns = @{
    "Asserts inutiles" = @(
        "assert!.*true",
        "assert!.*false",
        "assert!.*1.*==.*1",
        "assert!.*0.*==.*0"
    )
    "Placeholders" = @(
        "//.*Placeholder",
        "//.*TODO",
        "//.*FIXME",
        "//.*XXX",
        "//.*HACK",
        "//.*TEMP",
        "//.*Temporary",
        "//.*FIX.*THIS",
        "//.*REMOVE.*THIS"
    )
    "Suppositions" = @(
        "should.*work",
        "might.*work",
        "probably.*works",
        "assume",
        "hope",
        "guess",
        "think.*it.*works",
        "believe.*it.*works"
    )
    "Hypothèses non validées" = @(
        "HYPOTHÈSES",
        "À.*VALIDER",
        "TO.*BE.*VALIDATED",
        "NEEDS.*VALIDATION",
        "ASSUMPTION",
        "HYPOTHESIS"
    )
    "Commentaires vagues" = @(
        "ERREUR.*POSSIBLE",
        "À.*IMPLÉMENTER",
        "TO.*BE.*IMPLEMENTED",
        "NOT.*IMPLEMENTED",
        "MISSING.*IMPLEMENTATION",
        "SOMETHING.*WRONG",
        "PROBLEM.*HERE"
    )
    "Code mort" = @(
        "unimplemented!",
        "todo!",
        "panic!",
        "unreachable!",
        "unreachable_unchecked!"
    )
    "Credentials hardcodés" = @(
        "password.*=",
        "secret.*=",
        "key.*=",
        "token.*=",
        "api_key.*=",
        "private_key.*="
    )
    "Magic numbers" = @(
        "\b\d{4,}\b",
        "\b0x[0-9a-fA-F]{4,}\b"
    )
    "Patterns interdits" = @(
        "\.unwrap\s*\(\s*\)",
        "\.expect\s*\(\s*\"\s*\)",
        "println!",
        "print!",
        "eprintln!",
        "eprint!",
        "dbg!"
    )
}

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
    param($filePath, $lineContent, $lineNumber)
    
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

# Scanner les fichiers Rust
$rustFiles = Get-ChildItem -Path $Path -Recurse -Include "*.rs" | Where-Object {
    $_.FullName -notlike "*target*" -and 
    $_.FullName -notlike "*.git*"
}

Write-Host "Scanning $($rustFiles.Count) Rust files..." -ForegroundColor Yellow
Write-Host ""

$issuesFound = @()

foreach ($file in $rustFiles) {
    $relativePath = $file.FullName.Replace((Get-Location).Path + "\", "").Replace("\", "/")
    $lines = Get-Content $file.FullName -ErrorAction SilentlyContinue
    
    if (-not $lines) { continue }
    
    for ($i = 0; $i -lt $lines.Count; $i++) {
        $lineNumber = $i + 1
        $line = $lines[$i]
        
        # Vérifier les exceptions
        if (Test-Exception -filePath $relativePath -lineContent $line -lineNumber $lineNumber) {
            continue
        }
        
        # Tester chaque catégorie de patterns
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
                    $issuesByCategory[$category]++
                    $totalIssues++
                    
                    if ($Verbose) {
                        Write-Host "❌ $category" -ForegroundColor Red
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

# Rapport par catégorie
Write-Host "Issues by Category:" -ForegroundColor Yellow
foreach ($category in $issuesByCategory.Keys) {
    $count = $issuesByCategory[$category]
    if ($count -gt 0) {
        $color = if ($count -gt 5) { "Red" } elseif ($count -gt 2) { "Yellow" } else { "White" }
        Write-Host "  $category`: $count" -ForegroundColor $color
    }
}
Write-Host ""

# Top 10 des issues les plus critiques
Write-Host "Top Issues:" -ForegroundColor Red
$topIssues = $issuesFound | Sort-Object Category | Select-Object -First 10
foreach ($issue in $topIssues) {
    Write-Host "  $($issue.File):$($issue.Line) - $($issue.Category)" -ForegroundColor Red
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

# Instructions pour contourner temporairement
Write-Host "To temporarily bypass (with justification):" -ForegroundColor Cyan
Write-Host "  1. Add exception to $IgnoreFile" -ForegroundColor White
Write-Host "  2. Use format: path/pattern:regex_pattern" -ForegroundColor White
Write-Host "  3. Example: tests pattern" -ForegroundColor White
Write-Host ""

Write-Host "❌ COMMIT BLOCKED - Fix security theatre issues first" -ForegroundColor Red
exit 1