'use client';

import { Dialog, YStack, XStack, Text, Button, ScrollView, Tabs } from 'tamagui';
import { X, Download, Copy, Check } from 'lucide-react';
import { useState } from 'react';

interface CodePreviewDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  workflowCode: string;
  activitiesCode: string;
  workerCode: string;
  packageJson: string;
  tsConfig: string;
}

export function CodePreviewDialog({
  open,
  onOpenChange,
  workflowCode,
  activitiesCode,
  workerCode,
  packageJson,
  tsConfig,
}: CodePreviewDialogProps) {
  const [copiedTab, setCopiedTab] = useState<string | null>(null);

  const handleCopy = async (code: string, tabName: string) => {
    await navigator.clipboard.writeText(code);
    setCopiedTab(tabName);
    setTimeout(() => setCopiedTab(null), 2000);
  };

  const handleDownloadAll = () => {
    // Create a ZIP file with all the code (future enhancement)
    alert('Download all files feature coming soon!');
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay 
          key="overlay"
          animation="quick"
          opacity={0.5}
          enterStyle={{ opacity: 0 }}
          exitStyle={{ opacity: 0 }}
        />
        
        <Dialog.Content
          key="content"
          bordered
          elevate
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
          width="90vw"
          maxWidth={1200}
          height="90vh"
        >
          {/* Header */}
          <XStack ai="center" jc="space-between" p="$4" borderBottomWidth={1} borderBottomColor="$borderColor">
            <YStack gap="$1">
              <Dialog.Title fontSize="$6" fontWeight="600">
                Generated Code
              </Dialog.Title>
              <Dialog.Description fontSize="$3" color="$gray11">
                Preview and download your Temporal workflow code
              </Dialog.Description>
            </YStack>

            <XStack gap="$2">
              <Button
                icon={Download}
                size="$3"
                onPress={handleDownloadAll}
              >
                Download All
              </Button>
              <Dialog.Close asChild>
                <Button size="$3" icon={X} chromeless />
              </Dialog.Close>
            </XStack>
          </XStack>

          {/* Tabs */}
          <Tabs defaultValue="workflow" orientation="horizontal" f={1}>
            <Tabs.List paddingHorizontal="$4" paddingTop="$2" bg="$background">
              <Tabs.Tab value="workflow">
                <Text>workflow.ts</Text>
              </Tabs.Tab>
              <Tabs.Tab value="activities">
                <Text>activities.ts</Text>
              </Tabs.Tab>
              <Tabs.Tab value="worker">
                <Text>worker.ts</Text>
              </Tabs.Tab>
              <Tabs.Tab value="package">
                <Text>package.json</Text>
              </Tabs.Tab>
              <Tabs.Tab value="tsconfig">
                <Text>tsconfig.json</Text>
              </Tabs.Tab>
            </Tabs.List>

            {/* Workflow Tab */}
            <Tabs.Content value="workflow" f={1} p="$4">
              <YStack f={1} gap="$2">
                <XStack jc="space-between" ai="center">
                  <Text fontSize="$3" color="$gray11">src/workflows.ts</Text>
                  <Button
                    size="$2"
                    icon={copiedTab === 'workflow' ? Check : Copy}
                    onPress={() => handleCopy(workflowCode, 'workflow')}
                  >
                    {copiedTab === 'workflow' ? 'Copied!' : 'Copy'}
                  </Button>
                </XStack>
                <ScrollView f={1} bg="$gray2" borderRadius="$4" p="$3">
                  <Text fontFamily="$mono" fontSize="$2">
                    {workflowCode}
                  </Text>
                </ScrollView>
              </YStack>
            </Tabs.Content>

            {/* Activities Tab */}
            <Tabs.Content value="activities" f={1} p="$4">
              <YStack f={1} gap="$2">
                <XStack jc="space-between" ai="center">
                  <Text fontSize="$3" color="$gray11">src/activities.ts</Text>
                  <Button
                    size="$2"
                    icon={copiedTab === 'activities' ? Check : Copy}
                    onPress={() => handleCopy(activitiesCode, 'activities')}
                  >
                    {copiedTab === 'activities' ? 'Copied!' : 'Copy'}
                  </Button>
                </XStack>
                <ScrollView f={1} bg="$gray2" borderRadius="$4" p="$3">
                  <Text fontFamily="$mono" fontSize="$2">
                    {activitiesCode}
                  </Text>
                </ScrollView>
              </YStack>
            </Tabs.Content>

            {/* Worker Tab */}
            <Tabs.Content value="worker" f={1} p="$4">
              <YStack f={1} gap="$2">
                <XStack jc="space-between" ai="center">
                  <Text fontSize="$3" color="$gray11">src/worker.ts</Text>
                  <Button
                    size="$2"
                    icon={copiedTab === 'worker' ? Check : Copy}
                    onPress={() => handleCopy(workerCode, 'worker')}
                  >
                    {copiedTab === 'worker' ? 'Copied!' : 'Copy'}
                  </Button>
                </XStack>
                <ScrollView f={1} bg="$gray2" borderRadius="$4" p="$3">
                  <Text fontFamily="$mono" fontSize="$2">
                    {workerCode}
                  </Text>
                </ScrollView>
              </YStack>
            </Tabs.Content>

            {/* Package.json Tab */}
            <Tabs.Content value="package" f={1} p="$4">
              <YStack f={1} gap="$2">
                <XStack jc="space-between" ai="center">
                  <Text fontSize="$3" color="$gray11">package.json</Text>
                  <Button
                    size="$2"
                    icon={copiedTab === 'package' ? Check : Copy}
                    onPress={() => handleCopy(packageJson, 'package')}
                  >
                    {copiedTab === 'package' ? 'Copied!' : 'Copy'}
                  </Button>
                </XStack>
                <ScrollView f={1} bg="$gray2" borderRadius="$4" p="$3">
                  <Text fontFamily="$mono" fontSize="$2">
                    {packageJson}
                  </Text>
                </ScrollView>
              </YStack>
            </Tabs.Content>

            {/* TSConfig Tab */}
            <Tabs.Content value="tsconfig" f={1} p="$4">
              <YStack f={1} gap="$2">
                <XStack jc="space-between" ai="center">
                  <Text fontSize="$3" color="$gray11">tsconfig.json</Text>
                  <Button
                    size="$2"
                    icon={copiedTab === 'tsconfig' ? Check : Copy}
                    onPress={() => handleCopy(tsConfig, 'tsconfig')}
                  >
                    {copiedTab === 'tsconfig' ? 'Copied!' : 'Copy'}
                  </Button>
                </XStack>
                <ScrollView f={1} bg="$gray2" borderRadius="$4" p="$3">
                  <Text fontFamily="$mono" fontSize="$2">
                    {tsConfig}
                  </Text>
                </ScrollView>
              </YStack>
            </Tabs.Content>
          </Tabs>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog>
  );
}

