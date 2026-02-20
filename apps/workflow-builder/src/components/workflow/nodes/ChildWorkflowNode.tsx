/**
 * Child Workflow Node - Enhanced with parent communication indicators
 */

import { memo } from 'react';
import { Handle, Position } from 'react-flow-renderer';
import { ArrowUp, Search, PauseCircle } from 'lucide-react';

interface ChildWorkflowNodeProps {
  data: {
    label: string;
    workflowName?: string;
    signalToParent?: string;
    queryParent?: string;
    blockUntil?: string[];
    hasParentCommunication?: boolean;
    hasBlockingDependencies?: boolean;
  };
  selected?: boolean;
}

export const ChildWorkflowNode = memo(({ data, selected }: ChildWorkflowNodeProps) => {
  return (
    <div
      style={{
        padding: '12px 16px',
        borderRadius: '8px',
        backgroundColor: selected ? '#3b82f6' : '#eff6ff',
        border: `2px solid ${selected ? '#3b82f6' : '#bfdbfe'}`,
        minWidth: '200px',
        position: 'relative',
      }}
    >
      <Handle type="target" position={Position.Top} />
      
      {/* Parent Communication Indicators */}
      {data.hasParentCommunication && (
        <div
          style={{
            position: 'absolute',
            top: '-8px',
            right: '8px',
            display: 'flex',
            gap: '4px',
          }}
        >
          {data.signalToParent && (
            <div
              style={{
                backgroundColor: '#3b82f6',
                borderRadius: '50%',
                padding: '4px',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
              }}
              title={`Signals to parent: ${data.signalToParent}`}
            >
              <ArrowUp size={12} color="white" />
            </div>
          )}
          {data.queryParent && (
            <div
              style={{
                backgroundColor: '#14b8a6',
                borderRadius: '50%',
                padding: '4px',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
              }}
              title={`Queries parent: ${data.queryParent}`}
            >
              <Search size={12} color="white" />
            </div>
          )}
        </div>
      )}

      {/* Blocking Dependency Indicator */}
      {data.hasBlockingDependencies && (
        <div
          style={{
            position: 'absolute',
            top: '-8px',
            left: '8px',
            backgroundColor: '#f97316',
            borderRadius: '50%',
            padding: '4px',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
          }}
          title={`Blocked until: ${data.blockUntil?.join(', ')}`}
        >
          <PauseCircle size={12} color="white" />
        </div>
      )}

      <div>
        <div style={{ fontSize: '10px', color: '#6b7280', textTransform: 'uppercase', fontWeight: '600' }}>
          Child Workflow
        </div>
        <div style={{ fontSize: '14px', fontWeight: '600', color: selected ? 'white' : '#1f2937', marginTop: '4px' }}>
          {data.label}
        </div>
        {data.workflowName && (
          <div style={{ fontSize: '12px', color: selected ? '#dbeafe' : '#6b7280', marginTop: '2px', fontFamily: 'monospace' }}>
            {data.workflowName}
          </div>
        )}
      </div>

      {/* Communication Summary */}
      {(data.hasParentCommunication || data.hasBlockingDependencies) && (
        <div style={{ marginTop: '8px', paddingTop: '8px', borderTop: `1px solid ${selected ? 'rgba(255,255,255,0.2)' : '#e5e7eb'}` }}>
          <div style={{ fontSize: '10px', color: selected ? '#dbeafe' : '#9ca3af' }}>
            {data.signalToParent && '📤 Signal'}
            {data.signalToParent && data.queryParent && ' • '}
            {data.queryParent && '🔍 Query'}
            {(data.signalToParent || data.queryParent) && data.hasBlockingDependencies && ' • '}
            {data.hasBlockingDependencies && '🚫 Blocked'}
          </div>
        </div>
      )}

      <Handle type="source" position={Position.Bottom} />
    </div>
  );
});

ChildWorkflowNode.displayName = 'ChildWorkflowNode';

