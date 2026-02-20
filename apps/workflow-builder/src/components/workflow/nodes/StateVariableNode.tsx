/**
 * State Variable Node - Manages workflow state variables
 */

import { memo } from 'react';
import { Handle, Position } from 'react-flow-renderer';
import { Database } from 'lucide-react';

interface StateVariableNodeProps {
  data: {
    label: string;
    config?: {
      name?: string;
      operation?: 'set' | 'append' | 'increment' | 'decrement' | 'get';
      value?: string | number | boolean | object;
      scope?: 'workflow' | 'phase' | 'loop';
      initialValue?: string | number | boolean | object;
    };
  };
  selected?: boolean;
}

export const StateVariableNode = memo(({ data, selected }: StateVariableNodeProps) => {
  const varName = data.config?.name || data.label;
  const operation = data.config?.operation || 'set';
  const scope = data.config?.scope || 'workflow';

  const operationIcons: Record<string, string> = {
    set: '✏️',
    append: '➕',
    increment: '⬆️',
    decrement: '⬇️',
    get: '👁️',
  };

  const operationColors: Record<string, string> = {
    set: '#3b82f6',
    append: '#10b981',
    increment: '#f59e0b',
    decrement: '#ef4444',
    get: '#6366f1',
  };

  const icon = operationIcons[operation] || '✏️';
  const color = operationColors[operation] || '#3b82f6';

  return (
    <div
      style={{
        padding: '12px 16px',
        borderRadius: '8px',
        backgroundColor: selected ? color : `${color}15`,
        border: `2px solid ${selected ? color : `${color}40`}`,
        minWidth: '180px',
      }}
    >
      <Handle type="target" position={Position.Top} />
      
      <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '6px' }}>
        <Database size={14} color={selected ? 'white' : color} />
        <div style={{ fontSize: '10px', color: selected ? 'white' : color, textTransform: 'uppercase', fontWeight: '600' }}>
          State Variable
        </div>
      </div>
      
      <div style={{ fontSize: '13px', fontWeight: '600', color: selected ? 'white' : '#1f2937', marginBottom: '4px', fontFamily: 'monospace' }}>
        {icon} {varName}
      </div>
      
      <div style={{ marginTop: '8px', paddingTop: '8px', borderTop: `1px solid ${selected ? 'rgba(255,255,255,0.2)' : `${color}30`}` }}>
        <div style={{ fontSize: '10px', color: selected ? 'rgba(255,255,255,0.8)' : '#6b7280', display: 'flex', flexDirection: 'column', gap: '2px' }}>
          <div>Operation: {operation}</div>
          <div>Scope: {scope}</div>
          {data.config?.value !== undefined && (
            <div style={{ fontFamily: 'monospace', fontSize: '9px', marginTop: '4px', opacity: 0.8 }}>
              Value: {typeof data.config.value === 'string' ? data.config.value : JSON.stringify(data.config.value)}
            </div>
          )}
        </div>
      </div>

      <Handle type="source" position={Position.Bottom} />
    </div>
  );
});

StateVariableNode.displayName = 'StateVariableNode';

