//! Build script for radium-core.
//!
//! This script compiles the protobuf definitions into Rust code.
//!
//! Note: Build scripts require `std::env::var` and `println!` for cargo integration,
//! so we allow these disallowed items here.
#![allow(clippy::disallowed_methods)]
#![allow(clippy::disallowed_macros)]

use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR")?);

    // Configure and compile protobuf files
    tonic_build::configure()
        // Generate server code
        .build_server(true)
        // Generate client code (useful for testing and CLI)
        .build_client(true)
        // Generate file descriptor set for reflection
        .file_descriptor_set_path(out_dir.join("radium_descriptor.bin"))
        // Compile the proto files
        .compile_protos(&["proto/radium.proto"], &["proto/"])?;

    // Tell cargo to rerun this script if proto files change
    println!("cargo:rerun-if-changed=proto/");

    Ok(())
}
