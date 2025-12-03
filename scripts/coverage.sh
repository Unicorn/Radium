#!/bin/bash
# Coverage script for Radium project
# Requires: cargo-tarpaulin (install with: cargo install cargo-tarpaulin)

set -e

echo "Running test coverage analysis..."

# Install cargo-tarpaulin if not present
if ! command -v cargo-tarpaulin &> /dev/null; then
    echo "cargo-tarpaulin not found. Installing..."
    cargo install cargo-tarpaulin
fi

# Run coverage for all packages
cd "$(dirname "$0")/.." || exit 1

# Generate coverage report
cargo tarpaulin \
    --workspace \
    --out Xml \
    --output-dir ./coverage \
    --exclude-files '*/tests/*' \
    --exclude-files '*/proto/*' \
    --exclude-files '*/build.rs' \
    --timeout 120

# Also generate HTML report
cargo tarpaulin \
    --workspace \
    --out Html \
    --output-dir ./coverage/html \
    --exclude-files '*/tests/*' \
    --exclude-files '*/proto/*' \
    --exclude-files '*/build.rs' \
    --timeout 120

echo "Coverage report generated in ./coverage/"
echo "HTML report available at ./coverage/html/tarpaulin-report.html"

