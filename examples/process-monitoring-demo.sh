#!/bin/bash
# Process Monitoring Demo Script
# This script creates test scenarios for demonstrating the process monitoring features

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEMO_DIR="$SCRIPT_DIR/watch-mode-demo"

echo "🚀 Process Monitoring Demo Setup"
echo "=================================="
echo ""

# Create demo directory
mkdir -p "$DEMO_DIR"
cd "$DEMO_DIR"

# Create test files for different error scenarios

# 1. Rust compilation error demo
cat > demo_rust_error.rs << 'EOF'
fn main() {
    // This will cause a type error
    let x: i32 = "hello";  // Type mismatch error
    println!("x = {}", x);
}
EOF

# 2. Rust panic demo
cat > demo_rust_panic.rs << 'EOF'
fn main() {
    let numbers = vec![1, 2, 3];
    // This will panic - index out of bounds
    let value = numbers[10];
    println!("Value: {}", value);
}
EOF

# 3. TypeScript error demo (if Node.js available)
if command -v node &> /dev/null; then
    cat > demo_ts_error.js << 'EOF'
// This will cause a TypeError
const obj = null;
console.log(obj.property);  // Cannot read property of null
EOF

    cat > demo_syntax_error.js << 'EOF'
// This will cause a SyntaxError
const data = {
    name: "test"
    // Missing comma
    value: 123
};
console.log(data);
EOF
fi

# 4. Build error demo
cat > Cargo.toml << 'EOF'
[package]
name = "demo-project"
version = "0.1.0"
edition = "2021"

[dependencies]
# Missing dependency will cause build error
EOF

cat > src/main.rs << 'EOF'
use serde::Serialize;  // serde not in dependencies

fn main() {
    println!("Hello");
}
EOF

# Create test script for continuous errors
cat > continuous_errors.sh << 'EOF'
#!/bin/bash
# Simulates a process that continuously logs errors

echo "Starting continuous error simulator..."
sleep 2

while true; do
    echo "$(date): INFO - Processing request..."
    sleep 1

    echo "$(date): ERROR - Connection timeout to database"
    sleep 2

    echo "$(date): WARN - Deprecated API usage detected"
    sleep 1

    echo "$(date): ERROR - Failed to parse JSON response"
    sleep 3

    # Occasional panic
    if [ $((RANDOM % 10)) -eq 0 ]; then
        echo "$(date): FATAL - thread 'main' panicked at 'critical error'"
        sleep 1
    fi
done
EOF

chmod +x continuous_errors.sh

echo ""
echo "✅ Demo files created in: $DEMO_DIR"
echo ""
echo "📋 Test Scenarios Available:"
echo "----------------------------"
echo ""
echo "1. Rust Type Error:"
echo "   Command: rustc demo_rust_error.rs"
echo "   Expected: High severity, rust-expert-agent"
echo ""
echo "2. Rust Panic:"
echo "   Command: cargo run --manifest-path demo_rust_panic.rs"
echo "   Expected: Critical severity"
echo ""
if command -v node &> /dev/null; then
    echo "3. TypeScript TypeError:"
    echo "   Command: node demo_ts_error.js"
    echo "   Expected: High severity, typescript-agent"
    echo ""
    echo "4. JavaScript SyntaxError:"
    echo "   Command: node demo_syntax_error.js"
    echo "   Expected: High severity, typescript-agent"
    echo ""
fi
echo "5. Build Error:"
echo "   Command: cargo build"
echo "   Expected: High severity, build-agent"
echo ""
echo "6. Continuous Errors (Frequency Escalation):"
echo "   Command: ./continuous_errors.sh"
echo "   Expected: Errors escalate from Medium to High over time"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "🎯 How to Use with Radium TUI:"
echo "------------------------------"
echo ""
echo "1. Start Radium TUI:"
echo "   cd $SCRIPT_DIR/.."
echo "   ./dist/target/release/radium-tui"
echo ""
echo "2. Toggle Process Panel:"
echo "   Press: Ctrl+Shift+P"
echo ""
echo "3. Spawn a test process:"
echo "   Press: Ctrl+N"
echo "   Enter one of the commands above"
echo ""
echo "4. Observe error detection and classification"
echo ""
echo "5. Review and approve fix proposals:"
echo "   Press: Ctrl+A (when errors are detected)"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📚 For detailed documentation, see:"
echo "   PROCESS_MONITORING_GUIDE.md"
echo ""
echo "✨ Demo setup complete!"
echo ""
