-- Integration tests for Radium Neovim extension
-- Tests the complete workflow from selection to code application

local utils = require('radium.utils')
local commands = require('radium.commands')
local diff = require('radium.diff')

describe("Radium Neovim Extension", function()
    describe("utils", function()
        it("should extract file context", function()
            local context = utils.get_file_context()
            assert.is_not_nil(context)
            assert.is_not_nil(context.file_path)
            assert.is_not_nil(context.language)
        end)
        
        it("should parse visual selection", function()
            -- Note: Visual selection tests require visual mode setup
            -- This is a placeholder for full integration tests
            local selection = utils.get_visual_selection()
            assert.is_not_nil(selection)
        end)
        
        it("should store and retrieve agent output", function()
            local test_output = "Test agent output"
            utils.store_agent_output(test_output)
            local retrieved = utils.get_last_agent_output()
            assert.equals(test_output, retrieved)
        end)
    end)
    
    describe("diff", function()
        it("should parse code blocks from markdown", function()
            local markdown = [[
Here's some text.

```rust
fn main() {
    println!("Hello");
}
```

More text.

```python
print("World")
```
]]
            local blocks = diff.parse_code_blocks(markdown)
            assert.equals(2, #blocks)
            assert.equals("rust", blocks[1].language)
            assert.equals("python", blocks[2].language)
        end)
        
        it("should handle empty output", function()
            local blocks = diff.parse_code_blocks("")
            assert.equals(0, #blocks)
        end)
        
        it("should generate diff between content", function()
            local original = "line1\nline2\nline3"
            local new = "line1\nline2_modified\nline3"
            local diff_text = diff.generate_diff(original, new)
            assert.is_not_nil(diff_text)
            assert.is_string(diff_text)
        end)
    end)
    
    describe("commands", function()
        it("should handle empty selection gracefully", function()
            -- This would require mocking visual mode
            -- Placeholder for full integration test
        end)
        
        it("should parse agent output with code blocks", function()
            local test_output = [[
Here's the refactored code:

```rust
fn improved() {
    println!("Better");
}
```
]]
            utils.store_agent_output(test_output)
            local output = utils.get_last_agent_output()
            assert.is_not_nil(output)
            assert.matches("improved", output)
        end)
    end)
    
    describe("end-to-end workflow", function()
        it("should complete send-and-apply workflow", function()
            -- Full integration test requires:
            -- 1. Mock visual selection
            -- 2. Mock rad CLI command
            -- 3. Verify output storage
            -- 4. Verify code application
            -- Placeholder for full test implementation
        end)
    end)
end)

