# scripts/setup-ide.ps1
# Configuration IDE pour Monero Marketplace

param(
    [switch]$VSCode,
    [switch]$All
)

Write-Host "IDE Setup for Monero Marketplace" -ForegroundColor Cyan
Write-Host "=================================" -ForegroundColor Cyan
Write-Host ""

# Vérifier VS Code (activé par défaut)
if ($VSCode -or $All -or (-not $PSBoundParameters.ContainsKey('VSCode') -and -not $PSBoundParameters.ContainsKey('All'))) {
    Write-Host "Setting up VS Code..." -ForegroundColor Yellow
    
    if (Get-Command code -ErrorAction SilentlyContinue) {
        Write-Host "  Installing recommended extensions..." -ForegroundColor White
        
        # Extensions essentielles
        $extensions = @(
            "rust-lang.rust-analyzer",
            "vadimcn.vscode-lldb",
            "ms-vscode.powershell",
            "redhat.vscode-yaml"
        )
        
        foreach ($ext in $extensions) {
            Write-Host "    Installing $ext..." -ForegroundColor Gray
            code --install-extension $ext --force
        }
        
        Write-Host "  ✅ VS Code configured" -ForegroundColor Green
    }
    else {
        Write-Host "  ⚠️ VS Code not found in PATH" -ForegroundColor Yellow
    }
}

Write-Host ""
Write-Host "IDE configuration completed!" -ForegroundColor Green
Write-Host ""
Write-Host "Available tasks in VS Code:" -ForegroundColor White
Write-Host "  - Security Theatre Check (Ctrl+Shift+P > Tasks: Run Task)" -ForegroundColor Gray
Write-Host "  - Monero/Tor Patterns Check" -ForegroundColor Gray
Write-Host "  - Security Dashboard" -ForegroundColor Gray
Write-Host "  - Security Alerts" -ForegroundColor Gray
Write-Host "  - Pre-commit Check" -ForegroundColor Gray
