#!/usr/bin/env powershell
# Verification script for AUDIT.md fixes
# Run this after installing Rust to verify all fixes work correctly

Write-Host "===============================================" -ForegroundColor Cyan
Write-Host "  Monero Marketplace - Fix Verification" -ForegroundColor Cyan
Write-Host "===============================================" -ForegroundColor Cyan
Write-Host ""

# Check if Rust is installed
Write-Host "[1/6] Checking Rust installation..." -ForegroundColor Yellow
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Cargo not found. Please install Rust from https://rustup.rs/" -ForegroundColor Red
    exit 1
}

$rustVersion = cargo --version
Write-Host "✅ Rust installed: $rustVersion" -ForegroundColor Green
Write-Host ""

# Check workspace structure
Write-Host "[2/6] Verifying workspace structure..." -ForegroundColor Yellow
if (-not (Test-Path "Cargo.toml")) {
    Write-Host "❌ Cargo.toml not found. Run this from project root." -ForegroundColor Red
    exit 1
}
Write-Host "✅ Workspace structure valid" -ForegroundColor Green
Write-Host ""

# Run cargo check
Write-Host "[3/6] Running cargo check..." -ForegroundColor Yellow
$checkResult = cargo check --workspace 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Cargo check failed:" -ForegroundColor Red
    Write-Host $checkResult -ForegroundColor Red
    Write-Host ""
    Write-Host "CRITICAL: Compilation errors still exist. Check output above." -ForegroundColor Red
    exit 1
}
Write-Host "✅ Cargo check passed - code compiles!" -ForegroundColor Green
Write-Host ""

# Run cargo test (compilation only, tests may fail without RPC)
Write-Host "[4/6] Running cargo test (compile tests)..." -ForegroundColor Yellow
$testResult = cargo test --workspace --no-run 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Test compilation failed:" -ForegroundColor Red
    Write-Host $testResult -ForegroundColor Red
    exit 1
}
Write-Host "✅ All tests compile successfully" -ForegroundColor Green
Write-Host ""

# Run clippy
Write-Host "[5/6] Running clippy..." -ForegroundColor Yellow
$clippyResult = cargo clippy --workspace -- -D warnings 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "⚠️  Clippy warnings found:" -ForegroundColor Yellow
    Write-Host $clippyResult -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Note: Clippy warnings don't block compilation but should be addressed." -ForegroundColor Yellow
} else {
    Write-Host "✅ Clippy passed - no warnings!" -ForegroundColor Green
}
Write-Host ""

# Check formatting
Write-Host "[6/6] Checking code formatting..." -ForegroundColor Yellow
$fmtResult = cargo fmt --workspace --check 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "⚠️  Code formatting issues found" -ForegroundColor Yellow
    Write-Host "Run: cargo fmt --workspace" -ForegroundColor Yellow
} else {
    Write-Host "✅ Code formatting is correct" -ForegroundColor Green
}
Write-Host ""

# Final summary
Write-Host "===============================================" -ForegroundColor Cyan
Write-Host "  Verification Summary" -ForegroundColor Cyan
Write-Host "===============================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "✅ All critical compilation errors are FIXED!" -ForegroundColor Green
Write-Host ""
Write-Host "Fixes applied:" -ForegroundColor Cyan
Write-Host "  1. MoneroRpcClient::new() now accepts MoneroConfig" -ForegroundColor White
Write-Host "  2. get_version() method implemented" -ForegroundColor White
Write-Host "  3. get_balance() method implemented" -ForegroundColor White
Write-Host "  4. MoneroRpcClient now derives Clone" -ForegroundColor White
Write-Host "  5. CLI make_multisig includes threshold parameter" -ForegroundColor White
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Cyan
Write-Host "  • Build: cargo build --workspace" -ForegroundColor White
Write-Host "  • Test: cargo test --workspace" -ForegroundColor White
Write-Host "  • Run: cargo run --package cli -- --help" -ForegroundColor White
Write-Host ""
Write-Host "For detailed fixes, see FIXES-APPLIED.md" -ForegroundColor Yellow
Write-Host ""
