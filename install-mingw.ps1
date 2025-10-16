# Script pour installer MinGW et configurer l'environnement Rust
Write-Host "Installation de MinGW pour Rust..." -ForegroundColor Green

# Télécharger et installer MinGW
$mingwUrl = "https://github.com/niXman/mingw-builds-binaries/releases/download/13.2.0-rt_v11-rev1/winlibs-x86_64-posix-seh-gcc-13.2.0-mingw-w64-11.0.0-r1.zip"
$mingwZip = "mingw.zip"
$mingwDir = "C:\mingw64"

Write-Host "Téléchargement de MinGW..." -ForegroundColor Yellow
Invoke-WebRequest -Uri $mingwUrl -OutFile $mingwZip

Write-Host "Extraction de MinGW..." -ForegroundColor Yellow
Expand-Archive -Path $mingwZip -DestinationPath "C:\" -Force

Write-Host "Configuration de l'environnement..." -ForegroundColor Yellow
$env:PATH = "$mingwDir\bin;$env:PATH"

# Ajouter MinGW au PATH permanent
$currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($currentPath -notlike "*$mingwDir\bin*") {
    [Environment]::SetEnvironmentVariable("PATH", "$currentPath;$mingwDir\bin", "User")
}

Write-Host "Vérification de l'installation..." -ForegroundColor Yellow
& "$mingwDir\bin\gcc.exe" --version

Write-Host "MinGW installé avec succès !" -ForegroundColor Green
Write-Host "Redémarrez votre terminal pour que les changements prennent effet." -ForegroundColor Cyan
