/**
 * Validation Rule Builder
 *
 * Visual builder for creating validation rules.
 */

'use client';

import { memo, useCallback } from 'react';
import { YStack, XStack, Text, Button, Card, Input, Select } from 'tamagui';

export interface ValidationRule {
  id: string;
  field: string;
  rule: ValidationRuleType;
  params: Record<string, string | number | boolean>;
  errorMessage: string;
}

export type ValidationRuleType =
  | 'required'
  | 'email'
  | 'url'
  | 'min_length'
  | 'max_length'
  | 'min_value'
  | 'max_value'
  | 'pattern'
  | 'enum'
  | 'custom';

interface ValidationRuleConfig {
  type: ValidationRuleType;
  label: string;
  description: string;
  params: Array<{
    name: string;
    type: 'string' | 'number' | 'boolean';
    label: string;
    defaultValue?: string | number | boolean;
  }>;
  rustValidator: string;
}

const VALIDATION_RULES: ValidationRuleConfig[] = [
  {
    type: 'required',
    label: 'Required',
    description: 'Field must have a value',
    params: [],
    rustValidator: '#[validate(required)]',
  },
  {
    type: 'email',
    label: 'Email',
    description: 'Must be a valid email address',
    params: [],
    rustValidator: '#[validate(email)]',
  },
  {
    type: 'url',
    label: 'URL',
    description: 'Must be a valid URL',
    params: [],
    rustValidator: '#[validate(url)]',
  },
  {
    type: 'min_length',
    label: 'Min Length',
    description: 'Minimum string length',
    params: [{ name: 'min', type: 'number', label: 'Minimum', defaultValue: 1 }],
    rustValidator: '#[validate(length(min = {min}))]',
  },
  {
    type: 'max_length',
    label: 'Max Length',
    description: 'Maximum string length',
    params: [
      { name: 'max', type: 'number', label: 'Maximum', defaultValue: 100 },
    ],
    rustValidator: '#[validate(length(max = {max}))]',
  },
  {
    type: 'min_value',
    label: 'Min Value',
    description: 'Minimum numeric value',
    params: [{ name: 'min', type: 'number', label: 'Minimum', defaultValue: 0 }],
    rustValidator: '#[validate(range(min = {min}))]',
  },
  {
    type: 'max_value',
    label: 'Max Value',
    description: 'Maximum numeric value',
    params: [
      { name: 'max', type: 'number', label: 'Maximum', defaultValue: 100 },
    ],
    rustValidator: '#[validate(range(max = {max}))]',
  },
  {
    type: 'pattern',
    label: 'Pattern',
    description: 'Must match regex pattern',
    params: [
      { name: 'regex', type: 'string', label: 'Pattern', defaultValue: '.*' },
    ],
    rustValidator: '#[validate(regex(path = "{regex}"))]',
  },
  {
    type: 'enum',
    label: 'Enum Values',
    description: 'Must be one of specified values',
    params: [
      {
        name: 'values',
        type: 'string',
        label: 'Values (comma-separated)',
        defaultValue: 'value1,value2',
      },
    ],
    rustValidator: '#[validate(contains(collection = vec![{values}]))]',
  },
  {
    type: 'custom',
    label: 'Custom',
    description: 'Custom validation function',
    params: [
      {
        name: 'function',
        type: 'string',
        label: 'Function name',
        defaultValue: 'validate_custom',
      },
    ],
    rustValidator: '#[validate(custom(function = "{function}"))]',
  },
];

interface ValidationRuleBuilderProps {
  rules: ValidationRule[];
  availableFields: Array<{ name: string; type: string }>;
  onChange: (rules: ValidationRule[]) => void;
}

export const ValidationRuleBuilder = memo(function ValidationRuleBuilder({
  rules,
  availableFields,
  onChange,
}: ValidationRuleBuilderProps) {
  const addRule = useCallback(() => {
    const newRule: ValidationRule = {
      id: `rule-${Date.now()}`,
      field: availableFields[0]?.name || '',
      rule: 'required',
      params: {},
      errorMessage: 'Validation failed',
    };
    onChange([...rules, newRule]);
  }, [rules, availableFields, onChange]);

  const updateRule = useCallback(
    (id: string, updates: Partial<ValidationRule>) => {
      onChange(
        rules.map((rule) =>
          rule.id === id ? { ...rule, ...updates } : rule
        )
      );
    },
    [rules, onChange]
  );

  const removeRule = useCallback(
    (id: string) => {
      onChange(rules.filter((rule) => rule.id !== id));
    },
    [rules, onChange]
  );

  const getRuleConfig = (ruleType: ValidationRuleType): ValidationRuleConfig => {
    return (
      VALIDATION_RULES.find((r) => r.type === ruleType) || VALIDATION_RULES[0]!
    );
  };

  return (
    <YStack gap="$4">
      <XStack jc="space-between" ai="center">
        <YStack>
          <Text fontSize="$5" fontWeight="bold">
            Validation Rules
          </Text>
          <Text fontSize="$2" color="$gray10">
            Add validation rules to your schema fields
          </Text>
        </YStack>
        <Button size="$3" onPress={addRule} disabled={availableFields.length === 0}>
          + Add Rule
        </Button>
      </XStack>

      {availableFields.length === 0 && (
        <Card p="$4" bg="$yellow2" borderColor="$yellow8" borderWidth={1}>
          <Text color="$yellow11">
            Add fields to your schema before creating validation rules
          </Text>
        </Card>
      )}

      <YStack gap="$3">
        {rules.map((rule) => {
          const config = getRuleConfig(rule.rule);

          return (
            <Card key={rule.id} p="$3" bordered>
              <YStack gap="$3">
                {/* Field and Rule Type Selection */}
                <XStack gap="$3" flexWrap="wrap">
                  <YStack f={1} minWidth={150}>
                    <Text fontSize="$2" fontWeight="500" mb="$1">
                      Field
                    </Text>
                    <Select
                      value={rule.field}
                      onValueChange={(field: string) =>
                        updateRule(rule.id, { field })
                      }
                    >
                      <Select.Trigger>
                        <Select.Value placeholder="Select field" />
                      </Select.Trigger>
                      <Select.Content>
                        <Select.Viewport>
                          {availableFields.map((field, idx) => (
                            <Select.Item key={field.name} value={field.name} index={idx}>
                              <Select.ItemText>
                                {field.name} ({field.type})
                              </Select.ItemText>
                            </Select.Item>
                          ))}
                        </Select.Viewport>
                      </Select.Content>
                    </Select>
                  </YStack>

                  <YStack f={1} minWidth={150}>
                    <Text fontSize="$2" fontWeight="500" mb="$1">
                      Rule Type
                    </Text>
                    <Select
                      value={rule.rule}
                      onValueChange={(ruleType: string) =>
                        updateRule(rule.id, {
                          rule: ruleType as ValidationRuleType,
                          params: {},
                        })
                      }
                    >
                      <Select.Trigger>
                        <Select.Value placeholder="Select rule" />
                      </Select.Trigger>
                      <Select.Content>
                        <Select.Viewport>
                          {VALIDATION_RULES.map((r, idx) => (
                            <Select.Item key={r.type} value={r.type} index={idx}>
                              <Select.ItemText>{r.label}</Select.ItemText>
                            </Select.Item>
                          ))}
                        </Select.Viewport>
                      </Select.Content>
                    </Select>
                  </YStack>
                </XStack>

                {/* Rule Parameters */}
                {config.params.length > 0 && (
                  <XStack gap="$3" flexWrap="wrap">
                    {config.params.map((param) => (
                      <YStack key={param.name} f={1} minWidth={150}>
                        <Text fontSize="$2" fontWeight="500" mb="$1">
                          {param.label}
                        </Text>
                        <Input
                          value={String(
                            rule.params[param.name] ?? param.defaultValue ?? ''
                          )}
                          onChangeText={(value: string) =>
                            updateRule(rule.id, {
                              params: {
                                ...rule.params,
                                [param.name]:
                                  param.type === 'number'
                                    ? Number(value)
                                    : value,
                              },
                            })
                          }
                          placeholder={param.label}
                          keyboardType={
                            param.type === 'number' ? 'numeric' : 'default'
                          }
                        />
                      </YStack>
                    ))}
                  </XStack>
                )}

                {/* Error Message */}
                <YStack>
                  <Text fontSize="$2" fontWeight="500" mb="$1">
                    Error Message
                  </Text>
                  <Input
                    value={rule.errorMessage}
                    onChangeText={(errorMessage: string) =>
                      updateRule(rule.id, { errorMessage })
                    }
                    placeholder="Validation error message"
                  />
                </YStack>

                {/* Rule Preview */}
                <Card bg="$gray2" p="$2">
                  <Text fontSize="$2" fontFamily="$mono" color="$gray11">
                    {config.rustValidator}
                  </Text>
                </Card>

                {/* Remove Button */}
                <XStack jc="flex-end">
                  <Button
                    size="$2"
                    theme="red"
                    onPress={() => removeRule(rule.id)}
                  >
                    Remove
                  </Button>
                </XStack>
              </YStack>
            </Card>
          );
        })}
      </YStack>

      {rules.length === 0 && availableFields.length > 0 && (
        <Card p="$4" bg="$gray2" ai="center">
          <Text color="$gray10">
            No validation rules yet. Click "Add Rule" to create one.
          </Text>
        </Card>
      )}
    </YStack>
  );
});
