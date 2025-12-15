/**
 * Webhook Template
 *
 * Template for creating webhook sending components.
 */

import type { ComponentTemplate } from '../types';

export const webhookTemplate: ComponentTemplate = {
  id: 'webhook',
  name: 'Webhook Sender',
  description: 'Send HTTP webhooks to external services with retry support',
  category: 'integration',
  version: '1.0.0',
  author: 'Workflow Builder Team',
  complexity: 'simple',

  inputSchema: {
    fields: [
      {
        name: 'url',
        rustType: 'String',
        typescriptType: 'string',
        required: true,
        customizable: false,
        description: 'Webhook URL endpoint',
        validation: '#[validate(url)]',
      },
      {
        name: 'method',
        rustType: 'HttpMethod',
        typescriptType: "'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE'",
        required: false,
        customizable: true,
        description: 'HTTP method',
        default: 'POST',
      },
      {
        name: 'headers',
        rustType: 'HashMap<String, String>',
        typescriptType: 'Record<string, string>',
        required: false,
        customizable: true,
        description: 'Custom HTTP headers',
        default: '{}',
      },
      {
        name: 'payload',
        rustType: 'serde_json::Value',
        typescriptType: 'unknown',
        required: false,
        customizable: true,
        description: 'JSON payload to send',
      },
      {
        name: 'timeout_ms',
        rustType: 'u64',
        typescriptType: 'number',
        required: false,
        customizable: true,
        description: 'Request timeout in milliseconds',
        default: '30000',
      },
      {
        name: 'retry_count',
        rustType: 'u32',
        typescriptType: 'number',
        required: false,
        customizable: true,
        description: 'Number of retry attempts on failure',
        default: '3',
      },
      {
        name: 'secret',
        rustType: 'Option<String>',
        typescriptType: 'string | undefined',
        required: false,
        customizable: true,
        description: 'Secret for webhook signature (HMAC-SHA256)',
      },
    ],
    customizable: ['method', 'headers', 'timeout_ms', 'retry_count', 'secret'],
  },

  outputSchema: {
    fields: [
      {
        name: 'success',
        rustType: 'bool',
        typescriptType: 'boolean',
        required: true,
        customizable: false,
        description: 'Whether the webhook was delivered successfully',
      },
      {
        name: 'status_code',
        rustType: 'u16',
        typescriptType: 'number',
        required: true,
        customizable: false,
        description: 'HTTP response status code',
      },
      {
        name: 'response_body',
        rustType: 'Option<serde_json::Value>',
        typescriptType: 'unknown | undefined',
        required: false,
        customizable: false,
        description: 'Response body if any',
      },
      {
        name: 'duration_ms',
        rustType: 'u64',
        typescriptType: 'number',
        required: true,
        customizable: false,
        description: 'Request duration in milliseconds',
      },
      {
        name: 'attempt_count',
        rustType: 'u32',
        typescriptType: 'number',
        required: true,
        customizable: false,
        description: 'Number of attempts made',
      },
    ],
    customizable: [],
  },

  validationRules: [
    {
      field: 'url',
      ruleType: 'format',
      rule: 'url',
      customizable: false,
      errorMessage: 'Invalid URL format',
    },
    {
      field: 'timeout_ms',
      ruleType: 'range',
      rule: 'min=1000,max=300000',
      customizable: true,
      errorMessage: 'Timeout must be between 1 and 300 seconds',
    },
    {
      field: 'retry_count',
      ruleType: 'range',
      rule: 'min=0,max=10',
      customizable: true,
      errorMessage: 'Retry count must be between 0 and 10',
    },
  ],

  exampleUsage: `// Simple webhook
const result = await activities.sendWebhook({
  url: 'https://api.example.com/webhook',
  payload: {
    event: 'user.created',
    data: { userId: '123', email: 'user@example.com' },
  },
});

// Webhook with authentication
const result = await activities.sendWebhook({
  url: 'https://api.example.com/webhook',
  headers: {
    'Authorization': 'Bearer ' + token,
  },
  payload: eventData,
  secret: process.env.WEBHOOK_SECRET,
});`,

  tags: ['webhook', 'http', 'api', 'integration', 'notification'],
  icon: 'send',
  customizable: true,
  customizableFields: ['method', 'headers', 'timeout_ms', 'retry_count', 'secret'],
  dependencies: ['http-client'],
};
