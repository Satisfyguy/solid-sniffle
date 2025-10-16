# Script pour configurer Visual Studio après installation
Write-Host "Configuration de Visual Studio pour Rust..." -ForegroundColor Green

# Attendre que l'installation se termine
Write-Host "Vérification de l'installation Visual Studio..." -ForegroundColor Yellow
do {
    $vsInstaller = Get-Process -Name "vs_installer" -ErrorAction SilentlyContinue
    if ($vsInstaller) {
        Write-Host "Installation en cours..." -ForegroundColor Cyan
        Start-Sleep -Seconds 10
    }
} while ($vsInstaller)

Write-Host "Installation terminée !" -ForegroundColor Green

# Vérifier que Visual Studio est installé
$vsPath = "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"
if (Test-Path $vsPath) {
    Write-Host "Visual Studio trouvé !" -ForegroundColor Green
} else {
    Write-Host "Erreur: Visual Studio non trouvé" -ForegroundColor Red
    exit 1
}

# Charger les variables d'environnement
Write-Host "Chargement des variables d'environnement..." -ForegroundColor Yellow
& $vsPath

# Vérifier que le linker est disponible
Write-Host "Vérification du linker..." -ForegroundColor Yellow
$linker = Get-Command "link.exe" -ErrorAction SilentlyContinue
if ($linker) {
    Write-Host "Linker trouvé: $($linker.Source)" -ForegroundColor Green
} else {
    Write-Host "Erreur: Linker non trouvé" -ForegroundColor Red
    exit 1
}

# Vérifier les bibliothèques Windows
Write-Host "Vérification des bibliothèques Windows..." -ForegroundColor Yellow
$kernel32Path = "C:\Program Files (x86)\Windows Kits\10\Lib\10.0.22621.0\um\x64\kernel32.lib"
if (Test-Path $kernel32Path) {
    Write-Host "Bibliothèques Windows trouvées !" -ForegroundColor Green
} else {
    Write-Host "Avertissement: Bibliothèques Windows non trouvées" -ForegroundColor Yellow
}

# Configurer Cargo pour utiliser MSVC
Write-Host "Configuration de Cargo..." -ForegroundColor Yellow
$cargoConfig = @"
# Cargo configuration for Visual Studio MSVC
[build]
target = "x86_64-pc-windows-msvc"
"@

$cargoConfig | Out-File -FilePath ".cargo\config.toml" -Encoding UTF8
Write-Host "Configuration Cargo mise à jour !" -ForegroundColor Green

# Tester la compilation
Write-Host "Test de compilation..." -ForegroundColor Yellow
$result = cmd /c '"C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat" && cargo build --workspace' 2>&1

if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Compilation réussie !" -ForegroundColor Green
} else {
    Write-Host "❌ Compilation échouée" -ForegroundColor Red
    Write-Host $result
}

Write-Host "Configuration terminée !" -ForegroundColor Green
