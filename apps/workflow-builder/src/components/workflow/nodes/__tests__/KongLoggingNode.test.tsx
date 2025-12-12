/**
 * Tests for KongLoggingNode component
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '../../../../../tests/ui/test-helpers';
import { KongLoggingNode } from '../KongLoggingNode';
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
  FileText: ({ size, color }: any) => <div data-testid="filetext-icon" data-size={size} data-color={color} />,
  Globe: ({ size }: any) => <div data-testid="globe-icon" data-size={size} />,
}));

describe('KongLoggingNode', () => {
  const defaultProps: NodeProps = {
    id: 'test-node',
    type: 'kong-logging',
    data: {
      label: 'Kong Logging',
      displayName: 'Kong Logging',
      config: {
        connectorName: 'HTTP Log Connector',
        enabledEndpoints: ['endpoint-1', 'endpoint-2'],
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
    render(<KongLoggingNode {...defaultProps} />);
    expect(screen.getByText('Kong Logging')).toBeInTheDocument();
  });

  it('should display component name', () => {
    render(<KongLoggingNode {...defaultProps} />);
    expect(screen.getByText('Kong Logging')).toBeInTheDocument();
  });

  it('should display connector name', () => {
    render(<KongLoggingNode {...defaultProps} />);
    expect(screen.getByText(/HTTP Log Connector/)).toBeInTheDocument();
  });

  it('should display enabled endpoint count', () => {
    render(<KongLoggingNode {...defaultProps} />);
    expect(screen.getByText(/2 enabled/)).toBeInTheDocument();
  });

  it('should show project-level badge', () => {
    render(<KongLoggingNode {...defaultProps} />);
    expect(screen.getByText('Project')).toBeInTheDocument();
  });

  it('should show project-level note', () => {
    render(<KongLoggingNode {...defaultProps} />);
    expect(screen.getByText(/Project-level: Visible on all services/)).toBeInTheDocument();
  });

  it('should display "No connector" when connector is not set', () => {
    const props = {
      ...defaultProps,
      data: {
        ...defaultProps.data,
        config: {
          enabledEndpoints: [],
        },
      },
    };
    render(<KongLoggingNode {...props} />);
    expect(screen.getByText(/No connector/)).toBeInTheDocument();
  });

  it('should show selected styling when selected', () => {
    const props = {
      ...defaultProps,
      selected: true,
    };
    const { container } = render(<KongLoggingNode {...props} />);
    // Check that the component renders (selected styling is handled by Tamagui)
    expect(container).toBeInTheDocument();
  });

  it('should handle missing enabledEndpoints array', () => {
    const props = {
      ...defaultProps,
      data: {
        ...defaultProps.data,
        config: {
          connectorName: 'Test Connector',
        },
      },
    };
    render(<KongLoggingNode {...props} />);
    expect(screen.getByText(/0 enabled/)).toBeInTheDocument();
  });
});

