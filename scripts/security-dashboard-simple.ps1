# scripts/security-dashboard-simple.ps1
# Dashboard de sécurité simplifié

Clear-Host
Write-Host "🔒 MONERO MARKETPLACE - SECURITY DASHBOARD" -ForegroundColor Red
Write-Host "=============================================" -ForegroundColor Red
Write-Host "Timestamp: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')" -ForegroundColor White
Write-Host ""

# 1. Security Theatre Check
Write-Host "🎭 SECURITY THEATRE CHECK" -ForegroundColor Yellow
Write-Host "=========================" -ForegroundColor Yellow
if (Test-Path "scripts/check-security-theatre-simple.ps1") {
    & "scripts/check-security-theatre-simple.ps1" 2>&1 | Out-Null
    if ($LASTEXITCODE -eq 0) {
        Write-Host "  ✅ No security theatre detected" -ForegroundColor Green
    } else {
        Write-Host "  ❌ Security theatre detected!" -ForegroundColor Red
    }
} else {
    Write-Host "  ⚠️ Security theatre check script not found" -ForegroundColor Yellow
}
Write-Host ""

# 2. Code Metrics
Write-Host "📊 CODE METRICS" -ForegroundColor Cyan
Write-Host "===============" -ForegroundColor Cyan

# Lines of Code
$rustFiles = Get-ChildItem -Path "." -Recurse -Include "*.rs" | Where-Object { $_.FullName -notlike "*target*" }
$totalLines = 0
foreach ($file in $rustFiles) {
    $lines = Get-Content $file.FullName -ErrorAction SilentlyContinue
    if ($lines) { $totalLines += $lines.Count }
}
Write-Host "  Lines of Code: $totalLines" -ForegroundColor White

# Functions
$functions = (Select-String -Path "*/src/*.rs" -Pattern "fn\s+\w+\s*\(" -ErrorAction SilentlyContinue | Measure-Object).Count
Write-Host "  Functions: $functions" -ForegroundColor White

# Tests
$tests = (Get-ChildItem "*/tests/*.rs" -ErrorAction SilentlyContinue | Measure-Object).Count
Write-Host "  Tests: $tests" -ForegroundColor White

# Specs
$specs = (Get-ChildItem "docs/specs/*.md" -ErrorAction SilentlyContinue | Measure-Object).Count
Write-Host "  Specs: $specs" -ForegroundColor White

# Reality Checks
$realityChecks = (Get-ChildItem "docs/reality-checks/*.md" -ErrorAction SilentlyContinue | Measure-Object).Count
Write-Host "  Reality Checks: $realityChecks" -ForegroundColor White
Write-Host ""

# 3. Security Score
Write-Host "🛡️ SECURITY SCORE" -ForegroundColor Magenta
Write-Host "=================" -ForegroundColor Magenta

$score = 100
$issues = @()

# Check unwraps
$unwraps = (Select-String -Path "*/src/*.rs" -Pattern "\.unwrap\s*\(\s*\)" -ErrorAction SilentlyContinue | Measure-Object).Count
if ($unwraps -gt 0) {
    $score -= 20
    $issues += "Unwraps: $unwraps"
}

# Check TODOs
$todos = (Select-String -Path "*/src/*.rs" -Pattern "TODO|FIXME" -ErrorAction SilentlyContinue | Measure-Object).Count
if ($todos -gt 5) {
    $score -= 10
    $issues += "TODOs: $todos"
}

# Check functions without specs
if ($functions -gt $specs) {
    $score -= 15
    $issues += "Functions sans spec: $($functions - $specs)"
}

# Check tests
if ($tests -lt 3) {
    $score -= 10
    $issues += "Tests insuffisants: $tests"
}

$score = [Math]::Max(0, $score)
$color = if ($score -ge 90) { "Green" } elseif ($score -ge 70) { "Yellow" } else { "Red" }

Write-Host "  Score: $score/100" -ForegroundColor $color
Write-Host "  Level: $(if ($score -ge 90) { 'Excellent' } elseif ($score -ge 70) { 'Bon' } elseif ($score -ge 50) { 'Moyen' } else { 'Critique' })" -ForegroundColor $color

if ($issues.Count -gt 0) {
    Write-Host "  Issues:" -ForegroundColor Yellow
    foreach ($issue in $issues) {
        Write-Host "    - $issue" -ForegroundColor Yellow
    }
} else {
    Write-Host "  ✅ No issues detected" -ForegroundColor Green
}
Write-Host ""

# 4. Tor Status
Write-Host "🧅 TOR STATUS" -ForegroundColor DarkGreen
Write-Host "=============" -ForegroundColor DarkGreen
try {
    $response = Invoke-RestMethod -Uri "https://check.torproject.org/api/ip" -Proxy "http://127.0.0.1:9050" -TimeoutSec 5
    if ($response.IsTor) {
        Write-Host "  ✅ Connected via Tor" -ForegroundColor Green
        Write-Host "  IP: $($response.IP)" -ForegroundColor White
        Write-Host "  Country: $($response.Country)" -ForegroundColor White
    } else {
        Write-Host "  ❌ Not connected via Tor" -ForegroundColor Red
    }
} catch {
    Write-Host "  ❌ Tor connection failed: $($_.Exception.Message)" -ForegroundColor Red
}
Write-Host ""

# 5. Monero RPC Status
Write-Host "💰 MONERO RPC STATUS" -ForegroundColor DarkCyan
Write-Host "====================" -ForegroundColor DarkCyan
try {
    $response = Invoke-RestMethod -Uri "http://127.0.0.1:18082/json_rpc" -Method Post -ContentType "application/json" -Body '{"jsonrpc":"2.0","id":"0","method":"get_version"}' -TimeoutSec 5
    Write-Host "  ✅ Monero RPC connected" -ForegroundColor Green
    Write-Host "  Version: $($response.result.version)" -ForegroundColor White
} catch {
    Write-Host "  ❌ Monero RPC failed: $($_.Exception.Message)" -ForegroundColor Red
}
Write-Host ""

# 6. Alerts
Write-Host "🚨 ALERTS" -ForegroundColor Red
Write-Host "=========" -ForegroundColor Red
$alerts = @()

if ($score -lt 70) {
    $alerts += "Security score is below 70%"
}

if ($unwraps -gt 0) {
    $alerts += "$unwraps unwrap() found in production code"
}

if ($functions -gt $specs) {
    $alerts += "$($functions - $specs) functions without specs"
}

if ($alerts.Count -gt 0) {
    foreach ($alert in $alerts) {
        Write-Host "  ⚠️ $alert" -ForegroundColor Red
    }
} else {
    Write-Host "  ✅ No alerts" -ForegroundColor Green
}
Write-Host ""

Write-Host "=============================================" -ForegroundColor White
Write-Host "Dashboard completed at $(Get-Date -Format 'HH:mm:ss')" -ForegroundColor White
