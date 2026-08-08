#!/usr/bin/env pwsh
# Local quality check script (PowerShell).
# Runs the same checks as CI: format, clippy, tests.
$ErrorActionPreference = "Stop"

Write-Host "=== Checking formatting ==="
cargo fmt --all -- --check
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "=== Running clippy ==="
cargo clippy --workspace --all-targets -- -D warnings
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "=== Running tests ==="
cargo test --workspace
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "=== All checks passed ==="
