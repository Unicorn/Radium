/**
 * Retry Node - Wrapper for retry logic with exponential backoff
 */

import { memo } from 'react';
import { Handle, Position } from 'react-flow-renderer';
import { RotateCcw } from 'lucide-react';

interface RetryNodeProps {
  data: {
    label: string;
    config?: {
      maxAttempts?: number;
      retryOn?: 'failure' | 'error' | 'condition';
      condition?: string;
      backoff?: {
        type?: 'none' | 'linear' | 'exponential';
        initialInterval?: string;
        maxInterval?: string;
        multiplier?: number;
      };
      scope?: 'activity' | 'agent' | 'child-workflow' | 'block';
    };
  };
  selected?: boolean;
}

export const RetryNode = memo(({ data, selected }: RetryNodeProps) => {
  const maxAttempts = data.config?.maxAttempts || 3;
  const retryOn = data.config?.retryOn || 'failure';
  const backoffType = data.config?.backoff?.type || 'exponential';
  const scope = data.config?.scope || 'block';

  return (
    <div
      style={{
        padding: '14px 18px',
        borderRadius: '10px',
        backgroundColor: selected ? '#ec4899' : '#fce7f3',
        border: `2px dashed ${selected ? '#ec4899' : '#f9a8d4'}`,
        minWidth: '200px',
        position: 'relative',
      }}
    >
      <Handle type="target" position={Position.Top} />
      
      <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '8px' }}>
        <RotateCcw size={16} color={selected ? 'white' : '#be185d'} />
        <div style={{ fontSize: '10px', color: selected ? '#fce7f3' : '#9f1239', textTransform: 'uppercase', fontWeight: '600' }}>
          Retry Loop
        </div>
      </div>
      
      <div style={{ fontSize: '14px', fontWeight: '600', color: selected ? 'white' : '#831843', marginBottom: '8px' }}>
        {data.label}
      </div>
      
      <div style={{ marginTop: '8px', paddingTop: '8px', borderTop: `1px solid ${selected ? 'rgba(255,255,255,0.2)' : '#f9a8d4'}` }}>
        <div style={{ fontSize: '10px', color: selected ? '#fce7f3' : '#9f1239', display: 'flex', flexDirection: 'column', gap: '4px' }}>
          <div>🔄 Max: {maxAttempts} attempts</div>
          <div>📊 Retry on: {retryOn}</div>
          {backoffType !== 'none' && (
            <div>⏱️ Backoff: {backoffType}</div>
          )}
          <div>🎯 Scope: {scope}</div>
        </div>
      </div>

      <Handle type="source" position={Position.Bottom} />
    </div>
  );
});

RetryNode.displayName = 'RetryNode';

