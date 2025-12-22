# Gregorio LSP Configuration

## Environment Variables

### `DISABLE_TREE_SITTER`

Controls whether the tree-sitter parser is used for parsing GABC files.

- **Type**: Boolean (string)
- **Default**: `false`
- **Values**: 
  - `"true"` - Disable tree-sitter, use TypeScript fallback parser only
  - `"false"` or unset - Use tree-sitter if available, fallback to TypeScript parser

**Example**:
```bash
# Disable tree-sitter
DISABLE_TREE_SITTER=true node dist/server.js

# Enable tree-sitter (default)
node dist/server.js
```

### When to Disable Tree-sitter

You should disable tree-sitter when:

1. **Bundling for VS Code extensions**: Tree-sitter has native dependencies that are difficult to bundle
2. **Testing the fallback parser**: Ensure the TypeScript parser works correctly
3. **Deployment environments**: Where tree-sitter may not be available or needed

## Programmatic Configuration

You can also disable tree-sitter programmatically by creating a new instance:

```typescript
import { TreeSitterParser } from './parser/tree-sitter-integration';

// Disable tree-sitter
const parser = new TreeSitterParser({ disabled: true });

// Check availability
console.log(parser.isTreeSitterAvailable()); // false
```

## Integration with VS Code Extension

When bundling the LSP server for a VS Code extension, set the environment variable before starting the server:

```typescript
// In your extension.ts
const serverOptions: ServerOptions = {
  run: { 
    module: serverPath, 
    transport: TransportKind.ipc,
    options: { 
      env: { ...process.env, DISABLE_TREE_SITTER: 'true' }
    }
  },
  debug: {
    module: serverPath,
    transport: TransportKind.ipc,
    options: { 
      execArgv: ['--nolazy', '--inspect=6009'],
      env: { ...process.env, DISABLE_TREE_SITTER: 'true' }
    }
  }
};
```
