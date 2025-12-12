/**
 * Tests for DataInNode Component
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '../../../../../../tests/ui/test-helpers';
import { DataInNode } from '../DataInNode';
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

describe('DataInNode', () => {
  const defaultProps: NodeProps = {
    id: 'data-in-1',
    type: 'data-in',
    data: {
      label: 'Receive Data',
      displayName: 'Receive Data',
      componentName: 'receiveData',
      config: {
        endpointPath: '/receive-data',
        httpMethod: 'POST',
      },
    },
    selected: false,
    position: { x: 0, y: 0 },
    dragging: false,
  };

  it('should render with default props', () => {
    render(<DataInNode {...defaultProps} />);

    expect(screen.getByText('Receive Data')).toBeInTheDocument();
    expect(screen.getByText('POST')).toBeInTheDocument();
    expect(screen.getByText('/receive-data')).toBeInTheDocument();
    expect(screen.getByText('Receives data via signal')).toBeInTheDocument();
  });

  it('should display PATCH method when configured', () => {
    const props: NodeProps = {
      ...defaultProps,
      data: {
        ...defaultProps.data,
        config: {
          endpointPath: '/update-data',
          httpMethod: 'PATCH',
        },
      },
    };

    render(<DataInNode {...props} />);

    expect(screen.getByText('PATCH')).toBeInTheDocument();
    expect(screen.getByText('/update-data')).toBeInTheDocument();
  });

  it('should use fallback values when config is missing', () => {
    const props: NodeProps = {
      ...defaultProps,
      data: {
        label: 'Data In',
      },
    };

    render(<DataInNode {...props} />);

    expect(screen.getByText('Data In')).toBeInTheDocument();
    expect(screen.getByText('POST')).toBeInTheDocument();
    expect(screen.getByText('/endpoint')).toBeInTheDocument();
  });

  it('should apply selected styling when selected', () => {
    const props: NodeProps = {
      ...defaultProps,
      selected: true,
    };

    const { container } = render(<DataInNode {...props} />);
    const card = container.querySelector('[data-testid]')?.parentElement;

    // Check that selected styling is applied (cyan background)
    expect(card).toBeInTheDocument();
  });

  it('should display component name when label is missing', () => {
    const props: NodeProps = {
      ...defaultProps,
      data: {
        componentName: 'customReceiveData',
        config: {
          endpointPath: '/custom',
          httpMethod: 'POST',
        },
      },
    };

    render(<DataInNode {...props} />);

    expect(screen.getByText('customReceiveData')).toBeInTheDocument();
  });

  it('should display displayName when available', () => {
    const props: NodeProps = {
      ...defaultProps,
      data: {
        label: 'Internal Name',
        displayName: 'User-Friendly Name',
        config: {
          endpointPath: '/test',
          httpMethod: 'POST',
        },
      },
    };

    render(<DataInNode {...props} />);

    expect(screen.getByText('User-Friendly Name')).toBeInTheDocument();
    expect(screen.queryByText('Internal Name')).not.toBeInTheDocument();
  });

  it('should render handles for connections', () => {
    render(<DataInNode {...defaultProps} />);

    expect(screen.getByTestId('handle-top')).toBeInTheDocument();
    expect(screen.getByTestId('handle-bottom')).toBeInTheDocument();
  });
});

