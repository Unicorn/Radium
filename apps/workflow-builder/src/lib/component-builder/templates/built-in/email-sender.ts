/**
 * Email Sender Template
 *
 * Template for creating email sending components.
 */

import type { ComponentTemplate } from '../types';

export const emailSenderTemplate: ComponentTemplate = {
  id: 'email-sender',
  name: 'Email Sender',
  description: 'Send emails via SMTP with HTML template support and attachments',
  category: 'communication',
  version: '1.0.0',
  author: 'Workflow Builder Team',
  complexity: 'moderate',

  inputSchema: {
    fields: [
      {
        name: 'to',
        rustType: 'String',
        typescriptType: 'string',
        required: true,
        customizable: false,
        description: 'Recipient email address',
        validation: '#[validate(email)]',
      },
      {
        name: 'cc',
        rustType: 'Vec<String>',
        typescriptType: 'string[]',
        required: false,
        customizable: true,
        description: 'CC recipients',
        default: '[]',
      },
      {
        name: 'bcc',
        rustType: 'Vec<String>',
        typescriptType: 'string[]',
        required: false,
        customizable: true,
        description: 'BCC recipients',
        default: '[]',
      },
      {
        name: 'subject',
        rustType: 'String',
        typescriptType: 'string',
        required: true,
        customizable: false,
        description: 'Email subject line',
        validation: '#[validate(length(min = 1, max = 998))]',
      },
      {
        name: 'body',
        rustType: 'String',
        typescriptType: 'string',
        required: true,
        customizable: false,
        description: 'Email body content (HTML supported)',
      },
      {
        name: 'from',
        rustType: 'Option<String>',
        typescriptType: 'string | undefined',
        required: false,
        customizable: true,
        description: 'Sender email (uses default if not specified)',
        validation: '#[validate(email)]',
      },
      {
        name: 'reply_to',
        rustType: 'Option<String>',
        typescriptType: 'string | undefined',
        required: false,
        customizable: true,
        description: 'Reply-to email address',
      },
      {
        name: 'attachments',
        rustType: 'Vec<EmailAttachment>',
        typescriptType: 'EmailAttachment[]',
        required: false,
        customizable: true,
        description: 'File attachments',
        default: '[]',
      },
    ],
    customizable: ['cc', 'bcc', 'from', 'reply_to', 'attachments'],
  },

  outputSchema: {
    fields: [
      {
        name: 'success',
        rustType: 'bool',
        typescriptType: 'boolean',
        required: true,
        customizable: false,
        description: 'Whether the email was sent successfully',
      },
      {
        name: 'message_id',
        rustType: 'Option<String>',
        typescriptType: 'string | undefined',
        required: false,
        customizable: false,
        description: 'SMTP message ID',
      },
      {
        name: 'timestamp',
        rustType: 'String',
        typescriptType: 'string',
        required: true,
        customizable: false,
        description: 'ISO timestamp when email was sent',
      },
      {
        name: 'error',
        rustType: 'Option<String>',
        typescriptType: 'string | undefined',
        required: false,
        customizable: false,
        description: 'Error message if send failed',
      },
    ],
    customizable: [],
  },

  validationRules: [
    {
      field: 'to',
      ruleType: 'format',
      rule: 'email',
      customizable: false,
      errorMessage: 'Invalid email address format',
    },
    {
      field: 'subject',
      ruleType: 'length',
      rule: 'min=1,max=998',
      customizable: true,
      errorMessage: 'Subject must be between 1 and 998 characters',
    },
  ],

  exampleUsage: `// Send a simple email
const result = await activities.sendEmail({
  to: 'user@example.com',
  subject: 'Welcome to our platform!',
  body: '<h1>Hello!</h1><p>Welcome aboard.</p>',
});

// Send with attachments
const result = await activities.sendEmail({
  to: 'user@example.com',
  cc: ['manager@example.com'],
  subject: 'Monthly Report',
  body: '<p>Please find the report attached.</p>',
  attachments: [
    {
      filename: 'report.pdf',
      content: base64EncodedContent,
      contentType: 'application/pdf',
    },
  ],
});`,

  tags: ['email', 'smtp', 'communication', 'notification', 'attachment'],
  icon: 'mail',
  customizable: true,
  customizableFields: ['cc', 'bcc', 'from', 'reply_to', 'attachments'],
  dependencies: ['smtp-client'],
};
