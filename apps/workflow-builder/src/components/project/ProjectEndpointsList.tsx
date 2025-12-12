/**
 * Project Endpoints List
 * Displays all API endpoints for a project
 */

'use client';

import { YStack, XStack, Text, Card, Spinner } from 'tamagui';
import { api } from '@/lib/trpc/client';
import { Globe, ExternalLink } from 'lucide-react';

interface ProjectEndpointsListProps {
  projectId: string;
}

export function ProjectEndpointsList({ projectId }: ProjectEndpointsListProps) {
  // Fetch all workflows in the project
  const { data: workflowsData, isLoading: isLoadingWorkflows } = api.workflows.list.useQuery({
    projectId,
  });

  // Fetch public interfaces for all workflows
  const workflowIds = workflowsData?.workflows?.map(w => w.id) || [];
  const { data: publicInterfacesData, isLoading: isLoadingInterfaces } = api.publicInterfaces.list.useQuery(
    { projectId },
    { enabled: !!projectId && workflowIds.length > 0 }
  );

  const isLoading = isLoadingWorkflows || isLoadingInterfaces;

  if (isLoading) {
    return (
      <YStack padding="$4" alignItems="center" gap="$2">
        <Spinner size="small" />
        <Text fontSize="$2" color="$gray11">Loading endpoints...</Text>
      </YStack>
    );
  }

  const publicInterfaces = publicInterfacesData?.interfaces || [];
  const workflows = workflowsData?.workflows || [];

  // Group endpoints by workflow
  const endpointsByWorkflow = new Map<string, typeof publicInterfaces>();
  for (const endpoint of publicInterfaces) {
    const workflowId = (endpoint as any).service_interface?.workflow_id;
    if (workflowId) {
      if (!endpointsByWorkflow.has(workflowId)) {
        endpointsByWorkflow.set(workflowId, []);
      }
      endpointsByWorkflow.get(workflowId)!.push(endpoint);
    }
  }

  if (publicInterfaces.length === 0) {
    return (
      <YStack padding="$4" gap="$2">
        <Text fontSize="$3" color="$gray11">
          No endpoints available yet
        </Text>
        <Text fontSize="$2" color="$gray10">
          Endpoints will be available at publish time when you add interface components (data-in or data-out) to your services and mark them as public.
        </Text>
      </YStack>
    );
  }

  return (
    <YStack gap="$4">
      <YStack gap="$2">
        <Text fontSize="$4" fontWeight="600">
          API Endpoints ({publicInterfaces.length})
        </Text>
        <Text fontSize="$2" color="$gray11">
          Endpoints will be available at publish time. Each interface component creates one endpoint.
        </Text>
      </YStack>

      <YStack gap="$3">
        {Array.from(endpointsByWorkflow.entries()).map(([workflowId, endpoints]) => {
          const workflow = workflows.find(w => w.id === workflowId);
          const workflowName = workflow?.display_name || workflow?.name || 'Unknown Service';

          return (
            <Card key={workflowId} padding="$4" elevate>
              <YStack gap="$3">
                <XStack alignItems="center" gap="$2">
                  <Globe size={16} color="var(--blue9)" />
                  <Text fontSize="$4" fontWeight="600">
                    {workflowName}
                  </Text>
                </XStack>
                <YStack gap="$2">
                  {endpoints.map((endpoint: any) => {
                    const method = endpoint.http_method;
                    const path = endpoint.http_path;
                    const fullPath = endpoint.full_path || `/api/v1/...${path}`;
                    const methodColors: Record<string, string> = {
                      GET: '$blue9',
                      POST: '$green9',
                      PATCH: '$purple9',
                      PUT: '$orange9',
                      DELETE: '$red9',
                    };
                    const methodColor = methodColors[method] || '$gray9';

                    return (
                      <XStack
                        key={endpoint.id}
                        padding="$3"
                        backgroundColor="$gray2"
                        borderRadius="$3"
                        alignItems="center"
                        gap="$3"
                      >
                        <Text
                          fontSize="$2"
                          fontWeight="bold"
                          color={methodColor}
                          fontFamily="monospace"
                          minWidth={60}
                        >
                          {method}
                        </Text>
                        <Text
                          fontSize="$2"
                          color="$gray11"
                          fontFamily="monospace"
                          flex={1}
                        >
                          {fullPath}
                        </Text>
                        {endpoint.description && (
                          <Text fontSize="$2" color="$gray10" flex={1}>
                            {endpoint.description}
                          </Text>
                        )}
                      </XStack>
                    );
                  })}
                </YStack>
              </YStack>
            </Card>
          );
        })}
      </YStack>
    </YStack>
  );
}

