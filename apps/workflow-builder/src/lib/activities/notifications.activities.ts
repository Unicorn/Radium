/**
 * Notification Activities
 * 
 * Temporal activities for notifications and communication used by the UI component
 * for user feedback, error alerts, and workflow status updates.
 * 
 * @see plans/ui-compiler-activities.md for comprehensive requirements
 * @see plans/package-builder/future/activities/notification-communication.md for detailed specifications
 */

import { randomUUID } from 'crypto';

// ============================================================================
// Type Definitions
// ============================================================================

export interface SlackNotificationInput {
  webhookUrl: string;
  message: string;
  channel?: string;
  username?: string;
  iconEmoji?: string;
  attachments?: Array<{
    color?: string;
    title?: string;
    text?: string;
    fields?: Array<{
      title: string;
      value: string;
      short?: boolean;
    }>;
  }>;
}

export interface SlackNotificationResult {
  success: boolean;
  messageId?: string;
  error?: string;
}

export interface StatusUpdateInput {
  executionId: string;
  status: 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';
  message?: string;
  progress?: number; // 0-100
  metadata?: Record<string, any>;
}

export interface StatusUpdateResult {
  success: boolean;
  updated: boolean;
  error?: string;
}

export interface ErrorAlertInput {
  executionId: string;
  error: string;
  severity?: 'low' | 'medium' | 'high' | 'critical';
  context?: Record<string, any>;
  notifySlack?: boolean;
  slackWebhookUrl?: string;
}

export interface ErrorAlertResult {
  success: boolean;
  alertId: string;
  slackNotified?: boolean;
  error?: string;
}

export interface ProgressUpdateInput {
  executionId: string;
  step: string;
  progress: number; // 0-100
  message?: string;
  estimatedTimeRemaining?: number; // milliseconds
}

export interface ProgressUpdateResult {
  success: boolean;
  updated: boolean;
  error?: string;
}

// ============================================================================
// Notification Activities
// ============================================================================

/**
 * Send a Slack notification via webhook
 */
export async function sendSlackNotification(
  input: SlackNotificationInput
): Promise<SlackNotificationResult> {
  const {
    webhookUrl,
    message,
    channel,
    username = 'Workflow Builder',
    iconEmoji = ':robot_face:',
    attachments = [],
  } = input;

  try {
    const payload: any = {
      text: message,
      username,
      icon_emoji: iconEmoji,
    };

    if (channel) {
      payload.channel = channel;
    }

    if (attachments.length > 0) {
      payload.attachments = attachments;
    }

    const response = await fetch(webhookUrl, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(payload),
    });

    if (!response.ok) {
      const errorText = await response.text();
      return {
        success: false,
        error: `Slack API error: ${response.status} ${response.statusText} - ${errorText}`,
      };
    }

    const messageId = randomUUID();
    return {
      success: true,
      messageId,
    };
  } catch (error: any) {
    return {
      success: false,
      error: error.message || 'Failed to send Slack notification',
    };
  }
}

/**
 * Update workflow execution status in the database
 * 
 * Note: This activity assumes the database update logic is handled by the calling workflow
 * or through a separate database activity. This is a placeholder that can be extended
 * to directly update Supabase if needed.
 */
export async function updateWorkflowStatus(
  input: StatusUpdateInput
): Promise<StatusUpdateResult> {
  const {
    executionId,
    status,
    message,
    progress,
    metadata = {},
  } = input;

  try {
    // In a real implementation, this would update the database
    // For now, we'll just validate the input and return success
    // The actual database update should be handled by the workflow or a separate activity
    
    if (!executionId) {
      return {
        success: false,
        updated: false,
        error: 'executionId is required',
      };
    }

    const validStatuses = ['pending', 'running', 'completed', 'failed', 'cancelled'];
    if (!validStatuses.includes(status)) {
      return {
        success: false,
        updated: false,
        error: `Invalid status: ${status}. Must be one of: ${validStatuses.join(', ')}`,
      };
    }

    if (progress !== undefined && (progress < 0 || progress > 100)) {
      return {
        success: false,
        updated: false,
        error: 'Progress must be between 0 and 100',
      };
    }

    // Log the status update (in production, this would update the database)
    console.log(`[Status Update] ${executionId}: ${status}${message ? ` - ${message}` : ''}${progress !== undefined ? ` (${progress}%)` : ''}`);

    return {
      success: true,
      updated: true,
    };
  } catch (error: any) {
    return {
      success: false,
      updated: false,
      error: error.message || 'Failed to update workflow status',
    };
  }
}

/**
 * Send an error alert notification
 */
export async function sendErrorAlert(
  input: ErrorAlertInput
): Promise<ErrorAlertResult> {
  const {
    executionId,
    error,
    severity = 'medium',
    context = {},
    notifySlack = false,
    slackWebhookUrl,
  } = input;

  const alertId = randomUUID();

  try {
    let slackNotified = false;

    // Send Slack notification if requested
    if (notifySlack && slackWebhookUrl) {
      const slackResult = await sendSlackNotification({
        webhookUrl: slackWebhookUrl,
        message: `🚨 Workflow Error: ${error}`,
        attachments: [
          {
            color: severity === 'critical' ? 'danger' : severity === 'high' ? 'warning' : 'good',
            title: 'Error Details',
            text: error,
            fields: [
              {
                title: 'Execution ID',
                value: executionId,
                short: true,
              },
              {
                title: 'Severity',
                value: severity,
                short: true,
              },
              ...Object.entries(context).map(([key, value]) => ({
                title: key,
                value: String(value),
                short: true,
              })),
            ],
          },
        ],
      });

      slackNotified = slackResult.success;
    }

    // Log the error alert
    console.error(`[Error Alert] ${alertId} - ${executionId}: ${error} (Severity: ${severity})`);

    return {
      success: true,
      alertId,
      slackNotified,
    };
  } catch (error: any) {
    return {
      success: false,
      alertId,
      slackNotified: false,
      error: error.message || 'Failed to send error alert',
    };
  }
}

/**
 * Send a progress update for workflow execution
 */
export async function sendProgressUpdate(
  input: ProgressUpdateInput
): Promise<ProgressUpdateResult> {
  const {
    executionId,
    step,
    progress,
    message,
    estimatedTimeRemaining,
  } = input;

  try {
    if (!executionId) {
      return {
        success: false,
        updated: false,
        error: 'executionId is required',
      };
    }

    if (progress < 0 || progress > 100) {
      return {
        success: false,
        updated: false,
        error: 'Progress must be between 0 and 100',
      };
    }

    // Update status with progress information
    const statusUpdate = await updateWorkflowStatus({
      executionId,
      status: 'running',
      message: message || `Step: ${step}`,
      progress,
      metadata: {
        step,
        estimatedTimeRemaining,
      },
    });

    return {
      success: statusUpdate.success,
      updated: statusUpdate.updated,
      error: statusUpdate.error,
    };
  } catch (error: any) {
    return {
      success: false,
      updated: false,
      error: error.message || 'Failed to send progress update',
    };
  }
}

