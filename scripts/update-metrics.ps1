# Script: update-metrics.ps1
# Collecte et met a jour les metriques du projet
# Usage: .\scripts\update-metrics.ps1

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

# Date actuelle
$date = Get-Date -Format "yyyy-MM-dd"
$timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"

Write-Host "Collecte des metriques - $timestamp" -ForegroundColor Cyan

# 1. Lines of Code
$rustFiles = Get-ChildItem -Recurse -Include "*.rs" | Where-Object { $_.FullName -notlike "*target*" }
$totalLines = 0
foreach ($file in $rustFiles) {
    $lines = (Get-Content $file.FullName | Measure-Object -Line).Lines
    $totalLines += $lines
}

# 2. Nombre de fonctions (approximatif)
$functionCount = 0
foreach ($file in $rustFiles) {
    $content = Get-Content $file.FullName -Raw
    $matches = [regex]::Matches($content, "pub\s+(async\s+)?fn\s+\w+")
    $functionCount += $matches.Count
}

# 3. Nombre de specs
$specFiles = Get-ChildItem -Path "docs\specs" -Include "*.md" -ErrorAction SilentlyContinue
$specCount = if ($specFiles) { $specFiles.Count } else { 0 }

# 4. Nombre d'unwraps
$unwrapCount = 0
foreach ($file in $rustFiles) {
    $content = Get-Content $file.FullName -Raw
    $matches = [regex]::Matches($content, "\.unwrap\(")
    $unwrapCount += $matches.Count
}

# 5. Nombre de TODOs
$todoCount = 0
foreach ($file in $rustFiles) {
    $content = Get-Content $file.FullName -Raw
    $matches = [regex]::Matches($content, "TODO|FIXME", [System.Text.RegularExpressions.RegexOptions]::IgnoreCase)
    $todoCount += $matches.Count
}

# 6. Test coverage (approximatif)
$testFiles = Get-ChildItem -Recurse -Include "*test*.rs" | Where-Object { $_.FullName -notlike "*target*" }
$testCount = if ($testFiles) { $testFiles.Count } else { 0 }

# Creer l'objet metriques
$metrics = @{
    date = $date
    timestamp = $timestamp
    lines_of_code = $totalLines
    functions = $functionCount
    specs = $specCount
    unwraps = $unwrapCount
    todos = $todoCount
    test_files = $testCount
    coverage_estimate = if ($functionCount -gt 0) { [math]::Round(($testCount / $functionCount) * 100, 1) } else { 0 }
}

# Sauvegarder en JSON
$jsonFile = "$metricsDir\daily-$date.json"
$metrics | ConvertTo-Json -Depth 3 | Out-File -FilePath $jsonFile -Encoding UTF8

# Afficher les metriques
Write-Host "`nMETRIQUES COLLECTEES:" -ForegroundColor Green
Write-Host "  Lines of Code: $totalLines" -ForegroundColor White
Write-Host "  Functions: $functionCount" -ForegroundColor White
Write-Host "  Specs: $specCount" -ForegroundColor White
Write-Host "  Unwraps: $unwrapCount" -ForegroundColor $(if ($unwrapCount -gt 5) { "Red" } elseif ($unwrapCount -gt 0) { "Yellow" } else { "Green" })
Write-Host "  TODOs: $todoCount" -ForegroundColor $(if ($todoCount -gt 10) { "Red" } elseif ($todoCount -gt 0) { "Yellow" } else { "Green" })
Write-Host "  Test Files: $testCount" -ForegroundColor White
Write-Host "  Coverage Est.: $($metrics.coverage_estimate)%" -ForegroundColor White

# Verifier les seuils
$warnings = @()
$errors = @()

if ($totalLines -gt 5000) { $warnings += "LOC eleve (>5000)" }
if ($totalLines -gt 10000) { $errors += "LOC tres eleve (>10000)" }
if ($unwrapCount -gt 5) { $warnings += "Trop d'unwraps (>5)" }
if ($unwrapCount -gt 10) { $errors += "Beaucoup trop d'unwraps (>10)" }
if ($todoCount -gt 10) { $warnings += "Trop de TODOs (>10)" }
if ($todoCount -gt 20) { $errors += "Beaucoup trop de TODOs (>20)" }
if ($functionCount -gt 0 -and $specCount -lt $functionCount) { $warnings += "Fonctions sans spec" }

if ($warnings.Count -gt 0) {
    Write-Host "`nWARNINGS:" -ForegroundColor Yellow
    foreach ($warning in $warnings) {
        Write-Host "  - $warning" -ForegroundColor Yellow
    }
}

if ($errors.Count -gt 0) {
    Write-Host "`nERRORS:" -ForegroundColor Red
    foreach ($error in $errors) {
        Write-Host "  - $error" -ForegroundColor Red
    }
}

if ($warnings.Count -eq 0 -and $errors.Count -eq 0) {
    Write-Host "`nToutes les metriques sont dans les seuils acceptables!" -ForegroundColor Green
}

Write-Host "`nMetriques sauvegardees: $jsonFile" -ForegroundColor Cyan