# Simple Test Extension

A minimal extension for testing the extension system.

## Structure

```
test-extension-simple/
├── radium-extension.json
├── prompts/
│   └── test-agent.md
└── commands/
    └── test-command.toml
```

## Installation

```bash
rad extension install ./examples/extensions/test-extension-simple
```

## Testing

After installation:

1. List extensions: `rad extension list`
2. Show info: `rad extension info test-extension-simple`
3. Use command: `rad test-extension-simple:test-command`
4. Uninstall: `rad extension uninstall test-extension-simple`

