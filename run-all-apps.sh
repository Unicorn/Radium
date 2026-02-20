#!/usr/bin/env bash
#
# Run all Radium applications
# This script starts all Radium app variants in separate terminal windows/tabs
#

set -e

echo "🚀 Starting all Radium applications..."
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to open a new terminal tab/window and run a command
run_in_new_terminal() {
    local title=$1
    local command=$2

    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS - open new Terminal tab
        osascript <<EOF
tell application "Terminal"
    activate
    tell application "System Events" to keystroke "t" using {command down}
    delay 0.5
    do script "cd \"$(pwd)\" && echo \"$title\" && $command" in front window
end tell
EOF
    else
        echo -e "${YELLOW}⚠️  Please run the following command in a new terminal:${NC}"
        echo "   cd $(pwd) && $command"
        echo ""
    fi
}

echo "1️⃣  CLI (radium-cli)"
echo "   Command: nx run radium-cli:run"
echo ""

echo "2️⃣  TUI (radium-tui)"
echo "   Command: nx run radium-tui:run"
run_in_new_terminal "🖥️  Radium TUI" "nx run radium-tui:run"
echo -e "${GREEN}✓${NC} TUI starting in new terminal"
echo ""

echo "3️⃣  Desktop (Tauri/Electron)"
echo "   Command: nx run desktop:dev"
run_in_new_terminal "🪟  Radium Desktop" "nx run desktop:dev"
echo -e "${GREEN}✓${NC} Desktop starting in new terminal"
echo ""

echo "4️⃣  Mobile (React Native)"
echo "   Available commands:"
echo "   - Web: nx run mobile:web"
echo "   - iOS: nx run mobile:ios"
echo "   - Android: nx run mobile:android"
echo ""
read -p "Start mobile web version? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    run_in_new_terminal "📱 Radium Mobile (Web)" "nx run mobile:web"
    echo -e "${GREEN}✓${NC} Mobile web starting in new terminal"
fi
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${GREEN}✨ All applications started!${NC}"
echo ""
echo "To run individual apps manually:"
echo "  • CLI:     nx run radium-cli:run"
echo "  • TUI:     nx run radium-tui:run"
echo "  • Desktop: nx run desktop:dev"
echo "  • Mobile:  nx run mobile:web"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
