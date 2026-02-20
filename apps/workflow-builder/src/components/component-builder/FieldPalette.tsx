/**
 * Field Palette
 *
 * Draggable field types for the visual component builder.
 */

'use client';

import { memo, useCallback } from 'react';
import { YStack, XStack, Text, Card } from 'tamagui';

export interface FieldType {
  id: string;
  name: string;
  rustType: string;
  typescriptType: string;
  icon: string;
  category: 'primitive' | 'complex' | 'temporal' | 'custom';
  description: string;
}

export const FIELD_TYPES: FieldType[] = [
  // Primitive types
  {
    id: 'string',
    name: 'String',
    rustType: 'String',
    typescriptType: 'string',
    icon: 'Aa',
    category: 'primitive',
    description: 'Text value',
  },
  {
    id: 'number',
    name: 'Number',
    rustType: 'i64',
    typescriptType: 'number',
    icon: '#',
    category: 'primitive',
    description: 'Integer number',
  },
  {
    id: 'float',
    name: 'Float',
    rustType: 'f64',
    typescriptType: 'number',
    icon: '1.0',
    category: 'primitive',
    description: 'Decimal number',
  },
  {
    id: 'boolean',
    name: 'Boolean',
    rustType: 'bool',
    typescriptType: 'boolean',
    icon: '?',
    category: 'primitive',
    description: 'True/false value',
  },
  // Complex types
  {
    id: 'array',
    name: 'Array',
    rustType: 'Vec<String>',
    typescriptType: 'string[]',
    icon: '[]',
    category: 'complex',
    description: 'List of values',
  },
  {
    id: 'object',
    name: 'Object',
    rustType: 'serde_json::Value',
    typescriptType: 'Record<string, unknown>',
    icon: '{}',
    category: 'complex',
    description: 'JSON object',
  },
  {
    id: 'optional-string',
    name: 'Optional String',
    rustType: 'Option<String>',
    typescriptType: 'string | undefined',
    icon: 'Aa?',
    category: 'complex',
    description: 'Optional text value',
  },
  // Temporal types
  {
    id: 'datetime',
    name: 'DateTime',
    rustType: 'chrono::DateTime<chrono::Utc>',
    typescriptType: 'string',
    icon: '\uD83D\uDCC5',
    category: 'temporal',
    description: 'Date and time',
  },
  {
    id: 'duration',
    name: 'Duration',
    rustType: 'std::time::Duration',
    typescriptType: 'number',
    icon: '\u23F1',
    category: 'temporal',
    description: 'Time duration in milliseconds',
  },
];

interface FieldPaletteProps {
  onFieldSelect: (fieldType: FieldType) => void;
}

export const FieldPalette = memo(function FieldPalette({
  onFieldSelect,
}: FieldPaletteProps) {
  const categories = ['primitive', 'complex', 'temporal'] as const;
  const categoryLabels: Record<(typeof categories)[number], string> = {
    primitive: 'Basic Types',
    complex: 'Complex Types',
    temporal: 'Time Types',
  };

  const handleDragStart = useCallback(
    (e: React.DragEvent, fieldType: FieldType) => {
      e.dataTransfer.setData('application/json', JSON.stringify(fieldType));
      e.dataTransfer.effectAllowed = 'copy';
    },
    []
  );

  const handleClick = useCallback(
    (fieldType: FieldType) => {
      onFieldSelect(fieldType);
    },
    [onFieldSelect]
  );

  return (
    <YStack gap="$4" p="$3">
      <Text fontSize="$5" fontWeight="bold">
        Field Types
      </Text>
      <Text fontSize="$2" color="$gray10">
        Drag a field type to the schema or click to add
      </Text>

      {categories.map((category) => (
        <YStack key={category} gap="$2">
          <Text fontSize="$3" fontWeight="600" color="$gray11">
            {categoryLabels[category]}
          </Text>
          <XStack flexWrap="wrap" gap="$2">
            {FIELD_TYPES.filter((f) => f.category === category).map(
              (fieldType) => (
                <div
                  key={fieldType.id}
                  draggable
                  onDragStart={(e) => handleDragStart(e, fieldType)}
                  style={{ cursor: 'grab' }}
                >
                  <Card
                    p="$2"
                    br="$3"
                    hoverStyle={{ bg: '$blue2', borderColor: '$blue8' }}
                    pressStyle={{ scale: 0.98 }}
                    onPress={() => handleClick(fieldType)}
                    borderWidth={1}
                    borderColor="$borderColor"
                    minWidth={100}
                  >
                    <YStack ai="center" gap="$1">
                      <Text fontSize="$4">{fieldType.icon}</Text>
                      <Text fontSize="$2" fontWeight="500">
                        {fieldType.name}
                      </Text>
                      <Text fontSize="$1" color="$gray10" ta="center">
                        {fieldType.description}
                      </Text>
                    </YStack>
                  </Card>
                </div>
              )
            )}
          </XStack>
        </YStack>
      ))}
    </YStack>
  );
});
