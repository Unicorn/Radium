/**
 * End Node - Custom node for workflow end points
 */

import { memo } from 'react';
import { Handle, Position } from 'react-flow-renderer';

interface EndNodeProps {
  data: {
    label: string;
    componentName?: string;
  };
  selected?: boolean;
}

export const EndNode = memo(({ data, selected }: EndNodeProps) => {
  return (
    <div
      style={{
        padding: '14px 18px',
        borderRadius: '10px',
        backgroundColor: selected ? '#dc2626' : '#fef5f5',
        border: `2px solid ${selected ? '#dc2626' : '#f87171'}`,
        minWidth: '200px',
        boxShadow: selected
          ? '0 4px 12px rgba(220, 38, 38, 0.25)'
          : '0 2px 8px rgba(248, 113, 113, 0.12)',
        transition: 'all 0.2s ease',
      }}
    >
      <div>
        <div
          style={{
            fontSize: '11px',
            color: selected ? '#fee2e2' : '#991b1b',
            textTransform: 'uppercase',
            fontWeight: '700',
            letterSpacing: '0.5px',
          }}
        >
          End
        </div>
        <div
          style={{
            fontSize: '16px',
            fontWeight: '700',
            color: selected ? 'white' : '#1f2937',
            marginTop: '6px',
            lineHeight: '1.3',
          }}
        >
          {data.label || 'End Workflow'}
        </div>
        {data.componentName && (
          <div
            style={{
              fontSize: '13px',
              color: selected ? '#fecaca' : '#78716c',
              marginTop: '4px',
              fontFamily: 'ui-monospace, monospace',
              fontWeight: '500',
            }}
          >
            {data.componentName}
          </div>
        )}
      </div>
      <Handle
        type="target"
        position={Position.Top}
        style={{
          background: selected ? '#fff' : '#dc2626',
          width: '12px',
          height: '12px',
          border: `2px solid ${selected ? '#dc2626' : '#fff'}`,
        }}
      />
    </div>
  );
});

EndNode.displayName = 'EndNode';
