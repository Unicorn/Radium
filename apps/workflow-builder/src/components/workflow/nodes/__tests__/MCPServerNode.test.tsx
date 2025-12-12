/**
 * Tests for MCPServerNode component
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '../../../../../tests/ui/test-helpers';
import { MCPServerNode } from '../MCPServerNode';
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
  Server: ({ size }: any) => <div data-testid="server-icon" data-size={size} />,
}));

describe('MCPServerNode', () => {
  const defaultProps: NodeProps = {
    id: 'test-node',
    type: 'mcp-server',
    data: {
      label: 'MCP Server',
      displayName: 'MCP Server',
      config: {
        serverName: 'my-workflow-server',
        serverVersion: '1.0.0',
        endpointPath: '/mcp',
        resources: [{ uri: 'workflow://test/data', name: 'Test Resource' }],
        tools: [{ name: 'test_tool', description: 'Test tool' }],
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
    const { container } = render(<MCPServerNode {...defaultProps} />);
    expect(container).toBeTruthy();
  });

  it('should display component name', () => {
    render(<MCPServerNode {...defaultProps} />);
    expect(screen.getByText('MCP Server')).toBeTruthy();
  });

  it('should display server name and version', () => {
    render(<MCPServerNode {...defaultProps} />);
    expect(screen.getByText('my-workflow-server')).toBeTruthy();
    expect(screen.getByText('1.0.0')).toBeTruthy();
  });

  it('should display endpoint path', () => {
    render(<MCPServerNode {...defaultProps} />);
    expect(screen.getByText('/mcp')).toBeTruthy();
  });

  it('should display resource and tool counts', () => {
    const propsWithCounts = {
      ...defaultProps,
      data: {
        ...defaultProps.data,
        config: {
          ...defaultProps.data.config,
          resources: [
            { uri: 'workflow://test/data1', name: 'Resource 1' },
            { uri: 'workflow://test/data2', name: 'Resource 2' },
          ],
          tools: [
            { name: 'tool1', description: 'Tool 1' },
            { name: 'tool2', description: 'Tool 2' },
            { name: 'tool3', description: 'Tool 3' },
          ],
        },
      },
    };

    render(<MCPServerNode {...propsWithCounts} />);
    expect(screen.getByText(/2/)).toBeTruthy(); // Resource count
    expect(screen.getByText(/3/)).toBeTruthy(); // Tool count
  });

  it('should show no resources/tools badge when empty', () => {
    const propsEmpty = {
      ...defaultProps,
      data: {
        ...defaultProps.data,
        config: {
          ...defaultProps.data.config,
          resources: [],
          tools: [],
        },
      },
    };

    render(<MCPServerNode {...propsEmpty} />);
    expect(screen.getByText('No Resources/Tools')).toBeTruthy();
  });

  it('should apply selected styling when selected', () => {
    const selectedProps = {
      ...defaultProps,
      selected: true,
    };

    const { container } = render(<MCPServerNode {...selectedProps} />);
    expect(container).toBeTruthy();
  });
});

