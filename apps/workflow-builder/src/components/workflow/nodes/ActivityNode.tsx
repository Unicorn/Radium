/**
 * Activity Node - Custom node for activities
 */

import { memo, useState } from 'react';
import { Handle, Position } from 'react-flow-renderer';
import { Box } from 'lucide-react';

interface ActivityNodeProps {
  data: {
    label: string;
    componentName?: string;
  };
  selected?: boolean;
}

export const ActivityNode = memo(({ data, selected }: ActivityNodeProps) => {
  const [isExpanded, setIsExpanded] = useState(false);

  return (
    <div
      onClick={() => setIsExpanded(!isExpanded)}
      style={{
        padding: '12px 16px',
        borderRadius: '10px',
        backgroundColor: selected ? '#3b82f6' : '#f0f9ff',
        border: `2px solid ${selected ? '#3b82f6' : '#60a5fa'}`,
        minWidth: isExpanded ? '200px' : '160px',
        maxWidth: isExpanded ? '280px' : '200px',
        boxShadow: selected
          ? '0 4px 12px rgba(59, 130, 246, 0.25)'
          : '0 2px 8px rgba(96, 165, 250, 0.12)',
        transition: 'all 0.2s ease',
        cursor: 'pointer',
      }}
    >
      <Handle
        type="target"
        position={Position.Top}
        style={{
          background: selected ? '#fff' : '#3b82f6',
          width: '12px',
          height: '12px',
          border: `2px solid ${selected ? '#3b82f6' : '#fff'}`,
        }}
      />
      <div>
        {/* Icon + Label */}
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
          <Box
            size={18}
            color={selected ? 'white' : '#3b82f6'}
            strokeWidth={2.5}
            style={{ flexShrink: 0 }}
          />
          <div
            style={{
              fontSize: '15px',
              fontWeight: '700',
              color: selected ? 'white' : '#1f2937',
              lineHeight: '1.3',
              flex: 1,
            }}
          >
            {data.label}
          </div>
        </div>

        {/* Expanded details */}
        {isExpanded && (
          <>
            <div
              style={{
                fontSize: '11px',
                color: selected ? '#dbeafe' : '#1e40af',
                textTransform: 'uppercase',
                fontWeight: '700',
                letterSpacing: '0.5px',
                marginTop: '8px',
              }}
            >
              Activity
            </div>
            {data.componentName && (
              <div
                style={{
                  fontSize: '13px',
                  color: selected ? '#bfdbfe' : '#78716c',
                  marginTop: '4px',
                  fontFamily: 'ui-monospace, monospace',
                  fontWeight: '500',
                }}
              >
                {data.componentName}
              </div>
            )}
          </>
        )}
      </div>
      <Handle
        type="source"
        position={Position.Bottom}
        style={{
          background: selected ? '#fff' : '#3b82f6',
          width: '12px',
          height: '12px',
          border: `2px solid ${selected ? '#3b82f6' : '#fff'}`,
        }}
      />
    </div>
  );
});

ActivityNode.displayName = 'ActivityNode';

