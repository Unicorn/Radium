'use client';

import { YStack, XStack, Text, Input, TextArea, Label, Button, ScrollView, Card, Switch, Select, Adapt, Sheet, Separator } from 'tamagui';
import { X, Save, Check, ChevronDown } from 'lucide-react';
import { useState, useEffect } from 'react';
import { CronExpressionBuilder } from '../cron/CronExpressionBuilder';
import { api } from '@/lib/trpc/client';
import { ConnectorSelector } from '../connector/ConnectorSelector';

interface NodePropertyPanelProps {
  node: {
    id: string;
    type: 'activity' | 'agent' | 'signal' | 'query' | 'work-queue' | 'scheduled-workflow' | 'child-workflow' | 'api-endpoint' | 'condition' | 'phase' | 'retry' | 'state-variable' | 'data-in' | 'data-out' | 'kong-logging' | 'kong-cache' | 'kong-cors' | 'graphql-gateway' | 'mcp-server';
    data: Record<string, any>;
  } | null;
  onClose: () => void;
  onSave: (nodeId: string, updates: Record<string, any>) => void;
  availableSignals?: Array<{ id: string; signal_name: string }>;
  availableQueries?: Array<{ id: string; query_name: string }>;
  projectId?: string;
}

export function NodePropertyPanel({ node, onClose, onSave, availableSignals = [], availableQueries = [], projectId }: NodePropertyPanelProps) {
  const [properties, setProperties] = useState(node?.data || {});
  const [hasChanges, setHasChanges] = useState(false);

  if (!node) return null;

  const handlePropertyChange = (key: string, value: any) => {
    setProperties(prev => ({ ...prev, [key]: value }));
    setHasChanges(true);
  };

  const handleSave = () => {
    onSave(node.id, properties);
    setHasChanges(false);
  };

  return (
    <YStack
      f={1}
      gap="$3"
      p="$4"
      bg="$background"
      borderLeftWidth={1}
      borderLeftColor="$borderColor"
      maxWidth={350}
      position="relative"
      role="complementary"
      aria-label="Node properties panel"
    >
      {/* Header */}
      <XStack ai="center" jc="space-between">
        <YStack gap="$1">
          <Text fontSize="$5" fontWeight="600">Properties</Text>
          <Text fontSize="$2" color="$gray11" textTransform="capitalize">
            {node.type.replace('-', ' ')}
          </Text>
        </YStack>
        <Button
          size="$3"
          icon={X}
          onPress={onClose}
          chromeless
          aria-label="Close properties panel"
        />
      </XStack>

      {/* Content */}
      <ScrollView
        f={1}
        showsVerticalScrollIndicator={false}
        role="region"
        aria-label="Property fields"
      >
        <YStack gap="$4">
          {node.type === 'activity' && <ActivityProperties properties={properties} onChange={handlePropertyChange} node={node} />}
          {node.type === 'agent' && <AgentProperties properties={properties} onChange={handlePropertyChange} />}
          {node.type === 'signal' && <SignalProperties properties={properties} onChange={handlePropertyChange} />}
          {node.type === 'query' && <QueryProperties properties={properties} onChange={handlePropertyChange} />}
          {node.type === 'work-queue' && <WorkQueueProperties properties={properties} onChange={handlePropertyChange} />}
          {node.type === 'scheduled-workflow' && <ScheduledWorkflowProperties properties={properties} onChange={handlePropertyChange} />}
          {node.type === 'child-workflow' && <ChildWorkflowProperties properties={properties} onChange={handlePropertyChange} />}
          {node.type === 'api-endpoint' && <ApiEndpointProperties properties={properties} onChange={handlePropertyChange} availableSignals={availableSignals} availableQueries={availableQueries} />}
          {node.type === 'condition' && <ConditionProperties properties={properties} onChange={handlePropertyChange} />}
          {node.type === 'kong-logging' && <KongLoggingProperties properties={properties} onChange={handlePropertyChange} node={node} />}
          {node.type === 'kong-cache' && <KongCacheProperties properties={properties} onChange={handlePropertyChange} node={node} />}
          {node.type === 'kong-cors' && <KongCorsProperties properties={properties} onChange={handlePropertyChange} node={node} />}
          {node.type === 'graphql-gateway' && <GraphQLGatewayProperties properties={properties} onChange={handlePropertyChange} node={node} />}
          {node.type === 'mcp-server' && <MCPServerProperties properties={properties} onChange={handlePropertyChange} node={node} />}
          {node.type === 'phase' && <PhaseProperties properties={properties} onChange={handlePropertyChange} />}
          {node.type === 'retry' && <RetryProperties properties={properties} onChange={handlePropertyChange} />}
          {node.type === 'state-variable' && <StateVariableProperties properties={{ ...properties, projectId }} onChange={handlePropertyChange} />}
        </YStack>
      </ScrollView>

      {/* Actions */}
      <XStack
        gap="$2"
        jc="flex-end"
        borderTopWidth={1}
        borderTopColor="$borderColor"
        pt="$3"
        role="group"
        aria-label="Property panel actions"
      >
        <Button onPress={onClose} chromeless aria-label="Cancel changes">
          Cancel
        </Button>
        <Button
          themeInverse
          icon={hasChanges ? Save : Check}
          onPress={handleSave}
          disabled={!hasChanges}
          aria-label={hasChanges ? 'Save property changes' : 'Changes saved'}
        >
          {hasChanges ? 'Save Changes' : 'Saved'}
        </Button>
      </XStack>
    </YStack>
  );
}

// Activity Properties
function ActivityProperties({ properties, onChange, node }: any) {
  const componentName = properties.componentName || node?.data?.componentName;
  
  // Show specialized properties for database components
  if (componentName === 'postgresql-query') {
    return <PostgreSQLProperties properties={properties} onChange={onChange} />;
  }
  
  if (componentName === 'redis-command') {
    return <RedisProperties properties={properties} onChange={onChange} />;
  }
  
  if (componentName === 'typescript-processor') {
    return <TypeScriptProperties properties={properties} onChange={onChange} />;
  }
  
  return (
    <YStack gap="$3">
      <Text fontSize="$4" fontWeight="600">Activity Configuration</Text>

      <YStack gap="$2">
        <Label>Activity Name</Label>
        <Input
          value={properties.activityName || ''}
          onChangeText={(v) => onChange('activityName', v)}
          placeholder="e.g., processDataActivity"
        />
      </YStack>

      <YStack gap="$2">
        <Label>Channel</Label>
        <Input
          value={properties.taskQueue || properties.channel || ''}
          onChangeText={(v) => {
            onChange('taskQueue', v); // Keep legacy field
            onChange('channel', v); // New field
          }}
          placeholder="e.g., data-processing-channel"
        />
      </YStack>

      <YStack gap="$2">
        <Label>Timeout (seconds)</Label>
        <Input
          value={properties.timeout || ''}
          onChangeText={(v) => onChange('timeout', v)}
          placeholder="300"
          keyboardType="numeric"
        />
      </YStack>

      <Separator />
      
      <YStack gap="$2">
        <Label>Retry Policy</Label>
        <Select
          value={properties.retryPolicy?.strategy || 'none'}
          onValueChange={(v) => onChange('retryPolicy', {
            ...properties.retryPolicy,
            strategy: v,
          })}
        >
          <Select.Trigger iconAfter={ChevronDown}>
            <Select.Value placeholder="Select retry strategy" />
          </Select.Trigger>
          <Select.Content>
            <Select.Viewport>
              <Select.Item index={0} value="none">
                <Select.ItemText>No retries</Select.ItemText>
              </Select.Item>
              <Select.Item index={1} value="keep-trying">
                <Select.ItemText>Keep trying until success</Select.ItemText>
              </Select.Item>
              <Select.Item index={2} value="fail-after-x">
                <Select.ItemText>Fail after X attempts</Select.ItemText>
              </Select.Item>
              <Select.Item index={3} value="exponential-backoff">
                <Select.ItemText>Exponential backoff</Select.ItemText>
              </Select.Item>
            </Select.Viewport>
          </Select.Content>
        </Select>
      </YStack>

      {properties.retryPolicy?.strategy === 'fail-after-x' && (
        <YStack gap="$2">
          <Label>Max Attempts</Label>
          <Input
            value={properties.retryPolicy?.maxAttempts?.toString() || ''}
            onChangeText={(v) => onChange('retryPolicy', {
              ...properties.retryPolicy,
              maxAttempts: v ? parseInt(v, 10) : undefined,
            })}
            placeholder="e.g., 3"
            keyboardType="numeric"
          />
        </YStack>
      )}

      {properties.retryPolicy?.strategy === 'exponential-backoff' && (
        <YStack gap="$3">
          <YStack gap="$2">
            <Label>Max Attempts</Label>
            <Input
              value={properties.retryPolicy?.maxAttempts?.toString() || ''}
              onChangeText={(v) => onChange('retryPolicy', {
                ...properties.retryPolicy,
                maxAttempts: v ? parseInt(v, 10) : undefined,
              })}
              placeholder="e.g., 5"
              keyboardType="numeric"
            />
          </YStack>
          <YStack gap="$2">
            <Label>Initial Interval</Label>
            <Input
              value={properties.retryPolicy?.initialInterval || ''}
              onChangeText={(v) => onChange('retryPolicy', {
                ...properties.retryPolicy,
                initialInterval: v,
              })}
              placeholder="e.g., 1s, 5m"
            />
            <Text fontSize="$1" color="$gray11">Format: 1s, 5m, 1h</Text>
          </YStack>
          <YStack gap="$2">
            <Label>Max Interval</Label>
            <Input
              value={properties.retryPolicy?.maxInterval || ''}
              onChangeText={(v) => onChange('retryPolicy', {
                ...properties.retryPolicy,
                maxInterval: v,
              })}
              placeholder="e.g., 1h"
            />
          </YStack>
          <YStack gap="$2">
            <Label>Backoff Coefficient</Label>
            <Input
              value={properties.retryPolicy?.backoffCoefficient?.toString() || ''}
              onChangeText={(v) => onChange('retryPolicy', {
                ...properties.retryPolicy,
                backoffCoefficient: v ? parseFloat(v) : undefined,
              })}
              placeholder="e.g., 2.0"
              keyboardType="decimal-pad"
            />
          </YStack>
        </YStack>
      )}
    </YStack>
  );
}

// Agent Properties
function AgentProperties({ properties, onChange }: any) {
  return (
    <YStack gap="$3">
      <Text fontSize="$4" fontWeight="600">Agent Configuration</Text>

      <YStack gap="$2">
        <Label>Agent Prompt ID</Label>
        <Input
          value={properties.agentPromptId || ''}
          onChangeText={(v) => onChange('agentPromptId', v)}
          placeholder="Select or enter prompt ID"
        />
      </YStack>

      <YStack gap="$2">
        <Label>Model</Label>
        <Select value={properties.model || 'gpt-4'} onValueChange={(v) => onChange('model', v)}>
          <Select.Trigger iconAfter={ChevronDown}>
            <Select.Value placeholder="Select model" />
          </Select.Trigger>
          <Select.Content>
            <Select.Viewport>
              <Select.Item index={0} value="gpt-4">
                <Select.ItemText>GPT-4</Select.ItemText>
              </Select.Item>
              <Select.Item index={1} value="gpt-4-turbo">
                <Select.ItemText>GPT-4 Turbo</Select.ItemText>
              </Select.Item>
              <Select.Item index={2} value="claude-3-opus">
                <Select.ItemText>Claude 3 Opus</Select.ItemText>
              </Select.Item>
            </Select.Viewport>
          </Select.Content>
        </Select>
      </YStack>

      <YStack gap="$2">
        <Label>Temperature</Label>
        <Input
          value={properties.temperature || '0.7'}
          onChangeText={(v) => onChange('temperature', v)}
          keyboardType="decimal-pad"
        />
      </YStack>

      <YStack gap="$2">
        <Label>Max Tokens</Label>
        <Input
          value={properties.maxTokens || ''}
          onChangeText={(v) => onChange('maxTokens', v)}
          keyboardType="numeric"
        />
      </YStack>
    </YStack>
  );
}

// Signal Properties
function SignalProperties({ properties, onChange }: any) {
  return (
    <YStack gap="$3">
      <Text fontSize="$4" fontWeight="600">Signal Configuration</Text>

      <YStack gap="$2">
        <Label>Signal Name</Label>
        <Input
          value={properties.signalName || ''}
          onChangeText={(v) => onChange('signalName', v)}
          placeholder="e.g., updateStatus"
        />
      </YStack>

      <YStack gap="$2">
        <Label>Parameters Schema (JSON)</Label>
        <TextArea
          value={properties.parametersSchema || ''}
          onChangeText={(v) => onChange('parametersSchema', v)}
          placeholder='{"type": "object", "properties": {...}}'
          fontFamily="$mono"
          numberOfLines={5}
        />
      </YStack>
    </YStack>
  );
}

// Query Properties
function QueryProperties({ properties, onChange }: any) {
  return (
    <YStack gap="$3">
      <Text fontSize="$4" fontWeight="600">Query Configuration</Text>

      <YStack gap="$2">
        <Label>Query Name</Label>
        <Input
          value={properties.queryName || ''}
          onChangeText={(v) => onChange('queryName', v)}
          placeholder="e.g., getStatus"
        />
      </YStack>

      <YStack gap="$2">
        <Label>Return Type Schema (JSON)</Label>
        <TextArea
          value={properties.returnTypeSchema || ''}
          onChangeText={(v) => onChange('returnTypeSchema', v)}
          placeholder='{"type": "object", "properties": {...}}'
          fontFamily="$mono"
          numberOfLines={5}
        />
      </YStack>
    </YStack>
  );
}

// Work Queue Properties
function WorkQueueProperties({ properties, onChange }: any) {
  return (
    <YStack gap="$3">
      <Text fontSize="$4" fontWeight="600">Work Queue Configuration</Text>

      <YStack gap="$2">
        <Label>Queue Name</Label>
        <Input
          value={properties.queueName || ''}
          onChangeText={(v) => onChange('queueName', v)}
          placeholder="e.g., pendingTasks"
        />
      </YStack>

      <YStack gap="$2">
        <Label>Priority</Label>
        <Select value={properties.priority || 'fifo'} onValueChange={(v) => onChange('priority', v)}>
          <Select.Trigger iconAfter={ChevronDown}>
            <Select.Value placeholder="Select priority" />
          </Select.Trigger>
          <Select.Content>
            <Select.Viewport>
              <Select.Item index={0} value="fifo">
                <Select.ItemText>FIFO</Select.ItemText>
              </Select.Item>
              <Select.Item index={1} value="lifo">
                <Select.ItemText>LIFO</Select.ItemText>
              </Select.Item>
              <Select.Item index={2} value="priority">
                <Select.ItemText>Priority</Select.ItemText>
              </Select.Item>
            </Select.Viewport>
          </Select.Content>
        </Select>
      </YStack>

      <XStack ai="center" gap="$3">
        <Switch checked={properties.deduplicate || false} onCheckedChange={(v) => onChange('deduplicate', v)}>
          <Switch.Thumb animation="quick" />
        </Switch>
        <Label flex={1}>Enable Deduplication</Label>
      </XStack>
    </YStack>
  );
}

// Scheduled Workflow Properties
function ScheduledWorkflowProperties({ properties, onChange }: any) {
  return (
    <YStack gap="$3">
      <Text fontSize="$4" fontWeight="600">Scheduled Workflow Configuration</Text>

      <YStack gap="$2">
        <Label>Workflow Name</Label>
        <Input
          value={properties.workflowName || ''}
          onChangeText={(v) => onChange('workflowName', v)}
          placeholder="e.g., dailyCleanup"
        />
      </YStack>

      <Separator />

      <CronExpressionBuilder
        value={properties.cronExpression || ''}
        onChange={(v) => onChange('cronExpression', v)}
      />

      <Separator />

      <XStack ai="center" gap="$3">
        <Switch checked={properties.startImmediately || false} onCheckedChange={(v) => onChange('startImmediately', v)}>
          <Switch.Thumb animation="quick" />
        </Switch>
        <Label flex={1}>Start Immediately</Label>
      </XStack>

      <XStack ai="center" gap="$3">
        <Switch checked={properties.endWithParent || false} onCheckedChange={(v) => onChange('endWithParent', v)}>
          <Switch.Thumb animation="quick" />
        </Switch>
        <Label flex={1}>End with Parent</Label>
      </XStack>
    </YStack>
  );
}

// Child Workflow Properties
function ChildWorkflowProperties({ properties, onChange }: any) {
  return (
    <YStack gap="$3">
      <Text fontSize="$4" fontWeight="600">Child Workflow Configuration</Text>

      <YStack gap="$2">
        <Label>Workflow Type</Label>
        <Input
          value={properties.workflowType || ''}
          onChangeText={(v) => onChange('workflowType', v)}
          placeholder="e.g., processOrderWorkflow"
        />
      </YStack>

      <YStack gap="$2">
        <Label>Channel</Label>
        <Input
          value={properties.taskQueue || properties.channel || ''}
          onChangeText={(v) => {
            onChange('taskQueue', v); // Keep legacy field
            onChange('channel', v); // New field
          }}
          placeholder="e.g., order-processing"
        />
      </YStack>

      <Separator />

      <Text fontSize="$3" fontWeight="600">Parent Communication</Text>

      <YStack gap="$2">
        <Label>Signal to Parent (optional)</Label>
        <Input
          value={properties.signalToParent || ''}
          onChangeText={(v) => onChange('signalToParent', v)}
          placeholder="e.g., orderCompleted"
        />
      </YStack>

      <YStack gap="$2">
        <Label>Query Parent (optional)</Label>
        <Input
          value={properties.queryParent || ''}
          onChangeText={(v) => onChange('queryParent', v)}
          placeholder="e.g., getOrderStatus"
        />
      </YStack>

      <Separator />

      <Text fontSize="$3" fontWeight="600">Dependency Blocking</Text>

      <YStack gap="$2">
        <Label>Block Until Work Queue</Label>
        <Input
          value={properties.blockUntilQueue || ''}
          onChangeText={(v) => onChange('blockUntilQueue', v)}
          placeholder="e.g., prerequisites"
        />
      </YStack>
    </YStack>
  );
}

// API Endpoint Properties
function ApiEndpointProperties({ properties, onChange, availableSignals = [], availableQueries = [] }: any) {
  const config = properties.config || {};
  const targetType = config.targetType || 'start';

  const updateConfig = (key: string, value: any) => {
    onChange('config', { ...config, [key]: value });
  };

  return (
    <YStack gap="$3">
      <Text fontSize="$4" fontWeight="600">API Endpoint Configuration</Text>

      <YStack gap="$2">
        <Label>HTTP Method</Label>
        <Select value={config.method || 'POST'} onValueChange={(v) => updateConfig('method', v)}>
          <Select.Trigger>
            <Select.Value placeholder="Select method" />
          </Select.Trigger>
          <Adapt when="sm" platform="touch">
            <Sheet modal dismissOnSnapToBottom>
              <Sheet.Frame>
                <Sheet.ScrollView>
                  <Adapt.Contents />
                </Sheet.ScrollView>
              </Sheet.Frame>
              <Sheet.Overlay />
            </Sheet>
          </Adapt>
          <Select.Content zIndex={200000}>
            <Select.ScrollUpButton />
            <Select.Viewport>
              <Select.Item value="GET" index={0}>
                <Select.ItemText>GET</Select.ItemText>
              </Select.Item>
              <Select.Item value="POST" index={1}>
                <Select.ItemText>POST</Select.ItemText>
              </Select.Item>
              <Select.Item value="PUT" index={2}>
                <Select.ItemText>PUT</Select.ItemText>
              </Select.Item>
              <Select.Item value="DELETE" index={3}>
                <Select.ItemText>DELETE</Select.ItemText>
              </Select.Item>
              <Select.Item value="PATCH" index={4}>
                <Select.ItemText>PATCH</Select.ItemText>
              </Select.Item>
            </Select.Viewport>
            <Select.ScrollDownButton />
          </Select.Content>
        </Select>
      </YStack>

      <YStack gap="$2">
        <Label>Endpoint Path</Label>
        <Input
          value={config.endpointPath || ''}
          onChangeText={(v) => updateConfig('endpointPath', v)}
          placeholder="/orders"
        />
        <Text fontSize="$1" color="$gray10">
          Relative path (e.g., /orders, /api/v1/users)
        </Text>
      </YStack>

      <YStack gap="$2">
        <Label>Description</Label>
        <TextArea
          value={config.description || ''}
          onChangeText={(v) => updateConfig('description', v)}
          placeholder="What this endpoint does"
          minHeight={60}
        />
      </YStack>

      <Separator />

      <YStack gap="$2">
        <Label>Target Type</Label>
        <Select value={targetType} onValueChange={(v) => updateConfig('targetType', v)}>
          <Select.Trigger>
            <Select.Value />
          </Select.Trigger>
          <Adapt when="sm" platform="touch">
            <Sheet modal dismissOnSnapToBottom>
              <Sheet.Frame>
                <Sheet.ScrollView>
                  <Adapt.Contents />
                </Sheet.ScrollView>
              </Sheet.Frame>
              <Sheet.Overlay />
            </Sheet>
          </Adapt>
          <Select.Content zIndex={200000}>
            <Select.ScrollUpButton />
            <Select.Viewport>
              <Select.Item value="start" index={0}>
                <Select.ItemText>Start Workflow</Select.ItemText>
              </Select.Item>
              <Select.Item value="signal" index={1}>
                <Select.ItemText>Send Signal</Select.ItemText>
              </Select.Item>
              <Select.Item value="query" index={2}>
                <Select.ItemText>Query State</Select.ItemText>
              </Select.Item>
            </Select.Viewport>
            <Select.ScrollDownButton />
          </Select.Content>
        </Select>
      </YStack>

      {targetType === 'signal' && (
        <YStack gap="$2">
          <Label>Signal Name</Label>
          <Select
            value={config.targetName || ''}
            onValueChange={(v) => updateConfig('targetName', v)}
          >
            <Select.Trigger>
              <Select.Value placeholder="Select signal" />
            </Select.Trigger>
            <Adapt when="sm" platform="touch">
              <Sheet modal dismissOnSnapToBottom>
                <Sheet.Frame>
                  <Sheet.ScrollView>
                    <Adapt.Contents />
                  </Sheet.ScrollView>
                </Sheet.Frame>
                <Sheet.Overlay />
              </Sheet>
            </Adapt>
            <Select.Content zIndex={200000}>
              <Select.ScrollUpButton />
              <Select.Viewport>
                {availableSignals.map((signal: any, index: number) => (
                  <Select.Item key={signal.id} value={signal.signal_name} index={index}>
                    <Select.ItemText>{signal.signal_name}</Select.ItemText>
                  </Select.Item>
                ))}
              </Select.Viewport>
              <Select.ScrollDownButton />
            </Select.Content>
          </Select>
        </YStack>
      )}

      {targetType === 'query' && (
        <YStack gap="$2">
          <Label>Query Name</Label>
          <Select
            value={config.targetName || ''}
            onValueChange={(v) => updateConfig('targetName', v)}
          >
            <Select.Trigger>
              <Select.Value placeholder="Select query" />
            </Select.Trigger>
            <Adapt when="sm" platform="touch">
              <Sheet modal dismissOnSnapToBottom>
                <Sheet.Frame>
                  <Sheet.ScrollView>
                    <Adapt.Contents />
                  </Sheet.ScrollView>
                </Sheet.Frame>
                <Sheet.Overlay />
              </Sheet>
            </Adapt>
            <Select.Content zIndex={200000}>
              <Select.ScrollUpButton />
              <Select.Viewport>
                {availableQueries.map((query: any, index: number) => (
                  <Select.Item key={query.id} value={query.query_name} index={index}>
                    <Select.ItemText>{query.query_name}</Select.ItemText>
                  </Select.Item>
                ))}
              </Select.Viewport>
              <Select.ScrollDownButton />
            </Select.Content>
          </Select>
        </YStack>
      )}

      <Separator />

      <YStack gap="$2">
        <Label>Authentication</Label>
        <Select
          value={config.authType || 'api-key'}
          onValueChange={(v) => updateConfig('authType', v)}
        >
          <Select.Trigger>
            <Select.Value />
          </Select.Trigger>
          <Adapt when="sm" platform="touch">
            <Sheet modal dismissOnSnapToBottom>
              <Sheet.Frame>
                <Sheet.ScrollView>
                  <Adapt.Contents />
                </Sheet.ScrollView>
              </Sheet.Frame>
              <Sheet.Overlay />
            </Sheet>
          </Adapt>
          <Select.Content zIndex={200000}>
            <Select.ScrollUpButton />
            <Select.Viewport>
              <Select.Item value="none" index={0}>
                <Select.ItemText>None</Select.ItemText>
              </Select.Item>
              <Select.Item value="api-key" index={1}>
                <Select.ItemText>API Key</Select.ItemText>
              </Select.Item>
              <Select.Item value="jwt" index={2}>
                <Select.ItemText>JWT</Select.ItemText>
              </Select.Item>
            </Select.Viewport>
            <Select.ScrollDownButton />
          </Select.Content>
        </Select>
      </YStack>

      <YStack gap="$2">
        <Label>Rate Limit (per minute)</Label>
        <Input
          value={config.rateLimitPerMinute?.toString() || '60'}
          onChangeText={(v) => updateConfig('rateLimitPerMinute', parseInt(v) || 60)}
          keyboardType="numeric"
          placeholder="60"
        />
      </YStack>
    </YStack>
  );
}


// Condition Properties
function ConditionProperties({ properties, onChange }: any) {
  const config = properties.config || {};
  
  const updateConfig = (key: string, value: any) => {
    onChange('config', { ...config, [key]: value });
  };

  return (
    <YStack gap="$3">
      <Text fontSize="$4" fontWeight="600">Condition Configuration</Text>

      <YStack gap="$2">
        <Label>Label</Label>
        <Input
          value={properties.label || ''}
          onChangeText={(v) => onChange('label', v)}
          placeholder="Condition Name"
        />
      </YStack>

      <YStack gap="$2">
        <Label>Condition Expression</Label>
        <TextArea
          value={config.expression || ''}
          onChangeText={(v) => updateConfig('expression', v)}
          placeholder="result.success === true"
          rows={4}
        />
        <Text fontSize="$1" color="$gray11">
          JavaScript expression that evaluates to true/false
        </Text>
      </YStack>
    </YStack>
  );
}

// Phase Properties
function PhaseProperties({ properties, onChange }: any) {
  const config = properties.config || {};
  
  const updateConfig = (key: string, value: any) => {
    onChange('config', { ...config, [key]: value });
  };

  return (
    <YStack gap="$3">
      <Text fontSize="$4" fontWeight="600">Phase Configuration</Text>

      <YStack gap="$2">
        <Label>Phase Name</Label>
        <Input
          value={config.name || properties.label || ''}
          onChangeText={(v) => {
            updateConfig('name', v);
            onChange('label', v);
          }}
          placeholder="INITIALIZE"
        />
      </YStack>

      <YStack gap="$2">
        <Label>Description</Label>
        <TextArea
          value={config.description || ''}
          onChangeText={(v) => updateConfig('description', v)}
          placeholder="Phase description"
          rows={3}
        />
      </YStack>

      <YStack gap="$2">
        <Label>Execution Mode</Label>
        <Select
          value={config.sequential !== false ? 'sequential' : 'concurrent'}
          onValueChange={(v) => updateConfig('sequential', v === 'sequential')}
        >
          <Select.Trigger>
            <Select.Value />
          </Select.Trigger>
          <Adapt when="sm" platform="touch">
            <Sheet modal dismissOnSnapToBottom>
              <Sheet.Frame>
                <Sheet.ScrollView>
                  <Adapt.Contents />
                </Sheet.ScrollView>
              </Sheet.Frame>
              <Sheet.Overlay />
            </Sheet>
          </Adapt>
          <Select.Content zIndex={200000}>
            <Select.ScrollUpButton />
            <Select.Viewport>
              <Select.Item value="sequential" index={0}>
                <Select.ItemText>Sequential</Select.ItemText>
              </Select.Item>
              <Select.Item value="concurrent" index={1}>
                <Select.ItemText>Concurrent</Select.ItemText>
              </Select.Item>
            </Select.Viewport>
            <Select.ScrollDownButton />
          </Select.Content>
        </Select>
      </YStack>

      {config.sequential === false && (
        <YStack gap="$2">
          <Label>Max Concurrency</Label>
          <Input
            value={config.maxConcurrency?.toString() || '4'}
            onChangeText={(v) => updateConfig('maxConcurrency', parseInt(v) || 4)}
            keyboardType="numeric"
            placeholder="4"
          />
        </YStack>
      )}
    </YStack>
  );
}

// Retry Properties
function RetryProperties({ properties, onChange }: any) {
  const config = properties.config || {};
  
  const updateConfig = (key: string, value: any) => {
    onChange('config', { ...config, [key]: value });
  };

  const updateBackoff = (key: string, value: any) => {
    updateConfig('backoff', { ...(config.backoff || {}), [key]: value });
  };

  return (
    <YStack gap="$3">
      <Text fontSize="$4" fontWeight="600">Retry Configuration</Text>

      <YStack gap="$2">
        <Label>Label</Label>
        <Input
          value={properties.label || ''}
          onChangeText={(v) => onChange('label', v)}
          placeholder="Retry Loop"
        />
      </YStack>

      <YStack gap="$2">
        <Label>Max Attempts</Label>
        <Input
          value={config.maxAttempts?.toString() || '3'}
          onChangeText={(v) => updateConfig('maxAttempts', parseInt(v) || 3)}
          keyboardType="numeric"
          placeholder="3"
        />
      </YStack>

      <YStack gap="$2">
        <Label>Retry On</Label>
        <Select
          value={config.retryOn || 'failure'}
          onValueChange={(v) => updateConfig('retryOn', v)}
        >
          <Select.Trigger>
            <Select.Value />
          </Select.Trigger>
          <Adapt when="sm" platform="touch">
            <Sheet modal dismissOnSnapToBottom>
              <Sheet.Frame>
                <Sheet.ScrollView>
                  <Adapt.Contents />
                </Sheet.ScrollView>
              </Sheet.Frame>
              <Sheet.Overlay />
            </Sheet>
          </Adapt>
          <Select.Content zIndex={200000}>
            <Select.ScrollUpButton />
            <Select.Viewport>
              <Select.Item value="failure" index={0}>
                <Select.ItemText>Failure</Select.ItemText>
              </Select.Item>
              <Select.Item value="error" index={1}>
                <Select.ItemText>Error</Select.ItemText>
              </Select.Item>
              <Select.Item value="condition" index={2}>
                <Select.ItemText>Condition</Select.ItemText>
              </Select.Item>
            </Select.Viewport>
            <Select.ScrollDownButton />
          </Select.Content>
        </Select>
      </YStack>

      {config.retryOn === 'condition' && (
        <YStack gap="$2">
          <Label>Condition Expression</Label>
          <TextArea
            value={config.condition || ''}
            onChangeText={(v) => updateConfig('condition', v)}
            placeholder="result.status === 'retry'"
            rows={3}
          />
        </YStack>
      )}

      <YStack gap="$2">
        <Label>Backoff Type</Label>
        <Select
          value={config.backoff?.type || 'exponential'}
          onValueChange={(v) => updateBackoff('type', v)}
        >
          <Select.Trigger>
            <Select.Value />
          </Select.Trigger>
          <Adapt when="sm" platform="touch">
            <Sheet modal dismissOnSnapToBottom>
              <Sheet.Frame>
                <Sheet.ScrollView>
                  <Adapt.Contents />
                </Sheet.ScrollView>
              </Sheet.Frame>
              <Sheet.Overlay />
            </Sheet>
          </Adapt>
          <Select.Content zIndex={200000}>
            <Select.ScrollUpButton />
            <Select.Viewport>
              <Select.Item value="none" index={0}>
                <Select.ItemText>None</Select.ItemText>
              </Select.Item>
              <Select.Item value="linear" index={1}>
                <Select.ItemText>Linear</Select.ItemText>
              </Select.Item>
              <Select.Item value="exponential" index={2}>
                <Select.ItemText>Exponential</Select.ItemText>
              </Select.Item>
            </Select.Viewport>
            <Select.ScrollDownButton />
          </Select.Content>
        </Select>
      </YStack>

      {config.backoff?.type !== 'none' && (
        <>
          <YStack gap="$2">
            <Label>Initial Interval</Label>
            <Input
              value={config.backoff?.initialInterval || '1s'}
              onChangeText={(v) => updateBackoff('initialInterval', v)}
              placeholder="1s"
            />
            <Text fontSize="$1" color="$gray11">
              e.g., 1s, 1m, 5m
            </Text>
          </YStack>

          {config.backoff?.type === 'exponential' && (
            <YStack gap="$2">
              <Label>Multiplier</Label>
              <Input
                value={config.backoff?.multiplier?.toString() || '2'}
                onChangeText={(v) => updateBackoff('multiplier', parseFloat(v) || 2)}
                keyboardType="numeric"
                placeholder="2"
              />
            </YStack>
          )}
        </>
      )}

      <YStack gap="$2">
        <Label>Scope</Label>
        <Select
          value={config.scope || 'block'}
          onValueChange={(v) => updateConfig('scope', v)}
        >
          <Select.Trigger>
            <Select.Value />
          </Select.Trigger>
          <Adapt when="sm" platform="touch">
            <Sheet modal dismissOnSnapToBottom>
              <Sheet.Frame>
                <Sheet.ScrollView>
                  <Adapt.Contents />
                </Sheet.ScrollView>
              </Sheet.Frame>
              <Sheet.Overlay />
            </Sheet>
          </Adapt>
          <Select.Content zIndex={200000}>
            <Select.ScrollUpButton />
            <Select.Viewport>
              <Select.Item value="activity" index={0}>
                <Select.ItemText>Activity</Select.ItemText>
              </Select.Item>
              <Select.Item value="agent" index={1}>
                <Select.ItemText>Agent</Select.ItemText>
              </Select.Item>
              <Select.Item value="child-workflow" index={2}>
                <Select.ItemText>Child Workflow</Select.ItemText>
              </Select.Item>
              <Select.Item value="block" index={3}>
                <Select.ItemText>Block</Select.ItemText>
              </Select.Item>
            </Select.Viewport>
            <Select.ScrollDownButton />
          </Select.Content>
        </Select>
      </YStack>
    </YStack>
  );
}

// State Variable Properties
function StateVariableProperties({ properties, onChange }: any) {
  const config = properties.config || {};
  const projectId = properties.projectId; // Passed from NodePropertyPanel
  
  const updateConfig = (key: string, value: any) => {
    onChange('config', { ...config, [key]: value });
  };

  const { data: metricsData } = api.stateMonitoring.getMetrics.useQuery(
    { variableId: properties.id, scope: config.scope || 'workflow' },
    { enabled: !!properties.id && !!config.scope }
  );
  const { data: alertsData } = api.stateMonitoring.getAlerts.useQuery(
    { variableId: properties.id, scope: config.scope || 'workflow' },
    { enabled: !!properties.id && !!config.scope }
  );

  return (
    <YStack gap="$3">
      <Text fontSize="$4" fontWeight="600">State Variable Configuration</Text>

      <YStack gap="$2">
        <Label>Variable Name</Label>
        <Input
          value={config.name || properties.label || ''}
          onChangeText={(v) => {
            updateConfig('name', v);
            onChange('label', v);
          }}
          placeholder="variableName"
        />
      </YStack>

      <YStack gap="$2">
        <Label>Operation</Label>
        <Select
          value={config.operation || 'set'}
          onValueChange={(v) => updateConfig('operation', v)}
        >
          <Select.Trigger>
            <Select.Value />
          </Select.Trigger>
          <Adapt when="sm" platform="touch">
            <Sheet modal dismissOnSnapToBottom>
              <Sheet.Frame>
                <Sheet.ScrollView>
                  <Adapt.Contents />
                </Sheet.ScrollView>
              </Sheet.Frame>
              <Sheet.Overlay />
            </Sheet>
          </Adapt>
          <Select.Content zIndex={200000}>
            <Select.ScrollUpButton />
            <Select.Viewport>
              <Select.Item value="set" index={0}>
                <Select.ItemText>Set</Select.ItemText>
              </Select.Item>
              <Select.Item value="append" index={1}>
                <Select.ItemText>Append</Select.ItemText>
              </Select.Item>
              <Select.Item value="increment" index={2}>
                <Select.ItemText>Increment</Select.ItemText>
              </Select.Item>
              <Select.Item value="decrement" index={3}>
                <Select.ItemText>Decrement</Select.ItemText>
              </Select.Item>
              <Select.Item value="get" index={4}>
                <Select.ItemText>Get</Select.ItemText>
              </Select.Item>
            </Select.Viewport>
            <Select.ScrollDownButton />
          </Select.Content>
        </Select>
      </YStack>

      {(config.operation === 'set' || config.operation === 'append') && (
        <YStack gap="$2">
          <Label>Value</Label>
          <TextArea
            value={typeof config.value === 'string' ? config.value : JSON.stringify(config.value || '')}
            onChangeText={(v) => {
              try {
                const parsed = JSON.parse(v);
                updateConfig('value', parsed);
              } catch {
                updateConfig('value', v);
              }
            }}
            placeholder="Value or JSON"
            rows={3}
          />
        </YStack>
      )}

      <YStack gap="$2">
        <Label>Scope</Label>
        <Select
          value={config.scope || 'workflow'}
          onValueChange={(v) => updateConfig('scope', v)}
        >
          <Select.Trigger>
            <Select.Value />
          </Select.Trigger>
          <Adapt when="sm" platform="touch">
            <Sheet modal dismissOnSnapToBottom>
              <Sheet.Frame>
                <Sheet.ScrollView>
                  <Adapt.Contents />
                </Sheet.ScrollView>
              </Sheet.Frame>
              <Sheet.Overlay />
            </Sheet>
          </Adapt>
          <Select.Content zIndex={200000}>
            <Select.ScrollUpButton />
            <Select.Viewport>
              <Select.Item value="workflow" index={0}>
                <Select.ItemText>Workflow</Select.ItemText>
              </Select.Item>
              <Select.Item value="project" index={1}>
                <Select.ItemText>Project</Select.ItemText>
              </Select.Item>
            </Select.Viewport>
            <Select.ScrollDownButton />
          </Select.Content>
        </Select>
      </YStack>

      <YStack gap="$2">
        <Label>Storage Type</Label>
        <Select
          value={config.storageType || 'workflow'}
          onValueChange={(v) => updateConfig('storageType', v)}
        >
          <Select.Trigger>
            <Select.Value />
          </Select.Trigger>
          <Adapt when="sm" platform="touch">
            <Sheet modal dismissOnSnapToBottom>
              <Sheet.Frame>
                <Sheet.ScrollView>
                  <Adapt.Contents />
                </Sheet.ScrollView>
              </Sheet.Frame>
              <Sheet.Overlay />
            </Sheet>
          </Adapt>
          <Select.Content zIndex={200000}>
            <Select.ScrollUpButton />
            <Select.Viewport>
              <Select.Item value="workflow" index={0}>
                <Select.ItemText>Workflow (in-memory)</Select.ItemText>
              </Select.Item>
              <Select.Item value="database" index={1}>
                <Select.ItemText>Database (PostgreSQL)</Select.ItemText>
              </Select.Item>
              <Select.Item value="redis" index={2}>
                <Select.ItemText>Redis</Select.ItemText>
              </Select.Item>
            </Select.Viewport>
            <Select.ScrollDownButton />
          </Select.Content>
        </Select>
      </YStack>

      {(config.storageType === 'database' || config.storageType === 'redis') && projectId && (
        <ConnectorSelector
          projectId={projectId}
          classification={config.storageType === 'database' ? 'database' : 'redis'}
          value={config.connectorId}
          onChange={(v) => updateConfig('connectorId', v)}
        />
      )}

      {config.operation === 'set' && (
        <YStack gap="$2">
          <Label>Initial Value (optional)</Label>
          <TextArea
            value={typeof config.initialValue === 'string' ? config.initialValue : JSON.stringify(config.initialValue || '')}
            onChangeText={(v) => {
              try {
                const parsed = JSON.parse(v);
                updateConfig('initialValue', parsed);
              } catch {
                updateConfig('initialValue', v);
              }
            }}
            placeholder="Initial value or JSON"
            rows={2}
          />
        </YStack>
      )}

      {/* Monitoring and Best Practices Info */}
      <Card padding="$3" backgroundColor="$blue2" borderColor="$blue6">
        <YStack gap="$2">
          <Text fontSize="$3" fontWeight="600" color="$blue11">
            💡 Best Practices
          </Text>
          <YStack gap="$1">
            <Text fontSize="$2" color="$blue11">
              • <Text fontWeight="600">Small data (&lt;100 KB):</Text> Use workflow state (in-memory)
            </Text>
            <Text fontSize="$2" color="$blue11">
              • <Text fontWeight="600">Medium data (100 KB - 1 MB):</Text> Consider database storage
            </Text>
            <Text fontSize="$2" color="$blue11">
              • <Text fontWeight="600">Large data (&gt;1 MB):</Text> Use database or Redis storage
            </Text>
            <Text fontSize="$2" color="$blue11">
              • <Text fontWeight="600">High-frequency access:</Text> Redis provides best performance
            </Text>
            <Text fontSize="$2" color="$blue11">
              • <Text fontWeight="600">Project-level variables:</Text> Shared across all services in the project
            </Text>
          </YStack>
        </YStack>
      </Card>

      {/* Display Metrics and Alerts */}
      {metricsData && (
        <Card padding="$3" backgroundColor="$gray2" borderColor="$gray6">
          <YStack gap="$2">
            <Text fontSize="$3" fontWeight="600" color="$gray11">
              📊 Metrics
            </Text>
            <Text fontSize="$2" color="$gray11">
              Size: <Text fontWeight="600">{(metricsData.sizeBytes / 1024).toFixed(2)} KB</Text>
            </Text>
            <Text fontSize="$2" color="$gray11">
              Accesses: <Text fontWeight="600">{metricsData.accessCount}</Text>
            </Text>
            {metricsData.lastAccessed && (
              <Text fontSize="$2" color="$gray11">
                Last Accessed: <Text fontWeight="600">{metricsData.lastAccessed.toLocaleString()}</Text>
              </Text>
            )}
            {metricsData.recommendations.length > 0 && (
              <YStack gap="$1">
                <Text fontSize="$2" fontWeight="600" color="$gray11">Recommendations:</Text>
                {metricsData.recommendations.map((rec, i) => (
                  <Text key={i} fontSize="$2" color="$gray11">• {rec}</Text>
                ))}
              </YStack>
            )}
          </YStack>
        </Card>
      )}

      {alertsData && alertsData.length > 0 && (
        <Card padding="$3" backgroundColor="$red2" borderColor="$red6">
          <YStack gap="$2">
            <Text fontSize="$3" fontWeight="600" color="$red11">
              🚨 Alerts
            </Text>
            {alertsData.map((alert, i) => (
              <YStack key={i} gap="$1">
                <Text fontSize="$2" color="$red11">
                  • <Text fontWeight="600">{alert.severity.toUpperCase()}:</Text> {alert.message}
                </Text>
                {alert.recommendation && (
                  <Text fontSize="$2" color="$red11" marginLeft="$3">
                    Recommendation: {alert.recommendation}
                  </Text>
                )}
              </YStack>
            ))}
          </YStack>
        </Card>
      )}
    </YStack>
  );
}

// PostgreSQL Properties
function PostgreSQLProperties({ properties, onChange }: any) {
  const projectId = properties.projectId;

  return (
    <YStack gap="$3">
      <Text fontSize="$4" fontWeight="600">PostgreSQL Query</Text>

      {projectId && (
        <ConnectorSelector
          projectId={projectId}
          connectorType="database"
          value={properties.connectorId || properties.connectionId}
          onChange={(connectorId) => {
            // Support both connectorId (new) and connectionId (legacy) for backward compatibility
            onChange('connectorId', connectorId);
            if (connectorId) {
              onChange('connectionId', connectorId); // Keep legacy field for now
            }
          }}
        />
      )}

      <YStack gap="$2">
        <Label>Query</Label>
        <TextArea
          value={properties.query || ''}
          onChangeText={(v) => onChange('query', v)}
          placeholder="SELECT * FROM users WHERE id = $1"
          fontFamily="$mono"
          numberOfLines={6}
        />
      </YStack>

      <YStack gap="$2">
        <Label>Parameters (JSON array)</Label>
        <TextArea
          value={properties.parameters ? JSON.stringify(properties.parameters, null, 2) : ''}
          onChangeText={(v) => {
            try {
              const parsed = JSON.parse(v);
              onChange('parameters', parsed);
            } catch {
              // Invalid JSON, ignore
            }
          }}
          placeholder='["value1", "value2"]'
          fontFamily="$mono"
          numberOfLines={3}
        />
      </YStack>

      <Separator />
      
      <YStack gap="$2">
        <Label>Retry Policy</Label>
        <Select
          value={properties.retryPolicy?.strategy || 'exponential-backoff'}
          onValueChange={(v) => onChange('retryPolicy', {
            ...properties.retryPolicy,
            strategy: v,
          })}
        >
          <Select.Trigger iconAfter={ChevronDown}>
            <Select.Value placeholder="Select retry strategy" />
          </Select.Trigger>
          <Select.Content>
            <Select.Viewport>
              <Select.Item index={0} value="keep-trying">
                <Select.ItemText>Keep trying until success</Select.ItemText>
              </Select.Item>
              <Select.Item index={1} value="fail-after-x">
                <Select.ItemText>Fail after X attempts</Select.ItemText>
              </Select.Item>
              <Select.Item index={2} value="exponential-backoff">
                <Select.ItemText>Exponential backoff</Select.ItemText>
              </Select.Item>
              <Select.Item index={3} value="none">
                <Select.ItemText>No retries</Select.ItemText>
              </Select.Item>
            </Select.Viewport>
          </Select.Content>
        </Select>
      </YStack>

      {properties.retryPolicy?.strategy === 'fail-after-x' && (
        <YStack gap="$2">
          <Label>Max Attempts</Label>
          <Input
            value={properties.retryPolicy?.maxAttempts?.toString() || ''}
            onChangeText={(v) => onChange('retryPolicy', {
              ...properties.retryPolicy,
              maxAttempts: v ? parseInt(v, 10) : undefined,
            })}
            placeholder="e.g., 3"
            keyboardType="numeric"
          />
        </YStack>
      )}

      {properties.retryPolicy?.strategy === 'exponential-backoff' && (
        <YStack gap="$3">
          <YStack gap="$2">
            <Label>Max Attempts</Label>
            <Input
              value={properties.retryPolicy?.maxAttempts?.toString() || ''}
              onChangeText={(v) => onChange('retryPolicy', {
                ...properties.retryPolicy,
                maxAttempts: v ? parseInt(v, 10) : undefined,
              })}
              placeholder="e.g., 5"
              keyboardType="numeric"
            />
          </YStack>
          <YStack gap="$2">
            <Label>Initial Interval</Label>
            <Input
              value={properties.retryPolicy?.initialInterval || '1s'}
              onChangeText={(v) => onChange('retryPolicy', {
                ...properties.retryPolicy,
                initialInterval: v,
              })}
              placeholder="e.g., 1s, 5m"
            />
          </YStack>
        </YStack>
      )}
    </YStack>
  );
}

// Redis Properties
function RedisProperties({ properties, onChange }: any) {
  const projectId = properties.projectId;

  return (
    <YStack gap="$3">
      <Text fontSize="$4" fontWeight="600">Redis Command</Text>

      {projectId && (
        <ConnectorSelector
          projectId={projectId}
          connectorType="database"
          value={properties.connectorId || properties.connectionId}
          onChange={(connectorId) => {
            // Support both connectorId (new) and connectionId (legacy) for backward compatibility
            onChange('connectorId', connectorId);
            if (connectorId) {
              onChange('connectionId', connectorId); // Keep legacy field for now
            }
          }}
        />
      )}

      <YStack gap="$2">
        <Label>Command</Label>
        <Select
          value={properties.command || 'GET'}
          onValueChange={(v) => onChange('command', v)}
        >
          <Select.Trigger>
            <Select.Value />
          </Select.Trigger>
          <Select.Content>
            <Select.Viewport>
              <Select.Item value="GET">GET</Select.Item>
              <Select.Item value="SET">SET</Select.Item>
              <Select.Item value="DEL">DEL</Select.Item>
              <Select.Item value="EXISTS">EXISTS</Select.Item>
              <Select.Item value="INCR">INCR</Select.Item>
              <Select.Item value="DECR">DECR</Select.Item>
            </Select.Viewport>
          </Select.Content>
        </Select>
      </YStack>

      <YStack gap="$2">
        <Label>Key</Label>
        <Input
          value={properties.key || ''}
          onChangeText={(v) => onChange('key', v)}
          placeholder="e.g., user:123"
        />
      </YStack>

      {(properties.command === 'SET') && (
        <YStack gap="$2">
          <Label>Value</Label>
          <TextArea
            value={properties.value || ''}
            onChangeText={(v) => onChange('value', v)}
            placeholder="Value to set"
            numberOfLines={3}
          />
        </YStack>
      )}

      <Separator />
      
      <YStack gap="$2">
        <Label>Retry Policy</Label>
        <Select
          value={properties.retryPolicy?.strategy || 'exponential-backoff'}
          onValueChange={(v) => onChange('retryPolicy', {
            ...properties.retryPolicy,
            strategy: v,
          })}
        >
          <Select.Trigger iconAfter={ChevronDown}>
            <Select.Value placeholder="Select retry strategy" />
          </Select.Trigger>
          <Select.Content>
            <Select.Viewport>
              <Select.Item index={0} value="keep-trying">
                <Select.ItemText>Keep trying until success</Select.ItemText>
              </Select.Item>
              <Select.Item index={1} value="fail-after-x">
                <Select.ItemText>Fail after X attempts</Select.ItemText>
              </Select.Item>
              <Select.Item index={2} value="exponential-backoff">
                <Select.ItemText>Exponential backoff</Select.ItemText>
              </Select.Item>
              <Select.Item index={3} value="none">
                <Select.ItemText>No retries</Select.ItemText>
              </Select.Item>
            </Select.Viewport>
          </Select.Content>
        </Select>
      </YStack>

      {properties.retryPolicy?.strategy === 'fail-after-x' && (
        <YStack gap="$2">
          <Label>Max Attempts</Label>
          <Input
            value={properties.retryPolicy?.maxAttempts?.toString() || ''}
            onChangeText={(v) => onChange('retryPolicy', {
              ...properties.retryPolicy,
              maxAttempts: v ? parseInt(v, 10) : undefined,
            })}
            placeholder="e.g., 3"
            keyboardType="numeric"
          />
        </YStack>
      )}

      {properties.retryPolicy?.strategy === 'exponential-backoff' && (
        <YStack gap="$3">
          <YStack gap="$2">
            <Label>Max Attempts</Label>
            <Input
              value={properties.retryPolicy?.maxAttempts?.toString() || ''}
              onChangeText={(v) => onChange('retryPolicy', {
                ...properties.retryPolicy,
                maxAttempts: v ? parseInt(v, 10) : undefined,
              })}
              placeholder="e.g., 5"
              keyboardType="numeric"
            />
          </YStack>
          <YStack gap="$2">
            <Label>Initial Interval</Label>
            <Input
              value={properties.retryPolicy?.initialInterval || '1s'}
              onChangeText={(v) => onChange('retryPolicy', {
                ...properties.retryPolicy,
                initialInterval: v,
              })}
              placeholder="e.g., 1s, 5m"
            />
          </YStack>
        </YStack>
      )}
    </YStack>
  );
}

// TypeScript Properties
function TypeScriptProperties({ properties, onChange }: any) {
  return (
    <YStack gap="$3">
      <Text fontSize="$4" fontWeight="600">TypeScript Processor</Text>

      <YStack gap="$2">
        <Label>Code</Label>
        <TextArea
          value={properties.code || ''}
          onChangeText={(v) => onChange('code', v)}
          placeholder="// Process input data
export async function process(input: any): Promise<any> {
  // Your code here
  return input;
}"
          fontFamily="$mono"
          numberOfLines={12}
        />
      </YStack>

      <YStack gap="$2">
        <Label>Input Schema (JSON)</Label>
        <TextArea
          value={properties.inputSchema ? JSON.stringify(properties.inputSchema, null, 2) : ''}
          onChangeText={(v) => {
            try {
              const parsed = JSON.parse(v);
              onChange('inputSchema', parsed);
            } catch {
              // Invalid JSON, ignore
            }
          }}
          placeholder='{"type": "object", "properties": {...}}'
          fontFamily="$mono"
          numberOfLines={4}
        />
      </YStack>

      <YStack gap="$2">
        <Label>Output Schema (JSON)</Label>
        <TextArea
          value={properties.outputSchema ? JSON.stringify(properties.outputSchema, null, 2) : ''}
          onChangeText={(v) => {
            try {
              const parsed = JSON.parse(v);
              onChange('outputSchema', parsed);
            } catch {
              // Invalid JSON, ignore
            }
          }}
          placeholder='{"type": "object", "properties": {...}}'
          fontFamily="$mono"
          numberOfLines={4}
        />
      </YStack>
    </YStack>
  );
}


// Kong Cache Properties
function KongCacheProperties({ properties, onChange, node }: any) {
  const config = properties.config || {};
  const projectId = properties.projectId || node?.data?.projectId;
  const componentId = node?.id;
  const connectorId = config.connectorId;
  const cacheKey = config.cacheKey || '';
  const isSaved = config.isSaved || false;
  const ttlSeconds = config.ttlSeconds || 3600;
  const cacheKeyStrategy = config.cacheKeyStrategy || 'path';
  const contentTypes = config.contentTypes || ['application/json'];
  const responseCodes = config.responseCodes || [200, 201, 202];

  const updateConfig = (key: string, value: any) => {
    onChange('config', { ...config, [key]: value });
  };

  // Fetch existing cache key configuration
  const { data: cacheKeyConfig } = api.kongCache.get.useQuery(
    { componentId: componentId || '', projectId: projectId || '' },
    { enabled: !!componentId && !!projectId && isSaved }
  );

  // Generate new cache key mutation
  const generateKeyMutation = api.kongCache.generateKey.useMutation();

  // Upsert cache key mutation
  const upsertCacheKeyMutation = api.kongCache.upsert.useMutation();

  // Get connector name if connectorId is set
  const { data: connectorData } = api.connectors.get.useQuery(
    { id: connectorId || '' },
    { enabled: !!connectorId }
  );

  // Auto-generate cache key on mount if not set
  useEffect(() => {
    if (!cacheKey && projectId && !isSaved) {
      generateKeyMutation.mutate(
        { projectId },
        {
          onSuccess: (data) => {
            updateConfig('cacheKey', data.cacheKey);
          },
        }
      );
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Sync with database if component is saved
  useEffect(() => {
    if (cacheKeyConfig && isSaved) {
      updateConfig('cacheKey', cacheKeyConfig.cache_key);
      updateConfig('ttlSeconds', cacheKeyConfig.ttl_seconds);
      updateConfig('cacheKeyStrategy', cacheKeyConfig.cache_key_strategy);
      updateConfig('contentTypes', cacheKeyConfig.content_types);
      updateConfig('responseCodes', cacheKeyConfig.response_codes);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [cacheKeyConfig, isSaved]);

  const handleSave = () => {
    if (!connectorId || !cacheKey || !projectId || !componentId) {
      return;
    }

    // Validate cache key is UUID format
    const uuidRegex = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;
    if (!uuidRegex.test(cacheKey)) {
      alert('Cache key must be a valid UUID format');
      return;
    }

    // Save to database
    upsertCacheKeyMutation.mutate(
      {
        componentId,
        projectId,
        connectorId,
        cacheKey,
        ttlSeconds,
        cacheKeyStrategy,
        contentTypes,
        responseCodes,
        isSaved: true,
      },
      {
        onSuccess: () => {
          updateConfig('isSaved', true);
        },
      }
    );
  };

  return (
    <YStack gap="$3">
      <YStack gap="$1">
        <Text fontSize="$4" fontWeight="600">Kong Cache Configuration</Text>
        <Text fontSize="$2" color="$gray11">
          Redis-backed proxy caching for API endpoints
        </Text>
      </YStack>

      <YStack gap="$2">
        <Label>Redis Connector *</Label>
        {projectId ? (
          <>
            <ConnectorSelector
              projectId={projectId}
              classification="redis"
              value={connectorId}
              onChange={(id) => {
                updateConfig('connectorId', id);
                if (id && connectorData) {
                  onChange('connectorName', connectorData.display_name || connectorData.name);
                }
              }}
            />
            {!connectorId && (
              <Text fontSize="$1" color="$red10">
                A Redis connector is required
              </Text>
            )}
            <Text fontSize="$1" color="$gray11">
              Select a Redis connector (Upstash, Redis Cloud, etc.)
            </Text>
          </>
        ) : (
          <Text fontSize="$2" color="$red10">
            Project ID required to select connector
          </Text>
        )}
      </YStack>

      <Separator />

      <YStack gap="$2">
        <Label>Cache Key *</Label>
        {isSaved ? (
          <>
            <Input
              value={cacheKey}
              disabled
              fontFamily="monospace"
              fontSize="$2"
            />
            <Text fontSize="$1" color="$gray11">
              Cache key is immutable after saving. Remove and recreate the component to change it.
            </Text>
          </>
        ) : (
          <>
            <Input
              value={cacheKey}
              onChangeText={(text) => {
                updateConfig('cacheKey', text);
              }}
              placeholder="Auto-generated UUID"
              fontFamily="monospace"
              fontSize="$2"
            />
            <XStack gap="$2">
              <Button
                size="$2"
                onPress={() => {
                  if (projectId) {
                    generateKeyMutation.mutate(
                      { projectId },
                      {
                        onSuccess: (data) => {
                          updateConfig('cacheKey', data.cacheKey);
                        },
                      }
                    );
                  }
                }}
              >
                Generate New
              </Button>
            </XStack>
            <Text fontSize="$1" color="$gray11">
              Cache key will be auto-generated. You can edit it until you save the component.
            </Text>
          </>
        )}
      </YStack>

      <Separator />

      <YStack gap="$2">
        <Label>TTL (Time To Live) - Seconds</Label>
        <Input
          value={ttlSeconds.toString()}
          onChangeText={(text) => {
            const num = parseInt(text, 10);
            if (!isNaN(num) && num > 0) {
              updateConfig('ttlSeconds', num);
            }
          }}
          keyboardType="numeric"
        />
        <Text fontSize="$1" color="$gray11">
          How long cached responses should be stored (default: 3600 seconds = 1 hour)
        </Text>
      </YStack>

      <YStack gap="$2">
        <Label>Cache Key Strategy</Label>
        <Select
          value={cacheKeyStrategy}
          onValueChange={(value) => {
            updateConfig('cacheKeyStrategy', value);
          }}
        >
          <Select.Trigger>
            <Select.Value placeholder="Select strategy" />
          </Select.Trigger>
          <Select.Content>
            <Select.Viewport>
              <Select.Item value="path" index={0}>
                <Select.ItemText>Path</Select.ItemText>
              </Select.Item>
              <Select.Item value="query" index={1}>
                <Select.ItemText>Query String</Select.ItemText>
              </Select.Item>
              <Select.Item value="header" index={2}>
                <Select.ItemText>Header</Select.ItemText>
              </Select.Item>
              <Select.Item value="custom" index={3}>
                <Select.ItemText>Custom</Select.ItemText>
              </Select.Item>
            </Select.Viewport>
          </Select.Content>
        </Select>
        <Text fontSize="$1" color="$gray11">
          How to generate the cache key from the request
        </Text>
      </YStack>

      <YStack gap="$2">
        <Label>Content Types to Cache</Label>
        <TextArea
          value={contentTypes.join(', ')}
          onChangeText={(text) => {
            const types = text.split(',').map(t => t.trim()).filter(t => t);
            updateConfig('contentTypes', types);
          }}
          placeholder="application/json, text/html"
          minHeight={60}
        />
        <Text fontSize="$1" color="$gray11">
          Comma-separated list of content types to cache (default: application/json)
        </Text>
      </YStack>

      <YStack gap="$2">
        <Label>Response Codes to Cache</Label>
        <TextArea
          value={responseCodes.join(', ')}
          onChangeText={(text) => {
            const codes = text.split(',').map(c => parseInt(c.trim(), 10)).filter(c => !isNaN(c));
            updateConfig('responseCodes', codes);
          }}
          placeholder="200, 201, 202"
          minHeight={60}
        />
        <Text fontSize="$1" color="$gray11">
          Comma-separated list of HTTP response codes to cache (default: 200, 201, 202)
        </Text>
      </YStack>

      {!isSaved && (
        <YStack gap="$2">
          <Button
            onPress={handleSave}
            disabled={!connectorId || !cacheKey}
            backgroundColor="$blue9"
            color="white"
          >
            Save Cache Configuration
          </Button>
          <Text fontSize="$1" color="$gray11">
            Once saved, the cache key becomes immutable
          </Text>
        </YStack>
      )}

      {isSaved && (
        <Card padding="$2" backgroundColor="$green2" borderColor="$green6">
          <Text fontSize="$2" color="$green11" fontWeight="600">
            ✓ Cache configuration saved. Key is now immutable.
          </Text>
        </Card>
      )}
    </YStack>
  );
}

// Kong CORS Properties
function KongCorsProperties({ properties, onChange, node }: any) {
  const config = properties.config || {};
  const allowedOrigins = config.allowedOrigins || [];
  const allowedMethods = config.allowedMethods || ['GET', 'POST', 'PUT', 'DELETE', 'OPTIONS'];
  const allowedHeaders = config.allowedHeaders || ['Content-Type', 'Authorization'];
  const exposedHeaders = config.exposedHeaders || [];
  const credentials = config.credentials || false;
  const maxAge = config.maxAge || 3600;
  const preflightContinue = config.preflightContinue || false;

  const updateConfig = (key: string, value: any) => {
    onChange('config', { ...config, [key]: value });
  };

  const handleOriginsChange = (text: string) => {
    const origins = text.split(',').map(o => o.trim()).filter(o => o);
    updateConfig('allowedOrigins', origins);
  };

  const handleMethodsChange = (text: string) => {
    const methods = text.split(',').map(m => m.trim().toUpperCase()).filter(m => m);
    updateConfig('allowedMethods', methods);
  };

  const handleHeadersChange = (text: string) => {
    const headers = text.split(',').map(h => h.trim()).filter(h => h);
    updateConfig('allowedHeaders', headers);
  };

  const handleExposedHeadersChange = (text: string) => {
    const headers = text.split(',').map(h => h.trim()).filter(h => h);
    updateConfig('exposedHeaders', headers);
  };

  return (
    <YStack gap="$3">
      <YStack gap="$1">
        <Text fontSize="$4" fontWeight="600">Kong CORS Configuration</Text>
        <Text fontSize="$2" color="$gray11">
          Configure Cross-Origin Resource Sharing (CORS) headers for API endpoints
        </Text>
      </YStack>

      <YStack gap="$2">
        <Label>Allowed Origins *</Label>
        <TextArea
          value={allowedOrigins.join(', ')}
          onChangeText={handleOriginsChange}
          placeholder="https://example.com, https://app.example.com"
          minHeight={60}
        />
        <Text fontSize="$1" color="$gray11">
          Comma-separated list of allowed origins. Use * for all origins (not recommended for production).
        </Text>
      </YStack>

      <YStack gap="$2">
        <Label>Allowed Methods *</Label>
        <TextArea
          value={allowedMethods.join(', ')}
          onChangeText={handleMethodsChange}
          placeholder="GET, POST, PUT, DELETE, OPTIONS"
          minHeight={60}
        />
        <Text fontSize="$1" color="$gray11">
          Comma-separated list of allowed HTTP methods (default: GET, POST, PUT, DELETE, OPTIONS).
        </Text>
      </YStack>

      <YStack gap="$2">
        <Label>Allowed Headers</Label>
        <TextArea
          value={allowedHeaders.join(', ')}
          onChangeText={handleHeadersChange}
          placeholder="Content-Type, Authorization, X-Requested-With"
          minHeight={60}
        />
        <Text fontSize="$1" color="$gray11">
          Comma-separated list of allowed request headers (default: Content-Type, Authorization).
        </Text>
      </YStack>

      <YStack gap="$2">
        <Label>Exposed Headers</Label>
        <TextArea
          value={exposedHeaders.join(', ')}
          onChangeText={handleExposedHeadersChange}
          placeholder="X-Total-Count, X-Page-Number"
          minHeight={60}
        />
        <Text fontSize="$1" color="$gray11">
          Comma-separated list of headers that browsers are allowed to access.
        </Text>
      </YStack>

      <Separator />

      <YStack gap="$2">
        <XStack gap="$3" alignItems="center" justifyContent="space-between">
          <YStack flex={1}>
            <Text fontSize="$3" fontWeight="600">Allow Credentials</Text>
            <Text fontSize="$2" color="$gray11">
              Allow cookies and authentication headers to be sent with cross-origin requests
            </Text>
          </YStack>
          <Switch
            checked={credentials}
            onCheckedChange={(checked) => {
              updateConfig('credentials', checked);
            }}
            size="$3"
          />
        </XStack>
      </YStack>

      <YStack gap="$2">
        <Label>Max-Age (seconds)</Label>
        <Input
          value={maxAge.toString()}
          onChangeText={(text) => {
            const num = parseInt(text, 10);
            if (!isNaN(num) && num >= 0) {
              updateConfig('maxAge', num);
            }
          }}
          keyboardType="numeric"
        />
        <Text fontSize="$1" color="$gray11">
          How long the browser should cache preflight OPTIONS requests (default: 3600 seconds = 1 hour).
        </Text>
      </YStack>

      <YStack gap="$2">
        <XStack gap="$3" alignItems="center" justifyContent="space-between">
          <YStack flex={1}>
            <Text fontSize="$3" fontWeight="600">Preflight Continue</Text>
            <Text fontSize="$2" color="$gray11">
              Continue processing the request even if preflight fails (advanced)
            </Text>
          </YStack>
          <Switch
            checked={preflightContinue}
            onCheckedChange={(checked) => {
              updateConfig('preflightContinue', checked);
            }}
            size="$3"
          />
        </XStack>
      </YStack>

      <Card padding="$2" backgroundColor="$blue2" borderColor="$blue6">
        <Text fontSize="$2" color="$blue11">
          <Text fontWeight="600">Note:</Text> This CORS configuration will be applied to all data-in/data-out endpoints in the workflow when connected.
        </Text>
      </Card>
    </YStack>
  );
}

// GraphQL Gateway Properties
function GraphQLGatewayProperties({ properties, onChange, node }: any) {
  const config = properties.config || {};
  const endpointPath = config.endpointPath || '/graphql';
  const schema = config.schema || '';
  const queries = config.queries || [];
  const mutations = config.mutations || [];

  const updateConfig = (key: string, value: any) => {
    onChange('config', { ...config, [key]: value });
  };

  const handleSchemaChange = (text: string) => {
    updateConfig('schema', text);
    // TODO: Parse schema and extract queries/mutations
    // For now, we'll let users manually configure them
  };

  const addQuery = () => {
    const newQuery = {
      name: '',
      workflowId: '',
      inputSchema: {},
      outputSchema: {},
    };
    updateConfig('queries', [...queries, newQuery]);
  };

  const updateQuery = (index: number, field: string, value: any) => {
    const updatedQueries = [...queries];
    updatedQueries[index] = { ...updatedQueries[index], [field]: value };
    updateConfig('queries', updatedQueries);
  };

  const removeQuery = (index: number) => {
    updateConfig('queries', queries.filter((_: any, i: number) => i !== index));
  };

  const addMutation = () => {
    const newMutation = {
      name: '',
      workflowId: '',
      inputSchema: {},
      outputSchema: {},
    };
    updateConfig('mutations', [...mutations, newMutation]);
  };

  const updateMutation = (index: number, field: string, value: any) => {
    const updatedMutations = [...mutations];
    updatedMutations[index] = { ...updatedMutations[index], [field]: value };
    updateConfig('mutations', updatedMutations);
  };

  const removeMutation = (index: number) => {
    updateConfig('mutations', mutations.filter((_: any, i: number) => i !== index));
  };

  return (
    <YStack gap="$3">
      <YStack gap="$1">
        <Text fontSize="$4" fontWeight="600">GraphQL Gateway Configuration</Text>
        <Text fontSize="$2" color="$gray11">
          Configure GraphQL endpoint with schema definition and resolver mapping
        </Text>
      </YStack>

      <YStack gap="$2">
        <Label>Endpoint Path *</Label>
        <Input
          value={endpointPath}
          onChangeText={(text) => {
            updateConfig('endpointPath', text);
          }}
          placeholder="/graphql"
          fontFamily="monospace"
        />
        <Text fontSize="$1" color="$gray11">
          The path where the GraphQL endpoint will be available (default: /graphql)
        </Text>
      </YStack>

      <Separator />

      <YStack gap="$2">
        <Label>GraphQL Schema (SDL) *</Label>
        <TextArea
          value={schema}
          onChangeText={handleSchemaChange}
          placeholder={`type Query {
  hello: String
  user(id: ID!): User
}

type Mutation {
  createUser(input: UserInput!): User
}

type User {
  id: ID!
  name: String!
  email: String!
}

input UserInput {
  name: String!
  email: String!
}`}
          minHeight={200}
          fontFamily="monospace"
          fontSize="$2"
        />
        <Text fontSize="$1" color="$gray11">
          Define your GraphQL schema using Schema Definition Language (SDL). Include type definitions, queries, and mutations.
        </Text>
      </YStack>

      <Separator />

      <YStack gap="$2">
        <XStack gap="$2" alignItems="center" justifyContent="space-between">
          <Text fontSize="$3" fontWeight="600">Queries</Text>
          <Button size="$2" onPress={addQuery}>
            Add Query
          </Button>
        </XStack>
        <Text fontSize="$2" color="$gray11">
          Map GraphQL queries to workflows
        </Text>

        {queries.length === 0 ? (
          <Card padding="$3" backgroundColor="$gray2">
            <Text fontSize="$2" color="$gray11" textAlign="center">
              No queries defined. Add a query to map GraphQL queries to workflows.
            </Text>
          </Card>
        ) : (
          <YStack gap="$2">
            {queries.map((query: any, index: number) => (
              <Card key={index} padding="$3" backgroundColor="$gray1" borderColor="$gray4">
                <YStack gap="$2">
                  <XStack gap="$2" alignItems="center" justifyContent="space-between">
                    <Text fontSize="$3" fontWeight="600">Query {index + 1}</Text>
                    <Button size="$2" onPress={() => removeQuery(index)}>
                      Remove
                    </Button>
                  </XStack>
                  <YStack gap="$2">
                    <Label>Query Name *</Label>
                    <Input
                      value={query.name || ''}
                      onChangeText={(text) => updateQuery(index, 'name', text)}
                      placeholder="user"
                      fontFamily="monospace"
                    />
                    <Label>Workflow ID *</Label>
                    <Input
                      value={query.workflowId || ''}
                      onChangeText={(text) => updateQuery(index, 'workflowId', text)}
                      placeholder="workflow-uuid"
                      fontFamily="monospace"
                    />
                  </YStack>
                </YStack>
              </Card>
            ))}
          </YStack>
        )}
      </YStack>

      <Separator />

      <YStack gap="$2">
        <XStack gap="$2" alignItems="center" justifyContent="space-between">
          <Text fontSize="$3" fontWeight="600">Mutations</Text>
          <Button size="$2" onPress={addMutation}>
            Add Mutation
          </Button>
        </XStack>
        <Text fontSize="$2" color="$gray11">
          Map GraphQL mutations to workflows
        </Text>

        {mutations.length === 0 ? (
          <Card padding="$3" backgroundColor="$gray2">
            <Text fontSize="$2" color="$gray11" textAlign="center">
              No mutations defined. Add a mutation to map GraphQL mutations to workflows.
            </Text>
          </Card>
        ) : (
          <YStack gap="$2">
            {mutations.map((mutation: any, index: number) => (
              <Card key={index} padding="$3" backgroundColor="$gray1" borderColor="$gray4">
                <YStack gap="$2">
                  <XStack gap="$2" alignItems="center" justifyContent="space-between">
                    <Text fontSize="$3" fontWeight="600">Mutation {index + 1}</Text>
                    <Button size="$2" onPress={() => removeMutation(index)}>
                      Remove
                    </Button>
                  </XStack>
                  <YStack gap="$2">
                    <Label>Mutation Name *</Label>
                    <Input
                      value={mutation.name || ''}
                      onChangeText={(text) => updateMutation(index, 'name', text)}
                      placeholder="createUser"
                      fontFamily="monospace"
                    />
                    <Label>Workflow ID *</Label>
                    <Input
                      value={mutation.workflowId || ''}
                      onChangeText={(text) => updateMutation(index, 'workflowId', text)}
                      placeholder="workflow-uuid"
                      fontFamily="monospace"
                    />
                  </YStack>
                </YStack>
              </Card>
            ))}
          </YStack>
        )}
      </YStack>

      <Card padding="$2" backgroundColor="$blue2" borderColor="$blue6">
        <Text fontSize="$2" color="$blue11">
          <Text fontWeight="600">Note:</Text> The GraphQL schema will be validated and used to generate the GraphQL endpoint. Queries and mutations must be mapped to workflows.
        </Text>
      </Card>
    </YStack>
  );
}

// MCP Server Properties
function MCPServerProperties({ properties, onChange, node }: any) {
  const config = properties.config || {};
  const serverName = config.serverName || '';
  const serverVersion = config.serverVersion || '1.0.0';
  const endpointPath = config.endpointPath || '/mcp';
  const resources = config.resources || [];
  const tools = config.tools || [];

  const updateConfig = (key: string, value: any) => {
    onChange('config', { ...config, [key]: value });
  };

  const addResource = () => {
    const newResource = {
      uri: '',
      name: '',
      description: '',
      mimeType: 'application/json',
      workflowId: '',
    };
    updateConfig('resources', [...resources, newResource]);
  };

  const updateResource = (index: number, field: string, value: any) => {
    const updatedResources = [...resources];
    updatedResources[index] = { ...updatedResources[index], [field]: value };
    updateConfig('resources', updatedResources);
  };

  const removeResource = (index: number) => {
    updateConfig('resources', resources.filter((_: any, i: number) => i !== index));
  };

  const addTool = () => {
    const newTool = {
      name: '',
      description: '',
      inputSchema: {},
      workflowId: '',
    };
    updateConfig('tools', [...tools, newTool]);
  };

  const updateTool = (index: number, field: string, value: any) => {
    const updatedTools = [...tools];
    updatedTools[index] = { ...updatedTools[index], [field]: value };
    updateConfig('tools', updatedTools);
  };

  const removeTool = (index: number) => {
    updateConfig('tools', tools.filter((_: any, i: number) => i !== index));
  };

  return (
    <YStack gap="$3">
      <YStack gap="$1">
        <Text fontSize="$4" fontWeight="600">MCP Server Configuration</Text>
        <Text fontSize="$2" color="$gray11">
          Configure MCP (Model Context Protocol) server to expose workflows as resources and tools
        </Text>
      </YStack>

      <YStack gap="$2">
        <Label>Server Name *</Label>
        <Input
          value={serverName}
          onChangeText={(text) => {
            updateConfig('serverName', text);
          }}
          placeholder="my-workflow-server"
          fontFamily="monospace"
        />
        <Text fontSize="$1" color="$gray11">
          Unique identifier for this MCP server instance
        </Text>
      </YStack>

      <YStack gap="$2">
        <Label>Server Version</Label>
        <Input
          value={serverVersion}
          onChangeText={(text) => {
            updateConfig('serverVersion', text);
          }}
          placeholder="1.0.0"
          fontFamily="monospace"
        />
        <Text fontSize="$1" color="$gray11">
          Version of the MCP server (default: 1.0.0)
        </Text>
      </YStack>

      <YStack gap="$2">
        <Label>Endpoint Path *</Label>
        <Input
          value={endpointPath}
          onChangeText={(text) => {
            updateConfig('endpointPath', text);
          }}
          placeholder="/mcp"
          fontFamily="monospace"
        />
        <Text fontSize="$1" color="$gray11">
          The path where the MCP server endpoint will be available (default: /mcp)
        </Text>
      </YStack>

      <Separator />

      <YStack gap="$2">
        <XStack gap="$2" alignItems="center" justifyContent="space-between">
          <Text fontSize="$3" fontWeight="600">Resources</Text>
          <Button size="$2" onPress={addResource}>
            Add Resource
          </Button>
        </XStack>
        <Text fontSize="$2" color="$gray11">
          MCP resources expose data that can be read by clients
        </Text>

        {resources.length === 0 ? (
          <Card padding="$3" backgroundColor="$gray2">
            <Text fontSize="$2" color="$gray11" textAlign="center">
              No resources defined. Add a resource to expose workflow data.
            </Text>
          </Card>
        ) : (
          <YStack gap="$2">
            {resources.map((resource: any, index: number) => (
              <Card key={index} padding="$3" backgroundColor="$gray1" borderColor="$gray4">
                <YStack gap="$2">
                  <XStack gap="$2" alignItems="center" justifyContent="space-between">
                    <Text fontSize="$3" fontWeight="600">Resource {index + 1}</Text>
                    <Button size="$2" onPress={() => removeResource(index)}>
                      Remove
                    </Button>
                  </XStack>
                  <YStack gap="$2">
                    <Label>URI *</Label>
                    <Input
                      value={resource.uri || ''}
                      onChangeText={(text) => updateResource(index, 'uri', text)}
                      placeholder="workflow://my-workflow/data"
                      fontFamily="monospace"
                    />
                    <Label>Name *</Label>
                    <Input
                      value={resource.name || ''}
                      onChangeText={(text) => updateResource(index, 'name', text)}
                      placeholder="Workflow Data"
                    />
                    <Label>Description</Label>
                    <TextArea
                      value={resource.description || ''}
                      onChangeText={(text) => updateResource(index, 'description', text)}
                      placeholder="Description of this resource"
                      minHeight={60}
                    />
                    <Label>MIME Type</Label>
                    <Input
                      value={resource.mimeType || 'application/json'}
                      onChangeText={(text) => updateResource(index, 'mimeType', text)}
                      placeholder="application/json"
                      fontFamily="monospace"
                    />
                    <Label>Workflow ID *</Label>
                    <Input
                      value={resource.workflowId || ''}
                      onChangeText={(text) => updateResource(index, 'workflowId', text)}
                      placeholder="workflow-uuid"
                      fontFamily="monospace"
                    />
                  </YStack>
                </YStack>
              </Card>
            ))}
          </YStack>
        )}
      </YStack>

      <Separator />

      <YStack gap="$2">
        <XStack gap="$2" alignItems="center" justifyContent="space-between">
          <Text fontSize="$3" fontWeight="600">Tools</Text>
          <Button size="$2" onPress={addTool}>
            Add Tool
          </Button>
        </XStack>
        <Text fontSize="$2" color="$gray11">
          MCP tools expose executable functions that can be called by clients
        </Text>

        {tools.length === 0 ? (
          <Card padding="$3" backgroundColor="$gray2">
            <Text fontSize="$2" color="$gray11" textAlign="center">
              No tools defined. Add a tool to expose workflow functions.
            </Text>
          </Card>
        ) : (
          <YStack gap="$2">
            {tools.map((tool: any, index: number) => (
              <Card key={index} padding="$3" backgroundColor="$gray1" borderColor="$gray4">
                <YStack gap="$2">
                  <XStack gap="$2" alignItems="center" justifyContent="space-between">
                    <Text fontSize="$3" fontWeight="600">Tool {index + 1}</Text>
                    <Button size="$2" onPress={() => removeTool(index)}>
                      Remove
                    </Button>
                  </XStack>
                  <YStack gap="$2">
                    <Label>Name *</Label>
                    <Input
                      value={tool.name || ''}
                      onChangeText={(text) => updateTool(index, 'name', text)}
                      placeholder="execute_workflow"
                      fontFamily="monospace"
                    />
                    <Label>Description *</Label>
                    <TextArea
                      value={tool.description || ''}
                      onChangeText={(text) => updateTool(index, 'description', text)}
                      placeholder="Description of what this tool does"
                      minHeight={60}
                    />
                    <Label>Input Schema (JSON)</Label>
                    <TextArea
                      value={JSON.stringify(tool.inputSchema || {}, null, 2)}
                      onChangeText={(text) => {
                        try {
                          const parsed = JSON.parse(text);
                          updateTool(index, 'inputSchema', parsed);
                        } catch (e) {
                          // Invalid JSON, keep as is for now
                        }
                      }}
                      placeholder='{"type": "object", "properties": {...}}'
                      minHeight={100}
                      fontFamily="monospace"
                      fontSize="$2"
                    />
                    <Label>Workflow ID *</Label>
                    <Input
                      value={tool.workflowId || ''}
                      onChangeText={(text) => updateTool(index, 'workflowId', text)}
                      placeholder="workflow-uuid"
                      fontFamily="monospace"
                    />
                  </YStack>
                </YStack>
              </Card>
            ))}
          </YStack>
        )}
      </YStack>

      <Card padding="$2" backgroundColor="$blue2" borderColor="$blue6">
        <Text fontSize="$2" color="$blue11">
          <Text fontWeight="600">Note:</Text> MCP servers expose workflows as resources (readable data) and tools (executable functions) that can be used by AI assistants and other MCP clients.
        </Text>
      </Card>
    </YStack>
  );
}
