//! Clipboard operations for universal editor support.
//!
//! Provides cross-platform clipboard read/write operations and format detection
//! for clipboard-based editor integration.

pub mod parser;

use anyhow::{Context, Result};

/// Read text from system clipboard
pub fn read_clipboard() -> Result<String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use arboard::Clipboard;
        
        let mut clipboard = Clipboard::new()
            .context("Failed to initialize clipboard")?;
        
        let text = clipboard.get_text()
            .context("Failed to read from clipboard")?;
        
        Ok(text)
    }
    
    #[cfg(target_arch = "wasm32")]
    {
        // WASM doesn't have clipboard access directly
        anyhow::bail!("Clipboard operations not supported on WASM")
    }
}

/// Write text to system clipboard
pub fn write_clipboard(text: &str) -> Result<()> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use arboard::Clipboard;
        
        let mut clipboard = Clipboard::new()
            .context("Failed to initialize clipboard")?;
        
        clipboard.set_text(text)
            .context("Failed to write to clipboard")?;
        
        Ok(())
    }
    
    #[cfg(target_arch = "wasm32")]
    {
        // WASM doesn't have clipboard access directly
        anyhow::bail!("Clipboard operations not supported on WASM")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Requires system clipboard access
    fn test_clipboard_roundtrip() {
        let test_text = "Test clipboard content";
        write_clipboard(test_text).unwrap();
        let retrieved = read_clipboard().unwrap();
        assert_eq!(test_text, retrieved);
    }
}

