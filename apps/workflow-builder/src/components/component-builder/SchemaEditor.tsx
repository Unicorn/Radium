/**
 * Schema Editor Component
 *
 * Visual editor for component input/output schemas.
 */

'use client';

import { useState, useCallback } from 'react';
import {
  YStack,
  XStack,
  Text,
  Input,
  Button,
  Card,
  Select,
  Checkbox,
  H3,
} from 'tamagui';

/** Field definition */
export interface Field {
  name: string;
  type: string;
  required: boolean;
  default?: string;
  validation?: string;
  description?: string;
}

/** Props for SchemaEditor */
export interface SchemaEditorProps {
  title: string;
  fields: Field[];
  onChange: (fields: Field[]) => void;
  readOnly?: boolean;
}

/** Available Rust types */
const RUST_TYPES = [
  'String',
  'i32',
  'i64',
  'u32',
  'u64',
  'f32',
  'f64',
  'bool',
  'Vec<String>',
  'Vec<i32>',
  'Option<String>',
  'Option<i32>',
  'Option<bool>',
  'HashMap<String, String>',
  'HashMap<String, Value>',
  'serde_json::Value',
  'chrono::DateTime<Utc>',
];

/** Type to TypeScript mapping */
const TYPE_TO_TS: Record<string, string> = {
  String: 'string',
  i32: 'number',
  i64: 'number',
  u32: 'number',
  u64: 'number',
  f32: 'number',
  f64: 'number',
  bool: 'boolean',
  'Vec<String>': 'string[]',
  'Vec<i32>': 'number[]',
  'Option<String>': 'string | undefined',
  'Option<i32>': 'number | undefined',
  'Option<bool>': 'boolean | undefined',
  'HashMap<String, String>': 'Record<string, string>',
  'HashMap<String, Value>': 'Record<string, unknown>',
  'serde_json::Value': 'unknown',
  'chrono::DateTime<Utc>': 'string',
};

/**
 * Schema Editor Component
 */
export function SchemaEditor({
  title,
  fields,
  onChange,
  readOnly = false,
}: SchemaEditorProps) {
  const addField = useCallback(() => {
    onChange([
      ...fields,
      {
        name: '',
        type: 'String',
        required: true,
        description: '',
      },
    ]);
  }, [fields, onChange]);

  const updateField = useCallback(
    (index: number, updates: Partial<Field>) => {
      const newFields = [...fields];
      const currentField = newFields[index];
      if (currentField) {
        newFields[index] = { ...currentField, ...updates };
        onChange(newFields);
      }
    },
    [fields, onChange]
  );

  const removeField = useCallback(
    (index: number) => {
      onChange(fields.filter((_, i) => i !== index));
    },
    [fields, onChange]
  );

  const moveField = useCallback(
    (index: number, direction: 'up' | 'down') => {
      const newFields = [...fields];
      const targetIndex = direction === 'up' ? index - 1 : index + 1;
      if (targetIndex < 0 || targetIndex >= fields.length) return;

      const currentField = newFields[index];
      const targetField = newFields[targetIndex];
      if (currentField && targetField) {
        newFields[index] = targetField;
        newFields[targetIndex] = currentField;
        onChange(newFields);
      }
    },
    [fields, onChange]
  );

  return (
    <Card padding="$3" bordered>
      <XStack justifyContent="space-between" alignItems="center" marginBottom="$3">
        <H3>{title}</H3>
        {!readOnly && (
          <Button size="$2" onPress={addField} theme="blue">
            + Add Field
          </Button>
        )}
      </XStack>

      <YStack gap="$3">
        {fields.length === 0 ? (
          <Text color="$gray10" textAlign="center" padding="$4">
            No fields defined. Click &quot;Add Field&quot; to add one.
          </Text>
        ) : (
          fields.map((field, index) => (
            <FieldEditor
              key={index}
              field={field}
              index={index}
              totalFields={fields.length}
              onChange={(updates) => updateField(index, updates)}
              onRemove={() => removeField(index)}
              onMove={(direction) => moveField(index, direction)}
              readOnly={readOnly}
            />
          ))
        )}
      </YStack>
    </Card>
  );
}

/** Props for FieldEditor */
interface FieldEditorProps {
  field: Field;
  index: number;
  totalFields: number;
  onChange: (updates: Partial<Field>) => void;
  onRemove: () => void;
  onMove: (direction: 'up' | 'down') => void;
  readOnly: boolean;
}

/**
 * Single field editor
 */
function FieldEditor({
  field,
  index,
  totalFields,
  onChange,
  onRemove,
  onMove,
  readOnly,
}: FieldEditorProps) {
  const [expanded, setExpanded] = useState(false);

  const tsType = TYPE_TO_TS[field.type] || 'unknown';

  return (
    <Card
      padding="$2"
      backgroundColor="$gray2"
      bordered
      borderColor={field.required ? '$blue6' : '$gray6'}
    >
      <XStack gap="$2" alignItems="center">
        {/* Reorder buttons */}
        {!readOnly && (
          <YStack gap="$1">
            <Button
              size="$1"
              variant="outlined"
              disabled={index === 0}
              onPress={() => onMove('up')}
            >
              ↑
            </Button>
            <Button
              size="$1"
              variant="outlined"
              disabled={index === totalFields - 1}
              onPress={() => onMove('down')}
            >
              ↓
            </Button>
          </YStack>
        )}

        {/* Field name */}
        <Input
          flex={1}
          size="$2"
          value={field.name}
          onChangeText={(name) => onChange({ name })}
          placeholder="field_name"
          disabled={readOnly}
        />

        {/* Type selector */}
        {readOnly ? (
          <Text
            backgroundColor="$gray4"
            paddingHorizontal="$2"
            paddingVertical="$1"
            borderRadius="$2"
            fontFamily="$mono"
            fontSize="$2"
          >
            {field.type}
          </Text>
        ) : (
          <Select
            value={field.type}
            onValueChange={(type) => onChange({ type })}
          >
            <Select.Trigger width={180}>
              <Select.Value placeholder="Type" />
            </Select.Trigger>
            <Select.Content>
              <Select.ScrollUpButton />
              <Select.Viewport>
                {RUST_TYPES.map((type) => (
                  <Select.Item key={type} value={type} index={RUST_TYPES.indexOf(type)}>
                    <Select.ItemText>{type}</Select.ItemText>
                  </Select.Item>
                ))}
              </Select.Viewport>
              <Select.ScrollDownButton />
            </Select.Content>
          </Select>
        )}

        {/* Required checkbox */}
        <XStack alignItems="center" gap="$1">
          <Checkbox
            checked={field.required}
            onCheckedChange={(checked) =>
              onChange({ required: checked === true })
            }
            disabled={readOnly}
          >
            <Checkbox.Indicator>
              <Text>✓</Text>
            </Checkbox.Indicator>
          </Checkbox>
          <Text fontSize="$2" color="$gray11">
            Required
          </Text>
        </XStack>

        {/* Expand/collapse */}
        <Button
          size="$2"
          variant="outlined"
          onPress={() => setExpanded(!expanded)}
        >
          {expanded ? '−' : '+'}
        </Button>

        {/* Remove button */}
        {!readOnly && (
          <Button size="$2" theme="red" variant="outlined" onPress={onRemove}>
            ×
          </Button>
        )}
      </XStack>

      {/* TypeScript type preview */}
      <XStack marginTop="$1" paddingLeft="$8">
        <Text fontSize="$1" color="$gray10" fontFamily="$mono">
          TS: {field.name || 'field'}: {tsType}
        </Text>
      </XStack>

      {/* Expanded options */}
      {expanded && (
        <YStack gap="$2" marginTop="$2" paddingLeft="$8">
          <XStack gap="$2" alignItems="center">
            <Text fontSize="$2" width={80}>
              Default:
            </Text>
            <Input
              flex={1}
              size="$2"
              value={field.default || ''}
              onChangeText={(defaultValue) => onChange({ default: defaultValue })}
              placeholder="Default value"
              disabled={readOnly}
            />
          </XStack>

          <XStack gap="$2" alignItems="center">
            <Text fontSize="$2" width={80}>
              Validation:
            </Text>
            <Input
              flex={1}
              size="$2"
              value={field.validation || ''}
              onChangeText={(validation) => onChange({ validation })}
              placeholder="e.g., #[validate(length(min = 1))]"
              disabled={readOnly}
            />
          </XStack>

          <XStack gap="$2" alignItems="center">
            <Text fontSize="$2" width={80}>
              Description:
            </Text>
            <Input
              flex={1}
              size="$2"
              value={field.description || ''}
              onChangeText={(description) => onChange({ description })}
              placeholder="Field description"
              disabled={readOnly}
            />
          </XStack>
        </YStack>
      )}
    </Card>
  );
}

/**
 * Generate Rust struct code from fields
 */
export function generateRustStruct(
  structName: string,
  fields: Field[]
): string {
  const fieldLines = fields.map((field) => {
    const lines: string[] = [];

    // Add serde annotations
    if (!field.required) {
      lines.push('    #[serde(skip_serializing_if = "Option::is_none")]');
    }
    if (field.default) {
      lines.push(`    #[serde(default = "default_${field.name}")]`);
    }

    // Add validation
    if (field.validation) {
      lines.push(`    ${field.validation}`);
    }

    // Add field
    const rustType = field.required ? field.type : `Option<${field.type}>`;
    lines.push(`    pub ${field.name}: ${rustType},`);

    return lines.join('\n');
  });

  return `#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ${structName} {
${fieldLines.join('\n')}
}`;
}

/**
 * Generate TypeScript interface from fields
 */
export function generateTypeScriptInterface(
  interfaceName: string,
  fields: Field[]
): string {
  const fieldLines = fields.map((field) => {
    const tsType = TYPE_TO_TS[field.type] || 'unknown';
    const optional = field.required ? '' : '?';
    const comment = field.description ? `  /** ${field.description} */\n` : '';
    return `${comment}  ${field.name}${optional}: ${tsType};`;
  });

  return `export interface ${interfaceName} {
${fieldLines.join('\n')}
}`;
}
