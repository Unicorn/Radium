/**
 * Trigger Node - Custom node for workflow triggers
 */

import { memo, useState } from 'react';
import { Handle, Position } from 'react-flow-renderer';
import { Zap } from 'lucide-react';

interface TriggerNodeProps {
  data: {
    label: string;
    componentName?: string;
  };
  selected?: boolean;
}

export const TriggerNode = memo(({ data, selected }: TriggerNodeProps) => {
  const [isExpanded, setIsExpanded] = useState(false);

  return (
    <div
      onClick={() => setIsExpanded(!isExpanded)}
      style={{
        padding: '12px 16px',
        borderRadius: '10px',
        backgroundColor: selected ? '#f97316' : '#fffbf5',
        border: `2px solid ${selected ? '#f97316' : '#fb923c'}`,
        minWidth: isExpanded ? '200px' : '160px',
        maxWidth: isExpanded ? '280px' : '200px',
        boxShadow: selected
          ? '0 4px 12px rgba(249, 115, 22, 0.25)'
          : '0 2px 8px rgba(251, 146, 60, 0.12)',
        transition: 'all 0.2s ease',
        cursor: 'pointer',
      }}
    >
      <div>
        {/* Icon + Label */}
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
          <Zap
            size={18}
            color={selected ? 'white' : '#f97316'}
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
                color: selected ? '#ffedd5' : '#9a3412',
                textTransform: 'uppercase',
                fontWeight: '700',
                letterSpacing: '0.5px',
                marginTop: '8px',
              }}
            >
              Trigger
            </div>
            {data.componentName && (
              <div
                style={{
                  fontSize: '13px',
                  color: selected ? '#fed7aa' : '#78716c',
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
          background: selected ? '#fff' : '#f97316',
          width: '12px',
          height: '12px',
          border: `2px solid ${selected ? '#f97316' : '#fff'}`,
        }}
      />
    </div>
  );
});

TriggerNode.displayName = 'TriggerNode';
