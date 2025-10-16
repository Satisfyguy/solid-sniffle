# scripts/security-dashboard.ps1
# Dashboard de sécurité complet pour Monero Marketplace

param(
    [switch]$Live = $false,
    [int]$RefreshInterval = 30,
    [switch]$Export = $false,
    [string]$OutputPath = "docs/security-reports"
)

# Configuration des couleurs
$Colors = @{
    Critical = "Red"
    High = "Yellow" 
    Medium = "Cyan"
    Low = "Green"
    Info = "White"
    Success = "Green"
    Warning = "Yellow"
    Error = "Red"
}

# Fonction pour afficher le header
function Show-Header {
    Clear-Host
    Write-Host "🔒 MONERO MARKETPLACE - SECURITY DASHBOARD" -ForegroundColor $Colors.Critical
    Write-Host "=============================================" -ForegroundColor $Colors.Critical
    Write-Host "Timestamp: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')" -ForegroundColor $Colors.Info
    Write-Host ""
}

# Fonction pour calculer le score de sécurité
function Get-SecurityScore {
    $score = 100
    $issues = @()
    
    # Vérifier les unwraps
    $unwraps = (Select-String -Path "*/src/*.rs" -Pattern "\.unwrap\s*\(\s*\)" -ErrorAction SilentlyContinue | Measure-Object).Count
    if ($unwraps -gt 0) {
        $score -= 20
        $issues += "Unwraps: $unwraps"
    }
    
    # Vérifier les TODOs
    $todos = (Select-String -Path "*/src/*.rs" -Pattern "TODO|FIXME" -ErrorAction SilentlyContinue | Measure-Object).Count
    if ($todos -gt 5) {
        $score -= 10
        $issues += "TODOs: $todos"
    }
    
    # Vérifier les fonctions sans specs
    $functions = (Select-String -Path "*/src/*.rs" -Pattern "fn\s+\w+\s*\(" -ErrorAction SilentlyContinue | Measure-Object).Count
    $specs = (Get-ChildItem "docs/specs/*.md" -ErrorAction SilentlyContinue | Measure-Object).Count
    if ($functions -gt $specs) {
        $score -= 15
        $issues += "Functions sans spec: $($functions - $specs)"
    }
    
    # Vérifier les tests
    $testFiles = (Get-ChildItem "*/tests/*.rs" -ErrorAction SilentlyContinue | Measure-Object).Count
    if ($testFiles -lt 3) {
        $score -= 10
        $issues += "Tests insuffisants: $testFiles"
    }
    
    # Vérifier les reality checks
    $realityChecks = (Get-ChildItem "docs/reality-checks/*.md" -ErrorAction SilentlyContinue | Measure-Object).Count
    if ($realityChecks -lt 2) {
        $score -= 5
        $issues += "Reality checks manquants: $realityChecks"
    }
    
    return @{
        Score = [Math]::Max(0, $score)
        Issues = $issues
        Level = if ($score -ge 90) { "Excellent" } elseif ($score -ge 70) { "Bon" } elseif ($score -ge 50) { "Moyen" } else { "Critique" }
    }
}

# Fonction pour vérifier l'état de Tor
function Get-TorStatus {
    try {
        $response = Invoke-RestMethod -Uri "https://check.torproject.org/api/ip" -Proxy "http://127.0.0.1:9050" -TimeoutSec 5
        return @{
            Connected = $response.IsTor
            IP = $response.IP
            Country = $response.Country
            Status = if ($response.IsTor) { "Connected" } else { "Not Connected" }
        }
    } catch {
        return @{
            Connected = $false
            IP = "Unknown"
            Country = "Unknown"
            Status = "Error: $($_.Exception.Message)"
        }
    }
}

# Fonction pour vérifier l'état de Monero RPC
function Get-MoneroRpcStatus {
    try {
        $response = Invoke-RestMethod -Uri "http://127.0.0.1:18082/json_rpc" -Method Post -ContentType "application/json" -Body '{"jsonrpc":"2.0","id":"0","method":"get_version"}' -TimeoutSec 5
        return @{
            Connected = $true
            Version = $response.result.version
            Status = "Connected"
        }
    } catch {
        return @{
            Connected = $false
            Version = "Unknown"
            Status = "Error: $($_.Exception.Message)"
        }
    }
}

# Fonction pour analyser les métriques de sécurité
function Get-SecurityMetrics {
    $metrics = @{}
    
    # Lignes de code
    $rustFiles = Get-ChildItem -Path "." -Recurse -Include "*.rs" | Where-Object { $_.FullName -notlike "*target*" }
    $totalLines = 0
    foreach ($file in $rustFiles) {
        $lines = Get-Content $file.FullName -ErrorAction SilentlyContinue
        if ($lines) { $totalLines += $lines.Count }
    }
    $metrics.LinesOfCode = $totalLines
    
    # Fonctions
    $metrics.Functions = (Select-String -Path "*/src/*.rs" -Pattern "fn\s+\w+\s*\(" -ErrorAction SilentlyContinue | Measure-Object).Count
    
    # Tests
    $metrics.Tests = (Get-ChildItem "*/tests/*.rs" -ErrorAction SilentlyContinue | Measure-Object).Count
    
    # Specs
    $metrics.Specs = (Get-ChildItem "docs/specs/*.md" -ErrorAction SilentlyContinue | Measure-Object).Count
    
    # Reality Checks
    $metrics.RealityChecks = (Get-ChildItem "docs/reality-checks/*.md" -ErrorAction SilentlyContinue | Measure-Object).Count
    
    # Security Theatre Issues
    if (Test-Path "scripts/check-security-theatre-simple.ps1") {
        $stOutput = & "scripts/check-security-theatre-simple.ps1" 2>&1
        $metrics.SecurityTheatreIssues = if ($LASTEXITCODE -eq 0) { 0 } else { 1 }
    } else {
        $metrics.SecurityTheatreIssues = -1
    }
    
    return $metrics
}

# Fonction pour afficher les métriques
function Show-Metrics {
    param($Metrics)
    
    Write-Host "📊 CODE METRICS" -ForegroundColor $Colors.Info
    Write-Host "===============" -ForegroundColor $Colors.Info
    Write-Host "  Lines of Code: $($Metrics.LinesOfCode)" -ForegroundColor $Colors.Info
    Write-Host "  Functions: $($Metrics.Functions)" -ForegroundColor $Colors.Info
    Write-Host "  Tests: $($Metrics.Tests)" -ForegroundColor $Colors.Info
    Write-Host "  Specs: $($Metrics.Specs)" -ForegroundColor $Colors.Info
    Write-Host "  Reality Checks: $($Metrics.RealityChecks)" -ForegroundColor $Colors.Info
    Write-Host ""
}

# Fonction pour afficher le score de sécurité
function Show-SecurityScore {
    param($ScoreData)
    
    $color = switch ($ScoreData.Level) {
        "Excellent" { $Colors.Success }
        "Bon" { $Colors.Info }
        "Moyen" { $Colors.Warning }
        "Critique" { $Colors.Error }
    }
    
    Write-Host "🛡️ SECURITY SCORE" -ForegroundColor $Colors.Info
    Write-Host "=================" -ForegroundColor $Colors.Info
    Write-Host "  Score: $($ScoreData.Score)/100" -ForegroundColor $color
    Write-Host "  Level: $($ScoreData.Level)" -ForegroundColor $color
    
    if ($ScoreData.Issues.Count -gt 0) {
        Write-Host "  Issues:" -ForegroundColor $Colors.Warning
        foreach ($issue in $ScoreData.Issues) {
            Write-Host "    - $issue" -ForegroundColor $Colors.Warning
        }
    } else {
        Write-Host "  ✅ No issues detected" -ForegroundColor $Colors.Success
    }
    Write-Host ""
}

# Fonction pour afficher l'état de Tor
function Show-TorStatus {
    param($TorData)
    
    $color = if ($TorData.Connected) { $Colors.Success } else { $Colors.Error }
    
    Write-Host "🧅 TOR STATUS" -ForegroundColor $Colors.Info
    Write-Host "=============" -ForegroundColor $Colors.Info
    Write-Host "  Status: $($TorData.Status)" -ForegroundColor $color
    if ($TorData.Connected) {
        Write-Host "  IP: $($TorData.IP)" -ForegroundColor $Colors.Info
        Write-Host "  Country: $($TorData.Country)" -ForegroundColor $Colors.Info
    }
    Write-Host ""
}

# Fonction pour afficher l'état de Monero RPC
function Show-MoneroRpcStatus {
    param($RpcData)
    
    $color = if ($RpcData.Connected) { $Colors.Success } else { $Colors.Error }
    
    Write-Host "💰 MONERO RPC STATUS" -ForegroundColor $Colors.Info
    Write-Host "====================" -ForegroundColor $Colors.Info
    Write-Host "  Status: $($RpcData.Status)" -ForegroundColor $color
    if ($RpcData.Connected) {
        Write-Host "  Version: $($RpcData.Version)" -ForegroundColor $Colors.Info
    }
    Write-Host ""
}

# Fonction pour afficher les alertes
function Show-Alerts {
    param($ScoreData, $TorData, $RpcData)
    
    $alerts = @()
    
    if ($ScoreData.Score -lt 70) {
        $alerts += "🚨 Security score is below 70%"
    }
    
    if (-not $TorData.Connected) {
        $alerts += "🧅 Tor is not connected"
    }
    
    if (-not $RpcData.Connected) {
        $alerts += "💰 Monero RPC is not accessible"
    }
    
    if ($ScoreData.Issues.Count -gt 3) {
        $alerts += "⚠️ Multiple security issues detected"
    }
    
    if ($alerts.Count -gt 0) {
        Write-Host "🚨 ALERTS" -ForegroundColor $Colors.Error
        Write-Host "=========" -ForegroundColor $Colors.Error
        foreach ($alert in $alerts) {
            Write-Host "  $alert" -ForegroundColor $Colors.Error
        }
        Write-Host ""
    }
}

# Fonction pour exporter le rapport
function Export-SecurityReport {
    param($Metrics, $ScoreData, $TorData, $RpcData, $OutputPath)
    
    if (-not (Test-Path $OutputPath)) {
        New-Item -ItemType Directory -Path $OutputPath -Force | Out-Null
    }
    
    $timestamp = Get-Date -Format "yyyy-MM-dd_HH-mm-ss"
    $reportFile = Join-Path $OutputPath "security-report-$timestamp.json"
    
    $report = @{
        Timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
        Metrics = $Metrics
        SecurityScore = $ScoreData
        TorStatus = $TorData
        MoneroRpcStatus = $RpcData
        Alerts = @()
    }
    
    # Ajouter les alertes
    if ($ScoreData.Score -lt 70) { $report.Alerts += "Security score below 70%" }
    if (-not $TorData.Connected) { $report.Alerts += "Tor not connected" }
    if (-not $RpcData.Connected) { $report.Alerts += "Monero RPC not accessible" }
    
    $report | ConvertTo-Json -Depth 3 | Set-Content $reportFile
    Write-Host "📄 Security report exported to: $reportFile" -ForegroundColor $Colors.Success
}

# Fonction principale
function Show-SecurityDashboard {
    Show-Header
    
    # Collecter les données
    $metrics = Get-SecurityMetrics
    $scoreData = Get-SecurityScore
    $torData = Get-TorStatus
    $rpcData = Get-MoneroRpcStatus
    
    # Afficher les sections
    Show-Metrics $metrics
    Show-SecurityScore $scoreData
    Show-TorStatus $torData
    Show-MoneroRpcStatus $rpcData
    Show-Alerts $scoreData $torData $rpcData
    
    # Exporter si demandé
    if ($Export) {
        Export-SecurityReport $metrics $scoreData $torData $rpcData $OutputPath
    }
    
    # Footer
    Write-Host "=============================================" -ForegroundColor $Colors.Info
    Write-Host "Press Ctrl+C to exit" -ForegroundColor $Colors.Info
}

# Mode live avec refresh automatique
if ($Live) {
    while ($true) {
        Show-SecurityDashboard
        Start-Sleep -Seconds $RefreshInterval
    }
} else {
    Show-SecurityDashboard
}
