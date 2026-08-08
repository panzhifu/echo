#!/usr/bin/env bash
# Local quality check script.
# Runs the same checks as CI: format, clippy, tests.
set -euo pipefail

echo "=== Checking formatting ==="
cargo fmt --all -- --check

echo "=== Running clippy ==="
cargo clippy --workspace --all-targets -- -D warnings

echo "=== Running tests ==="
cargo test --workspace

echo "=== All checks passed ==="
