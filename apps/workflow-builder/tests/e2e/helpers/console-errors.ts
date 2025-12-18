/**
 * Console Error Capture Helper
 * 
 * Captures console errors and page errors during Playwright tests.
 * Can be configured to fail tests on errors or just log them.
 */

import { Page } from '@playwright/test';

export interface ConsoleError {
  type: 'console' | 'pageerror';
  message: string;
  timestamp: Date;
  stack?: string;
  args?: any[];
}

export class ConsoleErrorCollector {
  private errors: ConsoleError[] = [];
  private failOnError: boolean;

  constructor(failOnError: boolean = false) {
    this.failOnError = failOnError;
  }

  /**
   * Attach error listeners to a page
   */
  attachToPage(page: Page) {
    // Capture console errors
    page.on('console', (msg) => {
      const type = msg.type();
      if (type === 'error') {
        const error: ConsoleError = {
          type: 'console',
          message: msg.text(),
          timestamp: new Date(),
          args: msg.args().map(arg => arg.toString()),
        };
        this.errors.push(error);
        
        if (this.failOnError) {
          throw new Error(`Console error: ${error.message}`);
        }
      }
    });

    // Capture unhandled page errors
    page.on('pageerror', (error) => {
      const pageError: ConsoleError = {
        type: 'pageerror',
        message: error.message,
        timestamp: new Date(),
        stack: error.stack,
      };
      this.errors.push(pageError);
      
      if (this.failOnError) {
        throw error;
      }
    });

    // Capture unhandled promise rejections
    page.on('requestfailed', (request) => {
      const error: ConsoleError = {
        type: 'pageerror',
        message: `Request failed: ${request.url()} - ${request.failure()?.errorText || 'Unknown error'}`,
        timestamp: new Date(),
      };
      this.errors.push(error);
      
      if (this.failOnError) {
        throw new Error(error.message);
      }
    });
  }

  /**
   * Get all collected errors
   */
  getErrors(): ConsoleError[] {
    return [...this.errors];
  }

  /**
   * Clear collected errors
   */
  clear() {
    this.errors = [];
  }

  /**
   * Check if any errors were captured
   */
  hasErrors(): boolean {
    return this.errors.length > 0;
  }

  /**
   * Get formatted error summary
   */
  getSummary(): string {
    if (this.errors.length === 0) {
      return 'No console errors captured.';
    }

    const consoleErrors = this.errors.filter(e => e.type === 'console');
    const pageErrors = this.errors.filter(e => e.type === 'pageerror');

    let summary = `Captured ${this.errors.length} error(s):\n`;
    
    if (consoleErrors.length > 0) {
      summary += `\nConsole Errors (${consoleErrors.length}):\n`;
      consoleErrors.forEach((error, i) => {
        summary += `  ${i + 1}. ${error.message}\n`;
      });
    }

    if (pageErrors.length > 0) {
      summary += `\nPage Errors (${pageErrors.length}):\n`;
      pageErrors.forEach((error, i) => {
        summary += `  ${i + 1}. ${error.message}\n`;
        if (error.stack) {
          summary += `     Stack: ${error.stack.split('\n')[0]}\n`;
        }
      });
    }

    return summary;
  }
}

/**
 * Setup console error capture for a test
 * 
 * @param page - Playwright page object
 * @param failOnError - If true, tests will fail when errors are detected
 * @returns ConsoleErrorCollector instance
 */
export function setupConsoleErrorCapture(
  page: Page,
  failOnError: boolean = false
): ConsoleErrorCollector {
  const collector = new ConsoleErrorCollector(failOnError);
  collector.attachToPage(page);
  return collector;
}

