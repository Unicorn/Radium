/**
 * TypeScript Code Editor
 * Monaco-based code editor for writing custom activity code
 */

'use client';

import { YStack, XStack, Text, Button, Spinner } from 'tamagui';
import { useEffect, useRef, useState } from 'react';
import { CheckCircle, XCircle, AlertTriangle } from 'lucide-react';
import { api } from '@/lib/trpc/client';

interface TypeScriptEditorProps {
  value: string;
  onChange: (value: string) => void;
  height?: number;
  readOnly?: boolean;
  onValidationChange?: (isValid: boolean) => void;
}

export function TypeScriptEditor({
  value,
  onChange,
  height = 400,
  readOnly = false,
  onValidationChange,
}: TypeScriptEditorProps) {
  const editorRef = useRef<HTMLTextAreaElement>(null);
  const [localValue, setLocalValue] = useState(value);
  const [isValidating, setIsValidating] = useState(false);
  
  // Validate mutation
  const validateMutation = api.components.validateTypeScript.useMutation();

  // Validation state
  const validationResult = validateMutation.data;

  useEffect(() => {
    setLocalValue(value);
  }, [value]);

  // Notify parent of validation state
  useEffect(() => {
    if (validationResult && onValidationChange) {
      onValidationChange(validationResult.valid);
    }
  }, [validationResult, onValidationChange]);

  const handleChange = (newValue: string) => {
    setLocalValue(newValue);
    onChange(newValue);
  };

  const handleValidate = async () => {
    if (!localValue.trim()) {
      return;
    }
    
    setIsValidating(true);
    try {
      await validateMutation.mutateAsync({ code: localValue });
    } finally {
      setIsValidating(false);
    }
  };

  // Auto-validate on blur
  const handleBlur = () => {
    if (localValue.trim()) {
      handleValidate();
    }
  };

  return (
    <YStack gap="$2" width="100%">
      {/* Editor */}
      <YStack
        borderWidth={1}
        borderColor="$borderColor"
        borderRadius="$4"
        overflow="hidden"
        backgroundColor="$background"
      >
        <textarea
          ref={editorRef}
          value={localValue}
          onChange={(e) => handleChange(e.target.value)}
          onBlur={handleBlur}
          readOnly={readOnly}
          style={{
            width: '100%',
            height: `${height}px`,
            padding: '12px',
            fontFamily: 'Monaco, Menlo, "Ubuntu Mono", monospace',
            fontSize: '14px',
            lineHeight: '1.5',
            border: 'none',
            outline: 'none',
            resize: 'vertical',
            backgroundColor: 'transparent',
          }}
          placeholder="// Write your TypeScript activity code here
export async function myActivity(input: { message: string }) {
  // Your implementation
  return { result: 'success' };
}"
        />
      </YStack>

      {/* Validation Controls */}
      <XStack gap="$2" alignItems="center">
        <Button
          size="$3"
          onPress={handleValidate}
          disabled={isValidating || !localValue.trim() || readOnly}
          theme="blue"
        >
          {isValidating ? 'Validating...' : 'Validate TypeScript'}
        </Button>
        
        {isValidating && <Spinner size="small" />}
        
        {validationResult && (
          <XStack gap="$2" alignItems="center" flex={1}>
            {validationResult.valid ? (
              <>
                <CheckCircle size={16} color="green" />
                <Text fontSize="$2" color="$green11">
                  Valid TypeScript
                </Text>
              </>
            ) : (
              <>
                <XCircle size={16} color="red" />
                <Text fontSize="$2" color="$red11">
                  {validationResult.errors.length} error(s) found
                </Text>
              </>
            )}
            
            {validationResult.warnings.length > 0 && (
              <>
                <AlertTriangle size={16} color="orange" />
                <Text fontSize="$2" color="$orange11">
                  {validationResult.warnings.length} warning(s)
                </Text>
              </>
            )}
          </XStack>
        )}
      </XStack>

      {/* Validation Errors */}
      {validationResult && validationResult.errors.length > 0 && (
        <YStack gap="$2" padding="$3" backgroundColor="$red3" borderRadius="$4">
          <Text fontWeight="600" color="$red11">
            Errors:
          </Text>
          {validationResult.errors.map((error, idx) => (
            <Text key={idx} fontSize="$2" color="$red11" fontFamily="$mono">
              Line {error.line}:{error.column} - {error.message}
            </Text>
          ))}
        </YStack>
      )}

      {/* Validation Warnings */}
      {validationResult && validationResult.warnings.length > 0 && (
        <YStack gap="$2" padding="$3" backgroundColor="$orange3" borderRadius="$4">
          <Text fontWeight="600" color="$orange11">
            Warnings:
          </Text>
          {validationResult.warnings.map((warning, idx) => (
            <Text key={idx} fontSize="$2" color="$orange11" fontFamily="$mono">
              Line {warning.line}:{warning.column} - {warning.message}
            </Text>
          ))}
        </YStack>
      )}
    </YStack>
  );
}

