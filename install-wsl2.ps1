# install-wsl2.ps1
# Quick WSL2 + Ubuntu installation script for Monero Marketplace
# Run in PowerShell as Administrator

#Requires -RunAsAdministrator

$ErrorActionPreference = "Stop"

# Colors
function Write-Success { Write-Host $args -ForegroundColor Green }
function Write-Info { Write-Host $args -ForegroundColor Cyan }
function Write-Warning { Write-Host $args -ForegroundColor Yellow }
function Write-Error { Write-Host $args -ForegroundColor Red }

Write-Info @"
╔═══════════════════════════════════════╗
║  Monero Marketplace WSL2 Installer    ║
║  PowerShell Setup Script              ║
╚═══════════════════════════════════════╝
"@

# Check if running as Administrator
$currentPrincipal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
if (-not $currentPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    Write-Error "❌ This script must be run as Administrator!"
    Write-Warning "Right-click PowerShell and select 'Run as Administrator'"
    exit 1
}

Write-Success "✅ Running as Administrator"

# Step 1: Check if WSL is already installed
Write-Info "`n═══ Step 1: Checking WSL status ═══`n"

$wslStatus = wsl --status 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Warning "WSL is already installed"
    wsl --status

    $install = Read-Host "`nDo you want to install Ubuntu anyway? (y/N)"
    if ($install -ne "y") {
        Write-Info "Skipping WSL installation. Run 'wsl --install -d Ubuntu-22.04' manually if needed."
        exit 0
    }
} else {
    Write-Info "WSL not found. Installing..."
}

# Step 2: Enable WSL features (if needed)
Write-Info "`n═══ Step 2: Enabling Windows features ═══`n"

try {
    # Enable Virtual Machine Platform
    Write-Info "Enabling Virtual Machine Platform..."
    dism.exe /online /enable-feature /featurename:VirtualMachinePlatform /all /norestart

    # Enable WSL
    Write-Info "Enabling Windows Subsystem for Linux..."
    dism.exe /online /enable-feature /featurename:Microsoft-Windows-Subsystem-Linux /all /norestart

    Write-Success "✅ Windows features enabled"
} catch {
    Write-Warning "⚠️  Features may already be enabled"
}

# Step 3: Install Ubuntu
Write-Info "`n═══ Step 3: Installing Ubuntu 22.04 ═══`n"

try {
    wsl --install -d Ubuntu-22.04

    if ($LASTEXITCODE -eq 0) {
        Write-Success "✅ Ubuntu 22.04 installation initiated"
    } else {
        Write-Warning "⚠️  Installation may require restart or manual intervention"
    }
} catch {
    Write-Error "❌ Failed to install Ubuntu"
    Write-Info "Try manually: wsl --install -d Ubuntu-22.04"
    exit 1
}

# Step 4: Set WSL 2 as default (if WSL is installed)
Write-Info "`n═══ Step 4: Setting WSL 2 as default ═══`n"

try {
    wsl --set-default-version 2
    Write-Success "✅ WSL 2 set as default"
} catch {
    Write-Warning "⚠️  Could not set default version (may require restart)"
}

# Step 5: Next steps
Write-Info "`n═══════════════════════════════════════"
Write-Info "  Installation Complete!"
Write-Info "═══════════════════════════════════════`n"

Write-Warning "⚠️  RESTART REQUIRED ⚠️"
Write-Info "`nAfter restarting Windows:`n"

Write-Success "1. Launch Ubuntu from Start Menu"
Write-Success "2. Create your username and password"
Write-Success "3. Run the setup script:`n"
Write-Info "   cd /mnt/c/Users/Lenovo/monero-marketplace"
Write-Info "   cp -r . ~/monero-marketplace"
Write-Info "   cd ~/monero-marketplace"
Write-Info "   chmod +x scripts/ubuntu-setup.sh"
Write-Info "   ./scripts/ubuntu-setup.sh`n"

Write-Success "4. Read the migration guide:"
Write-Info "   cat MIGRATION-WSL2.md`n"

# Prompt for restart
$restart = Read-Host "`nDo you want to restart now? (y/N)"
if ($restart -eq "y") {
    Write-Info "Restarting in 10 seconds..."
    Start-Sleep -Seconds 10
    Restart-Computer -Force
} else {
    Write-Warning "`nPlease restart your computer manually to complete WSL2 installation."
    Write-Info "`nPress any key to exit..."
    $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
}
