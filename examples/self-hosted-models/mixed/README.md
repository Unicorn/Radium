# Mixed Cloud/Self-Hosted Example

This example demonstrates using a self-hosted model as the primary provider with cloud models as fallback and premium tiers.

## Configuration

The agent uses:
- **Primary**: Local Ollama (llama3.2) - Fast and free
- **Fallback**: Cloud Gemini - Reliable backup
- **Premium**: Cloud OpenAI (gpt-4o) - Best quality when needed

## Setup

1. Set up Ollama (see `../ollama/`)
2. Configure cloud API keys (Gemini, OpenAI)
3. Set environment variable:
   ```bash
   export UNIVERSAL_BASE_URL="http://localhost:11434/v1"
   ```
4. Test the agent:
   ```bash
   rad run mixed-agent "Hello!"
   ```

## Benefits

- **Cost Savings**: Most requests use free local model
- **Reliability**: Automatic fallback to cloud if local fails
- **Quality**: Premium tier available for critical tasks

