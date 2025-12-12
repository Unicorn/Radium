/**
 * Code Viewer Modal - Display generated TypeScript workflow code
 * Shows compiled workflow, activities, and worker code
 */

'use client';

import { useState } from 'react';
import { Dialog, XStack, Text, Button, ScrollView } from 'tamagui';
import { X, Copy, Check } from 'lucide-react';

interface CodeViewerModalProps {
  code: {
    workflowCode?: string;
    activitiesCode?: string;
    workerCode?: string;
  };
  open: boolean;
  onClose: () => void;
}

type CodeTab = 'workflow' | 'activities' | 'worker';

export function CodeViewerModal({ code, open, onClose }: CodeViewerModalProps) {
  const [activeTab, setActiveTab] = useState<CodeTab>('workflow');
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    const currentCode =
      activeTab === 'workflow'
        ? code.workflowCode
        : activeTab === 'activities'
        ? code.activitiesCode
        : code.workerCode;

    if (currentCode) {
      await navigator.clipboard.writeText(currentCode);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  const getCurrentCode = () => {
    switch (activeTab) {
      case 'workflow':
        return code.workflowCode || '// No workflow code generated';
      case 'activities':
        return code.activitiesCode || '// No activities code generated';
      case 'worker':
        return code.workerCode || '// No worker code generated';
      default:
        return '';
    }
  };

  return (
    <Dialog open={open} onOpenChange={(isOpen) => !isOpen && onClose()}>
      <Dialog.Portal>
        <Dialog.Overlay
          key="overlay"
          animation="quick"
          opacity={0.5}
          enterStyle={{ opacity: 0 }}
          exitStyle={{ opacity: 0 }}
        />
        <Dialog.Content
          bordered
          elevate
          key="content"
          animation={[
            'quick',
            {
              opacity: {
                overshootClamping: true,
              },
            },
          ]}
          enterStyle={{ x: 0, y: -20, opacity: 0, scale: 0.9 }}
          exitStyle={{ x: 0, y: 10, opacity: 0, scale: 0.95 }}
          gap="$4"
          width="90vw"
          maxWidth={1200}
          height="80vh"
          data-testid="code-viewer-modal"
        >
          {/* Header */}
          <Dialog.Title>
            <XStack justifyContent="space-between" alignItems="center">
              <Text fontSize="$7" fontWeight="bold">
                Generated TypeScript Code
              </Text>
              <Dialog.Close asChild>
                <Button
                  size="$3"
                  circular
                  icon={X}
                  variant="outlined"
                  onPress={onClose}
                />
              </Dialog.Close>
            </XStack>
          </Dialog.Title>

          <Dialog.Description>
            <Text fontSize="$4" color="$gray11">
              View and copy the compiled TypeScript code for your workflow
            </Text>
          </Dialog.Description>

          {/* Tab Navigation */}
          <XStack gap="$2" paddingBottom="$2" borderBottomWidth={1} borderColor="$borderColor">
            <Button
              size="$3"
              variant={activeTab === 'workflow' ? 'outlined' : 'ghost'}
              onPress={() => setActiveTab('workflow')}
              backgroundColor={activeTab === 'workflow' ? '$blue2' : 'transparent'}
              data-testid="code-tab-workflow"
            >
              Workflow
            </Button>
            <Button
              size="$3"
              variant={activeTab === 'activities' ? 'outlined' : 'ghost'}
              onPress={() => setActiveTab('activities')}
              backgroundColor={activeTab === 'activities' ? '$blue2' : 'transparent'}
              data-testid="code-tab-activities"
            >
              Activities
            </Button>
            <Button
              size="$3"
              variant={activeTab === 'worker' ? 'outlined' : 'ghost'}
              onPress={() => setActiveTab('worker')}
              backgroundColor={activeTab === 'worker' ? '$blue2' : 'transparent'}
              data-testid="code-tab-worker"
            >
              Worker
            </Button>
          </XStack>

          {/* Code Display */}
          <ScrollView flex={1} backgroundColor="$gray2" borderRadius="$4" padding="$4" data-testid="code-viewer-content">
            <pre
              style={{
                margin: 0,
                fontFamily: 'monospace',
                fontSize: 14,
                lineHeight: 1.5,
                whiteSpace: 'pre-wrap',
                wordBreak: 'break-word',
              }}
            >
              <code className="language-typescript" data-testid="code-content">{getCurrentCode()}</code>
            </pre>
          </ScrollView>

          {/* Actions */}
          <XStack gap="$3" justifyContent="flex-end">
            <Button
              size="$4"
              icon={copied ? Check : Copy}
              onPress={handleCopy}
              variant="outlined"
              theme={copied ? 'green' : undefined}
              data-testid="copy-code-button"
            >
              {copied ? 'Copied!' : 'Copy Code'}
            </Button>
            <Button size="$4" onPress={onClose} theme="blue" data-testid="close-code-viewer">
              Close
            </Button>
          </XStack>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog>
  );
}
