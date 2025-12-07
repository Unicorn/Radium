use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

// ============================================================================
// Basic CLI Tests
// ============================================================================

#[test]
fn test_version() {
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.arg("--version").assert().success().stdout(predicate::str::contains("rad 0.1.0"));
}

#[test]
fn test_help() {
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Radium (rad) is a high-performance"));
}

#[test]
fn test_no_command_shows_help() {
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    // When no command is provided, help is shown
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Usage: rad"))
        .stdout(predicate::str::contains("Commands:"));
}

// ============================================================================
// Init Command Tests
// ============================================================================

#[test]
fn test_init_command() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();

    // Run init in the temp directory
    cmd.arg("init")
        .arg("--use-defaults")
        .arg(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Workspace initialized successfully!"));

    // Verify directory structure
    let radium_dir = temp_dir.path().join(".radium");
    assert!(radium_dir.exists());

    let internals_dir = radium_dir.join("_internals");
    assert!(internals_dir.exists());
    assert!(internals_dir.join("agents").exists());
    assert!(internals_dir.join("prompts").exists());
    assert!(internals_dir.join("memory").exists());
    assert!(internals_dir.join("logs").exists());
    assert!(internals_dir.join("artifacts").exists());
    assert!(internals_dir.join("inputs").exists());

    let plan_dir = radium_dir.join("plan");
    assert!(plan_dir.exists());
    assert!(plan_dir.join("backlog").exists());
    assert!(plan_dir.join("development").exists());
    assert!(plan_dir.join("review").exists());
    assert!(plan_dir.join("testing").exists());
    assert!(plan_dir.join("docs").exists());
}

#[test]
fn test_init_command_current_dir() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("init")
        .arg("--use-defaults")
        .assert()
        .success()
        .stdout(predicate::str::contains("Workspace initialized successfully!"));

    // Verify directory structure
    assert!(temp_dir.path().join(".radium").exists());
}

#[test]
fn test_init_command_already_exists() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path().to_str().unwrap();

    // First init
    let mut cmd1 = Command::cargo_bin("radium-cli").unwrap();
    cmd1.arg("init").arg("--use-defaults").arg(temp_path).assert().success();

    // Second init should still succeed but may show warning
    let mut cmd2 = Command::cargo_bin("radium-cli").unwrap();
    cmd2.arg("init").arg("--use-defaults").arg(temp_path).assert().success();
}

// ============================================================================
// Status Command Tests
// ============================================================================

#[test]
fn test_status_command_no_workspace() {
    // Run status outside of a workspace
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();

    cmd.current_dir(temp_dir.path())
        .arg("status")
        .assert()
        .success() // It exits with 0 even if no workspace is found
        .stdout(predicate::str::contains("workspace not found"));
}

#[test]
fn test_status_command_in_workspace() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Initialize first
    let mut init_cmd = Command::cargo_bin("radium-cli").unwrap();
    init_cmd.arg("init").arg("--use-defaults").arg(temp_path.to_str().unwrap()).assert().success();

    // Then run status
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_path)
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("Radium Status"))
        .stdout(predicate::str::contains("Valid: ✓"));
}

#[test]
fn test_status_command_json_output() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Initialize workspace
    let mut init_cmd = Command::cargo_bin("radium-cli").unwrap();
    init_cmd.arg("init").arg("--use-defaults").arg(temp_path.to_str().unwrap()).assert().success();

    // Run status with JSON output
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_path)
        .arg("status")
        .arg("--json")
        .assert()
        .success()
        .stdout(predicate::str::contains("{"))
        .stdout(predicate::str::contains("workspace"));
}

// ============================================================================
// Clean Command Tests
// ============================================================================

#[test]
fn test_clean_command_empty_workspace() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Initialize workspace
    let mut init_cmd = Command::cargo_bin("radium-cli").unwrap();
    init_cmd.arg("init").arg("--use-defaults").arg(temp_path.to_str().unwrap()).assert().success();

    // Run clean on empty workspace
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_path).arg("clean").assert().success().stdout(
        predicate::str::contains("Workspace already clean")
            .or(predicate::str::contains("Removed 0 files")),
    );
}

#[test]
fn test_clean_command_with_artifacts() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Initialize workspace
    let mut init_cmd = Command::cargo_bin("radium-cli").unwrap();
    init_cmd.arg("init").arg("--use-defaults").arg(temp_path.to_str().unwrap()).assert().success();

    // Create some artifacts
    let artifacts_dir = temp_path.join(".radium/_internals/artifacts");
    fs::write(artifacts_dir.join("test1.txt"), "test content 1").unwrap();
    fs::write(artifacts_dir.join("test2.txt"), "test content 2").unwrap();

    let logs_dir = temp_path.join(".radium/_internals/logs");
    fs::write(logs_dir.join("test.log"), "log content").unwrap();

    // Run clean
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_path)
        .arg("clean")
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed 3 files"));

    // Verify files are removed
    assert!(!artifacts_dir.join("test1.txt").exists());
    assert!(!artifacts_dir.join("test2.txt").exists());
    assert!(!logs_dir.join("test.log").exists());
}

#[test]
fn test_clean_command_verbose() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Initialize workspace
    let mut init_cmd = Command::cargo_bin("radium-cli").unwrap();
    init_cmd.arg("init").arg("--use-defaults").arg(temp_path.to_str().unwrap()).assert().success();

    // Create artifact
    let artifacts_dir = temp_path.join(".radium/_internals/artifacts");
    fs::write(artifacts_dir.join("test.txt"), "test").unwrap();

    // Run clean with verbose flag
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_path)
        .arg("clean")
        .arg("--verbose")
        .assert()
        .success()
        .stdout(predicate::str::contains("Cleaning"));
}

#[test]
fn test_clean_command_no_workspace() {
    let temp_dir = TempDir::new().unwrap();

    // Run clean without workspace
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path()).arg("clean").assert().failure(); // Should fail without workspace
}

// ============================================================================
// Doctor Command Tests
// ============================================================================

#[test]
fn test_doctor_command() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Initialize workspace
    let mut init_cmd = Command::cargo_bin("radium-cli").unwrap();
    init_cmd.arg("init").arg("--use-defaults").arg(temp_path.to_str().unwrap()).assert().success();

    // Run doctor
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_path)
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("Radium Doctor"))
        .stdout(predicate::str::contains("Workspace:"));
}

#[test]
fn test_doctor_command_no_workspace() {
    let temp_dir = TempDir::new().unwrap();

    // Run doctor without workspace
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("doctor")
        .assert()
        .success() // Doctor doesn't fail, just reports issues
        .stdout(predicate::str::contains("Not found"));
}

#[test]
fn test_doctor_command_json_output() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Initialize workspace
    let mut init_cmd = Command::cargo_bin("radium-cli").unwrap();
    init_cmd.arg("init").arg("--use-defaults").arg(temp_path.to_str().unwrap()).assert().success();

    // Run doctor with JSON output
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_path)
        .arg("doctor")
        .arg("--json")
        .assert()
        .success()
        .stdout(predicate::str::contains("{"))
        .stdout(predicate::str::contains("workspace"))
        .stdout(predicate::str::contains("environment"));
}

// ============================================================================
// Agents Command Tests
// ============================================================================

#[test]
fn test_agents_list_no_agents() {
    let temp_dir = TempDir::new().unwrap();

    // Run agents list (no agents directory)
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("agents")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("No agents found"));
}

#[test]
fn test_agents_list_json() {
    let temp_dir = TempDir::new().unwrap();

    // Run agents list with JSON output
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path()).arg("agents").arg("list").arg("--json").assert().success();
}

#[test]
fn test_agents_search() {
    let temp_dir = TempDir::new().unwrap();

    // Run agents search
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path()).arg("agents").arg("search").arg("test").assert().success();
}

#[test]
fn test_agents_validate() {
    let temp_dir = TempDir::new().unwrap();

    // Run agents validate
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path()).arg("agents").arg("validate").assert().success();
}

#[test]
fn test_agents_create() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create agents directory
    fs::create_dir_all(temp_path.join("agents/test")).unwrap();
    fs::create_dir_all(temp_path.join("prompts/agents/test")).unwrap();

    // Run agents create (ID and NAME are positional arguments)
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_path)
        .arg("agents")
        .arg("create")
        .arg("test-agent") // ID (positional)
        .arg("Test Agent") // NAME (positional)
        .arg("--category")
        .arg("test")
        .assert()
        .success()
        .stdout(predicate::str::contains("Agent template created successfully"));

    // Verify files were created
    assert!(temp_path.join("agents/test/test-agent.toml").exists());
    assert!(temp_path.join("prompts/agents/test/test-agent.md").exists());
}

// ============================================================================
// Templates Command Tests
// ============================================================================

#[test]
fn test_templates_list() {
    let temp_dir = TempDir::new().unwrap();

    // Run templates list
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path()).arg("templates").arg("list").assert().success();
}

#[test]
fn test_templates_list_json() {
    let temp_dir = TempDir::new().unwrap();

    // Run templates list with JSON
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path()).arg("templates").arg("list").arg("--json").assert().success();
}

#[test]
fn test_templates_validate() {
    let temp_dir = TempDir::new().unwrap();

    // Run templates validate
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path()).arg("templates").arg("validate").assert().success();
}

// ============================================================================
// Auth Command Tests
// ============================================================================

#[test]
fn test_auth_status() {
    let temp_dir = TempDir::new().unwrap();

    // Run auth status
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path()).arg("auth").arg("status").assert().success();
}

#[test]
fn test_auth_status_json() {
    let temp_dir = TempDir::new().unwrap();

    // Run auth status with JSON
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("auth")
        .arg("status")
        .arg("--json")
        .assert()
        .success()
        .stdout(predicate::str::contains("{"));
}

// ============================================================================
// Step Command Tests
// ============================================================================

#[test]
fn test_step_command_no_workspace() {
    let temp_dir = TempDir::new().unwrap();

    // Run step without workspace
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("step")
        .arg("test-agent")
        .arg("test prompt")
        .assert()
        .failure(); // Should fail without workspace
}

// ============================================================================
// Run Command Tests
// ============================================================================

#[test]
fn test_run_command_no_workspace() {
    let temp_dir = TempDir::new().unwrap();

    // Run without workspace
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path()).arg("run").arg("test-agent 'test prompt'").assert().failure(); // Should fail without workspace
}

// ============================================================================
// Plan Command Tests
// ============================================================================

#[test]
fn test_plan_command_no_workspace() {
    let temp_dir = TempDir::new().unwrap();

    // Run plan without workspace
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path()).arg("plan").arg("test spec").assert().failure(); // Should fail without workspace
}

// ============================================================================
// Craft Command Tests
// ============================================================================

#[test]
fn test_craft_command_no_workspace() {
    let temp_dir = TempDir::new().unwrap();

    // Run craft without workspace
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path()).arg("craft").assert().failure(); // Should fail without workspace
}

#[test]
fn test_craft_command_dry_run() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Initialize workspace
    let mut init_cmd = Command::cargo_bin("radium-cli").unwrap();
    init_cmd.arg("init").arg("--use-defaults").arg(temp_path.to_str().unwrap()).assert().success();

    // Run craft with dry-run but no plan - should fail with helpful message
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_path)
        .arg("craft")
        .arg("--dry-run")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Plan identifier is required"));
}

// ============================================================================
// Workspace Flag Tests
// ============================================================================

#[test]
fn test_workspace_flag_override() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path().to_str().unwrap();

    // Initialize workspace
    let mut init_cmd = Command::cargo_bin("radium-cli").unwrap();
    init_cmd.arg("init").arg("--use-defaults").arg(temp_path).assert().success();

    // Run status from different directory with workspace flag
    let other_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(other_dir.path())
        .arg("--workspace")
        .arg(temp_path)
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("Radium Status"));
}

// ============================================================================
// Log Level Tests
// ============================================================================

#[test]
fn test_log_level_debug() {
    let temp_dir = TempDir::new().unwrap();

    // Run with debug log level
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("--log-level")
        .arg("debug")
        .arg("status")
        .assert()
        .success();
}

#[test]
fn test_log_level_error() {
    let temp_dir = TempDir::new().unwrap();

    // Run with error log level
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("--log-level")
        .arg("error")
        .arg("status")
        .assert()
        .success();
}

// ============================================================================
// Plan Command Tests - Success Paths
// ============================================================================

/// Helper to initialize workspace
fn init_workspace(temp_dir: &TempDir) {
    let temp_path = temp_dir.path().to_str().unwrap();
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.arg("init").arg("--use-defaults").arg(temp_path).assert().success();
}

#[test]
fn test_plan_command_with_file_input() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    init_workspace(&temp_dir);

    // Create a specification file
    let spec_file = temp_path.join("spec.md");
    let spec_content = r#"# Test Project

## Iteration 1
- [ ] Task 1
- [ ] Task 2

## Iteration 2
- [ ] Task 3
"#;
    fs::write(&spec_file, spec_content).unwrap();

    // Run plan with file input
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_path)
        .arg("plan")
        .arg(spec_file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("rad plan"))
        .stdout(predicate::str::contains("Plan generated successfully"));

    // Verify plan structure was created
    let backlog_dir = temp_path.join("radium/backlog");
    assert!(backlog_dir.exists());
    
    // Find the created plan directory
    let entries: Vec<_> = fs::read_dir(&backlog_dir).unwrap().collect();
    assert!(!entries.is_empty(), "Plan directory should be created");
}

#[test]
fn test_plan_command_with_direct_input() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    init_workspace(&temp_dir);

    // Run plan with direct input
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_path)
        .arg("plan")
        .arg("# My Project\n## Iteration 1\n- [ ] Task 1")
        .assert()
        .success()
        .stdout(predicate::str::contains("Plan generated successfully"));
}

#[test]
fn test_plan_command_with_custom_id() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    init_workspace(&temp_dir);

    let spec_file = temp_path.join("spec.md");
    fs::write(&spec_file, "# Test Project\n## Iteration 1\n- [ ] Task 1").unwrap();

    // Run plan with custom ID
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_path)
        .arg("plan")
        .arg(spec_file.to_str().unwrap())
        .arg("--id")
        .arg("REQ-999")
        .assert()
        .success()
        .stdout(predicate::str::contains("REQ-999"));
}

#[test]
fn test_plan_command_with_custom_name() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    init_workspace(&temp_dir);

    let spec_file = temp_path.join("spec.md");
    fs::write(&spec_file, "# Test Project\n## Iteration 1\n- [ ] Task 1").unwrap();

    // Run plan with custom name
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_path)
        .arg("plan")
        .arg(spec_file.to_str().unwrap())
        .arg("--name")
        .arg("custom-name")
        .assert()
        .success()
        .stdout(predicate::str::contains("custom-name"));
}

#[test]
fn test_plan_command_duplicate_directory() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    init_workspace(&temp_dir);

    let spec_file = temp_path.join("spec.md");
    fs::write(&spec_file, "# Test Project\n## Iteration 1\n- [ ] Task 1").unwrap();

    // Create first plan
    let mut cmd1 = Command::cargo_bin("radium-cli").unwrap();
    cmd1.current_dir(temp_path)
        .arg("plan")
        .arg(spec_file.to_str().unwrap())
        .arg("--id")
        .arg("REQ-001")
        .arg("--name")
        .arg("test")
        .assert()
        .success();

    // Try to create duplicate plan with same ID and name
    let mut cmd2 = Command::cargo_bin("radium-cli").unwrap();
    cmd2.current_dir(temp_path)
        .arg("plan")
        .arg(spec_file.to_str().unwrap())
        .arg("--id")
        .arg("REQ-001")
        .arg("--name")
        .arg("test")
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

// ============================================================================
// Craft Command Tests - Success Paths
// ============================================================================

#[test]
fn test_craft_command_with_plan() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    init_workspace(&temp_dir);

    // First create a plan
    let spec_file = temp_path.join("spec.md");
    fs::write(&spec_file, "# Test Project\n## Iteration 1\n- [ ] Task 1").unwrap();

    let mut plan_cmd = Command::cargo_bin("radium-cli").unwrap();
    plan_cmd.current_dir(temp_path)
        .arg("plan")
        .arg(spec_file.to_str().unwrap())
        .arg("--id")
        .arg("REQ-001")
        .assert()
        .success();

    // Now run craft with the plan
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_path)
        .arg("craft")
        .arg("REQ-001")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("rad craft"))
        .stdout(predicate::str::contains("Dry run mode"));
}

#[test]
fn test_craft_command_with_iteration_filter() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    init_workspace(&temp_dir);

    // Create a plan
    let spec_file = temp_path.join("spec.md");
    fs::write(&spec_file, "# Test Project\n## Iteration 1\n- [ ] Task 1\n## Iteration 2\n- [ ] Task 2").unwrap();

    let mut plan_cmd = Command::cargo_bin("radium-cli").unwrap();
    plan_cmd.current_dir(temp_path)
        .arg("plan")
        .arg(spec_file.to_str().unwrap())
        .arg("--id")
        .arg("REQ-001")
        .assert()
        .success();

    // Run craft with iteration filter
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_path)
        .arg("craft")
        .arg("REQ-001")
        .arg("--iteration")
        .arg("I1")
        .arg("--dry-run")
        .assert()
        .success();
}

#[test]
fn test_craft_command_json_output() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    init_workspace(&temp_dir);

    // Create a plan
    let spec_file = temp_path.join("spec.md");
    fs::write(&spec_file, "# Test Project\n## Iteration 1\n- [ ] Task 1").unwrap();

    let mut plan_cmd = Command::cargo_bin("radium-cli").unwrap();
    plan_cmd.current_dir(temp_path)
        .arg("plan")
        .arg(spec_file.to_str().unwrap())
        .arg("--id")
        .arg("REQ-001")
        .assert()
        .success();

    // Run craft with JSON output
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_path)
        .arg("craft")
        .arg("REQ-001")
        .arg("--dry-run")
        .arg("--json")
        .assert()
        .success();
}

// ============================================================================
// Step Command Tests - Success Paths
// ============================================================================

/// Helper to create a test agent
fn create_test_agent(temp_dir: &TempDir, agent_id: &str) {
    let agents_dir = temp_dir.path().join("agents");
    fs::create_dir_all(&agents_dir).unwrap();
    
    let prompts_dir = temp_dir.path().join("prompts/agents");
    fs::create_dir_all(&prompts_dir).unwrap();

    // Create agent config
    let config_content = format!(
        r#"[agent]
id = "{}"
name = "Test Agent"
description = "A test agent"
prompt_path = "prompts/agents/{}.md"
engine = "mock"
model = "test-model"
"#,
        agent_id, agent_id
    );
    fs::write(agents_dir.join(format!("{}.toml", agent_id)), config_content).unwrap();

    // Create prompt file
    let prompt_content = r#"# Test Agent Prompt

{{user_input}}
"#;
    fs::write(prompts_dir.join(format!("{}.md", agent_id)), prompt_content).unwrap();
}

#[test]
fn test_step_command_with_agent() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    init_workspace(&temp_dir);
    create_test_agent(&temp_dir, "test-agent");

    // Run step command
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_path)
        .arg("step")
        .arg("test-agent")
        .arg("Test prompt")
        .assert()
        .success()
        .stdout(predicate::str::contains("rad step"))
        .stdout(predicate::str::contains("test-agent"));
}

#[test]
fn test_step_command_with_model_override() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    init_workspace(&temp_dir);
    create_test_agent(&temp_dir, "test-agent");

    // Run step with model override
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_path)
        .arg("step")
        .arg("test-agent")
        .arg("Test prompt")
        .arg("--model")
        .arg("custom-model")
        .assert()
        .success()
        .stdout(predicate::str::contains("custom-model"));
}

#[test]
fn test_step_command_with_engine_override() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    init_workspace(&temp_dir);
    create_test_agent(&temp_dir, "test-agent");

    // Run step with engine override
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_path)
        .arg("step")
        .arg("test-agent")
        .arg("Test prompt")
        .arg("--engine")
        .arg("mock")
        .assert()
        .success();
}

#[test]
fn test_step_command_agent_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    init_workspace(&temp_dir);

    // Run step with non-existent agent
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_path)
        .arg("step")
        .arg("non-existent-agent")
        .arg("Test prompt")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

// ============================================================================
// Run Command Tests - Success Paths
// ============================================================================

#[test]
fn test_run_command_with_agent() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    init_workspace(&temp_dir);
    create_test_agent(&temp_dir, "test-agent");

    // Run command with agent script
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_path)
        .arg("run")
        .arg("test-agent Hello world")
        .assert()
        .success()
        .stdout(predicate::str::contains("rad run"))
        .stdout(predicate::str::contains("test-agent"));
}

#[test]
fn test_run_command_with_model_override() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    init_workspace(&temp_dir);
    create_test_agent(&temp_dir, "test-agent");

    // Run with model override
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_path)
        .arg("run")
        .arg("test-agent Test prompt")
        .arg("--model")
        .arg("custom-model")
        .assert()
        .success();
}

#[test]
fn test_run_command_with_working_directory() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    init_workspace(&temp_dir);
    create_test_agent(&temp_dir, "test-agent");

    // Run with working directory
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_path)
        .arg("run")
        .arg("test-agent Test prompt")
        .arg("--dir")
        .arg(temp_path.to_str().unwrap())
        .assert()
        .success();
}

#[test]
fn test_run_command_invalid_script_format() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    init_workspace(&temp_dir);

    // Run with invalid script format (no prompt)
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_path)
        .arg("run")
        .arg("test-agent")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid script format"));
}

// ============================================================================
// Agents Command Tests - Enhanced Coverage
// ============================================================================

#[test]
fn test_agents_list_with_agents() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    init_workspace(&temp_dir);
    create_test_agent(&temp_dir, "test-agent");

    // List agents
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_path)
        .arg("agents")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("test-agent"));
}

#[test]
fn test_agents_list_verbose() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    init_workspace(&temp_dir);
    create_test_agent(&temp_dir, "test-agent");

    // List agents with verbose flag
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_path)
        .arg("agents")
        .arg("list")
        .arg("--verbose")
        .assert()
        .success()
        .stdout(predicate::str::contains("test-agent"));
}

#[test]
fn test_agents_search_with_results() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    init_workspace(&temp_dir);
    create_test_agent(&temp_dir, "test-agent");

    // Search for agents
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_path)
        .arg("agents")
        .arg("search")
        .arg("test")
        .assert()
        .success()
        .stdout(predicate::str::contains("test-agent"));
}

#[test]
fn test_agents_search_no_results() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    init_workspace(&temp_dir);

    // Search with no matches
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_path)
        .arg("agents")
        .arg("search")
        .arg("nonexistent")
        .assert()
        .success()
        .stdout(predicate::str::contains("No agents found"));
}

#[test]
fn test_agents_info() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    init_workspace(&temp_dir);
    create_test_agent(&temp_dir, "test-agent");

    // Get agent info
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_path)
        .arg("agents")
        .arg("info")
        .arg("test-agent")
        .assert()
        .success()
        .stdout(predicate::str::contains("test-agent"))
        .stdout(predicate::str::contains("Test Agent"));
}

#[test]
fn test_agents_info_json() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    init_workspace(&temp_dir);
    create_test_agent(&temp_dir, "test-agent");

    // Get agent info as JSON
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_path)
        .arg("agents")
        .arg("info")
        .arg("test-agent")
        .arg("--json")
        .assert()
        .success()
        .stdout(predicate::str::contains("{"))
        .stdout(predicate::str::contains("test-agent"));
}

#[test]
fn test_agents_info_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    init_workspace(&temp_dir);

    // Get info for non-existent agent
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_path)
        .arg("agents")
        .arg("info")
        .arg("non-existent")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_agents_validate_with_valid_agents() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    init_workspace(&temp_dir);
    create_test_agent(&temp_dir, "test-agent");

    // Validate agents
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_path)
        .arg("agents")
        .arg("validate")
        .assert()
        .success()
        .stdout(predicate::str::contains("validated"));
}

#[test]
fn test_agents_validate_verbose() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    init_workspace(&temp_dir);
    create_test_agent(&temp_dir, "test-agent");

    // Validate with verbose flag
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_path)
        .arg("agents")
        .arg("validate")
        .arg("--verbose")
        .assert()
        .success();
}

// ============================================================================
// Templates Command Tests - Enhanced Coverage
// ============================================================================

/// Helper to create a test template
fn create_test_template(temp_dir: &TempDir, template_name: &str) {
    let templates_dir = temp_dir.path().join("templates");
    fs::create_dir_all(&templates_dir).unwrap();

    let template_content = format!(
        r#"{{
  "name": "{}",
  "description": "A test template",
  "steps": [
    {{
      "config": {{
        "agent_id": "test-agent",
        "agent_name": "Test Agent",
        "step_type": "main",
        "execute_once": false
      }}
    }}
  ],
  "sub_agent_ids": ["test-agent"]
}}
"#,
        template_name
    );
    fs::write(templates_dir.join(format!("{}.json", template_name)), template_content).unwrap();
}

#[test]
fn test_templates_list_with_templates() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    init_workspace(&temp_dir);
    create_test_template(&temp_dir, "test-template");

    // List templates
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_path)
        .arg("templates")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("test-template"));
}

#[test]
fn test_templates_list_verbose() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    init_workspace(&temp_dir);
    create_test_template(&temp_dir, "test-template");

    // List templates with verbose flag
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_path)
        .arg("templates")
        .arg("list")
        .arg("--verbose")
        .assert()
        .success()
        .stdout(predicate::str::contains("test-template"));
}

#[test]
fn test_templates_info() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    init_workspace(&temp_dir);
    create_test_template(&temp_dir, "test-template");

    // Get template info
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_path)
        .arg("templates")
        .arg("info")
        .arg("test-template")
        .assert()
        .success()
        .stdout(predicate::str::contains("test-template"));
}

#[test]
fn test_templates_info_json() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    init_workspace(&temp_dir);
    create_test_template(&temp_dir, "test-template");

    // Get template info as JSON
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_path)
        .arg("templates")
        .arg("info")
        .arg("test-template")
        .arg("--json")
        .assert()
        .success()
        .stdout(predicate::str::contains("{"))
        .stdout(predicate::str::contains("test-template"));
}

#[test]
fn test_templates_info_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    init_workspace(&temp_dir);

    // Get info for non-existent template
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_path)
        .arg("templates")
        .arg("info")
        .arg("non-existent")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_templates_validate_with_valid_templates() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    init_workspace(&temp_dir);
    create_test_template(&temp_dir, "test-template");

    // Validate templates
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_path)
        .arg("templates")
        .arg("validate")
        .assert()
        .success()
        .stdout(predicate::str::contains("validated"));
}

#[test]
fn test_templates_validate_verbose() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    init_workspace(&temp_dir);
    create_test_template(&temp_dir, "test-template");

    // Validate with verbose flag
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_path)
        .arg("templates")
        .arg("validate")
        .arg("--verbose")
        .assert()
        .success();
}
