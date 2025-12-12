'use client';

import { useState, useEffect } from 'react';
import { YStack, XStack, Text, Button, Input, Label, Select, Adapt, Sheet } from 'tamagui';
import { Badge } from '../shared/Badge';
import { Check, ChevronDown, Info } from 'lucide-react';
import { validateCronExpression, CRON_PRESETS, getHumanReadableDescription } from '@/utils/cron-validation';
import type { CronValidationResult } from '@/utils/cron-validation';

interface CronExpressionBuilderProps {
  value?: string;
  onChange?: (expression: string) => void;
  onValidationChange?: (isValid: boolean) => void;
  allowSeconds?: boolean;
}

export function CronExpressionBuilder({
  value = '',
  onChange,
  onValidationChange,
  allowSeconds = false,
}: CronExpressionBuilderProps) {
  const [expression, setExpression] = useState(value);
  const [validation, setValidation] = useState<CronValidationResult | null>(null);
  const [showPresets, setShowPresets] = useState(true);

  // Validate expression whenever it changes
  useEffect(() => {
    if (expression) {
      const result = validateCronExpression(expression, { allowSeconds });
      setValidation(result);
      onValidationChange?.(result.valid);
    } else {
      setValidation(null);
      onValidationChange?.(false);
    }
  }, [expression, allowSeconds, onValidationChange]);

  const handleExpressionChange = (newExpression: string) => {
    setExpression(newExpression);
    onChange?.(newExpression);
  };

  const handlePresetSelect = (preset: typeof CRON_PRESETS[0]) => {
    handleExpressionChange(preset.expression);
    setShowPresets(false);
  };

  return (
    <YStack gap="$3">
      {/* Preset Buttons */}
      {showPresets && (
        <YStack gap="$2">
          <XStack ai="center" jc="space-between">
            <Text fontSize="$3" fontWeight="600">Quick Presets</Text>
            <Button
              size="$2"
              chromeless
              onPress={() => setShowPresets(false)}
            >
              Hide
            </Button>
          </XStack>
          <XStack gap="$2" flexWrap="wrap">
            {CRON_PRESETS.slice(0, 6).map((preset) => (
              <Button
                key={preset.name}
                size="$2"
                onPress={() => handlePresetSelect(preset)}
                bg="$blue3"
                borderColor="$blue6"
                hoverStyle={{ bg: '$blue4' }}
              >
                {preset.name}
              </Button>
            ))}
          </XStack>
        </YStack>
      )}

      {/* Manual Expression Input */}
      <YStack gap="$2">
        <XStack ai="center" jc="space-between">
          <Label htmlFor="cronExpression">
            Cron Expression <Text color="$red10">*</Text>
          </Label>
          {!showPresets && (
            <Button
              size="$2"
              chromeless
              onPress={() => setShowPresets(true)}
            >
              Show Presets
            </Button>
          )}
        </XStack>
        <Input
          id="cronExpression"
          value={expression}
          onChangeText={handleExpressionChange}
          placeholder={allowSeconds ? "* * * * * *" : "* * * * *"}
          fontFamily="$mono"
        />
        <Text fontSize="$2" color="$gray11">
          Format: {allowSeconds ? 'second minute hour day month weekday' : 'minute hour day month weekday'}
        </Text>
      </YStack>

      {/* Validation Result */}
      {validation && (
        <YStack gap="$3" p="$3" bg={validation.valid ? "$green2" : "$red2"} borderRadius="$4" borderWidth={1} borderColor={validation.valid ? "$green6" : "$red6"}>
          {validation.valid ? (
            <>
              <XStack ai="center" gap="$2">
                <Text fontSize="$4" fontWeight="600" color="$green11">
                  ✓ Valid Cron Expression
                </Text>
                {validation.isHighFrequency && (
                  <Badge bg="$orange5">
                    High Frequency
                  </Badge>
                )}
              </XStack>

              {validation.humanReadable && (
                <Text fontSize="$3" color="$green11">
                  <Text fontWeight="600">Schedule:</Text> {validation.humanReadable}
                </Text>
              )}

              {validation.nextRuns && validation.nextRuns.length > 0 && (
                <YStack gap="$2">
                  <Text fontSize="$3" fontWeight="600" color="$green11">
                    Next {Math.min(3, validation.nextRuns.length)} Runs:
                  </Text>
                  {validation.nextRuns.slice(0, 3).map((date, i) => (
                    <Text key={i} fontSize="$2" color="$green11" fontFamily="$mono">
                      {i + 1}. {date.toLocaleString()}
                    </Text>
                  ))}
                </YStack>
              )}

              {validation.isHighFrequency && (
                <XStack ai="center" gap="$2" p="$2" bg="$orange3" borderRadius="$3">
                  <Info size={16} color="$orange11" />
                  <Text fontSize="$2" color="$orange11" flex={1}>
                    This schedule runs very frequently. Consider if this is intentional.
                  </Text>
                </XStack>
              )}
            </>
          ) : (
            <>
              <Text fontSize="$4" fontWeight="600" color="$red11">
                ✗ Invalid Expression
              </Text>
              <Text fontSize="$3" color="$red11">
                {validation.error}
              </Text>
            </>
          )}
        </YStack>
      )}

      {/* Help Text */}
      <YStack gap="$2" p="$3" bg="$gray2" borderRadius="$4" borderWidth={1} borderColor="$gray6">
        <Text fontSize="$3" fontWeight="600" color="$gray12">
          Cron Expression Examples
        </Text>
        <YStack gap="$1">
          <Text fontSize="$2" color="$gray11" fontFamily="$mono">
            */5 * * * * → Every 5 minutes
          </Text>
          <Text fontSize="$2" color="$gray11" fontFamily="$mono">
            0 */2 * * * → Every 2 hours
          </Text>
          <Text fontSize="$2" color="$gray11" fontFamily="$mono">
            0 9 * * 1-5 → 9 AM on weekdays
          </Text>
          <Text fontSize="$2" color="$gray11" fontFamily="$mono">
            0 0 1 * * → First day of every month
          </Text>
        </YStack>
        <Text fontSize="$2" color="$blue10" textDecorationLine="underline" onPress={() => {
          // Open cron docs
          if (typeof window !== 'undefined') {
            window.open('https://crontab.guru/', '_blank');
          }
        }}>
          Need help? Try crontab.guru
        </Text>
      </YStack>
    </YStack>
  );
}

