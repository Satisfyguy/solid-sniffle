# Script d'installation WSL - À exécuter en tant qu'administrateur
Write-Host "=== Installation de WSL ===" -ForegroundColor Green

# Vérifier les privilèges administrateur
if (-NOT ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator")) {
    Write-Host "ERREUR: Ce script doit être exécuté en tant qu'administrateur !" -ForegroundColor Red
    Write-Host "Clic droit sur PowerShell -> Exécuter en tant qu'administrateur" -ForegroundColor Yellow
    pause
    exit 1
}

Write-Host "Privilèges administrateur confirmés ✓" -ForegroundColor Green

# Activer les fonctionnalités Windows nécessaires
Write-Host "Activation des fonctionnalités Windows..." -ForegroundColor Yellow

try {
    Enable-WindowsOptionalFeature -Online -FeatureName Microsoft-Windows-Subsystem-Linux -NoRestart
    Write-Host "✓ Sous-système Windows pour Linux activé" -ForegroundColor Green
} catch {
    Write-Host "⚠ Erreur lors de l'activation du sous-système Linux: $($_.Exception.Message)" -ForegroundColor Yellow
}

try {
    Enable-WindowsOptionalFeature -Online -FeatureName VirtualMachinePlatform -NoRestart
    Write-Host "✓ Plateforme de machine virtuelle activée" -ForegroundColor Green
} catch {
    Write-Host "⚠ Erreur lors de l'activation de la plateforme VM: $($_.Exception.Message)" -ForegroundColor Yellow
}

# Installer WSL
Write-Host "Installation de WSL..." -ForegroundColor Yellow
try {
    wsl --install --no-distribution
    Write-Host "✓ WSL installé avec succès" -ForegroundColor Green
} catch {
    Write-Host "⚠ Erreur lors de l'installation WSL: $($_.Exception.Message)" -ForegroundColor Yellow
}

# Installer Ubuntu
Write-Host "Installation d'Ubuntu..." -ForegroundColor Yellow
try {
    wsl --install -d Ubuntu
    Write-Host "✓ Ubuntu installé" -ForegroundColor Green
} catch {
    Write-Host "⚠ Erreur lors de l'installation Ubuntu: $($_.Exception.Message)" -ForegroundColor Yellow
}

Write-Host "=== Installation terminée ===" -ForegroundColor Green
Write-Host "REDÉMARRAGE REQUIS !" -ForegroundColor Red
Write-Host "Après le redémarrage :" -ForegroundColor Cyan
Write-Host "1. Ouvrez WSL (tapez 'wsl' dans le menu Démarrer)" -ForegroundColor White
Write-Host "2. Créez un utilisateur Ubuntu" -ForegroundColor White
Write-Host "3. Clonez votre projet : git clone https://github.com/Satisfyguy/solid-sniffle.git" -ForegroundColor White
Write-Host "4. Installez Rust : curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh" -ForegroundColor White
Write-Host "5. Compilez : cargo build --workspace" -ForegroundColor White

$restart = Read-Host "Voulez-vous redémarrer maintenant ? (y/n)"
if ($restart -eq "y" -or $restart -eq "Y") {
    Write-Host "Redémarrage en cours..." -ForegroundColor Yellow
    Restart-Computer
} else {
    Write-Host "N'oubliez pas de redémarrer avant d'utiliser WSL !" -ForegroundColor Yellow
}
