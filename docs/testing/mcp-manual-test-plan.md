# MCP Integration Manual Testing Plan

## Overview

This document outlines the manual testing procedures for the MCP (Model Context Protocol) integration in Radium. These tests should be executed to verify functionality beyond automated unit and integration tests.

## Prerequisites

- Radium CLI built and available in PATH
- Access to test MCP servers (or ability to create mock servers)
- Terminal access for CLI testing
- Network access for HTTP/SSE transport testing

## Test Scenarios

### 1. Configuration Management

#### 1.1 Create MCP Server Configuration

**Steps:**
1. Create a `.radium/mcp-servers.toml` file
2. Add a stdio transport server configuration
3. Add an HTTP transport server configuration
4. Add an SSE transport server configuration

**Expected Results:**
- Configuration file is created successfully
- All three server types are recognized
- Configuration can be loaded without errors

**Test Command:**
```bash
rad mcp list
```

#### 1.2 Load Invalid Configuration

**Steps:**
1. Create a `.radium/mcp-servers.toml` with invalid TOML syntax
2. Attempt to load the configuration

**Expected Results:**
- Error message is displayed
- Application does not crash
- Error message indicates the issue

#### 1.3 Configuration Validation

**Steps:**
1. Create configurations with missing required fields:
   - Stdio transport without `command`
   - HTTP transport without `url`
   - SSE transport without `url`
2. Attempt to load each configuration

**Expected Results:**
- Validation errors are reported
- Specific field names are mentioned in error messages

### 2. Transport Layer Testing

#### 2.1 Stdio Transport

**Steps:**
1. Configure a stdio transport server pointing to a simple command (e.g., `echo`)
2. Attempt to connect
3. Send a test message
4. Receive response
5. Disconnect

**Expected Results:**
- Connection succeeds
- Messages can be sent and received
- Disconnection is clean

**Test Command:**
```bash
rad mcp test <server-name>
```

#### 2.2 HTTP Transport

**Steps:**
1. Configure an HTTP transport server (use a test endpoint)
2. Attempt to connect
3. Send a test message
4. Receive response
5. Disconnect

**Expected Results:**
- Connection succeeds (or fails gracefully if server unavailable)
- HTTP requests are properly formatted
- Responses are parsed correctly

#### 2.3 SSE Transport

**Steps:**
1. Configure an SSE transport server (use a test endpoint)
2. Attempt to connect
3. Verify event stream is received
4. Disconnect

**Expected Results:**
- Connection succeeds (or fails gracefully if server unavailable)
- SSE events are received
- Connection cleanup is proper

### 3. Tool Discovery and Execution

#### 3.1 Tool Discovery

**Steps:**
1. Connect to an MCP server with available tools
2. Discover tools using `tools/list` method
3. Verify tools are registered in the registry

**Expected Results:**
- Tools are discovered successfully
- Tool metadata (name, description, schema) is parsed correctly
- Tools are available in the registry

**Test Command:**
```bash
rad mcp tools
```

#### 3.2 Tool Execution

**Steps:**
1. Execute a tool with valid arguments
2. Execute a tool with invalid arguments
3. Execute a non-existent tool

**Expected Results:**
- Valid tool execution succeeds
- Invalid arguments produce appropriate errors
- Non-existent tool returns "tool not found" error

#### 3.3 Conflict Resolution

**Steps:**
1. Connect to multiple MCP servers
2. Register tools with the same name from different servers
3. Verify conflict resolution (prefixing)

**Expected Results:**
- Tools from different servers are distinguished
- Prefixed names are used (e.g., `server1:tool`, `server2:tool`)
- Both tools remain accessible

### 4. Rich Content Support

#### 4.1 Text Content

**Steps:**
1. Execute a tool that returns text content
2. Verify text is displayed correctly

**Expected Results:**
- Text content is displayed properly
- Special characters are handled correctly

#### 4.2 Image Content

**Steps:**
1. Execute a tool that returns image content
2. Verify image data is handled (base64 or URL)

**Expected Results:**
- Image content is recognized
- Base64 data is properly formatted
- URLs are preserved

#### 4.3 Audio Content

**Steps:**
1. Execute a tool that returns audio content
2. Verify audio data is handled

**Expected Results:**
- Audio content is recognized
- MIME types are preserved

### 5. OAuth Authentication

#### 5.1 OAuth Flow

**Steps:**
1. Configure an MCP server with OAuth authentication
2. Attempt to connect
3. Complete OAuth flow (if interactive)
4. Verify token is stored

**Expected Results:**
- OAuth flow initiates correctly
- Token is obtained and stored securely
- Authenticated requests succeed

#### 5.2 Token Refresh

**Steps:**
1. Use an expired token
2. Verify refresh logic triggers
3. Verify new token is stored

**Expected Results:**
- Token refresh is attempted automatically
- New token replaces old token
- Requests continue to work after refresh

### 6. CLI Commands

#### 6.1 List Servers

**Steps:**
1. Run `rad mcp list`
2. Verify all configured servers are displayed

**Expected Results:**
- Server list is displayed
- Server names, transport types, and status are shown

#### 6.2 List Tools

**Steps:**
1. Run `rad mcp tools`
2. Verify all discovered tools are displayed

**Expected Results:**
- Tool list is displayed
- Tool names, descriptions, and sources are shown

#### 6.3 Test Server

**Steps:**
1. Run `rad mcp test <server-name>`
2. Verify connection test is performed

**Expected Results:**
- Connection test runs
- Results are displayed clearly
- Errors are reported appropriately

### 7. Error Handling

#### 7.1 Connection Errors

**Steps:**
1. Attempt to connect to a non-existent server
2. Attempt to connect with invalid credentials
3. Attempt to connect with network issues

**Expected Results:**
- Appropriate error messages are displayed
- Application does not crash
- Errors are logged

#### 7.2 Protocol Errors

**Steps:**
1. Send malformed JSON-RPC messages
2. Send messages with invalid method names
3. Send messages with missing required fields

**Expected Results:**
- Protocol errors are caught
- Error messages indicate the issue
- Application recovers gracefully

### 8. Integration with Agent System

#### 8.1 Agent Tool Discovery

**Steps:**
1. Start Radium with MCP servers configured
2. Verify MCP tools are available to agents
3. Execute an agent that uses MCP tools

**Expected Results:**
- MCP tools are discoverable by agents
- Agents can execute MCP tools
- Tool results are returned correctly

#### 8.2 Slash Commands

**Steps:**
1. Configure MCP servers with prompts
2. Verify prompts are available as slash commands
3. Execute a slash command

**Expected Results:**
- Prompts are registered as slash commands
- Slash commands are executable
- Results are displayed correctly

## Test Results Template

For each test scenario, document:

- **Date**: Date of testing
- **Tester**: Name of person executing test
- **Environment**: OS, Radium version, etc.
- **Result**: Pass/Fail/Partial
- **Notes**: Any observations, issues, or deviations from expected behavior
- **Screenshots/Logs**: If applicable

## Known Limitations

- Some tests require actual MCP servers to be running
- OAuth testing may require interactive browser sessions
- Network-dependent tests may fail in offline environments

## Follow-up Actions

After manual testing:

1. Document any issues found
2. Create bug reports for failures
3. Update automated tests to cover discovered edge cases
4. Update documentation based on findings

