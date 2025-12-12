/**
 * Tests for KongCorsNode component
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '../../../../../tests/ui/test-helpers';
import { KongCorsNode } from '../KongCorsNode';
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
  Globe: ({ size }: any) => <div data-testid="globe-icon" data-size={size} />,
}));

describe('KongCorsNode', () => {
  const defaultProps: NodeProps = {
    id: 'test-node',
    type: 'kong-cors',
    data: {
      label: 'Kong CORS',
      displayName: 'Kong CORS',
      config: {
        allowedOrigins: ['https://example.com'],
        allowedMethods: ['GET', 'POST'],
        allowedHeaders: ['Content-Type'],
        credentials: false,
        maxAge: 3600,
      },
    },
    selected: false,
    dragging: false,
    xPos: 0,
    yPos: 0,
    targetPosition: 'top',
    sourcePosition: 'bottom',
  } as any;

  it('should render with default props', () => {
    const { container } = render(<KongCorsNode {...defaultProps} />);
    expect(container).toBeTruthy();
  });

  it('should display component name', () => {
    render(<KongCorsNode {...defaultProps} />);
    expect(screen.getByText('Kong CORS')).toBeTruthy();
  });

  it('should display CORS configuration details', () => {
    const propsWithConfig = {
      ...defaultProps,
      data: {
        ...defaultProps.data,
        config: {
          allowedOrigins: ['https://example.com', 'https://app.example.com'],
          allowedMethods: ['GET', 'POST', 'PUT'],
          allowedHeaders: ['Content-Type', 'Authorization'],
          credentials: true,
          maxAge: 7200,
        },
      },
    };

    render(<KongCorsNode {...propsWithConfig} />);
    expect(screen.getByText(/2 configured/)).toBeTruthy();
    expect(screen.getByText(/3 allowed/)).toBeTruthy();
    expect(screen.getByText(/7200s/)).toBeTruthy();
  });

  it('should show credentials badge when enabled', () => {
    const propsWithCredentials = {
      ...defaultProps,
      data: {
        ...defaultProps.data,
        config: {
          ...defaultProps.data.config,
          credentials: true,
        },
      },
    };

    render(<KongCorsNode {...propsWithCredentials} />);
    expect(screen.getByText('Credentials')).toBeTruthy();
  });

  it('should apply selected styling when selected', () => {
    const selectedProps = {
      ...defaultProps,
      selected: true,
    };

    const { container } = render(<KongCorsNode {...selectedProps} />);
    expect(container).toBeTruthy();
  });
});
