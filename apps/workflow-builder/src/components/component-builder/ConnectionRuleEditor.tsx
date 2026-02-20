/**
 * Connection Rule Editor
 *
 * Editor for defining which components can connect to this component.
 */

'use client';

import { memo, useCallback, useState } from 'react';
import { YStack, XStack, Text, Button, Card, Input, Checkbox, Label } from 'tamagui';

export interface ConnectionRules {
  allowedSources: string[];
  allowedTargets: string[];
  maxConnections: number;
  allowSelfLoop: boolean;
  requiredPrevious: string[];
}

interface ConnectionRuleEditorProps {
  rules: ConnectionRules;
  availableComponents: string[];
  onChange: (rules: ConnectionRules) => void;
}

const DEFAULT_COMPONENTS = [
  'trigger',
  'http_request',
  'database_query',
  'agent',
  'conditional',
  'loop',
  'parallel',
  'timer',
  'signal',
  'child_workflow',
];

export const ConnectionRuleEditor = memo(function ConnectionRuleEditor({
  rules,
  availableComponents = DEFAULT_COMPONENTS,
  onChange,
}: ConnectionRuleEditorProps) {
  const [customSource, setCustomSource] = useState('');
  const [customTarget, setCustomTarget] = useState('');

  const toggleSource = useCallback(
    (component: string) => {
      const newSources = rules.allowedSources.includes(component)
        ? rules.allowedSources.filter((s) => s !== component)
        : [...rules.allowedSources, component];
      onChange({ ...rules, allowedSources: newSources });
    },
    [rules, onChange]
  );

  const toggleTarget = useCallback(
    (component: string) => {
      const newTargets = rules.allowedTargets.includes(component)
        ? rules.allowedTargets.filter((t) => t !== component)
        : [...rules.allowedTargets, component];
      onChange({ ...rules, allowedTargets: newTargets });
    },
    [rules, onChange]
  );

  const addCustomSource = useCallback(() => {
    if (customSource && !rules.allowedSources.includes(customSource)) {
      onChange({
        ...rules,
        allowedSources: [...rules.allowedSources, customSource],
      });
      setCustomSource('');
    }
  }, [customSource, rules, onChange]);

  const addCustomTarget = useCallback(() => {
    if (customTarget && !rules.allowedTargets.includes(customTarget)) {
      onChange({
        ...rules,
        allowedTargets: [...rules.allowedTargets, customTarget],
      });
      setCustomTarget('');
    }
  }, [customTarget, rules, onChange]);

  const setAllowAll = useCallback(
    (type: 'sources' | 'targets') => {
      if (type === 'sources') {
        onChange({ ...rules, allowedSources: ['*'] });
      } else {
        onChange({ ...rules, allowedTargets: ['*'] });
      }
    },
    [rules, onChange]
  );

  const clearAll = useCallback(
    (type: 'sources' | 'targets') => {
      if (type === 'sources') {
        onChange({ ...rules, allowedSources: [] });
      } else {
        onChange({ ...rules, allowedTargets: [] });
      }
    },
    [rules, onChange]
  );

  const isAllAllowed = (list: string[]) => list.includes('*');

  return (
    <YStack gap="$4">
      <YStack>
        <Text fontSize="$5" fontWeight="bold">
          Connection Rules
        </Text>
        <Text fontSize="$2" color="$gray10">
          Define which components can connect to and from this component
        </Text>
      </YStack>

      {/* General Settings */}
      <Card p="$3" bordered>
        <YStack gap="$3">
          <Text fontSize="$4" fontWeight="600">
            General Settings
          </Text>

          <XStack gap="$4" flexWrap="wrap" ai="center">
            <YStack>
              <Text fontSize="$2" fontWeight="500" mb="$1">
                Max Connections
              </Text>
              <Input
                value={String(rules.maxConnections)}
                onChangeText={(value: string) =>
                  onChange({
                    ...rules,
                    maxConnections: Math.max(1, Number(value) || 1),
                  })
                }
                keyboardType="numeric"
                w={100}
              />
            </YStack>

            <XStack ai="center" gap="$2">
              <Checkbox
                id="self-loop"
                checked={rules.allowSelfLoop}
                onCheckedChange={(checked: boolean | 'indeterminate') =>
                  onChange({ ...rules, allowSelfLoop: checked === true })
                }
              >
                <Checkbox.Indicator>
                  <Text>X</Text>
                </Checkbox.Indicator>
              </Checkbox>
              <Label htmlFor="self-loop">
                <Text fontSize="$3">Allow self-loop connections</Text>
              </Label>
            </XStack>
          </XStack>
        </YStack>
      </Card>

      {/* Allowed Sources */}
      <Card p="$3" bordered>
        <YStack gap="$3">
          <XStack jc="space-between" ai="center">
            <YStack>
              <Text fontSize="$4" fontWeight="600">
                Allowed Sources
              </Text>
              <Text fontSize="$2" color="$gray10">
                Components that can connect TO this component
              </Text>
            </YStack>
            <XStack gap="$2">
              <Button size="$2" onPress={() => setAllowAll('sources')}>
                Allow All
              </Button>
              <Button size="$2" theme="gray" onPress={() => clearAll('sources')}>
                Clear
              </Button>
            </XStack>
          </XStack>

          {isAllAllowed(rules.allowedSources) ? (
            <Card bg="$green2" p="$3" borderColor="$green8" borderWidth={1}>
              <Text color="$green11" fontWeight="500">
                All components are allowed as sources
              </Text>
            </Card>
          ) : (
            <XStack flexWrap="wrap" gap="$2">
              {availableComponents.map((component) => (
                <Button
                  key={component}
                  size="$2"
                  theme={
                    rules.allowedSources.includes(component) ? 'blue' : 'gray'
                  }
                  onPress={() => toggleSource(component)}
                >
                  {component}
                </Button>
              ))}
            </XStack>
          )}

          <XStack gap="$2" ai="center">
            <Input
              f={1}
              value={customSource}
              onChangeText={setCustomSource}
              placeholder="Add custom source..."
              onSubmitEditing={addCustomSource}
            />
            <Button size="$2" onPress={addCustomSource}>
              Add
            </Button>
          </XStack>

          {rules.allowedSources.length > 0 &&
            !isAllAllowed(rules.allowedSources) && (
              <Text fontSize="$2" color="$gray10">
                Selected: {rules.allowedSources.join(', ')}
              </Text>
            )}
        </YStack>
      </Card>

      {/* Allowed Targets */}
      <Card p="$3" bordered>
        <YStack gap="$3">
          <XStack jc="space-between" ai="center">
            <YStack>
              <Text fontSize="$4" fontWeight="600">
                Allowed Targets
              </Text>
              <Text fontSize="$2" color="$gray10">
                Components this component can connect TO
              </Text>
            </YStack>
            <XStack gap="$2">
              <Button size="$2" onPress={() => setAllowAll('targets')}>
                Allow All
              </Button>
              <Button size="$2" theme="gray" onPress={() => clearAll('targets')}>
                Clear
              </Button>
            </XStack>
          </XStack>

          {isAllAllowed(rules.allowedTargets) ? (
            <Card bg="$green2" p="$3" borderColor="$green8" borderWidth={1}>
              <Text color="$green11" fontWeight="500">
                All components are allowed as targets
              </Text>
            </Card>
          ) : (
            <XStack flexWrap="wrap" gap="$2">
              {availableComponents.map((component) => (
                <Button
                  key={component}
                  size="$2"
                  theme={
                    rules.allowedTargets.includes(component) ? 'blue' : 'gray'
                  }
                  onPress={() => toggleTarget(component)}
                >
                  {component}
                </Button>
              ))}
            </XStack>
          )}

          <XStack gap="$2" ai="center">
            <Input
              f={1}
              value={customTarget}
              onChangeText={setCustomTarget}
              placeholder="Add custom target..."
              onSubmitEditing={addCustomTarget}
            />
            <Button size="$2" onPress={addCustomTarget}>
              Add
            </Button>
          </XStack>

          {rules.allowedTargets.length > 0 &&
            !isAllAllowed(rules.allowedTargets) && (
              <Text fontSize="$2" color="$gray10">
                Selected: {rules.allowedTargets.join(', ')}
              </Text>
            )}
        </YStack>
      </Card>

      {/* Required Previous Components */}
      <Card p="$3" bordered>
        <YStack gap="$3">
          <YStack>
            <Text fontSize="$4" fontWeight="600">
              Required Previous Components
            </Text>
            <Text fontSize="$2" color="$gray10">
              Components that MUST exist before this component in the workflow
            </Text>
          </YStack>

          <XStack flexWrap="wrap" gap="$2">
            {availableComponents.map((component) => (
              <Button
                key={component}
                size="$2"
                theme={
                  rules.requiredPrevious.includes(component) ? 'orange' : 'gray'
                }
                onPress={() => {
                  const newRequired = rules.requiredPrevious.includes(component)
                    ? rules.requiredPrevious.filter((c) => c !== component)
                    : [...rules.requiredPrevious, component];
                  onChange({ ...rules, requiredPrevious: newRequired });
                }}
              >
                {component}
              </Button>
            ))}
          </XStack>

          {rules.requiredPrevious.length > 0 && (
            <Card bg="$orange2" p="$2" borderColor="$orange8" borderWidth={1}>
              <Text fontSize="$2" color="$orange11">
                Requires: {rules.requiredPrevious.join(', ')} to be present in
                workflow
              </Text>
            </Card>
          )}
        </YStack>
      </Card>
    </YStack>
  );
});
