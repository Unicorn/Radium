//! Integration tests for TUI components.

#[cfg(test)]
mod tests {
    use radium_tui::views::markdown::render_markdown;

    #[test]
    fn test_markdown_rendering_integration() {
        // Test that markdown rendering works end-to-end
        let text = "This is **bold** and *italic* text with `code`.";
        let lines = render_markdown(text);
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_markdown_code_block_integration() {
        let text =
            "Here's some code:\n```rust\nfn main() {\n    println!(\"Hello\");\n}\n```\nThat's it!";
        let lines = render_markdown(text);
        // Should have multiple lines including code block
        assert!(lines.len() > 3);
    }

    #[test]
    fn test_markdown_list_integration() {
        let text = "- First item\n- Second item\n- Third item";
        let lines = render_markdown(text);
        assert!(!lines.is_empty());
    }
}
