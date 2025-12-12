/**
 * Work Queue Management Utilities
 * 
 * Utilities for managing work queues, priority ordering, and deduplication
 */

import type {
  WorkQueueStatus,
  WorkQueueItem,
  WorkflowWorkQueue,
} from '../types/advanced-patterns';

/**
 * Priority comparator functions
 */
export const PRIORITY_COMPARATORS = {
  fifo: (a: WorkQueueItem, b: WorkQueueItem) => {
    return a.added_at.getTime() - b.added_at.getTime();
  },
  lifo: (a: WorkQueueItem, b: WorkQueueItem) => {
    return b.added_at.getTime() - a.added_at.getTime();
  },
  priority: (a: WorkQueueItem, b: WorkQueueItem) => {
    // Higher priority number = higher priority
    const priorityDiff = (b.priority || 0) - (a.priority || 0);
    if (priorityDiff !== 0) return priorityDiff;
    
    // If same priority, use FIFO as tiebreaker
    return a.added_at.getTime() - b.added_at.getTime();
  },
} as const;

/**
 * Sort work queue items based on queue priority setting
 */
export function sortQueueItems(
  items: WorkQueueItem[],
  priority: 'fifo' | 'lifo' | 'priority'
): WorkQueueItem[] {
  const comparator = PRIORITY_COMPARATORS[priority];
  return [...items].sort(comparator);
}

/**
 * Check if work queue is full
 */
export function isQueueFull(
  currentCount: number,
  maxSize: number | null
): boolean {
  if (maxSize === null) return false;
  return currentCount >= maxSize;
}

/**
 * Calculate queue status
 */
export function calculateQueueStatus(
  queue: WorkflowWorkQueue,
  currentItems: WorkQueueItem[]
): WorkQueueStatus {
  const currentCount = currentItems.filter(
    item => item.status === 'pending' || item.status === 'processing'
  ).length;

  return {
    queue_id: queue.id,
    queue_name: queue.queue_name,
    current_count: currentCount,
    max_size: queue.max_size,
    is_full: isQueueFull(currentCount, queue.max_size),
    priority: queue.priority,
    last_updated: new Date(),
  };
}

/**
 * Generate unique item ID for deduplication
 * Can be based on hash of item data
 */
export function generateItemHash(data: Record<string, any>): string {
  // Simple stringification for hash
  // In production, use a proper hashing library
  const str = JSON.stringify(sortObjectKeys(data));
  
  // Simple hash function (djb2)
  let hash = 5381;
  for (let i = 0; i < str.length; i++) {
    hash = ((hash << 5) + hash) + str.charCodeAt(i);
  }
  
  return hash.toString(36);
}

/**
 * Sort object keys for consistent hashing
 */
function sortObjectKeys(obj: any): any {
  if (obj === null || typeof obj !== 'object') {
    return obj;
  }

  if (Array.isArray(obj)) {
    return obj.map(sortObjectKeys);
  }

  const sorted: Record<string, any> = {};
  Object.keys(obj).sort().forEach(key => {
    sorted[key] = sortObjectKeys(obj[key]);
  });

  return sorted;
}

/**
 * Check if item already exists in queue (for deduplication)
 */
export function isDuplicate(
  newItem: Record<string, any>,
  existingItems: WorkQueueItem[],
  deduplicate: boolean
): boolean {
  if (!deduplicate) return false;

  const newHash = generateItemHash(newItem);
  
  return existingItems.some(item => {
    if (item.status === 'completed' || item.status === 'failed') {
      return false;  // Don't compare with completed/failed items
    }
    
    const existingHash = generateItemHash(item.data);
    return existingHash === newHash;
  });
}

/**
 * Validate work queue configuration
 */
export function validateWorkQueueConfig(
  config: {
    queue_name: string;
    max_size?: number | null;
    priority?: 'fifo' | 'lifo' | 'priority';
    signal_name: string;
    query_name: string;
  }
): {
  valid: boolean;
  errors: Array<{ field: string; message: string }>;
} {
  const errors: Array<{ field: string; message: string }> = [];

  // Validate queue name
  if (!config.queue_name || config.queue_name.trim().length === 0) {
    errors.push({
      field: 'queue_name',
      message: 'Queue name is required',
    });
  } else if (config.queue_name.length > 100) {
    errors.push({
      field: 'queue_name',
      message: 'Queue name must be 100 characters or less',
    });
  } else if (!/^[a-zA-Z][a-zA-Z0-9_]*$/.test(config.queue_name)) {
    errors.push({
      field: 'queue_name',
      message: 'Queue name must start with a letter and contain only letters, numbers, and underscores',
    });
  }

  // Validate signal name
  if (!config.signal_name || config.signal_name.trim().length === 0) {
    errors.push({
      field: 'signal_name',
      message: 'Signal name is required',
    });
  } else if (config.signal_name.length > 100) {
    errors.push({
      field: 'signal_name',
      message: 'Signal name must be 100 characters or less',
    });
  } else if (!/^[a-zA-Z][a-zA-Z0-9_]*$/.test(config.signal_name)) {
    errors.push({
      field: 'signal_name',
      message: 'Signal name must start with a letter and contain only letters, numbers, and underscores',
    });
  }

  // Validate query name
  if (!config.query_name || config.query_name.trim().length === 0) {
    errors.push({
      field: 'query_name',
      message: 'Query name is required',
    });
  } else if (config.query_name.length > 100) {
    errors.push({
      field: 'query_name',
      message: 'Query name must be 100 characters or less',
    });
  } else if (!/^[a-zA-Z][a-zA-Z0-9_]*$/.test(config.query_name)) {
    errors.push({
      field: 'query_name',
      message: 'Query name must start with a letter and contain only letters, numbers, and underscores',
    });
  }

  // Validate max size
  if (config.max_size !== null && config.max_size !== undefined) {
    if (config.max_size < 1) {
      errors.push({
        field: 'max_size',
        message: 'Max size must be at least 1',
      });
    } else if (config.max_size > 10000) {
      errors.push({
        field: 'max_size',
        message: 'Max size cannot exceed 10,000',
      });
    }
  }

  // Validate priority
  if (config.priority && !['fifo', 'lifo', 'priority'].includes(config.priority)) {
    errors.push({
      field: 'priority',
      message: 'Priority must be one of: fifo, lifo, priority',
    });
  }

  return {
    valid: errors.length === 0,
    errors,
  };
}

/**
 * Get human-readable priority description
 */
export function getPriorityDescription(priority: 'fifo' | 'lifo' | 'priority'): string {
  const descriptions = {
    fifo: 'First In, First Out',
    lifo: 'Last In, First Out',
    priority: 'Priority-based (highest first)',
  };

  return descriptions[priority];
}

/**
 * Calculate queue capacity percentage
 */
export function getQueueCapacityPercent(
  currentCount: number,
  maxSize: number | null
): number | null {
  if (maxSize === null) return null;
  if (maxSize === 0) return 100;
  
  return Math.round((currentCount / maxSize) * 100);
}

/**
 * Check if queue is nearly full (>= 80% capacity)
 */
export function isQueueNearlyFull(
  currentCount: number,
  maxSize: number | null
): boolean {
  if (maxSize === null) return false;
  
  const percent = getQueueCapacityPercent(currentCount, maxSize);
  return percent !== null && percent >= 80;
}

/**
 * Get queue health status
 */
export function getQueueHealth(
  currentCount: number,
  maxSize: number | null
): 'healthy' | 'warning' | 'critical' {
  if (isQueueFull(currentCount, maxSize)) {
    return 'critical';
  }

  if (isQueueNearlyFull(currentCount, maxSize)) {
    return 'warning';
  }

  return 'healthy';
}

/**
 * Generate default signal name for work queue
 */
export function generateDefaultSignalName(queueName: string): string {
  // Convert camelCase or snake_case to camelCase
  const camelCase = queueName
    .replace(/_([a-z])/g, (_, letter) => letter.toUpperCase())
    .replace(/^[A-Z]/, letter => letter.toLowerCase());

  return `addTo${capitalize(camelCase)}`;
}

/**
 * Generate default query name for work queue
 */
export function generateDefaultQueryName(queueName: string): string {
  // Convert camelCase or snake_case to camelCase
  const camelCase = queueName
    .replace(/_([a-z])/g, (_, letter) => letter.toUpperCase())
    .replace(/^[A-Z]/, letter => letter.toLowerCase());

  return `get${capitalize(camelCase)}`;
}

/**
 * Helper: Capitalize first letter
 */
function capitalize(str: string): string {
  return str.charAt(0).toUpperCase() + str.slice(1);
}

/**
 * Get next item from queue based on priority
 */
export function getNextItem(
  items: WorkQueueItem[],
  priority: 'fifo' | 'lifo' | 'priority'
): WorkQueueItem | null {
  const pendingItems = items.filter(item => item.status === 'pending');
  
  if (pendingItems.length === 0) {
    return null;
  }

  const sorted = sortQueueItems(pendingItems, priority);
  return sorted[0];
}

/**
 * Bulk operations for work queues
 */
export function bulkAddToQueue(
  newItems: Array<Record<string, any>>,
  existingItems: WorkQueueItem[],
  queue: WorkflowWorkQueue
): {
  added: Array<Record<string, any>>;
  rejected: Array<{ item: Record<string, any>; reason: string }>;
} {
  const added: Array<Record<string, any>> = [];
  const rejected: Array<{ item: Record<string, any>; reason: string }> = [];

  let currentCount = existingItems.filter(
    item => item.status === 'pending' || item.status === 'processing'
  ).length;

  for (const item of newItems) {
    // Check if queue is full
    if (isQueueFull(currentCount, queue.max_size)) {
      rejected.push({
        item,
        reason: 'Queue is full',
      });
      continue;
    }

    // Check for duplicates
    if (isDuplicate(item, existingItems, queue.deduplicate)) {
      rejected.push({
        item,
        reason: 'Duplicate item',
      });
      continue;
    }

    added.push(item);
    currentCount++;
  }

  return { added, rejected };
}

/**
 * Clear completed/failed items from queue
 */
export function clearCompletedItems(items: WorkQueueItem[]): WorkQueueItem[] {
  return items.filter(
    item => item.status !== 'completed' && item.status !== 'failed'
  );
}

/**
 * Get queue statistics
 */
export function getQueueStatistics(items: WorkQueueItem[]): {
  total: number;
  pending: number;
  processing: number;
  completed: number;
  failed: number;
  oldestPending?: Date;
  averageWaitTime?: number;  // in milliseconds
} {
  const stats = {
    total: items.length,
    pending: 0,
    processing: 0,
    completed: 0,
    failed: 0,
    oldestPending: undefined as Date | undefined,
    averageWaitTime: undefined as number | undefined,
  };

  const waitTimes: number[] = [];
  const now = new Date();

  for (const item of items) {
    switch (item.status) {
      case 'pending':
        stats.pending++;
        if (!stats.oldestPending || item.added_at < stats.oldestPending) {
          stats.oldestPending = item.added_at;
        }
        waitTimes.push(now.getTime() - item.added_at.getTime());
        break;
      case 'processing':
        stats.processing++;
        break;
      case 'completed':
        stats.completed++;
        break;
      case 'failed':
        stats.failed++;
        break;
    }
  }

  if (waitTimes.length > 0) {
    stats.averageWaitTime = waitTimes.reduce((a, b) => a + b, 0) / waitTimes.length;
  }

  return stats;
}

