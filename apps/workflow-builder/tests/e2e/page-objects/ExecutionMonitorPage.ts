/**
 * ExecutionMonitor Page Object Model
 *
 * Encapsulates interactions with the execution monitoring UI.
 * Used for viewing execution history, details, and statistics.
 */

import type { Page, Locator } from '@playwright/test';
import { expect } from '@playwright/test';

export interface ExecutionSummary {
  id: string;
  status: 'completed' | 'failed' | 'running' | 'pending';
  startTime?: string;
  duration?: string;
}

export interface WorkflowStatistics {
  totalRuns?: number;
  successRate?: number;
  averageDuration?: string;
  mostUsedComponent?: string;
}

export class ExecutionMonitorPage {
  readonly page: Page;
  readonly baseUrl: string;

  // Main UI Elements
  readonly executionHistoryTab: Locator;
  readonly statisticsTab: Locator;
  readonly executionList: Locator;
  readonly executionDetailView: Locator;

  // Statistics Elements
  readonly totalRunsStat: Locator;
  readonly successRateStat: Locator;
  readonly averageDurationStat: Locator;

  // Execution Detail Elements
  readonly executionStatus: Locator;
  readonly executionDuration: Locator;
  readonly executionOutput: Locator;
  readonly executionError: Locator;
  readonly componentExecutions: Locator;

  constructor(page: Page, baseUrl: string = 'http://localhost:3010') {
    this.page = page;
    this.baseUrl = baseUrl;

    // Initialize locators
    this.executionHistoryTab = page.getByRole('tab', { name: /execution history/i });
    this.statisticsTab = page.getByRole('tab', { name: /statistics/i });
    this.executionList = page.locator('[data-testid="execution-history-list"]');
    this.executionDetailView = page.locator('[data-testid="execution-detail-view"]');

    // Statistics
    this.totalRunsStat = page.locator('[data-testid="total-runs"]').or(
      page.locator('text=/total runs/i').locator('..')
    );
    this.successRateStat = page.locator('[data-testid="success-rate"]').or(
      page.locator('text=/success rate/i').locator('..')
    );
    this.averageDurationStat = page.locator('[data-testid="average-duration"]').or(
      page.locator('text=/average duration/i').locator('..')
    );

    // Execution Details
    this.executionStatus = page.locator('[data-testid="execution-status"]');
    this.executionDuration = page.locator('[data-testid="execution-duration"]');
    this.executionOutput = page.locator('[data-testid="execution-output"]');
    this.executionError = page.locator('[data-testid="execution-error"]');
    this.componentExecutions = page.locator('[data-testid="component-executions"]');
  }

  /**
   * Navigate to workflow execution history tab
   */
  async navigateToExecutionHistory(workflowId: string): Promise<void> {
    await this.page.goto(`${this.baseUrl}/workflows/${workflowId}`);
    await this.page.waitForLoadState('networkidle');

    // Click on Execution History tab
    await this.executionHistoryTab.click();
    await this.page.waitForLoadState('networkidle');
  }

  /**
   * Navigate to workflow statistics tab
   */
  async navigateToStatistics(workflowId: string): Promise<void> {
    await this.page.goto(`${this.baseUrl}/workflows/${workflowId}`);
    await this.page.waitForLoadState('networkidle');

    // Click on Statistics tab
    await this.statisticsTab.click();
    await this.page.waitForLoadState('networkidle');
  }

  /**
   * Get list of executions from history
   */
  async getExecutionList(): Promise<ExecutionSummary[]> {
    const executionItems = this.page.locator('[data-testid="execution-item"]');
    const count = await executionItems.count();

    const executions: ExecutionSummary[] = [];

    for (let i = 0; i < count; i++) {
      const item = executionItems.nth(i);

      const id = await item.getAttribute('data-execution-id') || `exec-${i}`;
      const statusText = await item.locator('[data-testid="execution-status"]').textContent();
      const status = this.parseStatus(statusText || '');

      const startTimeElement = item.locator('[data-testid="execution-start-time"]');
      const startTime = await startTimeElement.count() > 0
        ? await startTimeElement.textContent()
        : undefined;

      const durationElement = item.locator('[data-testid="execution-duration"]');
      const duration = await durationElement.count() > 0
        ? await durationElement.textContent()
        : undefined;

      executions.push({
        id,
        status,
        startTime: startTime || undefined,
        duration: duration || undefined,
      });
    }

    return executions;
  }

  /**
   * Click on an execution to view details
   */
  async viewExecutionDetails(executionId: string): Promise<void> {
    const executionItem = this.page.locator(`[data-execution-id="${executionId}"]`);
    await expect(executionItem).toBeVisible();
    await executionItem.click();
    await this.page.waitForLoadState('networkidle');

    // Verify detail view loaded
    await expect(this.executionDetailView.or(
      this.page.locator('text=/execution details/i')
    ).first()).toBeVisible({ timeout: 5000 });
  }

  /**
   * Get execution status from detail view
   */
  async getExecutionStatus(): Promise<'completed' | 'failed' | 'running' | 'pending'> {
    const statusText = await this.executionStatus.textContent();
    return this.parseStatus(statusText || '');
  }

  /**
   * Get execution duration from detail view
   */
  async getExecutionDuration(): Promise<string | null> {
    if (await this.executionDuration.count() === 0) {
      return null;
    }
    return await this.executionDuration.textContent();
  }

  /**
   * Get execution output from detail view
   */
  async getExecutionOutput(): Promise<any> {
    if (await this.executionOutput.count() === 0) {
      return null;
    }

    const outputText = await this.executionOutput.textContent();
    if (!outputText) {
      return null;
    }

    try {
      return JSON.parse(outputText);
    } catch {
      return outputText;
    }
  }

  /**
   * Get execution error from detail view
   */
  async getExecutionErrorMessage(): Promise<string | null> {
    if (await this.executionError.count() === 0) {
      return null;
    }
    return await this.executionError.textContent();
  }

  /**
   * Get component executions from detail view
   */
  async getComponentExecutions(): Promise<any[]> {
    if (await this.componentExecutions.count() === 0) {
      return [];
    }

    const componentItems = this.componentExecutions.locator('[data-testid="component-execution-item"]');
    const count = await componentItems.count();

    const components: any[] = [];
    for (let i = 0; i < count; i++) {
      const item = componentItems.nth(i);
      const name = await item.locator('[data-testid="component-name"]').textContent();
      const status = await item.locator('[data-testid="component-status"]').textContent();
      const duration = await item.locator('[data-testid="component-duration"]').textContent();

      components.push({ name, status, duration });
    }

    return components;
  }

  /**
   * Navigate back to execution history from detail view
   */
  async backToHistory(): Promise<void> {
    const backButton = this.page.getByRole('button', { name: /back to history/i });
    await expect(backButton).toBeVisible();
    await backButton.click();
    await this.page.waitForLoadState('networkidle');
  }

  /**
   * Get workflow statistics
   */
  async getStatistics(): Promise<WorkflowStatistics> {
    const stats: WorkflowStatistics = {};

    // Extract total runs
    if (await this.totalRunsStat.count() > 0) {
      const totalRunsText = await this.totalRunsStat.textContent();
      const match = totalRunsText?.match(/\d+/);
      if (match) {
        stats.totalRuns = parseInt(match[0], 10);
      }
    }

    // Extract success rate
    if (await this.successRateStat.count() > 0) {
      const successRateText = await this.successRateStat.textContent();
      const match = successRateText?.match(/(\d+(?:\.\d+)?)/);
      if (match) {
        stats.successRate = parseFloat(match[1]);
      }
    }

    // Extract average duration
    if (await this.averageDurationStat.count() > 0) {
      const durationText = await this.averageDurationStat.textContent();
      stats.averageDuration = durationText || undefined;
    }

    // Extract most used component
    const mostUsedComponent = this.page.locator('[data-testid="most-used-component"]');
    if (await mostUsedComponent.count() > 0) {
      stats.mostUsedComponent = await mostUsedComponent.textContent() || undefined;
    }

    return stats;
  }

  /**
   * Check if execution history is empty
   */
  async isExecutionHistoryEmpty(): Promise<boolean> {
    const emptyState = this.page.locator('text=/no executions found/i');
    return await emptyState.count() > 0;
  }

  /**
   * Wait for execution to appear in history
   */
  async waitForExecutionInHistory(
    executionId: string,
    timeout: number = 10000
  ): Promise<void> {
    const executionItem = this.page.locator(`[data-execution-id="${executionId}"]`);
    await expect(executionItem).toBeVisible({ timeout });
  }

  /**
   * Verify statistics tab is visible
   */
  async verifyStatisticsTabVisible(): Promise<void> {
    await expect(this.statisticsTab).toBeVisible();
  }

  /**
   * Verify execution history tab is visible
   */
  async verifyExecutionHistoryTabVisible(): Promise<void> {
    await expect(this.executionHistoryTab).toBeVisible();
  }

  /**
   * Parse status text to typed status
   */
  private parseStatus(statusText: string): 'completed' | 'failed' | 'running' | 'pending' {
    const lower = statusText.toLowerCase();
    if (lower.includes('completed')) return 'completed';
    if (lower.includes('failed')) return 'failed';
    if (lower.includes('running')) return 'running';
    return 'pending';
  }

  /**
   * Retry a failed execution
   */
  async retryExecution(): Promise<void> {
    const retryButton = this.page.getByRole('button', { name: /retry|run again/i });
    await expect(retryButton).toBeVisible();
    await retryButton.click();

    // Wait for new execution to start
    await this.page.waitForTimeout(1000);
  }

  /**
   * Check if retry button is available
   */
  async hasRetryButton(): Promise<boolean> {
    const retryButton = this.page.getByRole('button', { name: /retry|run again/i });
    return await retryButton.count() > 0;
  }

  /**
   * Get pagination info (if exists)
   */
  async getPaginationInfo(): Promise<{ currentPage: number; totalPages: number } | null> {
    const paginationElement = this.page.locator('[data-testid="pagination"]');

    if (await paginationElement.count() === 0) {
      return null;
    }

    const currentPageText = await paginationElement.locator('[data-testid="current-page"]').textContent();
    const totalPagesText = await paginationElement.locator('[data-testid="total-pages"]').textContent();

    return {
      currentPage: parseInt(currentPageText || '1', 10),
      totalPages: parseInt(totalPagesText || '1', 10),
    };
  }

  /**
   * Navigate to next page of execution history
   */
  async goToNextPage(): Promise<void> {
    const nextButton = this.page.getByRole('button', { name: /next/i });
    await expect(nextButton).toBeVisible();
    await expect(nextButton).toBeEnabled();
    await nextButton.click();
    await this.page.waitForLoadState('networkidle');
  }

  /**
   * Navigate to previous page of execution history
   */
  async goToPreviousPage(): Promise<void> {
    const prevButton = this.page.getByRole('button', { name: /previous|prev/i });
    await expect(prevButton).toBeVisible();
    await expect(prevButton).toBeEnabled();
    await prevButton.click();
    await this.page.waitForLoadState('networkidle');
  }
}
