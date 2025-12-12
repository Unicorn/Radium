# Orchestration Troubleshooting Guide

This guide helps you diagnose and resolve common issues with Radium's orchestration system.

## Quick Diagnosis

Start by checking orchestration status:

```
/orchestrator
```

This shows:
- Whether orchestration is enabled
- Current provider and model
- Service initialization status

## Common Issues

### Orchestration Not Working

**Symptoms:**
- Natural language input doesn't trigger orchestration
- No "🤔 Analyzing..." indicator appears
- Input is treated as regular chat

**Diagnosis:**
1. Check if orchestration is enabled:
   ```
   /orchestrator
   ```
   Look for `Enabled: ✓ Yes`

2. Verify service is initialized:
   ```
   /orchestrator
   ```
   Should show `Service: [Provider name]` not `Service: Not initialized`

**Solutions:**
1. **Enable orchestration:**
   ```
   /orchestrator toggle
   ```

2. **Check API keys:**
   ```bash
   echo $GEMINI_API_KEY
   echo $ANTHROPIC_API_KEY
   echo $OPENAI_API_KEY
   ```
   At least one API key must be set.

3. **Reinitialize service:**
   ```
   /orchestrator toggle  # Disable
   /orchestrator toggle  # Re-enable
   ```

4. **Check logs:**
   ```bash
   tail -f logs/radium-core.log
   ```
   Look for initialization errors.

### "Function Calling Not Supported" Errors

**Symptoms:**
- Error message: "Function calling not supported"
- Orchestration fails immediately
- Fallback to prompt-based doesn't work

**Causes:**
1. Using prompt-based provider without fallback
2. Model doesn't support function calling
3. API key invalid or expired

**Solutions:**
1. **Switch to supported provider:**
   ```
   /orchestrator switch gemini
   # or
   /orchestrator switch claude
   ```

2. **Enable fallback:**
   Edit config file:
   ```toml
   [orchestration.fallback]
   enabled = true
   chain = ["gemini", "claude", "openai", "prompt_based"]
   ```

3. **Verify API key:**
   ```bash
   # Test Gemini
   curl -H "x-goog-api-key: $GEMINI_API_KEY" \
     https://generativelanguage.googleapis.com/v1/models
  
   # Test Claude
   curl -H "x-api-key: $ANTHROPIC_API_KEY" \
     https://api.anthropic.com/v1/messages
   ```

### Provider Authentication Issues

**Symptoms:**
- "API key not found" errors
- "Authentication failed" messages
- Provider switch fails

**Diagnosis:**
1. Check environment variables:
   ```bash
   env | grep -E "(GEMINI|ANTHROPIC|OPENAI)_API_KEY"
   ```

2. Verify API key format:
   - Gemini: Should start with `AIza...`
   - Claude: Should be a valid Anthropic API key
   - OpenAI: Should start with `sk-...`

**Solutions:**
1. **Set API key:**
   ```bash
   export GEMINI_API_KEY="your-key-here"
   ```

2. **Add to shell profile:**
   ```bash
   # Add to ~/.zshrc or ~/.bashrc
   echo 'export GEMINI_API_KEY="your-key-here"' >> ~/.zshrc
   source ~/.zshrc
   ```

3. **Verify key is valid:**
   - Check provider dashboard
   - Test with curl (see above)
   - Verify key hasn't expired

4. **Check key permissions:**
   - Ensure key has function calling enabled
   - Verify billing/quota is active

### Timeout Problems

**Symptoms:**
- "Finished with: max_iterations" message
- "Operation taking longer than expected" warnings
- Orchestration stops before completing

**Causes:**
1. Task too complex for iteration limit
2. Network latency
3. Provider API slow response

**Solutions:**
1. **Increase iteration limit:**
   Edit config:
   ```toml
   [orchestration.gemini]
   max_tool_iterations = 7  # Increase from default 5
   ```

2. **Break task into smaller requests:**
   Instead of:
   ```
   Build the entire user management system
   ```
   Try:
   ```
   Design the user database schema
   ```
   Then:
   ```
   Implement the User model
   ```

3. **Switch to faster provider:**
   ```
   /orchestrator switch gemini
   ```
   Gemini Flash is typically fastest.

4. **Check network connectivity:**
   ```bash
   ping api.anthropic.com
   ping generativelanguage.googleapis.com
   ```

### Tool Execution Failures

**Symptoms:**
- "Tool execution failed" errors
- "❌ Error" messages
- Specific agent not responding

**Diagnosis:**
1. Check which tool/agent failed:
   - Look at error message for tool name
   - Check if agent exists: `/agents`

2. Verify agent configuration:
   ```bash
   cat agents/agent-name.json
   ```

**Solutions:**
1. **Refresh agent registry:**
   ```
   /orchestrator refresh
   ```
   This reloads all agent definitions.

2. **Check agent file:**
   - Verify JSON syntax is valid
   - Ensure agent file is in correct location
   - Check file permissions

3. **Verify agent is accessible:**
   ```bash
   # Check project agents
   ls agents/
   
   # Check user agents
   ls ~/.radium/agents/
   ```

4. **Test agent directly:**
   ```
   /chat agent-name Test message
   ```
   If direct chat works but orchestration fails, check agent description.

### Network/Connectivity Issues

**Symptoms:**
- "Connection refused" errors
- "Timeout" errors
- Slow or intermittent responses

**Diagnosis:**
1. Test provider APIs:
   ```bash
   # Test Gemini
   curl -v https://generativelanguage.googleapis.com/v1/models
   
   # Test Claude
   curl -v https://api.anthropic.com/v1/messages
   ```

2. Check firewall/proxy:
   ```bash
   # Check if ports are blocked
   telnet api.anthropic.com 443
   ```

**Solutions:**
1. **Check internet connection:**
   ```bash
   ping google.com
   ```

2. **Configure proxy (if needed):**
   ```bash
   export HTTP_PROXY="http://proxy.example.com:8080"
   export HTTPS_PROXY="http://proxy.example.com:8080"
   ```

3. **Check DNS:**
   ```bash
   nslookup api.anthropic.com
   ```

4. **Try different provider:**
   If one provider fails, switch to another:
   ```
   /orchestrator switch claude
   ```

### Wrong Agent Selected

**Symptoms:**
- Orchestrator routes to incorrect agent
- Task not completed as expected
- Agent doesn't have required capabilities

**Solutions:**
1. **Be more specific:**
   Instead of:
   ```
   Fix the bug
   ```
   Try:
   ```
   Debug the authentication error in the login endpoint
   ```

2. **Provide context:**
   ```
   I'm working on a Rust web API. The authentication middleware is throwing errors when validating JWT tokens. Please review and fix.
   ```

3. **Use explicit command:**
   If you know which agent you need:
   ```
   /chat senior-developer Refactor the authentication module
   ```

4. **Check available agents:**
   ```
   /agents
   ```
   Review agent descriptions to understand capabilities.

5. **Improve agent descriptions:**
   Edit agent JSON files to include better descriptions:
   ```json
   {
     "description": "Expert in Rust web development, authentication, and API design"
   }
   ```

### Performance Issues

**Symptoms:**
- Slow orchestration responses
- High latency
- Timeout warnings

**Solutions:**
1. **Switch to faster provider:**
   ```
   /orchestrator switch gemini
   ```
   Gemini Flash is typically fastest.

2. **Reduce iteration limit:**
   Edit config:
   ```toml
   [orchestration.gemini]
   max_tool_iterations = 3  # Reduce from 5
   ```

3. **Lower temperature:**
   ```toml
   temperature = 0.5  # Reduce from 0.7
   ```

4. **Check provider status:**
   - Visit provider status pages
   - Check for service outages
   - Monitor API response times

5. **Optimize requests:**
   - Break complex tasks into smaller ones
   - Be specific to reduce back-and-forth
   - Provide context upfront

### Configuration Not Persisting

**Symptoms:**
- Changes via `/orchestrator` commands don't persist
- Configuration resets on restart
- File not found errors

**Solutions:**
1. **Check file location:**
   - Workspace config: `.radium/config/orchestration.toml`
   - Home config: `~/.radium/orchestration.toml`
   - Verify file exists and is writable

2. **Check file permissions:**
   ```bash
   ls -la .radium/config/orchestration.toml
   ```
   Should be readable/writable by your user.

3. **Verify workspace:**
   ```bash
   # Check if in workspace
   ls -la .radium/
   ```

4. **Create config directory:**
   ```bash
   mkdir -p .radium/config
   ```

5. **Manual save:**
   If automatic save fails, manually edit config file after using commands.

### Error Messages Reference

#### "API key not found for provider: X"

**Cause:** Environment variable not set

**Solution:**
```bash
export GEMINI_API_KEY="your-key"
```

#### "Orchestration service not available"

**Cause:** Service failed to initialize

**Solution:**
1. Check API keys
2. Verify provider is accessible
3. Check logs for errors
4. Try toggling: `/orchestrator toggle`

#### "Reached maximum iterations"

**Cause:** Task requires more tool calls than limit

**Solution:**
1. Increase `max_tool_iterations` in config
2. Break task into smaller requests

#### "Tool 'X' not found"

**Cause:** Agent not loaded or doesn't exist

**Solution:**
1. Refresh registry: `/orchestrator refresh`
2. Check agent file exists
3. Verify agent JSON is valid

#### "Orchestration timed out"

**Cause:** Operation exceeded 120 second timeout

**Solution:**
1. Break task into smaller pieces
2. Use faster provider
3. Simplify request

## Debugging Tips

### Enable Verbose Logging

Set log level to debug:

```bash
RUST_LOG=debug radium tui
```

### Check Logs

```bash
# View recent logs
tail -f logs/radium-core.log

# Search for errors
grep -i error logs/radium-core.log

# Search for orchestration
grep -i orchestration logs/radium-core.log
```

### Test Provider APIs

```bash
# Test Gemini
curl -H "x-goog-api-key: $GEMINI_API_KEY" \
  https://generativelanguage.googleapis.com/v1/models

# Test Claude
curl -H "x-api-key: $ANTHROPIC_API_KEY" \
  -H "anthropic-version: 2023-06-01" \
  https://api.anthropic.com/v1/messages
```

### Verify Configuration

```bash
# View current config
/orchestrator config

# Check config file
cat .radium/config/orchestration.toml
```

## Getting Help

If issues persist:

1. **Check documentation:**
   - [Orchestration User Guide](./orchestration.md)
   - [Configuration Guide](./orchestration-configuration.md)

2. **Review logs:**
   ```bash
   tail -100 logs/radium-core.log
   ```

3. **Test with minimal config:**
   ```toml
   [orchestration]
   enabled = true
   default_provider = "gemini"
   ```

4. **Try prompt-based fallback:**
   ```
   /orchestrator switch prompt_based
   ```
   If this works, issue is with API provider.

## Related Documentation

- [Orchestration User Guide](./orchestration.md) - Complete user guide
- [Orchestration Configuration](./orchestration-configuration.md) - Configuration details
- [Orchestration Workflows](../examples/orchestration-workflows.md) - Example workflows

