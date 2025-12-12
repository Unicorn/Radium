/**
 * Service Container Node - Custom React Flow node for visualizing services
 * with inside/outside boundaries and zone-based layout
 * 
 * Supports two view modes:
 * - 'builder': Full service builder view with all zones visible
 * - 'project': Compact overview for project view with multiple services
 */

import { memo } from 'react';
import { Handle, Position, NodeProps } from 'react-flow-renderer';
import { Building2, ArrowRight, ArrowLeft, Plug, Network } from 'lucide-react';

export interface ServiceContainerNodeData {
  serviceId: string;
  serviceName: string;
  // Zones
  incomingInterfaces?: InterfaceNode[];
  outgoingInterfaces?: InterfaceNode[];
  externalConnectors?: ConnectorNode[];
  internalComponents?: ComponentNode[];
  externalConnections?: ConnectionNode[];
  // View mode
  viewMode: 'builder' | 'project';
}

export interface InterfaceNode {
  id: string;
  name: string;
  displayName: string;
  interfaceType: 'signal' | 'query' | 'update' | 'start_child';
  position?: number; // Position along edge (0-100%)
}

export interface ConnectorNode {
  id: string;
  name: string;
  displayName: string;
  connectorType: string;
  position?: number; // Position along edge (0-100%)
}

export interface ComponentNode {
  id: string;
  type: string;
  label: string;
}

export interface ConnectionNode {
  id: string;
  targetServiceId: string;
  targetServiceName: string;
  interfaceName: string;
  position?: number;
}

interface ServiceContainerNodeProps extends NodeProps<ServiceContainerNodeData> {
  selected?: boolean;
}

export const ServiceContainerNode = memo(({ 
  data, 
  selected 
}: ServiceContainerNodeProps) => {
  const { 
    serviceId, 
    serviceName, 
    incomingInterfaces = [], 
    outgoingInterfaces = [],
    externalConnectors = [],
    internalComponents = [],
    externalConnections = [],
    viewMode = 'project'
  } = data;

  const isBuilderView = viewMode === 'builder';
  const isProjectView = viewMode === 'project';

  // Container dimensions based on view mode
  const containerStyle: React.CSSProperties = {
    position: 'relative',
    border: `3px solid ${selected ? '#6366f1' : '#818cf8'}`,
    borderRadius: '12px',
    background: isBuilderView 
      ? 'linear-gradient(to bottom, #eef2ff, #f9fafb)'
      : 'linear-gradient(to bottom, #f0f9ff, #ffffff)',
    minWidth: isBuilderView ? '1000px' : '300px',
    minHeight: isBuilderView ? '700px' : '200px',
    width: isBuilderView ? '100%' : '300px',
    height: isBuilderView ? '100%' : '200px',
    boxShadow: selected
      ? '0 4px 12px rgba(99, 102, 241, 0.25)'
      : '0 2px 8px rgba(99, 102, 241, 0.1)',
    transition: 'all 0.2s ease',
  };

  // Zone dimensions
  const zoneTopHeight = isBuilderView ? 100 : 40;
  const zoneLeftWidth = isBuilderView ? 150 : 0;
  const zoneRightWidth = isBuilderView ? 150 : 0;
  const zoneBottomHeight = isBuilderView ? 100 : 40;
  const zoneCenterTop = zoneTopHeight;
  const zoneCenterLeft = zoneLeftWidth;
  const zoneCenterRight = zoneRightWidth;
  const zoneCenterBottom = zoneBottomHeight;

  return (
    <div style={containerStyle}>
      {/* Header */}
      <div
        style={{
          padding: isBuilderView ? '16px 20px' : '12px 16px',
          borderBottom: '2px solid #c7d2fe',
          background: 'linear-gradient(to right, #6366f1, #818cf8)',
          color: 'white',
          borderRadius: '9px 9px 0 0',
          display: 'flex',
          alignItems: 'center',
          gap: '12px',
          fontWeight: '700',
          fontSize: isBuilderView ? '18px' : '14px',
        }}
      >
        <Building2 size={isBuilderView ? 24 : 18} />
        <span>{serviceName}</span>
      </div>

      {/* Top Zone - External Connectors (Input) */}
      {isBuilderView && (
        <div
          className="zone-top"
          style={{
            position: 'absolute',
            top: 0,
            left: 0,
            right: 0,
            height: `${zoneTopHeight}px`,
            borderBottom: '1px dashed #c7d2fe',
            background: 'rgba(199, 210, 254, 0.05)',
            display: 'flex',
            alignItems: 'center',
            padding: '8px',
            gap: '8px',
            overflowX: 'auto',
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', gap: '4px', marginRight: '8px' }}>
            <Plug size={14} color="#f59e0b" />
            <span style={{ fontSize: '11px', fontWeight: '600', color: '#f59e0b' }}>
              EXTERNAL CONNECTORS
            </span>
          </div>
          {externalConnectors.map((connector, index) => (
            <div
              key={connector.id}
              className="connector-port"
              style={{
                position: 'relative',
                padding: '6px 10px',
                borderRadius: '6px',
                background: '#fffbeb',
                border: '2px solid #f59e0b',
                fontSize: '11px',
                fontWeight: '600',
                color: '#92400e',
                whiteSpace: 'nowrap',
                cursor: 'pointer',
              }}
            >
              <Handle
                type="target"
                position={Position.Top}
                id={`connector-${connector.id}`}
                style={{
                  background: '#f59e0b',
                  width: '10px',
                  height: '10px',
                  border: '2px solid white',
                  top: '-6px',
                }}
              />
              {connector.displayName || connector.name}
            </div>
          ))}
        </div>
      )}

      {/* Left Zone - Incoming Interfaces */}
      {isBuilderView && (
        <div
          className="zone-left"
          style={{
            position: 'absolute',
            top: `${zoneCenterTop}px`,
            left: 0,
            bottom: `${zoneCenterBottom}px`,
            width: `${zoneLeftWidth}px`,
            borderRight: '1px dashed #c7d2fe',
            background: 'rgba(199, 210, 254, 0.05)',
            display: 'flex',
            flexDirection: 'column',
            padding: '8px',
            gap: '8px',
            overflowY: 'auto',
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', gap: '4px', marginBottom: '4px' }}>
            <ArrowRight size={14} color="#14b8a6" />
            <span style={{ fontSize: '11px', fontWeight: '600', color: '#14b8a6' }}>
              INCOMING
            </span>
          </div>
          {incomingInterfaces.map((interfaceItem) => (
            <div
              key={interfaceItem.id}
              className="interface-port incoming"
              style={{
                position: 'relative',
                padding: '8px 10px',
                borderRadius: '6px',
                background: '#f0fdfa',
                border: '2px solid #14b8a6',
                fontSize: '11px',
                fontWeight: '600',
                color: '#0f766e',
                cursor: 'pointer',
              }}
            >
              <Handle
                type="target"
                position={Position.Left}
                id={`interface-in-${interfaceItem.id}`}
                style={{
                  background: '#14b8a6',
                  width: '10px',
                  height: '10px',
                  border: '2px solid white',
                  left: '-6px',
                }}
              />
              <div>{interfaceItem.displayName || interfaceItem.name}</div>
              <div style={{ fontSize: '9px', color: '#5eead4', marginTop: '2px' }}>
                {interfaceItem.interfaceType}
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Center Zone - Internal Flow */}
      <div
        className="zone-center"
        style={{
          position: 'absolute',
          top: `${zoneCenterTop}px`,
          left: `${zoneCenterLeft}px`,
          right: `${zoneCenterRight}px`,
          bottom: `${zoneCenterBottom}px`,
          background: isBuilderView ? '#ffffff' : 'transparent',
          border: isBuilderView ? '1px dashed #c7d2fe' : 'none',
          borderRadius: isBuilderView ? '8px' : '0',
          padding: isBuilderView ? '16px' : '8px',
          overflow: 'auto',
        }}
      >
        {isBuilderView ? (
          <div>
            <div style={{ display: 'flex', alignItems: 'center', gap: '4px', marginBottom: '12px' }}>
              <Network size={14} color="#6366f1" />
              <span style={{ fontSize: '11px', fontWeight: '600', color: '#6366f1' }}>
                INTERNAL FLOW
              </span>
            </div>
            {internalComponents.length > 0 ? (
              <div style={{ fontSize: '12px', color: '#6b7280' }}>
                {internalComponents.length} component{internalComponents.length !== 1 ? 's' : ''}
              </div>
            ) : (
              <div style={{ fontSize: '12px', color: '#9ca3af', fontStyle: 'italic' }}>
                Drag components here
              </div>
            )}
          </div>
        ) : (
          <div style={{ fontSize: '12px', color: '#6b7280', textAlign: 'center', paddingTop: '8px' }}>
            {internalComponents.length} component{internalComponents.length !== 1 ? 's' : ''}
          </div>
        )}
      </div>

      {/* Right Zone - Outgoing Interfaces */}
      {isBuilderView && (
        <div
          className="zone-right"
          style={{
            position: 'absolute',
            top: `${zoneCenterTop}px`,
            right: 0,
            bottom: `${zoneCenterBottom}px`,
            width: `${zoneRightWidth}px`,
            borderLeft: '1px dashed #c7d2fe',
            background: 'rgba(199, 210, 254, 0.05)',
            display: 'flex',
            flexDirection: 'column',
            padding: '8px',
            gap: '8px',
            overflowY: 'auto',
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', gap: '4px', marginBottom: '4px' }}>
            <ArrowLeft size={14} color="#10b981" />
            <span style={{ fontSize: '11px', fontWeight: '600', color: '#10b981' }}>
              OUTGOING
            </span>
          </div>
          {outgoingInterfaces.map((interfaceItem) => (
            <div
              key={interfaceItem.id}
              className="interface-port outgoing"
              style={{
                position: 'relative',
                padding: '8px 10px',
                borderRadius: '6px',
                background: '#f0fdf4',
                border: '2px solid #10b981',
                fontSize: '11px',
                fontWeight: '600',
                color: '#047857',
                cursor: 'pointer',
              }}
            >
              <Handle
                type="source"
                position={Position.Right}
                id={`interface-out-${interfaceItem.id}`}
                style={{
                  background: '#10b981',
                  width: '10px',
                  height: '10px',
                  border: '2px solid white',
                  right: '-6px',
                }}
              />
              <div>{interfaceItem.displayName || interfaceItem.name}</div>
              <div style={{ fontSize: '9px', color: '#34d399', marginTop: '2px' }}>
                {interfaceItem.interfaceType}
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Bottom Zone - External Service Connections (Output) */}
      {isBuilderView && (
        <div
          className="zone-bottom"
          style={{
            position: 'absolute',
            bottom: 0,
            left: 0,
            right: 0,
            height: `${zoneBottomHeight}px`,
            borderTop: '1px dashed #c7d2fe',
            background: 'rgba(199, 210, 254, 0.05)',
            display: 'flex',
            alignItems: 'center',
            padding: '8px',
            gap: '8px',
            overflowX: 'auto',
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', gap: '4px', marginRight: '8px' }}>
            <Network size={14} color="#06b6d4" />
            <span style={{ fontSize: '11px', fontWeight: '600', color: '#06b6d4' }}>
              EXTERNAL CONNECTIONS
            </span>
          </div>
          {externalConnections.map((connection) => (
            <div
              key={connection.id}
              className="connection-port"
              style={{
                position: 'relative',
                padding: '6px 10px',
                borderRadius: '6px',
                background: '#ecfeff',
                border: '2px solid #06b6d4',
                fontSize: '11px',
                fontWeight: '600',
                color: '#0e7490',
                whiteSpace: 'nowrap',
                cursor: 'pointer',
              }}
            >
              <Handle
                type="source"
                position={Position.Bottom}
                id={`connection-${connection.id}`}
                style={{
                  background: '#06b6d4',
                  width: '10px',
                  height: '10px',
                  border: '2px solid white',
                  bottom: '-6px',
                }}
              />
              → {connection.targetServiceName}
            </div>
          ))}
        </div>
      )}

      {/* Project View - Compact Display */}
      {isProjectView && (
        <div style={{ padding: '12px', textAlign: 'center' }}>
          <div style={{ fontSize: '11px', color: '#6b7280', marginTop: '8px' }}>
            {incomingInterfaces.length} in • {outgoingInterfaces.length} out
          </div>
        </div>
      )}
    </div>
  );
});

ServiceContainerNode.displayName = 'ServiceContainerNode';

