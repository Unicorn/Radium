#!/bin/bash
# Setup script for LocalAI example

set -e

echo "Setting up LocalAI example..."

# Create directories
mkdir -p models config

# Start LocalAI
echo "Starting LocalAI..."
docker-compose up -d

# Wait for LocalAI to be ready
echo "Waiting for LocalAI to be ready..."
sleep 10

# Verify
echo "Verifying setup..."
curl -s http://localhost:8080/v1/models | jq '.' || curl -s http://localhost:8080/v1/models

echo ""
echo "Setup complete!"
echo ""
echo "Note: You may need to configure models in the config/ directory"
echo "See LocalAI documentation for model configuration"
echo ""
echo "Set environment variable:"
echo "  export UNIVERSAL_BASE_URL=\"http://localhost:8080/v1\""
echo ""
echo "Test the agent:"
echo "  rad run localai-agent \"Hello!\""

