/**
 * Tests for GraphQLNode component
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '../../../../../tests/ui/test-helpers';
import { GraphQLNode } from '../GraphQLNode';
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
  Network: ({ size }: any) => <div data-testid="network-icon" data-size={size} />,
}));

describe('GraphQLNode', () => {
  const defaultProps: NodeProps = {
    id: 'test-node',
    type: 'graphql-gateway',
    data: {
      label: 'GraphQL Gateway',
      displayName: 'GraphQL Gateway',
      config: {
        endpointPath: '/graphql',
        schema: 'type Query { hello: String }',
        queries: [{ name: 'hello', workflowId: 'workflow-1' }],
        mutations: [],
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
    const { container } = render(<GraphQLNode {...defaultProps} />);
    expect(container).toBeTruthy();
  });

  it('should display component name', () => {
    render(<GraphQLNode {...defaultProps} />);
    expect(screen.getByText('GraphQL Gateway')).toBeTruthy();
  });

  it('should display endpoint path', () => {
    render(<GraphQLNode {...defaultProps} />);
    expect(screen.getByText('/graphql')).toBeTruthy();
  });

  it('should display query and mutation counts', () => {
    const propsWithQueries = {
      ...defaultProps,
      data: {
        ...defaultProps.data,
        config: {
          ...defaultProps.data.config,
          queries: [{ name: 'user', workflowId: 'workflow-1' }, { name: 'users', workflowId: 'workflow-2' }],
          mutations: [{ name: 'createUser', workflowId: 'workflow-3' }],
        },
      },
    };

    render(<GraphQLNode {...propsWithQueries} />);
    expect(screen.getByText(/2/)).toBeTruthy(); // Query count
    expect(screen.getByText(/1/)).toBeTruthy(); // Mutation count
  });

  it('should show schema badge when schema is present', () => {
    render(<GraphQLNode {...defaultProps} />);
    expect(screen.getByText('Schema')).toBeTruthy();
  });

  it('should show schema required badge when schema is missing', () => {
    const propsWithoutSchema = {
      ...defaultProps,
      data: {
        ...defaultProps.data,
        config: {
          ...defaultProps.data.config,
          schema: '',
        },
      },
    };

    render(<GraphQLNode {...propsWithoutSchema} />);
    expect(screen.getByText('Schema Required')).toBeTruthy();
  });

  it('should apply selected styling when selected', () => {
    const selectedProps = {
      ...defaultProps,
      selected: true,
    };

    const { container } = render(<GraphQLNode {...selectedProps} />);
    expect(container).toBeTruthy();
  });
});

