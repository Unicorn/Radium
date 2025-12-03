#!/bin/bash
# Radium Build & Test Script
# Verifies all components build and run correctly

set -e

echo "================================"
echo "Radium Build & Test Suite"
echo "================================"
echo ""

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counter
PASSED=0
FAILED=0

run_test() {
    local name=$1
    local cmd=$2

    echo -e "${BLUE}Testing: ${name}${NC}"
    if eval "$cmd" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ PASS${NC}"
        ((PASSED++))
    else
        echo -e "${YELLOW}✗ FAIL${NC}"
        ((FAILED++))
    fi
    echo ""
}

# 1. Cargo Build
echo "📦 Building Rust Workspace..."
run_test "Cargo Build (radium-core)" "cargo build --package radium-core --quiet"
run_test "Cargo Build (radium-cli)" "cargo build --package radium-cli --quiet"
run_test "Cargo Build (radium-tui)" "cargo build --package radium-tui --quiet"
echo ""

# 2. Cargo Test
echo "🧪 Running Tests..."
run_test "Cargo Test Suite" "cargo test --workspace --quiet"
echo ""

# 3. Cargo Clippy
echo "🔍 Running Clippy..."
run_test "Clippy Check" "cargo clippy --workspace --all-targets --quiet -- -D warnings 2>&1 | grep -q 'error:' && exit 1 || exit 0"
echo ""

# 4. Binary Verification
echo "🔧 Verifying Binaries..."
run_test "CLI Binary Exists" "test -f dist/target/debug/radium-cli"
run_test "TUI Binary Exists" "test -f dist/target/debug/radium-tui"
run_test "Core Binary Exists" "test -f dist/target/debug/radium-core"
echo ""

# 5. CLI Help
echo "📖 Testing CLI..."
run_test "CLI Help Command" "cargo run --bin radium-cli --quiet -- --help"
run_test "CLI Status Command" "cargo run --bin radium-cli --quiet -- status"
echo ""

# Results
echo "================================"
echo "Test Results"
echo "================================"
echo -e "${GREEN}Passed: $PASSED${NC}"
if [ $FAILED -gt 0 ]; then
    echo -e "${YELLOW}Failed: $FAILED${NC}"
fi
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All tests passed!${NC}"
    exit 0
else
    echo -e "${YELLOW}⚠ Some tests failed${NC}"
    exit 1
fi
