# scripts/validate-github-workflows.ps1
# Validation des workflows GitHub Actions

param(
    [switch]$Verbose = $false
)

Write-Host "GitHub Actions Workflow Validation" -ForegroundColor Cyan
Write-Host "====================================" -ForegroundColor Cyan
Write-Host ""

$workflowsDir = ".github/workflows"
$errors = 0
$warnings = 0

# Vérifier que le répertoire .github/workflows existe
if (-not (Test-Path $workflowsDir)) {
    Write-Host "❌ Directory $workflowsDir not found" -ForegroundColor Red
    $errors++
    exit 1
}

# Lister tous les fichiers YAML dans .github/workflows
$workflowFiles = Get-ChildItem -Path $workflowsDir -Filter "*.yml" -ErrorAction SilentlyContinue

if ($workflowFiles.Count -eq 0) {
    Write-Host "❌ No workflow files found in $workflowsDir" -ForegroundColor Red
    $errors++
    exit 1
}

Write-Host "Found $($workflowFiles.Count) workflow files:" -ForegroundColor Yellow
foreach ($file in $workflowFiles) {
    Write-Host "  - $($file.Name)" -ForegroundColor White
}
Write-Host ""

# Valider chaque workflow
foreach ($file in $workflowFiles) {
    Write-Host "Validating $($file.Name)..." -ForegroundColor Yellow
    
    try {
        $content = Get-Content $file.FullName -Raw
        
        # Vérifications de base
        $checks = @{
            "Has 'name' field" = $content -match "name:"
            "Has 'on' trigger" = $content -match "on:"
            "Has 'jobs' section" = $content -match "jobs:"
            "Uses checkout action" = $content -match "actions/checkout"
            "Uses Rust toolchain" = $content -match "rust-toolchain|actions-rs/toolchain"
        }
        
        foreach ($check in $checks.GetEnumerator()) {
            if ($check.Value) {
                Write-Host "  ✅ $($check.Key)" -ForegroundColor Green
            } else {
                Write-Host "  ❌ $($check.Key)" -ForegroundColor Red
                $errors++
            }
        }
        
        # Vérifications spécifiques au projet
        if ($file.Name -eq "ci.yml") {
            $ciChecks = @{
                "Has security theatre check" = $content -match "check-security-theatre"
                "Has cargo check" = $content -match "cargo check"
                "Has cargo clippy" = $content -match "cargo clippy"
                "Has cargo test" = $content -match "cargo test"
            }
            
            foreach ($check in $ciChecks.GetEnumerator()) {
                if ($check.Value) {
                    Write-Host "  ✅ $($check.Key)" -ForegroundColor Green
                } else {
                    Write-Host "  ⚠️ $($check.Key)" -ForegroundColor Yellow
                    $warnings++
                }
            }
        }
        
        if ($file.Name -eq "security-audit.yml") {
            $securityChecks = @{
                "Has cargo audit" = $content -match "cargo audit"
                "Has semgrep" = $content -match "semgrep"
                "Has security theatre check" = $content -match "check-security-theatre"
            }
            
            foreach ($check in $securityChecks.GetEnumerator()) {
                if ($check.Value) {
                    Write-Host "  ✅ $($check.Key)" -ForegroundColor Green
                } else {
                    Write-Host "  ⚠️ $($check.Key)" -ForegroundColor Yellow
                    $warnings++
                }
            }
        }
        
        if ($file.Name -eq "monero-integration.yml") {
            $moneroChecks = @{
                "Has Monero setup" = $content -match "monero|Monero"
                "Has testnet configuration" = $content -match "testnet"
                "Has RPC tests" = $content -match "rpc|RPC"
            }
            
            foreach ($check in $moneroChecks.GetEnumerator()) {
                if ($check.Value) {
                    Write-Host "  ✅ $($check.Key)" -ForegroundColor Green
                } else {
                    Write-Host "  ⚠️ $($check.Key)" -ForegroundColor Yellow
                    $warnings++
                }
            }
        }
        
        # Vérifier la syntaxe YAML basique
        if ($content -match "^\s*-\s*name:" -and $content -match "^\s*run:") {
            Write-Host "  ✅ Basic YAML structure looks correct" -ForegroundColor Green
        } else {
            Write-Host "  ⚠️ YAML structure might have issues" -ForegroundColor Yellow
            $warnings++
        }
        
    } catch {
        Write-Host "  ❌ Error reading file: $_" -ForegroundColor Red
        $errors++
    }
    
    Write-Host ""
}

# Vérifier les dépendances entre workflows
Write-Host "Checking workflow dependencies..." -ForegroundColor Yellow

$ciWorkflow = Get-Content "$workflowsDir/ci.yml" -Raw -ErrorAction SilentlyContinue
$securityWorkflow = Get-Content "$workflowsDir/security-audit.yml" -Raw -ErrorAction SilentlyContinue
$moneroWorkflow = Get-Content "$workflowsDir/monero-integration.yml" -Raw -ErrorAction SilentlyContinue

if ($ciWorkflow -and $securityWorkflow) {
    if ($securityWorkflow -match "needs:.*security-check") {
        Write-Host "  ✅ Security audit depends on CI security check" -ForegroundColor Green
    } else {
        Write-Host "  ⚠️ Security audit should depend on CI security check" -ForegroundColor Yellow
        $warnings++
    }
}

if ($moneroWorkflow -and $ciWorkflow) {
    if ($moneroWorkflow -match "needs:.*security-check") {
        Write-Host "  ✅ Monero integration depends on security check" -ForegroundColor Green
    } else {
        Write-Host "  ⚠️ Monero integration should depend on security check" -ForegroundColor Yellow
        $warnings++
    }
}

Write-Host ""

# Résumé
Write-Host "Validation Summary" -ForegroundColor Cyan
Write-Host "=================" -ForegroundColor Cyan
Write-Host ""

if ($errors -eq 0 -and $warnings -eq 0) {
    Write-Host "✅ All workflows are valid!" -ForegroundColor Green
    Write-Host ""
    Write-Host "Workflows created:" -ForegroundColor White
    Write-Host "  - ci.yml: Continuous Integration with security checks" -ForegroundColor White
    Write-Host "  - security-audit.yml: Weekly security audit" -ForegroundColor White
    Write-Host "  - monero-integration.yml: Monero testnet integration tests" -ForegroundColor White
    Write-Host "  - config.yml: Shared configuration" -ForegroundColor White
    Write-Host ""
    Write-Host "Next steps:" -ForegroundColor Yellow
    Write-Host "  1. Commit these workflows to your repository" -ForegroundColor White
    Write-Host "  2. Push to trigger the first CI run" -ForegroundColor White
    Write-Host "  3. Check the Actions tab in GitHub" -ForegroundColor White
    Write-Host "  4. Configure branch protection rules" -ForegroundColor White
    exit 0
} elseif ($errors -eq 0) {
    Write-Host "⚠️ Workflows are valid but have $warnings warning(s)" -ForegroundColor Yellow
    Write-Host "Consider addressing the warnings for better CI/CD" -ForegroundColor Yellow
    exit 0
} else {
    Write-Host "❌ Found $errors error(s) and $warnings warning(s)" -ForegroundColor Red
    Write-Host "Fix the errors before committing workflows" -ForegroundColor Red
    exit 1
}
