# scripts/security-dashboard-basic.ps1
# Dashboard de sécurité basique

Clear-Host
Write-Host "MONERO MARKETPLACE - SECURITY DASHBOARD" -ForegroundColor Red
Write-Host "========================================" -ForegroundColor Red
Write-Host ""

# Security Theatre Check
Write-Host "Security Theatre Check:" -ForegroundColor Yellow
if (Test-Path "scripts/check-security-theatre-simple.ps1") {
    & "scripts/check-security-theatre-simple.ps1" 2>&1 | Out-Null
    if ($LASTEXITCODE -eq 0) {
        Write-Host "  No security theatre detected" -ForegroundColor Green
    } else {
        Write-Host "  Security theatre detected!" -ForegroundColor Red
    }
} else {
    Write-Host "  Security theatre check script not found" -ForegroundColor Yellow
}
Write-Host ""

# Code Metrics
Write-Host "Code Metrics:" -ForegroundColor Cyan
$rustFiles = Get-ChildItem -Path "." -Recurse -Include "*.rs" | Where-Object { $_.FullName -notlike "*target*" }
$totalLines = 0
foreach ($file in $rustFiles) {
    $lines = Get-Content $file.FullName -ErrorAction SilentlyContinue
    if ($lines) { $totalLines += $lines.Count }
}
Write-Host "  Lines of Code: $totalLines" -ForegroundColor White

$functions = (Select-String -Path "*/src/*.rs" -Pattern "fn\s+\w+\s*\(" -ErrorAction SilentlyContinue | Measure-Object).Count
Write-Host "  Functions: $functions" -ForegroundColor White

$tests = (Get-ChildItem "*/tests/*.rs" -ErrorAction SilentlyContinue | Measure-Object).Count
Write-Host "  Tests: $tests" -ForegroundColor White

$specs = (Get-ChildItem "docs/specs/*.md" -ErrorAction SilentlyContinue | Measure-Object).Count
Write-Host "  Specs: $specs" -ForegroundColor White
Write-Host ""

# Security Score
Write-Host "Security Score:" -ForegroundColor Magenta
$score = 100
$issues = @()

$unwraps = (Select-String -Path "*/src/*.rs" -Pattern "\.unwrap\s*\(\s*\)" -ErrorAction SilentlyContinue | Measure-Object).Count
if ($unwraps -gt 0) {
    $score -= 20
    $issues += "Unwraps: $unwraps"
}

$todos = (Select-String -Path "*/src/*.rs" -Pattern "TODO|FIXME" -ErrorAction SilentlyContinue | Measure-Object).Count
if ($todos -gt 5) {
    $score -= 10
    $issues += "TODOs: $todos"
}

if ($functions -gt $specs) {
    $score -= 15
    $issues += "Functions sans spec: $($functions - $specs)"
}

$score = [Math]::Max(0, $score)
$color = if ($score -ge 90) { "Green" } elseif ($score -ge 70) { "Yellow" } else { "Red" }

Write-Host "  Score: $score/100" -ForegroundColor $color
Write-Host "  Level: $(if ($score -ge 90) { 'Excellent' } elseif ($score -ge 70) { 'Bon' } else { 'Critique' })" -ForegroundColor $color

if ($issues.Count -gt 0) {
    Write-Host "  Issues:" -ForegroundColor Yellow
    foreach ($issue in $issues) {
        Write-Host "    - $issue" -ForegroundColor Yellow
    }
} else {
    Write-Host "  No issues detected" -ForegroundColor Green
}
Write-Host ""

Write-Host "Dashboard completed" -ForegroundColor White
