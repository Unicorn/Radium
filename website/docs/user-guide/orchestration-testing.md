---
id: "orchestration-testing"
title: "Orchestration System Testing Guide"
sidebar_label: "Orchestration System Testing Guide"
---

# Orchestration System Testing Guide

This guide provides step-by-step instructions for manually testing the orchestration system in Radium TUI.

## Overview

The orchestration system allows users to interact naturally with Radium without explicitly selecting agents. The orchestrator automatically routes tasks to appropriate specialist agents and coordinates multi-agent workflows.

## Prerequisites

- Radium TUI installed and configured
- At least one AI provider configured (Gemini, Claude, or OpenAI)
- API keys set up in credential store

## Test Scenarios

### 1. Basic Orchestration - Natural Language Input

**Objective**: Verify that natural language input routes to orchestration automatically.

**Steps**:
1. Start Radium TUI: `radium-tui`
2. Type a natural language request (without `/` prefix), for example:
   - "I need to refactor the authentication module"
   - "Create a new feature for task templates"
   - "Help me debug this error in the API"
3. Press Enter

**Expected Results**:
- System displays "🤔 Analyzing task..." indicator
- Orchestrator selects appropriate agent(s)
- Agent execution progress is shown
- Final response is displayed with results

**Troubleshooting**:
- If orchestration is disabled, you'll see: "Orchestration disabled. Use /chat or /agents for interaction."
- Enable orchestration with: `/orchestrator toggle`

---

### 2. Orchestration Status Command

**Objective**: Verify the `/orchestrator` command displays current configuration.

**Steps**:
1. In TUI, type: `/orchestrator`
2. Press Enter

**Expected Results**:
- Status display shows:
  - Enabled: ✓ Yes or ✗ No
  - Provider: Current provider name (e.g., "gemini", "claude")
  - Default: Default provider setting
  - Service: Initialization status

**Troubleshooting**:
- If service shows "Not initialized", orchestration may need to be enabled first
- Check API keys are configured if initialization fails

---

### 3. Enable/Disable Orchestration

**Objective**: Verify orchestration can be toggled on/off.

**Steps**:
1. Type: `/orchestrator toggle`
2. Press Enter
3. Type: `/orchestrator` to verify status changed
4. Type: `/orchestrator toggle` again to toggle back

**Expected Results**:
- First toggle: "Orchestration enabled" or "Orchestration disabled"
- Status command shows updated enabled state
- When disabled, natural input falls back to direct chat mode

**Troubleshooting**:
- If toggle doesn't work, check that orchestration service initialized correctly
- Verify configuration file is writable

---

### 4. Provider Switching

**Objective**: Test switching between different orchestration providers.

**Steps**:
1. Type: `/orchestrator switch gemini`
2. Press Enter
3. Verify status: `/orchestrator`
4. Switch to another provider: `/orchestrator switch claude`
5. Verify status again

**Expected Results**:
- Confirmation message: "✅ Switched to {provider} successfully"
- Status shows new provider
- Next orchestration request uses new provider

**Available Providers**:
- `gemini` - Google Gemini models
- `claude` - Anthropic Claude models
- `openai` - OpenAI GPT models
- `prompt_based` or `prompt-based` - Prompt-based fallback

**Error Cases**:
- Invalid provider: `/orchestrator switch invalid`
  - Expected: Error message listing available providers
- Missing provider: `/orchestrator switch`
  - Expected: Usage message showing correct syntax

**Troubleshooting**:
- Ensure API keys are configured for the provider you're switching to
- Some providers may require specific model configurations

---

### 5. Multi-Agent Workflows

**Objective**: Test orchestration coordinating multiple agents for complex tasks.

**Steps**:
1. Type a complex request that likely requires multiple agents, for example:
   - "Create a new feature for task templates with tests and documentation"
   - "Refactor the authentication system and add comprehensive tests"
2. Press Enter

**Expected Results**:
- Orchestrator shows multi-agent workflow plan:
  - "📋 Multi-agent workflow planned:"
  - List of agents to be invoked
  - Sequential or parallel execution indicators
- Progress shown for each agent
- Final synthesized result combining all agent outputs

**Troubleshooting**:
- If only one agent is invoked, the task may be simple enough for single-agent handling
- Check agent registry has multiple agents available
- Verify agents are properly configured and discoverable

---

### 6. Error Handling and Fallback

**Objective**: Test graceful error handling and fallback behavior.

**Test Cases**:

#### 6.1 API Rate Limit
- Make multiple rapid orchestration requests
- **Expected**: Clear error message about rate limits, suggestion to retry

#### 6.2 Invalid API Key
- Temporarily remove or invalidate API key
- Attempt orchestration
- **Expected**: Authentication error message with instructions to check API key

#### 6.3 Provider Unavailable
- Switch to a provider without API key configured
- Attempt orchestration
- **Expected**: Fallback to prompt-based orchestration (if enabled) or clear error

#### 6.4 Timeout Handling
- Request a very complex task that may timeout
- **Expected**: Timeout message after 5 minutes, suggestion to simplify task

**Troubleshooting**:
- Check logs for detailed error information
- Verify fallback configuration in config file
- Ensure network connectivity

---

### 7. Command Input Bypass

**Objective**: Verify that explicit commands bypass orchestration.

**Steps**:
1. Ensure orchestration is enabled: `/orchestrator toggle`
2. Type: `/chat my-agent` (explicit command)
3. Press Enter
4. Type: `/agents` (another explicit command)
5. Press Enter

**Expected Results**:
- Commands starting with `/` are handled by command system
- Orchestration is bypassed for explicit commands
- Direct agent interaction works as expected

**Troubleshooting**:
- If commands don't work, check command parsing
- Verify agent IDs are correct

---

### 8. Configuration Management

**Objective**: Test configuration loading and runtime changes.

**Steps**:
1. Check current configuration: `/orchestrator`
2. Modify config file manually (if applicable)
3. Restart TUI
4. Verify configuration loaded correctly

**Expected Results**:
- Configuration persists across restarts
- Runtime changes (via commands) take effect immediately
- Config file changes require restart

**Configuration File Location**:
- Default: `~/.radium/config.toml`
- Check `[orchestration]` section

**Key Settings**:
```toml
[orchestration]
enabled = true
provider = "gemini"  # or "claude", "openai", "prompt-based"

[orchestration.gemini]
model = "gemini-2.0-flash-thinking-exp"
temperature = 0.7
max_tool_iterations = 5
```

---

## Performance Validation

### Benchmark Targets

Run performance benchmarks to verify orchestration overhead:

```bash
cargo bench -p radium-orchestrator --bench orchestration_benchmark
```

**Target Metrics**:
- Engine creation: < 10µs
- Provider selection: < 1µs
- Tool registry build (100 tools): < 10ms
- Single tool call overhead: < 5ms
- Multi-tool iteration (5 iterations): < 50ms

**Overall Target**: < 500ms overhead for orchestration layer (excluding API calls)

---

## Common Issues and Solutions

### Issue: Orchestration Not Responding

**Symptoms**: Natural input doesn't trigger orchestration

**Solutions**:
1. Check orchestration is enabled: `/orchestrator`
2. Enable if disabled: `/orchestrator toggle`
3. Verify service initialized: Check status output
4. Check API keys are configured

### Issue: Wrong Agent Selected

**Symptoms**: Orchestrator selects incorrect agent for task

**Solutions**:
1. Check agent descriptions are clear and accurate
2. Verify agent registry loaded correctly
3. Try rephrasing the request more specifically
4. Use explicit `/chat <agent-id>` for direct control

### Issue: Multi-Agent Workflow Fails

**Symptoms**: Multi-agent tasks don't complete or fail midway

**Solutions**:
1. Check all required agents are available
2. Verify agent execution permissions
3. Check logs for specific agent failures
4. Try breaking task into smaller pieces

### Issue: Provider Switch Fails

**Symptoms**: Cannot switch to different provider

**Solutions**:
1. Verify API key for target provider is configured
2. Check provider name spelling (case-insensitive)
3. Verify provider is supported
4. Check configuration file permissions

---

## Test Checklist

Use this checklist to verify all orchestration features:

- [ ] Natural language input routes to orchestration
- [ ] `/orchestrator` command shows status
- [ ] `/orchestrator toggle` enables/disables orchestration
- [ ] `/orchestrator switch <provider>` changes provider
- [ ] Invalid provider shows helpful error
- [ ] Multi-agent workflows execute correctly
- [ ] Error handling works (rate limits, auth failures, timeouts)
- [ ] Fallback to prompt-based works when configured
- [ ] Command input (with `/`) bypasses orchestration
- [ ] Configuration persists across restarts
- [ ] Performance benchmarks meet targets

---

## Additional Resources

- [Orchestration Architecture](../developer-guide/agent-system-architecture.md)
- [Agent Configuration](../user-guide/agent-configuration.md)
- [Command Reference](../user-guide/commands.md)

---

## Reporting Issues

If you encounter issues during testing:

1. Note the exact steps to reproduce
2. Check TUI output and error messages
3. Review logs: `~/.radium/logs/radium-core.log`
4. Report with:
   - Radium version
   - Provider being used
   - Error messages
   - Relevant log excerpts

