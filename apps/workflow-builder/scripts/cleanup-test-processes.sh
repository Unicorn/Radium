#!/bin/bash
# Cleanup script for E2E test processes
# Kills any lingering Playwright, Chromium, or Next.js dev server processes

echo "Cleaning up test processes..."

# Kill Playwright browsers
pkill -f "playwright" || true
pkill -f "chromium" || true
pkill -f "chrome" || true
pkill -f "firefox" || true
pkill -f "webkit" || true

# Kill Next.js dev server on port 3010 (if not being reused)
if [ -z "$CI" ]; then
  echo "Not killing Next.js dev server (reuseExistingServer is enabled in non-CI)"
else
  lsof -ti:3010 | xargs kill -9 2>/dev/null || true
fi

echo "Cleanup complete"
echo ""
echo "Remaining Node processes:"
ps aux | grep -E "node.*workflow-builder" | grep -v grep || echo "  None found"
