# Clipboard Mode Guide

Universal editor support via clipboard operations - works with any editor without extensions.

## Overview

Clipboard mode provides a fallback integration method that works with any editor through simple copy/paste operations. It's ideal when:

- Your editor doesn't have a Radium extension
- You want a lightweight integration
- You need cross-editor compatibility
- Extension installation is not possible

## How It Works

1. **Copy code** with optional file path annotation
2. **Run `rad clipboard send`** to process with Radium
3. **Copy result** from output
4. **Paste back** into your editor

## Installation

No installation required! Just ensure Radium CLI is in your PATH:

```bash
rad --version  # Verify CLI is installed
```

## Usage

### Sending Code to Radium

#### Step 1: Copy Code with Annotation (Optional)

Add a file path annotation to your code when copying:

**For C-style languages** (Rust, C, Java, JavaScript, etc.):
```rust
// File: src/main.rs
fn main() {
    println!("Hello, world!");
}
```

**For hash-comment languages** (Python, Ruby, Shell, YAML, etc.):
```python
# File: main.py
def main():
    print("Hello, world!")
```

**For HTML/XML:**
```html
<!-- File: index.html -->
<html>
  <body>Hello</body>
</html>
```

#### Step 2: Send to Radium

```bash
rad clipboard send
```

This will:
- Read from clipboard
- Parse file path and language (if annotated)
- Detect language automatically (if not annotated)
- Prepare context for Radium processing

**Output:**
```
rad clipboard send

  • Reading from clipboard...
  ✓ Content parsed
    File path: src/main.rs
    Language: rust

  • Sending to Radium...

Context:
{
  "file_path": "src/main.rs",
  "language": "rust",
  "selection": "fn main() {\n    println!(\"Hello, world!\");\n}",
  "surrounding_lines": ""
}

✓ Processed code from clipboard and prepared for Radium.
  Use 'rad step <agent-id>' with this context to process the code.
```

### Receiving Code from Radium

```bash
rad clipboard receive
```

Formats the last agent output and writes to clipboard with file path annotation for easy pasting.

## Language Detection

If no file path annotation is provided, Radium attempts to detect language from:

1. **Shebang** (`#!/usr/bin/env python3`)
2. **Code patterns**:
   - `fn main()` → Rust
   - `def ` + `import ` → Python
   - `function` + `=>` → JavaScript
   - `package ` + `func ` → Go

## File Path Annotation Format

The annotation format supports several comment styles:

| Language Type | Format | Example |
|--------------|--------|---------|
| C-style | `// File: path` | `// File: src/main.rs` |
| Hash | `# File: path` | `# File: main.py` |
| HTML/XML | `<!-- File: path -->` | `<!-- File: index.html -->` |

**Notes:**
- Annotation must be on the first line of copied content
- Path can be relative or absolute
- Language is auto-detected from file extension if path provided

## Workflow Examples

### Example 1: Quick Refactor

1. Copy code with annotation:
   ```
   // File: utils.rs
   pub fn old_name() {}
   ```

2. Send to Radium:
   ```bash
   rad clipboard send
   rad step refactor-agent "Rename function to new_name"
   ```

3. Copy result from Radium output

4. Paste into editor

### Example 2: Multi-file Context

1. Copy code from multiple files sequentially
2. Each copy includes file annotation
3. Send each to Radium with context

### Example 3: Any Editor Workflow

1. **In Vim/Emacs/Any Editor:**
   - Select code
   - Copy with annotation comment
   
2. **Terminal:**
   ```bash
   rad clipboard send
   rad step code-agent "Add error handling"
   ```

3. **Back in Editor:**
   - Copy result from terminal
   - Paste into file

## Integration with rad step

Clipboard mode works seamlessly with `rad step`:

```bash
# Read from clipboard and process
rad clipboard send | rad step code-agent

# Or process with specific context
rad step code-agent --context "$(rad clipboard send)"
```

## Limitations

- No automatic context injection (must annotate manually)
- No diff preview (compare manually)
- No chat integration (use `rad chat` separately)
- Manual copy/paste required

## Tips

1. **Create Editor Macros**: Set up keyboard shortcuts in your editor to add file path annotations automatically
2. **Use Aliases**: Create shell aliases for common workflows
   ```bash
   alias rad-copy='rad clipboard send | rad step code-agent'
   ```
3. **Template Comments**: Keep annotation templates in a snippet/abbreviation system

## Troubleshooting

### Clipboard Empty

**Issue:** "Clipboard is empty" error

**Solution:**
- Ensure code is copied before running command
- Test clipboard access: `rad clipboard send` (should show content)
- Check system clipboard permissions

### Language Not Detected

**Issue:** Language shows as "unknown"

**Solution:**
- Add file path annotation with extension
- Use explicit language hints in annotation
- Check code patterns match known languages

### File Path Not Parsed

**Issue:** File path annotation not recognized

**Solution:**
- Ensure annotation is on first line
- Use correct comment style for language
- Format: `// File: path` or `# File: path` or `<!-- File: path -->`

## See Also

- [Overview](./overview.md) - Comparison of integration methods
- [Architecture](./architecture.md) - Technical details
- [Troubleshooting](./troubleshooting.md) - Common issues

