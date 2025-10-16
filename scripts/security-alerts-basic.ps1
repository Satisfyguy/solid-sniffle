# scripts/security-alerts-basic.ps1
# Alertes automatiques de sécurité basiques

param(
    [switch]$Test = $false
)

$alerts = @()

# 1. Security Theatre Check
if (Test-Path "scripts/check-security-theatre-simple.ps1") {
    & "scripts/check-security-theatre-simple.ps1" 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0) {
        $alerts += "Security theatre detected in codebase"
    }
}

# 2. Vérifier les unwraps
$unwraps = (Select-String -Path "*/src/*.rs" -Pattern "\.unwrap\s*\(\s*\)" -ErrorAction SilentlyContinue | Measure-Object).Count
if ($unwraps -gt 0) {
    $alerts += "$unwraps unwrap() found in production code"
}

# 3. Vérifier les TODOs
$todos = (Select-String -Path "*/src/*.rs" -Pattern "TODO|FIXME" -ErrorAction SilentlyContinue | Measure-Object).Count
if ($todos -gt 10) {
    $alerts += "$todos TODO/FIXME items need attention"
}

# 4. Vérifier les fonctions sans specs
$functions = (Select-String -Path "*/src/*.rs" -Pattern "fn\s+\w+\s*\(" -ErrorAction SilentlyContinue | Measure-Object).Count
$specs = (Get-ChildItem "docs/specs/*.md" -ErrorAction SilentlyContinue | Measure-Object).Count
if ($functions -gt $specs) {
    $alerts += "$($functions - $specs) functions without specs"
}

# Afficher les alertes
if ($alerts.Count -gt 0) {
    Write-Host "SECURITY ALERTS:" -ForegroundColor Red
    foreach ($alert in $alerts) {
        Write-Host "  - $alert" -ForegroundColor Red
    }
    Write-Host "Total alerts: $($alerts.Count)" -ForegroundColor Yellow
} else {
    Write-Host "No security alerts" -ForegroundColor Green
}
