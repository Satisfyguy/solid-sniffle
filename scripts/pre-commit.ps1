# Script: pre-commit.ps1
# Verifications avant commit
# Usage: .\scripts\pre-commit.ps1

Write-Host "PRE-COMMIT CHECKS" -ForegroundColor Cyan
Write-Host "===================" -ForegroundColor Cyan

# Verifier que nous sommes dans le bon repertoire
if (-not (Test-Path ".cursorrules")) {
    Write-Host "ERREUR: Execute ce script depuis la racine du projet" -ForegroundColor Red
    exit 1
}

$errors = 0
$warnings = 0

# 1. Verifier que le projet compile
Write-Host "`n1. Verification compilation..." -ForegroundColor Yellow
try {
    $result = cargo check 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "   Projet compile correctement" -ForegroundColor Green
    } else {
        Write-Host "   Erreurs de compilation:" -ForegroundColor Red
        Write-Host $result -ForegroundColor Red
        $errors++
    }
} catch {
    Write-Host "   Erreur lors de la verification: $_" -ForegroundColor Red
    $errors++
}

# 2. Format du code
Write-Host "`n2. Verification format..." -ForegroundColor Yellow
try {
    $result = cargo fmt --check 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "   Code bien formate" -ForegroundColor Green
    } else {
        Write-Host "   Code mal formate, correction automatique..." -ForegroundColor Yellow
        cargo fmt
        if ($LASTEXITCODE -eq 0) {
            Write-Host "   Code reformate automatiquement" -ForegroundColor Green
        } else {
            Write-Host "   Erreur lors du formatage" -ForegroundColor Red
            $errors++
        }
    }
} catch {
    Write-Host "   Erreur lors de la verification du format: $_" -ForegroundColor Red
    $errors++
}

# 3. Clippy (linter)
Write-Host "`n3. Verification Clippy..." -ForegroundColor Yellow
try {
    $result = cargo clippy -- -D warnings 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "   Aucun warning Clippy" -ForegroundColor Green
    } else {
        Write-Host "   Warnings Clippy detectes:" -ForegroundColor Yellow
        Write-Host $result -ForegroundColor Yellow
        $warnings++
    }
} catch {
    Write-Host "   Erreur lors de la verification Clippy: $_" -ForegroundColor Red
    $errors++
}

# 4. Tests
Write-Host "`n4. Execution des tests..." -ForegroundColor Yellow
try {
    $result = cargo test 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "   Tous les tests passent" -ForegroundColor Green
    } else {
        Write-Host "   Tests echoues:" -ForegroundColor Red
        Write-Host $result -ForegroundColor Red
        $errors++
    }
} catch {
    Write-Host "   Erreur lors de l'execution des tests: $_" -ForegroundColor Red
    $errors++
}

# 5. Verifier que les specs existent
Write-Host "`n5. Verification des specs..." -ForegroundColor Yellow
$rustFiles = Get-ChildItem -Recurse -Include "*.rs" | Where-Object { $_.FullName -notlike "*target*" }
$functionCount = 0
foreach ($file in $rustFiles) {
    $content = Get-Content $file.FullName -Raw
    $matches = [regex]::Matches($content, "pub\s+(async\s+)?fn\s+\w+")
    $functionCount += $matches.Count
}

$specFiles = Get-ChildItem -Path "docs\specs" -Include "*.md" -ErrorAction SilentlyContinue
$specCount = if ($specFiles) { $specFiles.Count } else { 0 }

if ($specCount -ge $functionCount) {
    Write-Host "   Toutes les fonctions ont une spec" -ForegroundColor Green
} else {
    Write-Host "   $($functionCount - $specCount) fonction(s) sans spec" -ForegroundColor Yellow
    $warnings++
}

# 6. Verification des unwraps
Write-Host "`n6. Verification des unwraps..." -ForegroundColor Yellow
$unwrapCount = 0
foreach ($file in $rustFiles) {
    $content = Get-Content $file.FullName -Raw
    $matches = [regex]::Matches($content, "\.unwrap\(")
    $unwrapCount += $matches.Count
}

if ($unwrapCount -eq 0) {
    Write-Host "   Aucun unwrap() trouve" -ForegroundColor Green
} elseif ($unwrapCount -le 5) {
    Write-Host "   $unwrapCount unwrap() trouve(s) (seuil: 5)" -ForegroundColor Yellow
    $warnings++
} else {
    Write-Host "   $unwrapCount unwrap() trouve(s) (seuil: 5)" -ForegroundColor Red
    $errors++
}

# 7. Verification des TODOs
Write-Host "`n7. Verification des TODOs..." -ForegroundColor Yellow
$todoCount = 0
foreach ($file in $rustFiles) {
    $content = Get-Content $file.FullName -Raw
    $matches = [regex]::Matches($content, "TODO|FIXME", [System.Text.RegularExpressions.RegexOptions]::IgnoreCase)
    $todoCount += $matches.Count
}

if ($todoCount -eq 0) {
    Write-Host "   Aucun TODO trouve" -ForegroundColor Green
} elseif ($todoCount -le 10) {
    Write-Host "   $todoCount TODO trouve(s) (seuil: 10)" -ForegroundColor Yellow
    $warnings++
} else {
    Write-Host "   $todoCount TODO trouve(s) (seuil: 10)" -ForegroundColor Red
    $errors++
}

# 8. Check Security Theatre
Write-Host "`n8. Checking for security theatre..." -ForegroundColor Yellow
try {
    & ".\scripts\check-security-theatre-simple.ps1"
    if ($LASTEXITCODE -eq 0) {
        Write-Host "   No security theatre detected" -ForegroundColor Green
    } else {
        Write-Host "   Security theatre detected!" -ForegroundColor Red
        $errors++
    }
} catch {
    Write-Host "   Error during security theatre check: $_" -ForegroundColor Red
    $errors++
}

# 9. Check Monero/Tor Security
Write-Host "`n9. Checking Monero/Tor security..." -ForegroundColor Yellow
try {
    & ".\scripts\check-monero-tor-final.ps1"
    if ($LASTEXITCODE -eq 0) {
        Write-Host "   No Monero/Tor security issues detected" -ForegroundColor Green
    } else {
        Write-Host "   Monero/Tor security issues detected!" -ForegroundColor Red
        $errors++
    }
} catch {
    Write-Host "   Error during Monero/Tor security check: $_" -ForegroundColor Red
    $errors++
}

# 10. Mise a jour des metriques
Write-Host "`n10. Mise a jour des metriques..." -ForegroundColor Yellow
try {
    & ".\scripts\update-metrics.ps1"
    if ($LASTEXITCODE -eq 0) {
        Write-Host "   Metriques mises a jour" -ForegroundColor Green
    } else {
        Write-Host "   Erreur lors de la mise a jour des metriques" -ForegroundColor Yellow
        $warnings++
    }
} catch {
    Write-Host "   Erreur lors de la mise a jour des metriques: $_" -ForegroundColor Yellow
    $warnings++
}

# Resume final
Write-Host "`nRESUME PRE-COMMIT" -ForegroundColor Cyan
Write-Host "===================" -ForegroundColor Cyan

if ($errors -eq 0 -and $warnings -eq 0) {
    Write-Host "TOUS LES CHECKS PASSENT - Pret pour le commit!" -ForegroundColor Green
    exit 0
} elseif ($errors -eq 0) {
    Write-Host "$warnings warning(s) detecte(s) - Commit possible mais attention" -ForegroundColor Yellow
    Write-Host "Considerez corriger les warnings avant de commiter" -ForegroundColor Cyan
    exit 0
} else {
    Write-Host "$errors erreur(s) detectee(s) - COMMIT BLOQUE" -ForegroundColor Red
    Write-Host "Corrigez les erreurs avant de pouvoir commiter" -ForegroundColor Yellow
    exit 1
}