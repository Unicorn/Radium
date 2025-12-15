/**
 * Visual Component Builder
 *
 * Main visual builder component with drag-and-drop, live preview,
 * undo/redo, and keyboard shortcuts.
 */

'use client';

import { useState, useCallback, useEffect, useMemo, useReducer, memo } from 'react';
import {
  YStack,
  XStack,
  Text,
  Button,
  Card,
  Input,
  Tabs,
  ScrollView,
  Separator,
  TextArea,
} from 'tamagui';
import { FieldPalette, FIELD_TYPES, FieldType } from './FieldPalette';
import { SchemaEditor, Field } from './SchemaEditor';
import {
  ValidationRuleBuilder,
  ValidationRule,
} from './ValidationRuleBuilder';
import {
  ConnectionRuleEditor,
  ConnectionRules,
} from './ConnectionRuleEditor';

// Types
interface ComponentDesign {
  name: string;
  description: string;
  category: string;
  temporalType: 'activity' | 'workflow' | 'signal' | 'query';
  inputFields: Field[];
  outputFields: Field[];
  validationRules: ValidationRule[];
  connectionRules: ConnectionRules;
}

interface BuilderState {
  design: ComponentDesign;
  history: ComponentDesign[];
  historyIndex: number;
  isDirty: boolean;
}

type BuilderAction =
  | { type: 'UPDATE_DESIGN'; payload: Partial<ComponentDesign> }
  | { type: 'UPDATE_INPUT_FIELDS'; payload: Field[] }
  | { type: 'UPDATE_OUTPUT_FIELDS'; payload: Field[] }
  | { type: 'UPDATE_VALIDATION_RULES'; payload: ValidationRule[] }
  | { type: 'UPDATE_CONNECTION_RULES'; payload: ConnectionRules }
  | { type: 'UNDO' }
  | { type: 'REDO' }
  | { type: 'RESET'; payload: ComponentDesign };

const DEFAULT_DESIGN: ComponentDesign = {
  name: '',
  description: '',
  category: 'activities',
  temporalType: 'activity',
  inputFields: [],
  outputFields: [],
  validationRules: [],
  connectionRules: {
    allowedSources: ['*'],
    allowedTargets: ['*'],
    maxConnections: 10,
    allowSelfLoop: false,
    requiredPrevious: [],
  },
};

const MAX_HISTORY = 50;

function builderReducer(state: BuilderState, action: BuilderAction): BuilderState {
  switch (action.type) {
    case 'UPDATE_DESIGN':
    case 'UPDATE_INPUT_FIELDS':
    case 'UPDATE_OUTPUT_FIELDS':
    case 'UPDATE_VALIDATION_RULES':
    case 'UPDATE_CONNECTION_RULES': {
      let newDesign: ComponentDesign;

      if (action.type === 'UPDATE_DESIGN') {
        newDesign = { ...state.design, ...action.payload };
      } else if (action.type === 'UPDATE_INPUT_FIELDS') {
        newDesign = { ...state.design, inputFields: action.payload };
      } else if (action.type === 'UPDATE_OUTPUT_FIELDS') {
        newDesign = { ...state.design, outputFields: action.payload };
      } else if (action.type === 'UPDATE_VALIDATION_RULES') {
        newDesign = { ...state.design, validationRules: action.payload };
      } else {
        newDesign = { ...state.design, connectionRules: action.payload };
      }

      // Add to history (truncate future if we're not at the end)
      const newHistory = [
        ...state.history.slice(0, state.historyIndex + 1),
        newDesign,
      ].slice(-MAX_HISTORY);

      return {
        design: newDesign,
        history: newHistory,
        historyIndex: newHistory.length - 1,
        isDirty: true,
      };
    }

    case 'UNDO': {
      if (state.historyIndex <= 0) return state;
      const newIndex = state.historyIndex - 1;
      return {
        ...state,
        design: state.history[newIndex]!,
        historyIndex: newIndex,
        isDirty: true,
      };
    }

    case 'REDO': {
      if (state.historyIndex >= state.history.length - 1) return state;
      const newIndex = state.historyIndex + 1;
      return {
        ...state,
        design: state.history[newIndex]!,
        historyIndex: newIndex,
        isDirty: true,
      };
    }

    case 'RESET': {
      return {
        design: action.payload,
        history: [action.payload],
        historyIndex: 0,
        isDirty: false,
      };
    }

    default:
      return state;
  }
}

interface VisualBuilderProps {
  initialDesign?: Partial<ComponentDesign>;
  onSave?: (design: ComponentDesign) => void;
  onGenerate?: (design: ComponentDesign) => void;
}

export const VisualBuilder = memo(function VisualBuilder({
  initialDesign,
  onSave,
  onGenerate,
}: VisualBuilderProps) {
  const [state, dispatch] = useReducer(builderReducer, {
    design: { ...DEFAULT_DESIGN, ...initialDesign },
    history: [{ ...DEFAULT_DESIGN, ...initialDesign }],
    historyIndex: 0,
    isDirty: false,
  });

  const [activeTab, setActiveTab] = useState('metadata');
  const [previewType, setPreviewType] = useState<'rust' | 'typescript'>('rust');

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;
      const modifier = isMac ? e.metaKey : e.ctrlKey;

      if (modifier && e.key === 'z') {
        e.preventDefault();
        if (e.shiftKey) {
          dispatch({ type: 'REDO' });
        } else {
          dispatch({ type: 'UNDO' });
        }
      } else if (modifier && e.key === 's') {
        e.preventDefault();
        if (onSave) {
          onSave(state.design);
        }
      } else if (modifier && e.key === 'g') {
        e.preventDefault();
        if (onGenerate) {
          onGenerate(state.design);
        }
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [state.design, onSave, onGenerate]);

  // Handle drag and drop for fields
  const handleDrop = useCallback(
    (e: React.DragEvent, target: 'input' | 'output') => {
      e.preventDefault();
      try {
        const data = e.dataTransfer.getData('application/json');
        const fieldType: FieldType = JSON.parse(data);

        const newField: Field = {
          name: `field_${Date.now()}`,
          type: fieldType.rustType,
          required: true,
          description: fieldType.description,
        };

        if (target === 'input') {
          dispatch({
            type: 'UPDATE_INPUT_FIELDS',
            payload: [...state.design.inputFields, newField],
          });
        } else {
          dispatch({
            type: 'UPDATE_OUTPUT_FIELDS',
            payload: [...state.design.outputFields, newField],
          });
        }
      } catch {
        console.error('Invalid drop data');
      }
    },
    [state.design.inputFields, state.design.outputFields]
  );

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'copy';
  }, []);

  // Add field from palette click
  const handleFieldSelect = useCallback(
    (fieldType: FieldType) => {
      const newField: Field = {
        name: `field_${Date.now()}`,
        type: fieldType.rustType,
        required: true,
        description: fieldType.description,
      };

      dispatch({
        type: 'UPDATE_INPUT_FIELDS',
        payload: [...state.design.inputFields, newField],
      });
    },
    [state.design.inputFields]
  );

  // Generate preview code
  const rustPreview = useMemo(() => {
    const { name, description, inputFields, outputFields, validationRules } =
      state.design;

    const structName = name
      ? name
          .split(/[-_\s]+/)
          .map((w) => w.charAt(0).toUpperCase() + w.slice(1).toLowerCase())
          .join('')
      : 'NewComponent';

    const inputStruct = `/// ${description || 'Input schema for ' + structName}
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ${structName}Input {
${inputFields
  .map((f) => {
    const validation = validationRules.find((r) => r.field === f.name);
    const validationAttr = validation
      ? `    #[validate(/* ${validation.rule} */)]\n`
      : '';
    const serdeAttr = !f.required
      ? '    #[serde(skip_serializing_if = "Option::is_none")]\n'
      : '';
    const fieldType = !f.required ? `Option<${f.type}>` : f.type;
    return `${validationAttr}${serdeAttr}    /// ${f.description || f.name}\n    pub ${f.name}: ${fieldType},`;
  })
  .join('\n\n')}
}`;

    const outputStruct = `/// Output schema for ${structName}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ${structName}Output {
${outputFields
  .map((f) => {
    const fieldType = !f.required ? `Option<${f.type}>` : f.type;
    return `    /// ${f.description || f.name}\n    pub ${f.name}: ${fieldType},`;
  })
  .join('\n\n')}
}`;

    return `use serde::{Deserialize, Serialize};
use validator::Validate;

${inputStruct}

${outputStruct}`;
  }, [state.design]);

  const typescriptPreview = useMemo(() => {
    const { name, description, inputFields, outputFields } = state.design;

    const interfaceName = name
      ? name
          .split(/[-_\s]+/)
          .map((w) => w.charAt(0).toUpperCase() + w.slice(1).toLowerCase())
          .join('')
      : 'NewComponent';

    const rustToTs: Record<string, string> = {
      String: 'string',
      i32: 'number',
      i64: 'number',
      f64: 'number',
      bool: 'boolean',
      'Vec<String>': 'string[]',
      'serde_json::Value': 'Record<string, unknown>',
      'Option<String>': 'string | undefined',
      'chrono::DateTime<chrono::Utc>': 'string',
      'std::time::Duration': 'number',
    };

    const mapType = (rustType: string): string => {
      if (rustToTs[rustType]) return rustToTs[rustType];
      if (rustType.startsWith('Option<')) {
        const inner = rustType.slice(7, -1);
        return `${mapType(inner)} | undefined`;
      }
      if (rustType.startsWith('Vec<')) {
        const inner = rustType.slice(4, -1);
        return `${mapType(inner)}[]`;
      }
      return 'unknown';
    };

    return `/**
 * ${description || interfaceName + ' Activity'}
 */

/**
 * Input interface for ${interfaceName}
 */
export interface ${interfaceName}Input {
${inputFields
  .map((f) => {
    const tsType = mapType(f.type);
    const optional = !f.required ? '?' : '';
    return `  /** ${f.description || f.name} */\n  ${f.name}${optional}: ${tsType};`;
  })
  .join('\n\n')}
}

/**
 * Output interface for ${interfaceName}
 */
export interface ${interfaceName}Output {
${outputFields
  .map((f) => {
    const tsType = mapType(f.type);
    const optional = !f.required ? '?' : '';
    return `  /** ${f.description || f.name} */\n  ${f.name}${optional}: ${tsType};`;
  })
  .join('\n\n')}
}`;
  }, [state.design]);

  const canUndo = state.historyIndex > 0;
  const canRedo = state.historyIndex < state.history.length - 1;

  return (
    <XStack f={1} h="100%">
      {/* Left Panel - Field Palette */}
      <YStack w={280} borderRightWidth={1} borderColor="$borderColor">
        <ScrollView>
          <FieldPalette onFieldSelect={handleFieldSelect} />
        </ScrollView>
      </YStack>

      {/* Center Panel - Schema Builder */}
      <YStack f={1}>
        {/* Toolbar */}
        <XStack
          p="$3"
          borderBottomWidth={1}
          borderColor="$borderColor"
          jc="space-between"
          ai="center"
        >
          <XStack gap="$2">
            <Button
              size="$2"
              onPress={() => dispatch({ type: 'UNDO' })}
              disabled={!canUndo}
              theme={canUndo ? undefined : 'gray'}
            >
              Undo
            </Button>
            <Button
              size="$2"
              onPress={() => dispatch({ type: 'REDO' })}
              disabled={!canRedo}
              theme={canRedo ? undefined : 'gray'}
            >
              Redo
            </Button>
            <Separator vertical />
            <Button
              size="$2"
              onPress={() => dispatch({ type: 'RESET', payload: DEFAULT_DESIGN })}
              theme="gray"
            >
              Reset
            </Button>
          </XStack>

          <XStack gap="$2">
            {state.isDirty && (
              <Text fontSize="$2" color="$orange10">
                Unsaved changes
              </Text>
            )}
            <Button size="$2" onPress={() => onSave?.(state.design)}>
              Save
            </Button>
            <Button size="$2" theme="blue" onPress={() => onGenerate?.(state.design)}>
              Generate Code
            </Button>
          </XStack>
        </XStack>

        {/* Keyboard shortcuts help */}
        <XStack px="$3" py="$1" bg="$gray2">
          <Text fontSize="$1" color="$gray10">
            Shortcuts: Ctrl/Cmd+Z (Undo) | Ctrl/Cmd+Shift+Z (Redo) | Ctrl/Cmd+S
            (Save) | Ctrl/Cmd+G (Generate)
          </Text>
        </XStack>

        {/* Main Content */}
        <ScrollView f={1}>
          <YStack p="$4" gap="$4">
            <Tabs
              value={activeTab}
              onValueChange={setActiveTab}
              orientation="horizontal"
            >
              <Tabs.List>
                <Tabs.Tab value="metadata">
                  <Text>Metadata</Text>
                </Tabs.Tab>
                <Tabs.Tab value="input">
                  <Text>Input Schema</Text>
                </Tabs.Tab>
                <Tabs.Tab value="output">
                  <Text>Output Schema</Text>
                </Tabs.Tab>
                <Tabs.Tab value="validation">
                  <Text>Validation</Text>
                </Tabs.Tab>
                <Tabs.Tab value="connections">
                  <Text>Connections</Text>
                </Tabs.Tab>
              </Tabs.List>

              <Tabs.Content value="metadata">
                <Card p="$4" mt="$3">
                  <YStack gap="$4">
                    <YStack>
                      <Text fontSize="$2" fontWeight="500" mb="$1">
                        Component Name
                      </Text>
                      <Input
                        value={state.design.name}
                        onChangeText={(name: string) =>
                          dispatch({ type: 'UPDATE_DESIGN', payload: { name } })
                        }
                        placeholder="my_component"
                      />
                    </YStack>

                    <YStack>
                      <Text fontSize="$2" fontWeight="500" mb="$1">
                        Description
                      </Text>
                      <TextArea
                        value={state.design.description}
                        onChangeText={(description: string) =>
                          dispatch({
                            type: 'UPDATE_DESIGN',
                            payload: { description },
                          })
                        }
                        placeholder="What does this component do?"
                        numberOfLines={3}
                      />
                    </YStack>

                    <XStack gap="$4">
                      <YStack f={1}>
                        <Text fontSize="$2" fontWeight="500" mb="$1">
                          Category
                        </Text>
                        <Input
                          value={state.design.category}
                          onChangeText={(category: string) =>
                            dispatch({
                              type: 'UPDATE_DESIGN',
                              payload: { category },
                            })
                          }
                          placeholder="activities"
                        />
                      </YStack>

                      <YStack f={1}>
                        <Text fontSize="$2" fontWeight="500" mb="$1">
                          Temporal Type
                        </Text>
                        <XStack gap="$2" flexWrap="wrap">
                          {(
                            ['activity', 'workflow', 'signal', 'query'] as const
                          ).map((type) => (
                            <Button
                              key={type}
                              size="$2"
                              theme={
                                state.design.temporalType === type
                                  ? 'blue'
                                  : 'gray'
                              }
                              onPress={() =>
                                dispatch({
                                  type: 'UPDATE_DESIGN',
                                  payload: { temporalType: type },
                                })
                              }
                            >
                              {type}
                            </Button>
                          ))}
                        </XStack>
                      </YStack>
                    </XStack>
                  </YStack>
                </Card>
              </Tabs.Content>

              <Tabs.Content value="input">
                <div
                  onDrop={(e) => handleDrop(e, 'input')}
                  onDragOver={handleDragOver}
                  style={{ marginTop: 12 }}
                >
                  <Card
                    p="$4"
                    borderWidth={2}
                    borderColor="$borderColor"
                    borderStyle="dashed"
                  >
                    <YStack gap="$3">
                      <Text fontSize="$2" color="$gray10">
                        Drag fields from the palette or click to add
                      </Text>
                      <SchemaEditor
                        title="Input Fields"
                        fields={state.design.inputFields}
                        onChange={(fields) =>
                          dispatch({
                            type: 'UPDATE_INPUT_FIELDS',
                            payload: fields,
                          })
                        }
                      />
                    </YStack>
                  </Card>
                </div>
              </Tabs.Content>

              <Tabs.Content value="output">
                <div
                  onDrop={(e) => handleDrop(e, 'output')}
                  onDragOver={handleDragOver}
                  style={{ marginTop: 12 }}
                >
                  <Card
                    p="$4"
                    borderWidth={2}
                    borderColor="$borderColor"
                    borderStyle="dashed"
                  >
                    <YStack gap="$3">
                      <Text fontSize="$2" color="$gray10">
                        Drag fields from the palette or click to add
                      </Text>
                      <SchemaEditor
                        title="Output Fields"
                        fields={state.design.outputFields}
                        onChange={(fields) =>
                          dispatch({
                            type: 'UPDATE_OUTPUT_FIELDS',
                            payload: fields,
                          })
                        }
                      />
                    </YStack>
                  </Card>
                </div>
              </Tabs.Content>

              <Tabs.Content value="validation">
                <Card p="$4" mt="$3">
                  <ValidationRuleBuilder
                    rules={state.design.validationRules}
                    availableFields={state.design.inputFields.map((f) => ({
                      name: f.name,
                      type: f.type,
                    }))}
                    onChange={(rules) =>
                      dispatch({
                        type: 'UPDATE_VALIDATION_RULES',
                        payload: rules,
                      })
                    }
                  />
                </Card>
              </Tabs.Content>

              <Tabs.Content value="connections">
                <Card p="$4" mt="$3">
                  <ConnectionRuleEditor
                    rules={state.design.connectionRules}
                    availableComponents={[]}
                    onChange={(rules) =>
                      dispatch({
                        type: 'UPDATE_CONNECTION_RULES',
                        payload: rules,
                      })
                    }
                  />
                </Card>
              </Tabs.Content>
            </Tabs>
          </YStack>
        </ScrollView>
      </YStack>

      {/* Right Panel - Live Preview */}
      <YStack w={400} borderLeftWidth={1} borderColor="$borderColor">
        <XStack p="$3" borderBottomWidth={1} borderColor="$borderColor" gap="$2">
          <Button
            size="$2"
            theme={previewType === 'rust' ? 'blue' : 'gray'}
            onPress={() => setPreviewType('rust')}
          >
            Rust
          </Button>
          <Button
            size="$2"
            theme={previewType === 'typescript' ? 'blue' : 'gray'}
            onPress={() => setPreviewType('typescript')}
          >
            TypeScript
          </Button>
        </XStack>

        <ScrollView f={1} p="$3">
          <Card bg="$gray2" p="$3" br="$3">
            <Text fontFamily="$mono" fontSize="$2" whiteSpace="pre-wrap">
              {previewType === 'rust' ? rustPreview : typescriptPreview}
            </Text>
          </Card>
        </ScrollView>
      </YStack>
    </XStack>
  );
});

export default VisualBuilder;
