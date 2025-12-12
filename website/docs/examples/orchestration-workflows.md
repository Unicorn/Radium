---
id: "orchestration-workflows"
title: "Orchestration Workflow Examples"
sidebar_label: "Orchestration Workflow Exam..."
---

# Orchestration Workflow Examples

This document provides real-world examples of using Radium's orchestration system for common development tasks.

## Example 1: Simple Refactoring Task

**Goal:** Refactor an authentication module to improve code quality

**Input:**
```
You: I need to refactor the authentication module
```

**Orchestration Flow:**
1. Orchestrator analyzes request
2. Routes to `senior-developer` agent
3. Agent analyzes code and refactors
4. Results returned

**Expected Output:**
```
🤔 Analyzing...
📋 Invoking: senior-developer
✅ Complete (8.5s)

Assistant: I've refactored the authentication module with the following improvements:

- Extracted authentication logic into separate service class
- Improved error handling with specific exception types
- Added input validation
- Improved code documentation

Files modified:
- crates/radium-core/src/auth/mod.rs
- crates/radium-core/src/auth/service.rs
- crates/radium-core/src/auth/errors.rs
```

---

## Example 2: Multi-Agent Feature Development

**Goal:** Create a complete feature from requirements to implementation

**Input:**
```
You: Create a new feature for task templates
```

**Orchestration Flow:**
1. Orchestrator analyzes complex request
2. Plans multi-agent workflow
3. Routes to multiple agents in sequence:
   - `product-manager` - Define requirements
   - `architect` - Design architecture
   - `senior-developer` - Implement code
   - `tester` - Create tests

**Expected Output:**
```
🤔 Analyzing...
1. 📐 product-manager - Define feature requirements
2. 🏗️ architect - Design implementation approach
3. 💻 senior-developer - Implement feature
4. 🧪 tester - Create test suite

Executing 4 agents...

Step 1: Product Manager defining requirements...
Step 2: Architect designing implementation...
Step 3: Senior Developer implementing...
Step 4: Tester creating test suite...

✅ Complete (24.3s)

Assistant: Feature "Task Templates" has been created successfully!

Components:
- TaskTemplate model and database migration
- TaskTemplateService for business logic
- TaskTemplateController for API endpoints
- Comprehensive test suite with 85% coverage

Files created:
- crates/radium-core/src/templates/mod.rs
- crates/radium-core/src/templates/service.rs
- crates/radium-core/src/templates/controller.rs
- tests/templates_test.rs
```

---

## Example 3: Code Review Workflow

**Goal:** Get a comprehensive code review of recent changes

**Input:**
```
You: Review my recent changes to the API module
```

**Orchestration Flow:**
1. Orchestrator identifies code review request
2. Routes to `code-reviewer` agent
3. Agent analyzes git changes and provides feedback

**Expected Output:**
```
🤔 Analyzing...
📋 Invoking: code-reviewer
✅ Complete (12.1s)

Assistant: I've reviewed your recent changes to the API module. Here's my analysis:

✅ Strengths:
- Good error handling patterns
- Consistent code style
- Proper logging

⚠️ Areas for Improvement:
- Missing input validation in UserController.create()
- Potential SQL injection risk in UserRepository.find_by_email()
- Consider adding rate limiting for authentication endpoints

🔧 Suggested Changes:
1. Add validation middleware for all POST endpoints
2. Use parameterized queries in repository methods
3. Implement rate limiting middleware

Files reviewed:
- apps/api/src/controllers/user_controller.rs
- apps/api/src/repositories/user_repository.rs
- apps/api/src/routes/auth.rs
```

---

## Example 4: Documentation Generation

**Goal:** Generate API documentation from codebase

**Input:**
```
You: Generate API documentation for the user management endpoints
```

**Orchestration Flow:**
1. Orchestrator identifies documentation task
2. Routes to `documentation-agent` or `senior-developer`
3. Agent analyzes code and generates documentation

**Expected Output:**
```
🤔 Analyzing...
📋 Invoking: documentation-agent
✅ Complete (6.8s)

Assistant: I've generated comprehensive API documentation for user management endpoints:

# User Management API

## POST /api/users
Creates a new user account.

**Request Body:**
```json
{
  "email": "user@example.com",
  "password": "securepassword",
  "name": "John Doe"
}
```

**Response:**
- 201 Created: User successfully created
- 400 Bad Request: Validation errors
- 409 Conflict: Email already exists

## GET /api/users/:id
Retrieves user information by ID.

**Path Parameters:**
- `id` (string, required): User UUID

**Response:**
- 200 OK: User data
- 404 Not Found: User does not exist

[... additional endpoints ...]

Documentation saved to:
- docs/api/user-management.md
```

---

## Example 5: Debugging Assistance

**Goal:** Debug a failing test or error

**Input:**
```
You: Help me debug this error: "Connection refused when connecting to database"
```

**Orchestration Flow:**
1. Orchestrator identifies debugging request
2. Routes to `senior-developer` or `debugging-agent`
3. Agent analyzes error and provides solutions

**Expected Output:**
```
🤔 Analyzing...
📋 Invoking: senior-developer
✅ Complete (4.2s)

Assistant: I've analyzed the database connection error. Here are the likely causes and solutions:

🔍 Diagnosis:
The error "Connection refused" typically indicates:
1. Database service is not running
2. Incorrect connection host/port
3. Firewall blocking connection
4. Wrong database URL in configuration

✅ Solutions:

1. Check database service:
   ```bash
   # For PostgreSQL
   sudo systemctl status postgresql
   
   # For MySQL
   sudo systemctl status mysql
   ```

2. Verify connection settings in `.env`:
   ```env
   DATABASE_URL=postgresql://user:password@localhost:5432/radium
   ```

3. Test connection:
   ```bash
   psql $DATABASE_URL
   ```

4. Check logs for more details:
   ```bash
   tail -f logs/radium-core.log
   ```

🔧 Quick Fix:
If using Docker, ensure database container is running:
```bash
docker-compose up -d postgres
```

Most likely cause: Database service is not started. Check service status first.
```

---

## Example 6: Quick Code Snippet Generation

**Goal:** Generate a code snippet for a specific task

**Input:**
```
You: Generate a function to validate email addresses in Rust
```

**Orchestration Flow:**
1. Orchestrator identifies code generation request
2. Routes to `senior-developer`
3. Agent generates code with best practices

**Expected Output:**
```
🤔 Analyzing...
📋 Invoking: senior-developer
✅ Complete (2.1s)

Assistant: Here's a robust email validation function in Rust:

```rust
use regex::Regex;

pub fn validate_email(email: &str) -> bool {
    // RFC 5322 compliant email regex (simplified version)
    let email_regex = Regex::new(
        r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$"
    ).unwrap();
    
    email_regex.is_match(email)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_emails() {
        assert!(validate_email("user@example.com"));
        assert!(validate_email("test.email+tag@example.co.uk"));
    }

    #[test]
    fn test_invalid_emails() {
        assert!(!validate_email("invalid"));
        assert!(!validate_email("@example.com"));
        assert!(!validate_email("user@"));
    }
}
```

**Usage:**
```rust
if validate_email(&user_input) {
    println!("Valid email!");
} else {
    eprintln!("Invalid email format");
}
```

**Note:** For production use, consider using a library like `validator` crate which provides more comprehensive validation.
```

---

## Example 7: Testing Workflow

**Goal:** Create comprehensive tests for a module

**Input:**
```
You: Create unit tests for the authentication service
```

**Orchestration Flow:**
1. Orchestrator identifies testing request
2. Routes to `tester` agent
3. Agent analyzes service and creates test suite

**Expected Output:**
```
🤔 Analyzing...
📋 Invoking: tester
✅ Complete (9.3s)

Assistant: I've created a comprehensive test suite for the authentication service:

```rust
// tests/auth_service_test.rs

use radium_core::auth::service::AuthService;
use radium_core::auth::credentials::Credentials;

#[tokio::test]
async fn test_successful_authentication() {
    let service = AuthService::new();
    let credentials = Credentials::new("user@example.com", "password123");
    
    let result = service.authenticate(&credentials).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_invalid_credentials() {
    let service = AuthService::new();
    let credentials = Credentials::new("user@example.com", "wrongpassword");
    
    let result = service.authenticate(&credentials).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_missing_email() {
    let service = AuthService::new();
    let credentials = Credentials::new("", "password123");
    
    let result = service.authenticate(&credentials).await;
    assert!(result.is_err());
}

// ... additional test cases ...
```

**Test Coverage:**
- Authentication success scenarios
- Invalid credentials handling
- Input validation
- Error handling
- Edge cases

**Run tests:**
```bash
cargo test --package radium-core --lib auth_service
```

Test file saved to: `tests/auth_service_test.rs`
```

---

## Tips for Effective Orchestration

### 1. Be Specific and Clear

**Good:**
```
"Refactor the UserService to use dependency injection and improve error handling"
```

**Not as Good:**
```
"fix user service"
```

### 2. Break Down Complex Tasks

**Good Approach:**
```
Step 1: "Design the database schema for user profiles"
Step 2: "Create the UserProfile model"
Step 3: "Implement the UserProfileService"
Step 4: "Add API endpoints for profile management"
```

**Less Effective:**
```
"Build the entire user profile system"
```

### 3. Provide Context

**Good:**
```
"I'm working on a Rust web API. I need to add JWT authentication middleware that validates tokens and extracts user information."
```

**Not as Good:**
```
"add auth"
```

### 4. Use Explicit Commands When Needed

If you know exactly which agent you want:
```
/chat senior-developer Refactor the payment processing module
```

### 5. Iterate on Results

You can ask follow-up questions:
```
You: Create a function to parse JSON config files
[... response ...]

You: Now add validation for required fields
[... response ...]

You: Add error handling for malformed JSON
[... response ...]
```

---

## Advanced Patterns

### Pattern 1: Multi-Step Feature Development

```
Step 1: "Design the API for task templates"
Step 2: "Review the API design"
Step 3: "Implement the API endpoints"
Step 4: "Create tests for the API"
Step 5: "Generate API documentation"
```

### Pattern 2: Code Review and Improvement

```
Step 1: "Review my authentication module"
Step 2: "Implement the suggested improvements"
Step 3: "Add tests for the improved code"
```

### Pattern 3: Research and Implementation

```
Step 1: "Research best practices for JWT authentication in Rust"
Step 2: "Implement JWT authentication based on research"
Step 3: "Review the implementation"
```

---

## Example 8: Workspace-Based Configuration

**Goal:** Configure orchestration for a specific project

**Setup:**
1. Navigate to your Radium workspace
2. Configuration is automatically saved to `.radium/config/orchestration.toml`

**View Configuration:**
```
/orchestrator config
```

**Switch Provider:**
```
/orchestrator switch claude
```

**Refresh Agents:**
After adding new agents to `agents/` directory:
```
/orchestrator refresh
```

**Expected Output:**
```
🔄 Refreshing agent tool registry...
✅ Agent tool registry refreshed successfully

All available agents have been reloaded and are ready for use.
```

---

## Example 9: Using Configuration Commands

**Goal:** Manage orchestration settings via TUI commands

**View Status:**
```
/orchestrator
```

**View Full Configuration:**
```
/orchestrator config
```

**Switch Provider:**
```
/orchestrator switch gemini
```

**Toggle Orchestration:**
```
/orchestrator toggle
```

**Refresh Agent Registry:**
```
/orchestrator refresh
```

All changes are automatically saved to your configuration file (workspace config preferred, home directory as fallback).

---

## See Also

- [Orchestration User Guide](../user-guide/orchestration.md) - Complete user guide
- [Orchestration Configuration Guide](../user-guide/orchestration-configuration.md) - Configuration details
- [Orchestration Troubleshooting Guide](../user-guide/orchestration-troubleshooting.md) - Common issues
- [Agent Configuration](../user-guide/agent-configuration.md) - Agent setup
- [Orchestration Testing Guide](../user-guide/orchestration-testing.md) - Testing procedures

