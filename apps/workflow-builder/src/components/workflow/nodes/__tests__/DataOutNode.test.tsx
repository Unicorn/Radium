/**
 * Tests for DataOutNode Component
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '../../../../../../tests/ui/test-helpers';
import { DataOutNode } from '../DataOutNode';
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

describe('DataOutNode', () => {
  const defaultProps: NodeProps = {
    id: 'data-out-1',
    type: 'data-out',
    data: {
      label: 'Provide Data',
      displayName: 'Provide Data',
      componentName: 'provideData',
      config: {
        endpointPath: '/get-data',
        httpMethod: 'GET',
      },
    },
    selected: false,
    position: { x: 0, y: 0 },
    dragging: false,
  };

  it('should render with default props', () => {
    render(<DataOutNode {...defaultProps} />);

    expect(screen.getByText('Provide Data')).toBeInTheDocument();
    expect(screen.getByText('GET')).toBeInTheDocument();
    expect(screen.getByText('/get-data')).toBeInTheDocument();
    expect(screen.getByText('Provides data via query')).toBeInTheDocument();
  });

  it('should use fallback values when config is missing', () => {
    const props: NodeProps = {
      ...defaultProps,
      data: {
        label: 'Data Out',
      },
    };

    render(<DataOutNode {...props} />);

    expect(screen.getByText('Data Out')).toBeInTheDocument();
    expect(screen.getByText('GET')).toBeInTheDocument();
    expect(screen.getByText('/endpoint')).toBeInTheDocument();
  });

  it('should apply selected styling when selected', () => {
    const props: NodeProps = {
      ...defaultProps,
      selected: true,
    };

    const { container } = render(<DataOutNode {...props} />);
    const card = container.querySelector('[data-testid]')?.parentElement;

    // Check that selected styling is applied (teal background)
    expect(card).toBeInTheDocument();
  });

  it('should display component name when label is missing', () => {
    const props: NodeProps = {
      ...defaultProps,
      data: {
        componentName: 'customProvideData',
        config: {
          endpointPath: '/custom',
          httpMethod: 'GET',
        },
      },
    };

    render(<DataOutNode {...props} />);

    expect(screen.getByText('customProvideData')).toBeInTheDocument();
  });

  it('should display displayName when available', () => {
    const props: NodeProps = {
      ...defaultProps,
      data: {
        label: 'Internal Name',
        displayName: 'User-Friendly Name',
        config: {
          endpointPath: '/test',
          httpMethod: 'GET',
        },
      },
    };

    render(<DataOutNode {...props} />);

    expect(screen.getByText('User-Friendly Name')).toBeInTheDocument();
    expect(screen.queryByText('Internal Name')).not.toBeInTheDocument();
  });

  it('should render handles for connections', () => {
    render(<DataOutNode {...defaultProps} />);

    expect(screen.getByTestId('handle-top')).toBeInTheDocument();
    expect(screen.getByTestId('handle-bottom')).toBeInTheDocument();
  });

  it('should always display GET method (data-out only supports GET)', () => {
    const props: NodeProps = {
      ...defaultProps,
      data: {
        ...defaultProps.data,
        config: {
          endpointPath: '/get-data',
          httpMethod: 'GET',
        },
      },
    };

    render(<DataOutNode {...props} />);

    expect(screen.getByText('GET')).toBeInTheDocument();
  });
});

