# scripts/security-alerts.ps1
# Alertes automatiques de sécurité

param(
    [string]$WebhookUrl = "",
    [string]$Email = "",
    [switch]$Test = $false
)

# Vérifier les conditions d'alerte
$alerts = @()

# 1. Security Theatre Check
if (Test-Path "scripts/check-security-theatre-simple.ps1") {
    & "scripts/check-security-theatre-simple.ps1" 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0) {
        $alerts += "🚨 Security theatre detected in codebase"
    }
}

# 2. Vérifier les unwraps
$unwraps = (Select-String -Path "*/src/*.rs" -Pattern "\.unwrap\s*\(\s*\)" -ErrorAction SilentlyContinue | Measure-Object).Count
if ($unwraps -gt 0) {
    $alerts += "⚠️ $unwraps unwrap() found in production code"
}

# 3. Vérifier les TODOs
$todos = (Select-String -Path "*/src/*.rs" -Pattern "TODO|FIXME" -ErrorAction SilentlyContinue | Measure-Object).Count
if ($todos -gt 10) {
    $alerts += "📝 $todos TODO/FIXME items need attention"
}

# 4. Vérifier les fonctions sans specs
$functions = (Select-String -Path "*/src/*.rs" -Pattern "fn\s+\w+\s*\(" -ErrorAction SilentlyContinue | Measure-Object).Count
$specs = (Get-ChildItem "docs/specs/*.md" -ErrorAction SilentlyContinue | Measure-Object).Count
if ($functions -gt $specs) {
    $alerts += "📋 $($functions - $specs) functions without specs"
}

# Envoyer les alertes
if ($alerts.Count -gt 0) {
    $message = "🔒 Monero Marketplace Security Alerts`n`n" + ($alerts -join "`n")
    
    if ($Test) {
        Write-Host $message -ForegroundColor Red
    }
    
    if ($WebhookUrl) {
        $body = @{ text = $message } | ConvertTo-Json
        Invoke-RestMethod -Uri $WebhookUrl -Method Post -Body $body -ContentType "application/json"
    }
    
    if ($Email) {
        Send-MailMessage -To $Email -Subject "Security Alert - Monero Marketplace" -Body $message -SmtpServer "localhost"
    }
    
    Write-Host "Sent $($alerts.Count) security alerts" -ForegroundColor Yellow
} else {
    Write-Host "✅ No security alerts" -ForegroundColor Green
}
