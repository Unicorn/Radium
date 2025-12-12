/**
 * Cron Expression Validation and Parsing Utilities
 * 
 * Provides validation, parsing, and human-readable descriptions for cron expressions
 * Used for scheduled workflow configuration
 */

import type { CronValidationResult } from '../types/advanced-patterns';

/**
 * Common cron presets with descriptions
 */
export const CRON_PRESETS = {
  'everyMinute': {
    expression: '* * * * *',
    description: 'Every minute',
  },
  'every5Minutes': {
    expression: '*/5 * * * *',
    description: 'Every 5 minutes',
  },
  'every15Minutes': {
    expression: '*/15 * * * *',
    description: 'Every 15 minutes',
  },
  'every30Minutes': {
    expression: '*/30 * * * *',
    description: 'Every 30 minutes',
  },
  'hourly': {
    expression: '0 * * * *',
    description: 'Every hour (at :00)',
  },
  'every2Hours': {
    expression: '0 */2 * * *',
    description: 'Every 2 hours',
  },
  'every6Hours': {
    expression: '0 */6 * * *',
    description: 'Every 6 hours',
  },
  'daily': {
    expression: '0 0 * * *',
    description: 'Daily at midnight',
  },
  'dailyAt9AM': {
    expression: '0 9 * * *',
    description: 'Daily at 9:00 AM',
  },
  'dailyAtNoon': {
    expression: '0 12 * * *',
    description: 'Daily at noon',
  },
  'weeklyMonday': {
    expression: '0 0 * * 1',
    description: 'Weekly on Monday at midnight',
  },
  'weekdaysAt9AM': {
    expression: '0 9 * * 1-5',
    description: 'Weekdays at 9:00 AM',
  },
  'monthly': {
    expression: '0 0 1 * *',
    description: 'Monthly on the 1st at midnight',
  },
  'yearly': {
    expression: '0 0 1 1 *',
    description: 'Yearly on January 1st at midnight',
  },
} as const;

/**
 * Validate a cron expression
 */
export function validateCronExpression(expression: string): CronValidationResult {
  if (!expression || typeof expression !== 'string') {
    return {
      valid: false,
      error: 'Cron expression is required',
    };
  }

  const trimmed = expression.trim();
  
  // Check length
  if (trimmed.length === 0) {
    return {
      valid: false,
      error: 'Cron expression cannot be empty',
    };
  }

  if (trimmed.length > 100) {
    return {
      valid: false,
      error: 'Cron expression is too long (max 100 characters)',
    };
  }

  // Split into fields
  const fields = trimmed.split(/\s+/);
  
  // Standard cron: 5 fields (minute hour day month day-of-week)
  // Extended cron: 6 fields (second minute hour day month day-of-week)
  if (fields.length !== 5 && fields.length !== 6) {
    return {
      valid: false,
      error: `Cron expression must have 5 or 6 fields, got ${fields.length}`,
    };
  }

  // Validate each field
  const fieldNames = fields.length === 5
    ? ['minute', 'hour', 'day', 'month', 'day-of-week']
    : ['second', 'minute', 'hour', 'day', 'month', 'day-of-week'];

  const fieldRanges = fields.length === 5
    ? [
        { min: 0, max: 59, name: 'minute' },
        { min: 0, max: 23, name: 'hour' },
        { min: 1, max: 31, name: 'day' },
        { min: 1, max: 12, name: 'month' },
        { min: 0, max: 7, name: 'day-of-week' },  // 0 and 7 both = Sunday
      ]
    : [
        { min: 0, max: 59, name: 'second' },
        { min: 0, max: 59, name: 'minute' },
        { min: 0, max: 23, name: 'hour' },
        { min: 1, max: 31, name: 'day' },
        { min: 1, max: 12, name: 'month' },
        { min: 0, max: 7, name: 'day-of-week' },
      ];

  for (let i = 0; i < fields.length; i++) {
    const field = fields[i];
    const range = fieldRanges[i];

    const fieldResult = validateCronField(field, range.min, range.max, range.name);
    if (!fieldResult.valid) {
      return {
        valid: false,
        error: fieldResult.error,
      };
    }
  }

  // Try to get human-readable description
  const humanReadable = getCronDescription(trimmed);

  // Try to calculate next runs (simplified - would need full cron library for accuracy)
  const nextRuns = calculateNextRuns(trimmed, 3);

  return {
    valid: true,
    humanReadable,
    nextRuns,
  };
}

/**
 * Validate a single cron field
 */
function validateCronField(
  field: string,
  min: number,
  max: number,
  fieldName: string
): { valid: boolean; error?: string } {
  // Wildcard
  if (field === '*') {
    return { valid: true };
  }

  // Step values: */5, 0-23/2, etc.
  if (field.includes('/')) {
    const [range, step] = field.split('/');
    
    // Validate step is a number
    const stepNum = parseInt(step, 10);
    if (isNaN(stepNum) || stepNum <= 0) {
      return {
        valid: false,
        error: `Invalid step value in ${fieldName}: ${step}`,
      };
    }

    // Validate range part
    if (range === '*') {
      return { valid: true };
    }

    // Range like 0-23
    if (range.includes('-')) {
      return validateCronRange(range, min, max, fieldName);
    }

    // Single value
    const value = parseInt(range, 10);
    if (isNaN(value) || value < min || value > max) {
      return {
        valid: false,
        error: `${fieldName} value ${range} is out of range [${min}-${max}]`,
      };
    }

    return { valid: true };
  }

  // Range: 1-5, 9-17, etc.
  if (field.includes('-')) {
    return validateCronRange(field, min, max, fieldName);
  }

  // List: 1,3,5 or MON,WED,FRI
  if (field.includes(',')) {
    const values = field.split(',');
    for (const value of values) {
      const result = validateCronField(value.trim(), min, max, fieldName);
      if (!result.valid) {
        return result;
      }
    }
    return { valid: true };
  }

  // Single value or named value (MON, JAN, etc.)
  // For simplicity, just check if it's a valid number
  const value = parseInt(field, 10);
  if (isNaN(value)) {
    // Could be a named value like MON, TUE, JAN, FEB
    // For now, accept it (full validation would require mapping)
    return { valid: true };
  }

  if (value < min || value > max) {
    return {
      valid: false,
      error: `${fieldName} value ${value} is out of range [${min}-${max}]`,
    };
  }

  return { valid: true };
}

/**
 * Validate a cron range (e.g., "1-5", "9-17")
 */
function validateCronRange(
  range: string,
  min: number,
  max: number,
  fieldName: string
): { valid: boolean; error?: string } {
  const [startStr, endStr] = range.split('-');
  
  const start = parseInt(startStr, 10);
  const end = parseInt(endStr, 10);

  if (isNaN(start) || isNaN(end)) {
    return {
      valid: false,
      error: `Invalid range in ${fieldName}: ${range}`,
    };
  }

  if (start < min || start > max) {
    return {
      valid: false,
      error: `${fieldName} range start ${start} is out of range [${min}-${max}]`,
    };
  }

  if (end < min || end > max) {
    return {
      valid: false,
      error: `${fieldName} range end ${end} is out of range [${min}-${max}]`,
    };
  }

  if (start > end) {
    return {
      valid: false,
      error: `${fieldName} range start ${start} is greater than end ${end}`,
    };
  }

  return { valid: true };
}

/**
 * Get human-readable description of cron expression
 */
export function getCronDescription(expression: string): string {
  // Check if it matches a preset
  for (const [key, preset] of Object.entries(CRON_PRESETS)) {
    if (preset.expression === expression) {
      return preset.description;
    }
  }

  // Try to generate description
  const fields = expression.split(/\s+/);
  
  if (fields.length === 5) {
    const [minute, hour, day, month, dayOfWeek] = fields;

    // Every minute
    if (minute === '*' && hour === '*' && day === '*' && month === '*' && dayOfWeek === '*') {
      return 'Every minute';
    }

    // Every N minutes
    if (minute.startsWith('*/') && hour === '*' && day === '*' && month === '*' && dayOfWeek === '*') {
      const interval = minute.split('/')[1];
      return `Every ${interval} minutes`;
    }

    // Hourly
    if (minute !== '*' && hour === '*' && day === '*' && month === '*' && dayOfWeek === '*') {
      return `Hourly at :${minute.padStart(2, '0')}`;
    }

    // Every N hours
    if (hour.startsWith('*/') && day === '*' && month === '*' && dayOfWeek === '*') {
      const interval = hour.split('/')[1];
      const atMinute = minute === '0' ? '' : ` at :${minute.padStart(2, '0')}`;
      return `Every ${interval} hours${atMinute}`;
    }

    // Daily
    if (day === '*' && month === '*' && dayOfWeek === '*' && hour !== '*' && minute !== '*') {
      return `Daily at ${hour.padStart(2, '0')}:${minute.padStart(2, '0')}`;
    }

    // Weekly
    if (day === '*' && month === '*' && dayOfWeek !== '*' && dayOfWeek !== '**') {
      const dayName = getDayName(dayOfWeek);
      return `Weekly on ${dayName} at ${hour.padStart(2, '0')}:${minute.padStart(2, '0')}`;
    }

    // Monthly
    if (day !== '*' && month === '*' && dayOfWeek === '*') {
      const dayNum = day;
      return `Monthly on the ${dayNum}${getOrdinalSuffix(parseInt(dayNum))} at ${hour.padStart(2, '0')}:${minute.padStart(2, '0')}`;
    }
  }

  // Fallback: just return the expression
  return expression;
}

/**
 * Calculate next N run times for a cron expression
 * NOTE: This is a simplified implementation
 * For production, use a proper cron library like 'cron-parser'
 */
export function calculateNextRuns(expression: string, count: number = 5): Date[] {
  // This is a simplified placeholder
  // In production, you'd use a library like 'cron-parser'
  // For now, return empty array
  
  // TODO: Integrate with cron-parser library
  return [];
}

/**
 * Check if cron expression represents a high-frequency schedule
 * (warns users about potentially expensive schedules)
 */
export function isHighFrequency(expression: string): boolean {
  const fields = expression.split(/\s+/);
  if (fields.length < 5) return false;

  const minute = fields[0];

  // Every minute
  if (minute === '*') return true;

  // Every N minutes where N <= 5
  if (minute.startsWith('*/')) {
    const interval = parseInt(minute.split('/')[1], 10);
    return !isNaN(interval) && interval <= 5;
  }

  return false;
}

/**
 * Helper functions
 */

function getDayName(dayField: string): string {
  const dayMap: Record<string, string> = {
    '0': 'Sunday',
    '1': 'Monday',
    '2': 'Tuesday',
    '3': 'Wednesday',
    '4': 'Thursday',
    '5': 'Friday',
    '6': 'Saturday',
    '7': 'Sunday',
  };

  return dayMap[dayField] || dayField;
}

function getOrdinalSuffix(num: number): string {
  const j = num % 10;
  const k = num % 100;
  
  if (j === 1 && k !== 11) return 'st';
  if (j === 2 && k !== 12) return 'nd';
  if (j === 3 && k !== 13) return 'rd';
  return 'th';
}

/**
 * Get all presets for UI dropdown
 */
export function getCronPresets() {
  return Object.entries(CRON_PRESETS).map(([key, preset]) => ({
    key,
    expression: preset.expression,
    description: preset.description,
  }));
}

