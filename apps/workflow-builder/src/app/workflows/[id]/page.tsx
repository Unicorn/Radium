/**
 * Workflow Detail Page
 */

'use client';

import { YStack, XStack, H1, Text, Button, Card, Separator, Spinner, Tabs } from 'tamagui';
import { Badge } from '@/components/shared/Badge';
import { useRouter, useParams } from 'next/navigation';
import { AuthGuardWithLoading } from '@/components/shared/AuthGuard';
import { Header } from '@/components/shared/Header';
import { Sidebar } from '@/components/shared/Sidebar';
import { api } from '@/lib/trpc/client';
import { formatDistanceToNow } from 'date-fns';
import { Edit, Play, Pause, Trash, History, BarChart3, Network } from 'lucide-react';
import { ExecutionHistoryList } from '@/components/execution/ExecutionHistoryList';
import { ExecutionDetailView } from '@/components/execution/ExecutionDetailView';
import { WorkflowStatisticsPanel } from '@/components/execution/WorkflowStatisticsPanel';
import { ServiceInterfaces } from '@/components/service/ServiceInterfaces';
import { useState } from 'react';

function WorkflowDetailContent() {
  const router = useRouter();
  const params = useParams();
  const workflowId = params.id as string;
  const utils = api.useUtils();
  const [selectedExecutionId, setSelectedExecutionId] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState('overview');

  const { data: workflowData, isLoading, error } = api.workflows.get.useQuery({ 
    id: workflowId 
  });
  
  const workflow = workflowData?.workflow;

  const deployMutation = api.workflows.deploy.useMutation({
    onSuccess: (data) => {
      utils.workflows.get.invalidate({ id: workflowId });
      utils.workflows.list.invalidate();
      
      // Show endpoint URLs if any were registered
      if (data?.endpoints && data.endpoints.length > 0) {
        const endpointList = data.endpoints
          .map((ep: any) => `${ep.method} ${ep.url}`)
          .join('\n');
        alert(`✅ Workflow deployed!\n\nAPI Endpoints:\n${endpointList}`);
      }
    },
  });

  const pauseMutation = api.workflows.pause.useMutation({
    onSuccess: () => {
      utils.workflows.get.invalidate({ id: workflowId });
      utils.workflows.list.invalidate();
    },
  });

  const archiveMutation = api.workflows.archive.useMutation({
    onSuccess: () => {
      router.push('/workflows');
    },
  });

  if (isLoading) {
    return (
      <YStack flex={1}>
        <Header />
        <XStack flex={1}>
          <Sidebar />
          <YStack flex={1} alignItems="center" justifyContent="center">
            <Spinner size="large" />
            <Text marginTop="$4">Loading service...</Text>
          </YStack>
        </XStack>
      </YStack>
    );
  }

  if (error || !workflow) {
    return (
      <YStack flex={1}>
        <Header />
        <XStack flex={1}>
          <Sidebar />
          <YStack flex={1} alignItems="center" justifyContent="center">
            <Text color="$red10">
              {error?.message || 'Service not found'}
            </Text>
          </YStack>
        </XStack>
      </YStack>
    );
  }

  const nodeCount = (workflow.definition as any)?.nodes?.length || 0;
  const edgeCount = (workflow.definition as any)?.edges?.length || 0;

  return (
    <YStack flex={1}>
      <Header />
      <XStack flex={1}>
        <Sidebar />
        <YStack flex={1} padding="$6" gap="$4">
          {/* Header */}
          <YStack gap="$2">
            <XStack justifyContent="space-between" alignItems="flex-start">
              <YStack gap="$2">
                <H1>{workflow.display_name}</H1>
                <Text color="$gray11">{workflow.description}</Text>
              </YStack>
              <Badge 
                backgroundColor={workflow.status?.color || '$gray10'}
                size="3"
              >
                <Text fontSize="$3" color="white">
                  {workflow.status?.name || 'Unknown'}
                </Text>
              </Badge>
            </XStack>
          </YStack>

          {/* Actions */}
          <XStack gap="$2" flexWrap="wrap">
            <Button
              size="$3"
              icon={Edit}
              onPress={() => router.push(`/workflows/${workflowId}/edit`)}
            >
              Edit
            </Button>
            
            {workflow.status?.name === 'draft' && (
              <Button
                size="$3"
                theme="blue"
                icon={Play}
                onPress={() => deployMutation.mutate({ id: workflowId })}
                disabled={deployMutation.isLoading || nodeCount === 0}
              >
                {deployMutation.isLoading ? 'Deploying...' : 'Deploy'}
              </Button>
            )}

            {workflow.status?.name === 'active' && (
              <Button
                size="$3"
                theme="orange"
                icon={Pause}
                onPress={() => pauseMutation.mutate({ id: workflowId })}
                disabled={pauseMutation.isLoading}
              >
                {pauseMutation.isLoading ? 'Pausing...' : 'Pause'}
              </Button>
            )}

            <Button
              size="$3"
              theme="orange"
              icon={Trash}
              onPress={() => {
                if (confirm('Are you sure you want to archive this service? You can unarchive it later.')) {
                  archiveMutation.mutate({ id: workflowId });
                }
              }}
              disabled={archiveMutation.isLoading}
            >
              {archiveMutation.isLoading ? 'Archiving...' : 'Archive'}
            </Button>
          </XStack>

          {/* Tabs */}
          <Tabs value={activeTab} onValueChange={setActiveTab} flex={1}>
            <Tabs.List>
              <Tabs.Tab value="overview">
                <Text>Overview</Text>
              </Tabs.Tab>
              <Tabs.Tab value="history">
                <XStack gap="$2" ai="center">
                  <History size={16} />
                  <Text>Execution History</Text>
                </XStack>
              </Tabs.Tab>
              <Tabs.Tab value="statistics">
                <XStack gap="$2" ai="center">
                  <BarChart3 size={16} />
                  <Text>Statistics</Text>
                </XStack>
              </Tabs.Tab>
              <Tabs.Tab value="interfaces">
                <XStack gap="$2" ai="center">
                  <Network size={16} />
                  <Text>Interfaces</Text>
                </XStack>
              </Tabs.Tab>
            </Tabs.List>

            <Tabs.Content value="overview">
              {/* Metadata */}
              <Card padding="$4" elevate>
                <YStack gap="$3">
                  <Text fontSize="$5" fontWeight="600">Service Details</Text>
                  <Separator />

                  <XStack justifyContent="space-between">
                    <Text color="$gray11">ID</Text>
                    <Text fontFamily="monospace" fontSize="$2">{workflow.id}</Text>
                  </XStack>

                  <XStack justifyContent="space-between">
                    <Text color="$gray11">Identifier</Text>
                    <Text fontFamily="monospace">{workflow.kebab_name}</Text>
                  </XStack>

                  <XStack justifyContent="space-between">
                    <Text color="$gray11">Version</Text>
                    <Text>{workflow.version}</Text>
                  </XStack>

                  <XStack justifyContent="space-between">
                    <Text color="$gray11">Channel</Text>
                    <Text>{workflow.task_queue?.name || 'Not set'}</Text>
                  </XStack>

                  <XStack justifyContent="space-between">
                    <Text color="$gray11">Nodes</Text>
                    <Text>{nodeCount}</Text>
                  </XStack>

                  <XStack justifyContent="space-between">
                    <Text color="$gray11">Connections</Text>
                    <Text>{edgeCount}</Text>
                  </XStack>

                  {workflow.deployed_at && (
                    <XStack justifyContent="space-between">
                      <Text color="$gray11">Deployed</Text>
                      <Text>
                        {formatDistanceToNow(new Date(workflow.deployed_at), { addSuffix: true })}
                      </Text>
                    </XStack>
                  )}

                  <XStack justifyContent="space-between">
                    <Text color="$gray11">Created</Text>
                    <Text>
                      {workflow.created_at 
                        ? formatDistanceToNow(new Date(workflow.created_at), { addSuffix: true })
                        : 'Unknown'}
                    </Text>
                  </XStack>

                  <XStack justifyContent="space-between">
                    <Text color="$gray11">Updated</Text>
                    <Text>
                      {workflow.updated_at 
                        ? formatDistanceToNow(new Date(workflow.updated_at), { addSuffix: true })
                        : 'Unknown'}
                    </Text>
                  </XStack>
                </YStack>
              </Card>
            </Tabs.Content>

            <Tabs.Content value="history">
              <YStack gap="$3" flex={1}>
                {selectedExecutionId ? (
                  <>
                    <Button
                      size="$3"
                      onPress={() => setSelectedExecutionId(null)}
                      variant="outlined"
                    >
                      ← Back to History
                    </Button>
                    <ExecutionDetailView
                      executionId={selectedExecutionId}
                      workflowId={workflowId}
                    />
                  </>
                ) : (
                  <ExecutionHistoryList
                    workflowId={workflowId}
                    onSelectExecution={setSelectedExecutionId}
                  />
                )}
              </YStack>
            </Tabs.Content>

            <Tabs.Content value="statistics">
              <WorkflowStatisticsPanel workflowId={workflowId} />
            </Tabs.Content>

            <Tabs.Content value="interfaces">
              <ServiceInterfaces serviceId={workflowId} />
            </Tabs.Content>
          </Tabs>
        </YStack>
      </XStack>
    </YStack>
  );
}

export default function WorkflowDetailPage() {
  return (
    <AuthGuardWithLoading>
      <WorkflowDetailContent />
    </AuthGuardWithLoading>
  );
}

