/**
 * Signal Node - Custom node for signals
 */

import { memo } from 'react';
import { Handle, Position } from 'react-flow-renderer';

interface SignalNodeProps {
  data: {
    label: string;
    componentName?: string;
  };
  selected?: boolean;
}

export const SignalNode = memo(({ data, selected }: SignalNodeProps) => {
  return (
    <div
      style={{
        padding: '12px 16px',
        borderRadius: '8px',
        backgroundColor: selected ? '#10b981' : '#f0fdf4',
        border: `2px solid ${selected ? '#10b981' : '#bbf7d0'}`,
        minWidth: '180px',
      }}
    >
      <Handle type="target" position={Position.Top} />
      <div>
        <div style={{ fontSize: '10px', color: '#6b7280', textTransform: 'uppercase' }}>
          Signal
        </div>
        <div style={{ fontSize: '14px', fontWeight: '600', color: selected ? 'white' : '#1f2937', marginTop: '4px' }}>
          {data.label}
        </div>
        {data.componentName && (
          <div style={{ fontSize: '12px', color: selected ? '#d1fae5' : '#6b7280', marginTop: '2px', fontFamily: 'monospace' }}>
            {data.componentName}
          </div>
        )}
      </div>
      <Handle type="source" position={Position.Bottom} />
    </div>
  );
});

SignalNode.displayName = 'SignalNode';

