#!/bin/bash
# Setup script for vLLM example

set -e

echo "Setting up vLLM example..."

# Check for GPU
if ! command -v nvidia-smi &> /dev/null; then
    echo "Warning: nvidia-smi not found. GPU may not be available."
fi

# Start vLLM
echo "Starting vLLM..."
docker-compose up -d

# Wait for vLLM to be ready
echo "Waiting for vLLM to be ready..."
sleep 30

# Verify
echo "Verifying setup..."
curl -s http://localhost:8000/v1/models | jq '.' || curl -s http://localhost:8000/v1/models

echo ""
echo "Setup complete!"
echo ""
echo "Set environment variable:"
echo "  export UNIVERSAL_BASE_URL=\"http://localhost:8000/v1\""
echo ""
echo "Test the agent:"
echo "  rad run vllm-agent \"Hello!\""

