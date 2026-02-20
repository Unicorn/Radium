/**
 * Phase Node - Container for organizing workflow into phases
 */

import { memo } from 'react';
import { Handle, Position } from 'react-flow-renderer';
import { Layers } from 'lucide-react';

interface PhaseNodeProps {
  data: {
    label: string;
    phaseName?: string;
    config?: {
      name?: string;
      description?: string;
      sequential?: boolean;
      maxConcurrency?: number;
    };
  };
  selected?: boolean;
}

export const PhaseNode = memo(({ data, selected }: PhaseNodeProps) => {
  const phaseName = data.config?.name || data.phaseName || data.label;
  const isSequential = data.config?.sequential !== false;
  const maxConcurrent = data.config?.maxConcurrency;

  return (
    <div
      style={{
        padding: '16px 20px',
        borderRadius: '12px',
        backgroundColor: selected ? '#6366f1' : '#eef2ff',
        border: `3px solid ${selected ? '#6366f1' : '#c7d2fe'}`,
        minWidth: '220px',
        boxShadow: selected ? '0 4px 12px rgba(99, 102, 241, 0.3)' : '0 2px 8px rgba(0, 0, 0, 0.1)',
      }}
    >
      <Handle type="target" position={Position.Top} />
      
      <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '8px' }}>
        <Layers size={18} color={selected ? 'white' : '#4f46e5'} />
        <div style={{ fontSize: '10px', color: selected ? '#e0e7ff' : '#6366f1', textTransform: 'uppercase', fontWeight: '700', letterSpacing: '0.5px' }}>
          Phase
        </div>
      </div>
      
      <div style={{ fontSize: '16px', fontWeight: '700', color: selected ? 'white' : '#312e81', marginBottom: '4px' }}>
        {phaseName}
      </div>
      
      {data.config?.description && (
        <div style={{ fontSize: '11px', color: selected ? '#c7d2fe' : '#6366f1', marginTop: '4px', fontStyle: 'italic' }}>
          {data.config.description}
        </div>
      )}
      
      <div style={{ marginTop: '12px', paddingTop: '12px', borderTop: `1px solid ${selected ? 'rgba(255,255,255,0.2)' : '#c7d2fe'}` }}>
        <div style={{ fontSize: '10px', color: selected ? '#c7d2fe' : '#6366f1', display: 'flex', gap: '12px', alignItems: 'center' }}>
          {isSequential ? (
            <span>📋 Sequential</span>
          ) : (
            <span>⚡ Concurrent {maxConcurrent ? `(max ${maxConcurrent})` : ''}</span>
          )}
        </div>
      </div>

      <Handle type="source" position={Position.Bottom} />
    </div>
  );
});

PhaseNode.displayName = 'PhaseNode';

