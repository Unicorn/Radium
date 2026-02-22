#!/usr/bin/env bash
set -euo pipefail

echo "Starting Neo4j for local development..."
docker compose -f docker-compose.neo4j.yml up -d

echo "Waiting for Neo4j to be ready..."
until docker compose -f docker-compose.neo4j.yml exec neo4j cypher-shell -u neo4j -p radium-dev "RETURN 1" > /dev/null 2>&1; do
    sleep 2
    echo "  waiting..."
done

echo ""
echo "Neo4j is ready!"
echo "  Browser: http://localhost:7474"
echo "  Bolt:    bolt://localhost:7687"
echo "  User:    neo4j"
echo "  Pass:    radium-dev"
echo ""
echo "Set these env vars for radium-discovery:"
echo "  export NEO4J_URI=bolt://localhost:7687"
echo "  export NEO4J_USER=neo4j"
echo "  export NEO4J_PASSWORD=radium-dev"
