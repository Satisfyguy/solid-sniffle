# scripts/validate-reality-check-tor.ps1
# Valide reality check Tor avant merge en production

param(
    [Parameter(Mandatory=$true)]
    [string]$FunctionName
)

$date = Get-Date -Format "yyyy-MM-dd"
$checkPath = "docs/reality-checks/tor-$FunctionName-$date.md"

if (-not (Test-Path $checkPath)) {
    Write-Host "Reality check Tor non trouve: $checkPath" -ForegroundColor Red
    exit 1
}

Write-Host "Validation Reality Check TOR: $FunctionName" -ForegroundColor Cyan
Write-Host ""

$content = Get-Content $checkPath -Raw

# Parse metadata JSON
if ($content -match '```json\s*({[\s\S]*?})\s*```') {
    $metadata = $matches[1] | ConvertFrom-Json
    $criticalIssues = $metadata.critical_issues
    $autoTestsPassed = $metadata.auto_tests_passed
} else {
    Write-Host "Metadata JSON non trouve" -ForegroundColor Yellow
    $criticalIssues = 999
    $autoTestsPassed = $false
}

# Checks obligatoires
$checks = @{
    "Tests auto passes" = $autoTestsPassed -eq $true
    "Issues critiques = 0" = $criticalIssues -eq 0
    "Test DNS leak complete" = $content -match "\[x\].*DNS via Tor uniquement"
    "Test fingerprint complete" = $content -match "\[x\].*Fingerprint anonyme"
    "Decision prise" = $content -match "\[x\].*\*\*(APPROUVE|CONDITIONNEL|REJETE)\*\*"
    "Signature presente" = $content -match "Teste par:.*\[([^\]]+)\]" -and $matches[1] -ne "Nom"
    "Checklist finale complete" = ($content -match "\[x\].*Tous les tests auto passent") -and 
                                    ($content -match "\[x\].*Tests manuels completes")
}

$allValid = $true
$blockersCount = 0

Write-Host "Verifications:" -ForegroundColor Cyan

foreach ($check in $checks.GetEnumerator()) {
    $symbol = if ($check.Value) { "OK" } else { "FAIL" }
    $color = if ($check.Value) { "Green" } else { "Red" }
    
    Write-Host "  $symbol $($check.Key)" -ForegroundColor $color
    
    if (-not $check.Value) {
        $allValid = $false
        
        # Certains checks sont bloquants
        if ($check.Key -match "Tests auto|Issues critiques|Decision") {
            $blockersCount++
        }
    }
}

Write-Host ""

# Checks specifiques Tor
Write-Host "Verifications Tor Specifiques:" -ForegroundColor Cyan

$torChecks = @{
    "IP leak test fait" = $content -match "IP via Tor:"
    "Port exposure verifie" = $content -match "Port Exposure Check:"
    "Logs audites" = $content -match "Audit automatique des logs:"
    "Aucune fuite .onion" = -not ($content -match "fuite.*\.onion")
    "RPC isole" = $content -match "RPC isolated on localhost" -or $content -match "\[x\].*RPC NOT exposed publicly"
}

foreach ($check in $torChecks.GetEnumerator()) {
    $symbol = if ($check.Value) { "OK" } else { "FAIL" }
    $color = if ($check.Value) { "Green" } else { "Yellow" }
    
    Write-Host "  $symbol $($check.Key)" -ForegroundColor $color
    
    if (-not $check.Value -and $check.Key -match "RPC isole|fuite") {
        $blockersCount++
    }
}

Write-Host ""

# Checks de decision
Write-Host "Verifications Decision:" -ForegroundColor Cyan

$decisionChecks = @{
    "Justification presente" = $content -match "Justification" -and $content -notmatch "\[Expliquer"
    "Actions requises" = if ($content -match "CONDITIONNEL|REJETE") { 
        $content -match "Actions Requises" -and $content -match "\[Action" 
    } else { $true }
}

foreach ($check in $decisionChecks.GetEnumerator()) {
    $symbol = if ($check.Value) { "OK" } else { "FAIL" }
    $color = if ($check.Value) { "Green" } else { "Red" }
    
    Write-Host "  $symbol $($check.Key)" -ForegroundColor $color
    
    if (-not $check.Value) {
        $blockersCount++
    }
}

Write-Host ""

# Resultat final
if ($blockersCount -eq 0 -and $allValid) {
    Write-Host "VALIDATION REUSSIE" -ForegroundColor Green
    Write-Host ""
    Write-Host "Reality Check Tor complet et valide." -ForegroundColor Green
    Write-Host "Issues critiques: $criticalIssues OK" -ForegroundColor Green
    Write-Host ""
    Write-Host "Pret pour merge en production Tor" -ForegroundColor Cyan
    
    # Marquer comme valide
    $content = $content -replace "Status:.*", "Status: [x] Valide pour production"
    Set-Content -Path $checkPath -Value $content
    
    exit 0
    
} elseif ($blockersCount -gt 0) {
    Write-Host "VALIDATION ECHOUE - $blockersCount BLOCKER(S)" -ForegroundColor Red
    Write-Host ""
    Write-Host "Issues critiques detectees: $criticalIssues" -ForegroundColor Red
    Write-Host ""
    Write-Host "NE PAS MERGER EN PRODUCTION" -ForegroundColor Red
    Write-Host "Fix les issues critiques dans: $checkPath" -ForegroundColor Yellow
    
    exit 1
    
} else {
    Write-Host "VALIDATION PARTIELLE" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Complete les sections manquantes avant merge." -ForegroundColor Yellow
    Write-Host "Fichier: $checkPath" -ForegroundColor Cyan
    
    exit 2
}