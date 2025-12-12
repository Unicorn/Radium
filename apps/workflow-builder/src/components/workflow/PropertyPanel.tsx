/**
 * Property Panel - Configure selected workflow node
 */

'use client';

import { YStack, XStack, Text, Button, Input, Label, Separator } from 'tamagui';
import { useState, useEffect } from 'react';
import { X } from 'lucide-react';
import type { WorkflowNode } from '@/types/workflow';
import { ActivityConfigPanel } from './ActivityConfigPanel';

interface PropertyPanelProps {
  node: WorkflowNode;
  onUpdate: (updated: Partial<WorkflowNode>) => void;
  onClose: () => void;
}

export function PropertyPanel({ node, onUpdate, onClose }: PropertyPanelProps) {
  const [label, setLabel] = useState(node.data.label);
  const [config, setConfig] = useState(
    JSON.stringify(node.data.config || {}, null, 2)
  );

  useEffect(() => {
    setLabel(node.data.label);
    setConfig(JSON.stringify(node.data.config || {}, null, 2));
  }, [node]);

  const handleSave = () => {
    try {
      const parsedConfig = JSON.parse(config);
      onUpdate({
        data: {
          ...node.data,
          label,
          config: parsedConfig,
        },
      });
    } catch (err) {
      alert('Invalid JSON configuration');
    }
  };

  return (
    <YStack
      width={320}
      backgroundColor="$background"
      borderLeftWidth={1}
      borderColor="$borderColor"
      padding="$4"
      gap="$4"
    >
      {/* Header */}
      <XStack justifyContent="space-between" alignItems="center">
        <Text fontSize="$6" fontWeight="bold">Properties</Text>
        <Button
          size="$3"
          circular
          icon={X}
          onPress={onClose}
          variant="outlined"
        />
      </XStack>

      <Separator />

      {/* Node Info */}
      <YStack gap="$2">
        <Text fontSize="$3" color="$gray11">Type</Text>
        <Text fontSize="$4" fontWeight="600" textTransform="capitalize">
          {node.type}
        </Text>
      </YStack>

      <YStack gap="$2">
        <Text fontSize="$3" color="$gray11">Component</Text>
        <Text fontSize="$4" fontWeight="600">
          {node.data.componentName || 'No component'}
        </Text>
      </YStack>

      <Separator />

      {/* Label */}
      <YStack gap="$2">
        <Label htmlFor="label" fontSize="$3" fontWeight="600">
          Label
        </Label>
        <Input
          id="label"
          size="$4"
          value={label}
          onChangeText={setLabel}
          onBlur={handleSave}
        />
      </YStack>

      {/* Activity-specific configuration */}
      {(node.type === 'activity' || node.type === 'agent') && (
        <>
          <Separator />
          <Text fontSize="$4" fontWeight="bold">Activity Settings</Text>
          <ActivityConfigPanel node={node} onUpdate={onUpdate} />
        </>
      )}

      {/* Configuration */}
      <YStack gap="$2">
        <Label htmlFor="config" fontSize="$3" fontWeight="600">
          Configuration (JSON)
        </Label>
        <YStack
          height={200}
          borderWidth={1}
          borderColor="$borderColor"
          borderRadius="$4"
          padding="$2"
          backgroundColor="$gray2"
        >
          <textarea
            id="config"
            value={config}
            onChange={(e) => setConfig(e.target.value)}
            onBlur={handleSave}
            style={{
              flex: 1,
              border: 'none',
              background: 'transparent',
              fontFamily: 'monospace',
              fontSize: 12,
              resize: 'none',
              outline: 'none',
            }}
          />
        </YStack>
        <Text fontSize="$2" color="$gray11">
          Configuration options for this component
        </Text>
      </YStack>

      {/* Position */}
      <YStack gap="$2">
        <Text fontSize="$3" fontWeight="600">Position</Text>
        <XStack gap="$2">
          <YStack flex={1}>
            <Text fontSize="$2" color="$gray11">X: {Math.round(node.position.x)}</Text>
          </YStack>
          <YStack flex={1}>
            <Text fontSize="$2" color="$gray11">Y: {Math.round(node.position.y)}</Text>
          </YStack>
        </XStack>
      </YStack>
    </YStack>
  );
}

