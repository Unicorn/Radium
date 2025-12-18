#!/bin/bash
# Start test database infrastructure
# Usage: ./scripts/test-db-start.sh [--with-rust-compiler]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
COMPOSE_FILE="$PROJECT_DIR/docker-compose.test.yml"

# Parse arguments
WITH_RUST_COMPILER=false
for arg in "$@"; do
  case $arg in
    --with-rust-compiler)
      WITH_RUST_COMPILER=true
      shift
      ;;
  esac
done

echo "Starting test infrastructure..."

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
  echo "Error: Docker is not running. Please start Docker first."
  exit 1
fi

# Start containers (without the optional 'tools' profile)
if [ "$WITH_RUST_COMPILER" = true ]; then
  echo "Including Rust compiler service..."
  docker compose -f "$COMPOSE_FILE" --profile rust-compiler up -d
else
  docker compose -f "$COMPOSE_FILE" up -d
fi

echo "Waiting for services to be healthy..."

# Wait for PostgreSQL
echo -n "  PostgreSQL: "
max_attempts=30
attempt=0
until docker compose -f "$COMPOSE_FILE" exec -T db pg_isready -U postgres -h localhost > /dev/null 2>&1 || [ $attempt -ge $max_attempts ]; do
  echo -n "."
  sleep 1
  attempt=$((attempt + 1))
done
if [ $attempt -ge $max_attempts ]; then
  echo " timeout"
  exit 1
else
  echo " ready"
fi

# Wait for Kong (main API gateway - this is the Supabase URL)
# We check if Kong responds (even with 401) - that means it's running
echo -n "  Kong API Gateway: "
attempt=0
until curl -s -o /dev/null -w "%{http_code}" http://localhost:54331/rest/v1/ 2>/dev/null | grep -qE "^[0-9]" || [ $attempt -ge $max_attempts ]; do
  echo -n "."
  sleep 2
  attempt=$((attempt + 1))
done
if [ $attempt -ge $max_attempts ]; then
  echo " timeout (may still be starting)"
else
  echo " ready"
fi

# Wait for GoTrue Auth (check if auth container is healthy via Docker)
echo -n "  GoTrue Auth: "
attempt=0
until docker inspect --format='{{.State.Health.Status}}' radium-test-auth 2>/dev/null | grep -q "healthy" || [ $attempt -ge $max_attempts ]; do
  echo -n "."
  sleep 2
  attempt=$((attempt + 1))
done
if [ $attempt -ge $max_attempts ]; then
  echo " timeout (may still be starting)"
else
  echo " ready"
fi

# Wait for Temporal (best effort - doesn't block)
echo -n "  Temporal: "
attempt=0
max_temporal_attempts=10
until docker exec radium-test-temporal sh -c 'temporal operator cluster health --address $(hostname -i):7233' > /dev/null 2>&1 || [ $attempt -ge $max_temporal_attempts ]; do
  echo -n "."
  sleep 3
  attempt=$((attempt + 1))
done
if [ $attempt -ge $max_temporal_attempts ]; then
  echo " starting (may take a few more seconds)"
else
  echo " ready"
fi

# Wait for Rust Compiler if enabled
if [ "$WITH_RUST_COMPILER" = true ]; then
  echo -n "  Rust Compiler: "
  attempt=0
  max_compiler_attempts=60  # Building can take a while
  until curl -s -o /dev/null -w "%{http_code}" http://localhost:3020/health 2>/dev/null | grep -q "200" || [ $attempt -ge $max_compiler_attempts ]; do
    echo -n "."
    sleep 3
    attempt=$((attempt + 1))
  done
  if [ $attempt -ge $max_compiler_attempts ]; then
    echo " timeout (may still be building)"
  else
    echo " ready"
  fi
fi

echo ""
echo "Test infrastructure is ready!"
echo ""
echo "Services available at:"
echo "  Supabase API:  http://localhost:54331"
echo "  PostgreSQL:    localhost:54332"
echo "  Temporal:      localhost:7233"
echo "  Inbucket UI:   http://localhost:54334"
if [ "$WITH_RUST_COMPILER" = true ]; then
  echo "  Rust Compiler: http://localhost:3020"
fi
echo ""
echo "Run tests with: npm test"
echo "Stop with: npm run test:db:stop"
