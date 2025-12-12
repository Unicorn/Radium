/**
 * Condition Node - Branching logic with true/false paths
 */

import { memo } from 'react';
import { Handle, Position } from 'react-flow-renderer';
import { GitBranch } from 'lucide-react';

interface ConditionNodeProps {
  data: {
    label: string;
    expression?: string;
    config?: {
      expression?: string;
      conditions?: Array<{
        variable: string;
        operator: string;
        value: any;
      }>;
      operator?: 'AND' | 'OR';
    };
  };
  selected?: boolean;
}

export const ConditionNode = memo(({ data, selected }: ConditionNodeProps) => {
  const expression = data.config?.expression || data.expression || 'condition';
  const isSimple = expression.length < 50;

  return (
    <div
      style={{
        padding: '18px',
        borderRadius: '10px',
        backgroundColor: selected ? '#f59e0b' : '#fffbeb',
        border: `2px solid ${selected ? '#f59e0b' : '#fbbf24'}`,
        minWidth: '200px',
        transform: 'rotate(45deg)',
        position: 'relative',
        boxShadow: selected
          ? '0 4px 12px rgba(245, 158, 11, 0.25)'
          : '0 2px 8px rgba(251, 191, 36, 0.12)',
        transition: 'all 0.2s ease',
      }}
    >
      {/* Rotate content back */}
      <div style={{ transform: 'rotate(-45deg)' }}>
        <Handle
          type="target"
          position={Position.Top}
          style={{
            transform: 'rotate(-45deg)',
            top: '-8px',
            background: selected ? '#fff' : '#f59e0b',
            width: '12px',
            height: '12px',
            border: `2px solid ${selected ? '#f59e0b' : '#fff'}`,
          }}
        />

        <div style={{ display: 'flex', alignItems: 'center', gap: '8px', justifyContent: 'center' }}>
          <GitBranch size={18} color={selected ? 'white' : '#92400e'} strokeWidth={2.5} />
          <div style={{ fontSize: '11px', color: selected ? 'white' : '#92400e', textTransform: 'uppercase', fontWeight: '700', letterSpacing: '0.5px' }}>
            Condition
          </div>
        </div>

        <div
          style={{
            fontSize: '13px',
            fontWeight: '600',
            color: selected ? 'white' : '#78350f',
            marginTop: '8px',
            textAlign: 'center',
            fontFamily: 'ui-monospace, monospace',
            wordBreak: 'break-word',
          }}
        >
          {isSimple ? expression : `${expression.substring(0, 30)}...`}
        </div>
      </div>

      {/* True output (right) */}
      <Handle
        type="source"
        position={Position.Right}
        id="true"
        style={{
          transform: 'rotate(-45deg)',
          right: '-8px',
          backgroundColor: '#10b981',
          width: '12px',
          height: '12px',
          border: '2px solid #fff',
        }}
      >
        <div
          style={{
            position: 'absolute',
            right: '22px',
            top: '50%',
            transform: 'translateY(-50%)',
            fontSize: '10px',
            color: '#10b981',
            fontWeight: '700',
            whiteSpace: 'nowrap',
            backgroundColor: 'white',
            padding: '2px 6px',
            borderRadius: '4px',
            border: '1px solid #10b981',
          }}
        >
          TRUE
        </div>
      </Handle>

      {/* False output (left) */}
      <Handle
        type="source"
        position={Position.Left}
        id="false"
        style={{
          transform: 'rotate(-45deg)',
          left: '-8px',
          backgroundColor: '#ef4444',
          width: '12px',
          height: '12px',
          border: '2px solid #fff',
        }}
      >
        <div
          style={{
            position: 'absolute',
            left: '22px',
            top: '50%',
            transform: 'translateY(-50%)',
            fontSize: '10px',
            color: '#ef4444',
            fontWeight: '700',
            whiteSpace: 'nowrap',
            backgroundColor: 'white',
            padding: '2px 6px',
            borderRadius: '4px',
            border: '1px solid #ef4444',
          }}
        >
          FALSE
        </div>
      </Handle>
    </div>
  );
});

ConditionNode.displayName = 'ConditionNode';

