#!/bin/bash
# Setup script for Ollama example

set -e

echo "Setting up Ollama example..."

# Start Ollama
echo "Starting Ollama..."
docker-compose up -d

# Wait for Ollama to be ready
echo "Waiting for Ollama to be ready..."
sleep 5

# Pull model
echo "Pulling llama3.2 model..."
docker exec ollama ollama pull llama3.2 || ollama pull llama3.2

# Verify
echo "Verifying setup..."
curl -s http://localhost:11434/api/tags | jq '.' || curl -s http://localhost:11434/api/tags

echo ""
echo "Setup complete!"
echo ""
echo "Set environment variable:"
echo "  export UNIVERSAL_BASE_URL=\"http://localhost:11434/v1\""
echo ""
echo "Test the agent:"
echo "  rad run ollama-agent \"Hello!\""

