/**
 * Notifications Router
 * 
 * tRPC endpoints that wrap notification activities for UI use.
 * These endpoints allow the UI to send user feedback, error alerts,
 * and workflow status updates.
 */

import { z } from 'zod';
import { createTRPCRouter, protectedProcedure } from '../trpc';
import { TRPCError } from '@trpc/server';
import {
  sendSlackNotification,
  sendErrorAlert,
  sendProgressUpdate,
  type SlackNotificationInput,
  type ErrorAlertInput,
  type ProgressUpdateInput,
} from '@/lib/activities/notifications.activities';

export const notificationsRouter = createTRPCRouter({
  /**
   * Send Slack notification (for user feedback, workflow deployments)
   */
  sendSlack: protectedProcedure
    .input(z.object({
      webhookUrl: z.string(),
      message: z.string(),
      channel: z.string().optional(),
      username: z.string().optional(),
      iconEmoji: z.string().optional(),
      attachments: z.array(z.object({
        color: z.string().optional(),
        title: z.string().optional(),
        text: z.string().optional(),
        fields: z.array(z.object({
          title: z.string(),
          value: z.string(),
          short: z.boolean().optional(),
        })).optional(),
      })).optional(),
    }))
    .mutation(async ({ input }) => {
      try {
        const result = await sendSlackNotification(input as SlackNotificationInput);
        if (!result.success) {
          throw new TRPCError({
            code: 'INTERNAL_SERVER_ERROR',
            message: result.error || 'Failed to send Slack notification',
          });
        }
        return result;
      } catch (error) {
        if (error instanceof TRPCError) throw error;
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error instanceof Error ? error.message : 'Failed to send Slack notification',
        });
      }
    }),

  /**
   * Send error alert (for user feedback on errors)
   */
  sendError: protectedProcedure
    .input(z.object({
      error: z.object({
        message: z.string(),
        stack: z.string().optional(),
      }),
      severity: z.enum(['low', 'medium', 'high', 'critical']).optional(),
      context: z.record(z.any()).optional(),
      slackWebhookUrl: z.string().optional(),
    }))
    .mutation(async ({ input }) => {
      try {
        const result = await sendErrorAlert({
          error: new Error(input.error.message),
          severity: input.severity,
          context: input.context,
          slackWebhookUrl: input.slackWebhookUrl,
        } as ErrorAlertInput);
        if (!result.success) {
          throw new TRPCError({
            code: 'INTERNAL_SERVER_ERROR',
            message: result.error || 'Failed to send error alert',
          });
        }
        return result;
      } catch (error) {
        if (error instanceof TRPCError) throw error;
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error instanceof Error ? error.message : 'Failed to send error alert',
        });
      }
    }),

  /**
   * Send progress update (for long-running operations)
   */
  sendProgress: protectedProcedure
    .input(z.object({
      message: z.string(),
      progress: z.number().min(0).max(100).optional(),
      stage: z.string().optional(),
      slackWebhookUrl: z.string().optional(),
    }))
    .mutation(async ({ input }) => {
      try {
        const result = await sendProgressUpdate({
          message: input.message,
          progress: input.progress,
          stage: input.stage,
          slackWebhookUrl: input.slackWebhookUrl,
        } as ProgressUpdateInput);
        if (!result.success) {
          throw new TRPCError({
            code: 'INTERNAL_SERVER_ERROR',
            message: result.error || 'Failed to send progress update',
          });
        }
        return result;
      } catch (error) {
        if (error instanceof TRPCError) throw error;
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error instanceof Error ? error.message : 'Failed to send progress update',
        });
      }
    }),
});

