//! Radium Desktop - Entry Point
//!
//! This is the main entry point for the Tauri desktop application.

// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    radium_desktop_lib::run()
}
