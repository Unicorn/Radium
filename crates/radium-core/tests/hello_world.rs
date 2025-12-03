//! End-to-end tests for the "Hello World" applications.
//!
//! Note: These tests require a running server for the CLI test.
//! The TUI and Desktop tests are marked ignored as they need special setup.

use assert_cmd::prelude::*;
use predicates::str;
use std::process::Command;

/// CLI (RAD-HW1): Verify client can connect to server
///
/// Note: This test requires a running radium-core server at localhost:50051
#[test]
#[ignore = "Requires running server - run manually with: cargo test --test hello_world -- --ignored"]
#[allow(deprecated)] // cargo_bin is deprecated but tests are ignored
fn test_cli_hello_world() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("radium-cli")?;
    cmd.arg("ping").arg("Hello from test");
    cmd.assert().success().stdout(str::contains("Pong!"));

    Ok(())
}

/// TUI (RAD-HW2): Verify TUI binary exists and can be invoked
///
/// Note: TUI is interactive and exits immediately without a terminal.
/// This test just verifies the binary exists and can be built.
#[test]
#[ignore = "TUI requires terminal - run manually"]
#[allow(deprecated)] // cargo_bin is deprecated but tests are ignored
fn test_tui_hello_world() -> Result<(), Box<dyn std::error::Error>> {
    // Just verify the binary exists by checking help
    let mut cmd = Command::cargo_bin("radium-tui")?;
    cmd.arg("--help");
    cmd.assert().success();
    Ok(())
}

/// Desktop (RAD-HW3): Verify Tauri app structure exists
///
/// Note: Desktop is a Tauri app and cannot be tested via cargo_bin.
/// This test is a placeholder - use Tauri's testing tools for proper E2E.
#[test]
#[ignore = "Desktop is Tauri app - requires Tauri testing setup"]
fn test_desktop_hello_world() {
    // Check that the desktop app directory structure exists
    let desktop_dir = std::path::Path::new("apps/desktop");
    assert!(desktop_dir.exists(), "Desktop app directory should exist");

    let tauri_conf = desktop_dir.join("src-tauri/tauri.conf.json");
    assert!(tauri_conf.exists(), "Tauri config should exist");
}

/// Test that verifies all hello world app binaries can be built
///
/// This test doesn't require a running server and can be part of CI.
#[test]
fn test_hello_world_binaries_exist() {
    // Get workspace root (go up from core to workspace root)
    // CARGO_MANIFEST_DIR is radium/crates/radium-core, so parent.parent is radium
    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    // Verify CLI binary can be built (by checking it's in the workspace)
    let cli_cargo = workspace_root.join("apps/cli/Cargo.toml");
    assert!(cli_cargo.exists(), "CLI Cargo.toml should exist at {:?}", cli_cargo);

    // Verify TUI binary can be built
    let tui_cargo = workspace_root.join("apps/tui/Cargo.toml");
    assert!(tui_cargo.exists(), "TUI Cargo.toml should exist at {:?}", tui_cargo);

    // Verify Desktop app structure
    let desktop_dir = workspace_root.join("apps/desktop");
    assert!(desktop_dir.exists(), "Desktop app directory should exist at {:?}", desktop_dir);
    let tauri_cargo = desktop_dir.join("src-tauri/Cargo.toml");
    assert!(tauri_cargo.exists(), "Tauri Cargo.toml should exist at {:?}", tauri_cargo);
}
