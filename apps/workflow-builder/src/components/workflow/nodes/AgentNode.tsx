/**
 * Agent Node - Custom node for AI agents
 */

import { memo, useState } from 'react';
import { Handle, Position } from 'react-flow-renderer';
import { Radio } from 'lucide-react';

interface AgentNodeProps {
  data: {
    label: string;
    componentName?: string;
  };
  selected?: boolean;
}

export const AgentNode = memo(({ data, selected }: AgentNodeProps) => {
  const [isExpanded, setIsExpanded] = useState(false);

  return (
    <div
      onClick={() => setIsExpanded(!isExpanded)}
      style={{
        padding: '12px 16px',
        borderRadius: '10px',
        backgroundColor: selected ? '#a855f7' : '#faf5ff',
        border: `2px solid ${selected ? '#a855f7' : '#c084fc'}`,
        minWidth: isExpanded ? '200px' : '160px',
        maxWidth: isExpanded ? '280px' : '200px',
        boxShadow: selected
          ? '0 4px 12px rgba(168, 85, 247, 0.25)'
          : '0 2px 8px rgba(192, 132, 252, 0.12)',
        transition: 'all 0.2s ease',
        cursor: 'pointer',
      }}
    >
      <Handle
        type="target"
        position={Position.Top}
        style={{
          background: selected ? '#fff' : '#a855f7',
          width: '12px',
          height: '12px',
          border: `2px solid ${selected ? '#a855f7' : '#fff'}`,
        }}
      />
      <div>
        {/* Icon + Label */}
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
          <Radio
            size={18}
            color={selected ? 'white' : '#a855f7'}
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
                color: selected ? '#f3e8ff' : '#6b21a8',
                textTransform: 'uppercase',
                fontWeight: '700',
                letterSpacing: '0.5px',
                marginTop: '8px',
              }}
            >
              Agent
            </div>
            {data.componentName && (
              <div
                style={{
                  fontSize: '13px',
                  color: selected ? '#e9d5ff' : '#78716c',
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
          background: selected ? '#fff' : '#a855f7',
          width: '12px',
          height: '12px',
          border: `2px solid ${selected ? '#a855f7' : '#fff'}`,
        }}
      />
    </div>
  );
});

AgentNode.displayName = 'AgentNode';
