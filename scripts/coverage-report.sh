#!/bin/bash
# Generate test coverage report for hooks system

set -e

echo "🔍 Generating test coverage report for hooks system..."

# Install cargo-llvm-cov if not present
if ! command -v cargo-llvm-cov &> /dev/null; then
    echo "Installing cargo-llvm-cov..."
    cargo install cargo-llvm-cov
fi

# Generate coverage report
cd crates/radium-core

echo "Running tests with coverage..."
cargo llvm-cov --locked --workspace --lcov --output-path ../../coverage.lcov --tests

echo "Generating HTML report..."
cargo llvm-cov --locked --workspace --html --output-path ../../coverage-html --tests

# Focus on hooks module
echo "Analyzing hooks module coverage..."
cargo llvm-cov --locked --workspace --lcov --output-path ../../hooks-coverage.lcov --tests --lib --hooks

# Generate summary
echo "📊 Coverage Summary:"
cargo llvm-cov --locked --workspace --summary --tests --lib --hooks 2>&1 | grep -A 20 "hooks"

echo "✅ Coverage report generated:"
echo "   - LCOV: coverage.lcov"
echo "   - HTML: coverage-html/index.html"
echo "   - Hooks: hooks-coverage.lcov"

cd ../..

