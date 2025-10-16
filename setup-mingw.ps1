# Script simple pour installer MinGW
Write-Host "Configuration de MinGW pour Rust..." -ForegroundColor Green

# Créer le répertoire MinGW
$mingwDir = "C:\mingw64"
if (-not (Test-Path $mingwDir)) {
    New-Item -ItemType Directory -Path $mingwDir -Force
}

# Télécharger MinGW depuis une source fiable
$mingwUrl = "https://github.com/brechtsanders/winlibs_mingw/releases/download/13.2.0-16.0.6-11.0.0-ucrt-r1/winlibs-x86_64-posix-seh-gcc-13.2.0-mingw-w64-11.0.0-r1.zip"
$mingwZip = "mingw-temp.zip"

Write-Host "Téléchargement de MinGW..." -ForegroundColor Yellow
try {
    Invoke-WebRequest -Uri $mingwUrl -OutFile $mingwZip -UseBasicParsing
    Write-Host "Téléchargement réussi !" -ForegroundColor Green
} catch {
    Write-Host "Erreur de téléchargement: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host "Tentative avec une URL alternative..." -ForegroundColor Yellow
    
    # URL alternative
    $mingwUrl2 = "https://sourceforge.net/projects/mingw-w64/files/Toolchains%20targetting%20Win64/Personal%20Builds/mingw-builds/8.1.0/threads-posix/seh/x86_64-8.1.0-release-posix-seh-rt_v6-rev0.7z"
    try {
        Invoke-WebRequest -Uri $mingwUrl2 -OutFile $mingwZip -UseBasicParsing
        Write-Host "Téléchargement alternatif réussi !" -ForegroundColor Green
    } catch {
        Write-Host "Échec du téléchargement alternatif: $($_.Exception.Message)" -ForegroundColor Red
        exit 1
    }
}

Write-Host "Extraction de MinGW..." -ForegroundColor Yellow
try {
    Expand-Archive -Path $mingwZip -DestinationPath "C:\" -Force
    Write-Host "Extraction réussie !" -ForegroundColor Green
} catch {
    Write-Host "Erreur d'extraction: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

# Nettoyer le fichier temporaire
Remove-Item $mingwZip -Force

# Vérifier l'installation
$gccPath = "$mingwDir\bin\gcc.exe"
if (Test-Path $gccPath) {
    Write-Host "MinGW installé avec succès !" -ForegroundColor Green
    & $gccPath --version
} else {
    Write-Host "Erreur: GCC non trouvé après installation" -ForegroundColor Red
    exit 1
}

Write-Host "Configuration terminée !" -ForegroundColor Green
