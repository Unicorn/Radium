/**
 * Component Templates Module
 *
 * Provides reusable component templates as starting points
 * for new component creation.
 */

export * from './types';
export * from './library';

// Re-export built-in templates
export { emailSenderTemplate } from './built-in/email-sender';
export { webhookTemplate } from './built-in/webhook';
export { databaseQueryTemplate } from './built-in/database-query';
