/**
 * Database Query Template
 *
 * Template for creating database query components.
 */

import type { ComponentTemplate } from '../types';

export const databaseQueryTemplate: ComponentTemplate = {
  id: 'database-query',
  name: 'Database Query',
  description: 'Execute SQL queries against PostgreSQL databases with parameterized queries',
  category: 'data',
  version: '1.0.0',
  author: 'Workflow Builder Team',
  complexity: 'moderate',

  inputSchema: {
    fields: [
      {
        name: 'query',
        rustType: 'String',
        typescriptType: 'string',
        required: true,
        customizable: false,
        description: 'SQL query to execute (use $1, $2, etc. for parameters)',
        validation: '#[validate(length(min = 1))]',
      },
      {
        name: 'parameters',
        rustType: 'Vec<serde_json::Value>',
        typescriptType: 'unknown[]',
        required: false,
        customizable: true,
        description: 'Query parameters (positional)',
        default: '[]',
      },
      {
        name: 'connection_id',
        rustType: 'String',
        typescriptType: 'string',
        required: true,
        customizable: false,
        description: 'Database connection identifier',
      },
      {
        name: 'timeout_ms',
        rustType: 'u64',
        typescriptType: 'number',
        required: false,
        customizable: true,
        description: 'Query timeout in milliseconds',
        default: '30000',
      },
      {
        name: 'fetch_size',
        rustType: 'Option<u32>',
        typescriptType: 'number | undefined',
        required: false,
        customizable: true,
        description: 'Maximum rows to fetch (null for unlimited)',
      },
      {
        name: 'read_only',
        rustType: 'bool',
        typescriptType: 'boolean',
        required: false,
        customizable: true,
        description: 'Execute in read-only transaction',
        default: 'false',
      },
    ],
    customizable: ['parameters', 'timeout_ms', 'fetch_size', 'read_only'],
  },

  outputSchema: {
    fields: [
      {
        name: 'rows',
        rustType: 'Vec<serde_json::Value>',
        typescriptType: 'Record<string, unknown>[]',
        required: true,
        customizable: false,
        description: 'Query result rows',
      },
      {
        name: 'row_count',
        rustType: 'u64',
        typescriptType: 'number',
        required: true,
        customizable: false,
        description: 'Number of rows returned/affected',
      },
      {
        name: 'columns',
        rustType: 'Vec<String>',
        typescriptType: 'string[]',
        required: true,
        customizable: false,
        description: 'Column names in result set',
      },
      {
        name: 'duration_ms',
        rustType: 'u64',
        typescriptType: 'number',
        required: true,
        customizable: false,
        description: 'Query execution time in milliseconds',
      },
      {
        name: 'truncated',
        rustType: 'bool',
        typescriptType: 'boolean',
        required: true,
        customizable: false,
        description: 'Whether results were truncated by fetch_size',
      },
    ],
    customizable: [],
  },

  validationRules: [
    {
      field: 'query',
      ruleType: 'length',
      rule: 'min=1,max=100000',
      customizable: false,
      errorMessage: 'Query must not be empty',
    },
    {
      field: 'timeout_ms',
      ruleType: 'range',
      rule: 'min=1000,max=600000',
      customizable: true,
      errorMessage: 'Timeout must be between 1 second and 10 minutes',
    },
    {
      field: 'fetch_size',
      ruleType: 'range',
      rule: 'min=1,max=100000',
      customizable: true,
      errorMessage: 'Fetch size must be between 1 and 100,000',
    },
  ],

  exampleUsage: `// Simple SELECT query
const result = await activities.databaseQuery({
  connectionId: 'main-db',
  query: 'SELECT id, name, email FROM users WHERE active = $1',
  parameters: [true],
});

// INSERT with returning
const result = await activities.databaseQuery({
  connectionId: 'main-db',
  query: \`
    INSERT INTO orders (user_id, total, status)
    VALUES ($1, $2, $3)
    RETURNING id, created_at
  \`,
  parameters: [userId, total, 'pending'],
});

// Paginated query
const result = await activities.databaseQuery({
  connectionId: 'main-db',
  query: 'SELECT * FROM products ORDER BY created_at DESC',
  fetchSize: 100,
  readOnly: true,
});`,

  tags: ['database', 'sql', 'postgresql', 'query', 'data'],
  icon: 'database',
  customizable: true,
  customizableFields: ['parameters', 'timeout_ms', 'fetch_size', 'read_only'],
  dependencies: ['database-pool'],
};
