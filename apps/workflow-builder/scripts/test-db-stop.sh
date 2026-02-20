#!/bin/bash
# Stop test database infrastructure
# Usage: ./scripts/test-db-stop.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
COMPOSE_FILE="$PROJECT_DIR/docker-compose.test.yml"

echo "Stopping test infrastructure..."

# Stop and remove containers
docker compose -f "$COMPOSE_FILE" down --volumes --remove-orphans

echo "Test infrastructure stopped."
