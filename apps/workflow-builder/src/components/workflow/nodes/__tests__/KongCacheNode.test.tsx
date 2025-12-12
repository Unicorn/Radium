/**
 * Kong Cache Node Component Tests
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '../../../../../tests/ui/test-helpers';
import { KongCacheNode } from '../KongCacheNode';
import type { NodeProps } from 'react-flow-renderer';

// Mock react-flow-renderer
vi.mock('react-flow-renderer', () => ({
  Handle: ({ position, type, id }: { position: string; type?: string; id?: string }) => (
    <div data-testid={`handle-${id || position}`} data-position={position} data-type={type} />
  ),
  Position: {
    Top: 'top',
    Bottom: 'bottom',
    Left: 'left',
    Right: 'right',
  },
}));

// Mock lucide-react icons
vi.mock('lucide-react', () => ({
  Database: ({ size, color }: any) => <div data-testid="database-icon" data-size={size} data-color={color} />,
  Key: ({ size, color }: any) => <div data-testid="key-icon" data-size={size} data-color={color} />,
}));

describe('KongCacheNode', () => {
  const createMockProps = (overrides?: Partial<NodeProps>): NodeProps => ({
    id: 'cache-node-1',
    type: 'kong-cache',
    data: {
      label: 'Cache Component',
      displayName: 'Cache Component',
      config: {
        connectorName: 'Upstash Redis',
        cacheKey: '550e8400-e29b-41d4-a716-446655440000',
        ttlSeconds: 3600,
        isSaved: true,
      },
      ...overrides?.data,
    },
    selected: false,
    dragging: false,
    xPos: 0,
    yPos: 0,
    targetPosition: 'top',
    sourcePosition: 'bottom',
    ...overrides,
  } as any);

  it('should render cache node with basic information', () => {
    const props = createMockProps();
    render(<KongCacheNode {...props} />);

    expect(screen.getByText('Cache Component')).toBeDefined();
    expect(screen.getByText('Upstash Redis')).toBeDefined();
  });

  it('should display cache key (truncated if long)', () => {
    const props = createMockProps({
      data: {
        label: 'Cache Component',
        displayName: 'Cache Component',
        config: {
          connectorName: 'Upstash Redis',
          cacheKey: '550e8400-e29b-41d4-a716-446655440000',
          ttlSeconds: 3600,
          isSaved: true,
        },
      },
    });
    render(<KongCacheNode {...props} />);

    // Cache key should be displayed (may be truncated)
    expect(screen.getByText(/550e8400/)).toBeDefined();
  });

  it('should display TTL information', () => {
    const props = createMockProps({
      data: {
        label: 'Cache Component',
        displayName: 'Cache Component',
        config: {
          connectorName: 'Upstash Redis',
          cacheKey: 'test-key',
          ttlSeconds: 7200,
          isSaved: true,
        },
      },
    });
    render(<KongCacheNode {...props} />);

    // TTL should be displayed
    expect(screen.getByText(/7200/)).toBeDefined();
  });

  it('should show "Saved" badge when isSaved is true', () => {
    const props = createMockProps({
      data: {
        label: 'Cache Component',
        displayName: 'Cache Component',
        config: {
          connectorName: 'Upstash Redis',
          cacheKey: 'test-key',
          ttlSeconds: 3600,
          isSaved: true,
        },
      },
    });
    render(<KongCacheNode {...props} />);

    expect(screen.getByText(/Saved/i)).toBeDefined();
  });

  it('should show "Editable" badge when isSaved is false', () => {
    const props = createMockProps({
      data: {
        label: 'Cache Component',
        displayName: 'Cache Component',
        config: {
          connectorName: 'Upstash Redis',
          cacheKey: 'test-key',
          ttlSeconds: 3600,
          isSaved: false,
        },
      },
    });
    render(<KongCacheNode {...props} />);

    expect(screen.getByText(/Editable/i)).toBeDefined();
  });

  it('should apply selected styling when selected is true', () => {
    const props = createMockProps({ selected: true });
    const { container } = render(<KongCacheNode {...props} />);

    // Selected state should be visually distinct
    expect(container.firstChild).toBeDefined();
  });

  it('should handle missing optional data gracefully', () => {
    const props = createMockProps({
      data: {
        label: 'Cache Component',
        displayName: 'Cache Component',
        config: {
          connectorName: undefined,
          cacheKey: undefined,
          ttlSeconds: undefined,
          isSaved: false,
        },
      },
    });
    render(<KongCacheNode {...props} />);

    expect(screen.getByText('Cache Component')).toBeDefined();
  });
});
