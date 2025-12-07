# Complex Test Extension

A comprehensive extension for testing all extension system features.

## Structure

```
test-extension-complex/
├── radium-extension.json
├── agents/
│   ├── agent1.toml
│   └── agent2.toml
├── templates/
│   ├── template1.json
│   └── template2.json
├── commands/
│   ├── command1.toml
│   └── deploy/
│       └── deploy.toml
├── prompts/
│   ├── prompt1.md
│   └── frameworks/
│       └── framework1.md
└── mcp/
    ├── server1.json
    └── server2.json
```

## Dependencies

This extension depends on `test-extension-simple`. Install it first:

```bash
rad extension install ./examples/extensions/test-extension-simple
rad extension install ./examples/extensions/test-extension-complex --install-deps
```

## Testing

This extension tests:

- Multiple component types
- Glob patterns in component declarations
- Nested directory structures
- Extension dependencies
- Component discovery and integration

## See Also

- [Extension System Guide](../../../docs/guides/extension-system.md)
- [Creating Extensions](../../../docs/extensions/creating-extensions.md)

