/**
 * Error Logging and Monitoring Service
 * Centralizes error logging for server-side and client-side errors
 */

import type { WorkflowError } from './workflow-errors';

/**
 * Error severity levels
 */
export enum ErrorSeverity {
  DEBUG = 'debug',
  INFO = 'info',
  WARNING = 'warning',
  ERROR = 'error',
  CRITICAL = 'critical',
}

/**
 * Error log entry
 */
export interface ErrorLogEntry {
  timestamp: string;
  severity: ErrorSeverity;
  message: string;
  code?: string;
  userId?: string;
  sessionId?: string;
  context?: Record<string, any>;
  stack?: string;
  userAgent?: string;
  url?: string;
}

/**
 * Error logging interface
 */
export interface IErrorLogger {
  log(entry: ErrorLogEntry): void | Promise<void>;
  debug(message: string, context?: Record<string, any>): void;
  info(message: string, context?: Record<string, any>): void;
  warn(message: string, context?: Record<string, any>): void;
  error(message: string, context?: Record<string, any>): void;
  critical(message: string, context?: Record<string, any>): void;
  logWorkflowError(error: WorkflowError, context?: Record<string, any>): void;
}

/**
 * Console Error Logger (Development)
 */
export class ConsoleErrorLogger implements IErrorLogger {
  log(entry: ErrorLogEntry): void {
    const emoji = {
      [ErrorSeverity.DEBUG]: '🔍',
      [ErrorSeverity.INFO]: 'ℹ️',
      [ErrorSeverity.WARNING]: '⚠️',
      [ErrorSeverity.ERROR]: '❌',
      [ErrorSeverity.CRITICAL]: '🚨',
    }[entry.severity];

    const prefix = `${emoji} [${entry.severity.toUpperCase()}]`;
    const timestamp = new Date(entry.timestamp).toISOString();

    console.log(`${prefix} ${timestamp} - ${entry.message}`);

    if (entry.context) {
      console.log('Context:', entry.context);
    }

    if (entry.stack) {
      console.log('Stack:', entry.stack);
    }
  }

  debug(message: string, context?: Record<string, any>): void {
    this.log({
      timestamp: new Date().toISOString(),
      severity: ErrorSeverity.DEBUG,
      message,
      context,
    });
  }

  info(message: string, context?: Record<string, any>): void {
    this.log({
      timestamp: new Date().toISOString(),
      severity: ErrorSeverity.INFO,
      message,
      context,
    });
  }

  warn(message: string, context?: Record<string, any>): void {
    this.log({
      timestamp: new Date().toISOString(),
      severity: ErrorSeverity.WARNING,
      message,
      context,
    });
  }

  error(message: string, context?: Record<string, any>): void {
    this.log({
      timestamp: new Date().toISOString(),
      severity: ErrorSeverity.ERROR,
      message,
      context,
    });
  }

  critical(message: string, context?: Record<string, any>): void {
    this.log({
      timestamp: new Date().toISOString(),
      severity: ErrorSeverity.CRITICAL,
      message,
      context,
    });
  }

  logWorkflowError(error: WorkflowError, context?: Record<string, any>): void {
    const severity =
      error.statusCode >= 500 ? ErrorSeverity.ERROR : ErrorSeverity.WARNING;

    this.log({
      timestamp: new Date().toISOString(),
      severity,
      message: error.message,
      code: error.code,
      context: {
        ...context,
        ...error.toLogFormat(),
      },
      stack: error.stack,
    });
  }
}

/**
 * Database Error Logger (Production)
 * Logs errors to Supabase for monitoring and analysis
 */
export class DatabaseErrorLogger implements IErrorLogger {
  private supabase: any;
  private userId?: string;
  private sessionId?: string;

  constructor(supabase: any, userId?: string, sessionId?: string) {
    this.supabase = supabase;
    this.userId = userId;
    this.sessionId = sessionId;
  }

  async log(entry: ErrorLogEntry): Promise<void> {
    try {
      // Add user and session info
      const logEntry = {
        ...entry,
        userId: entry.userId || this.userId,
        sessionId: entry.sessionId || this.sessionId,
      };

      // Insert into error_logs table
      await this.supabase.from('error_logs').insert({
        timestamp: logEntry.timestamp,
        severity: logEntry.severity,
        message: logEntry.message,
        code: logEntry.code,
        user_id: logEntry.userId,
        session_id: logEntry.sessionId,
        context: logEntry.context,
        stack: logEntry.stack,
        user_agent: logEntry.userAgent,
        url: logEntry.url,
      });

      // Also log to console in development
      if (process.env.NODE_ENV === 'development') {
        const consoleLogger = new ConsoleErrorLogger();
        consoleLogger.log(entry);
      }
    } catch (error) {
      // Fallback to console if database logging fails
      console.error('Failed to log error to database:', error);
      console.error('Original error:', entry);
    }
  }

  debug(message: string, context?: Record<string, any>): void {
    void this.log({
      timestamp: new Date().toISOString(),
      severity: ErrorSeverity.DEBUG,
      message,
      context,
    });
  }

  info(message: string, context?: Record<string, any>): void {
    void this.log({
      timestamp: new Date().toISOString(),
      severity: ErrorSeverity.INFO,
      message,
      context,
    });
  }

  warn(message: string, context?: Record<string, any>): void {
    void this.log({
      timestamp: new Date().toISOString(),
      severity: ErrorSeverity.WARNING,
      message,
      context,
    });
  }

  error(message: string, context?: Record<string, any>): void {
    void this.log({
      timestamp: new Date().toISOString(),
      severity: ErrorSeverity.ERROR,
      message,
      context,
    });
  }

  critical(message: string, context?: Record<string, any>): void {
    void this.log({
      timestamp: new Date().toISOString(),
      severity: ErrorSeverity.CRITICAL,
      message,
      context,
    });
  }

  logWorkflowError(error: WorkflowError, context?: Record<string, any>): void {
    const severity =
      error.statusCode >= 500 ? ErrorSeverity.ERROR : ErrorSeverity.WARNING;

    void this.log({
      timestamp: new Date().toISOString(),
      severity,
      message: error.message,
      code: error.code,
      context: {
        ...context,
        ...error.toLogFormat(),
      },
      stack: error.stack,
    });
  }
}

/**
 * Composite Error Logger
 * Logs to multiple destinations (console + database)
 */
export class CompositeErrorLogger implements IErrorLogger {
  private loggers: IErrorLogger[];

  constructor(loggers: IErrorLogger[]) {
    this.loggers = loggers;
  }

  log(entry: ErrorLogEntry): void {
    for (const logger of this.loggers) {
      try {
        logger.log(entry);
      } catch (error) {
        console.error('Logger failed:', error);
      }
    }
  }

  debug(message: string, context?: Record<string, any>): void {
    for (const logger of this.loggers) {
      logger.debug(message, context);
    }
  }

  info(message: string, context?: Record<string, any>): void {
    for (const logger of this.loggers) {
      logger.info(message, context);
    }
  }

  warn(message: string, context?: Record<string, any>): void {
    for (const logger of this.loggers) {
      logger.warn(message, context);
    }
  }

  error(message: string, context?: Record<string, any>): void {
    for (const logger of this.loggers) {
      logger.error(message, context);
    }
  }

  critical(message: string, context?: Record<string, any>): void {
    for (const logger of this.loggers) {
      logger.critical(message, context);
    }
  }

  logWorkflowError(error: WorkflowError, context?: Record<string, any>): void {
    for (const logger of this.loggers) {
      logger.logWorkflowError(error, context);
    }
  }
}

/**
 * Global error logger instance
 */
let globalLogger: IErrorLogger = new ConsoleErrorLogger();

/**
 * Set the global error logger
 */
export function setGlobalErrorLogger(logger: IErrorLogger): void {
  globalLogger = logger;
}

/**
 * Get the global error logger
 */
export function getGlobalErrorLogger(): IErrorLogger {
  return globalLogger;
}

/**
 * Convenience functions for logging
 */
export const logger = {
  debug: (message: string, context?: Record<string, any>) =>
    globalLogger.debug(message, context),
  info: (message: string, context?: Record<string, any>) =>
    globalLogger.info(message, context),
  warn: (message: string, context?: Record<string, any>) =>
    globalLogger.warn(message, context),
  error: (message: string, context?: Record<string, any>) =>
    globalLogger.error(message, context),
  critical: (message: string, context?: Record<string, any>) =>
    globalLogger.critical(message, context),
  logWorkflowError: (error: WorkflowError, context?: Record<string, any>) =>
    globalLogger.logWorkflowError(error, context),
};

/**
 * Error metrics aggregation
 */
export interface ErrorMetrics {
  totalErrors: number;
  errorsByCode: Record<string, number>;
  errorsBySeverity: Record<ErrorSeverity, number>;
  errorsByHour: Record<string, number>;
  topErrors: Array<{ message: string; count: number }>;
}

/**
 * Calculate error metrics from logs
 */
export function calculateErrorMetrics(logs: ErrorLogEntry[]): ErrorMetrics {
  const metrics: ErrorMetrics = {
    totalErrors: logs.length,
    errorsByCode: {},
    errorsBySeverity: {
      [ErrorSeverity.DEBUG]: 0,
      [ErrorSeverity.INFO]: 0,
      [ErrorSeverity.WARNING]: 0,
      [ErrorSeverity.ERROR]: 0,
      [ErrorSeverity.CRITICAL]: 0,
    },
    errorsByHour: {},
    topErrors: [],
  };

  const messageCounts = new Map<string, number>();

  for (const log of logs) {
    // Count by code
    if (log.code) {
      metrics.errorsByCode[log.code] = (metrics.errorsByCode[log.code] || 0) + 1;
    }

    // Count by severity
    metrics.errorsBySeverity[log.severity]++;

    // Count by hour
    const hour = new Date(log.timestamp).toISOString().slice(0, 13);
    metrics.errorsByHour[hour] = (metrics.errorsByHour[hour] || 0) + 1;

    // Count by message
    messageCounts.set(log.message, (messageCounts.get(log.message) || 0) + 1);
  }

  // Get top errors
  metrics.topErrors = Array.from(messageCounts.entries())
    .map(([message, count]) => ({ message, count }))
    .sort((a, b) => b.count - a.count)
    .slice(0, 10);

  return metrics;
}
