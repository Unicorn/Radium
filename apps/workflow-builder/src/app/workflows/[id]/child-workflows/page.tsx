'use client';

import { useParams } from 'next/navigation';
import { api as trpc } from '@/lib/trpc/client';
import { YStack, XStack, Heading, Text, Spinner, Button } from 'tamagui';
import { Plus, ArrowLeft } from 'lucide-react';
import { useRouter } from 'next/navigation';
import { ChildWorkflowCard } from '@/components/child-workflow';
import type { EnhancedWorkflowNode } from '@/types/advanced-patterns';

export default function ChildWorkflowsPage() {
  const params = useParams();
  const router = useRouter();
  const workflowId = params.id as string;

  // Fetch workflow details
  const { data: workflowData, isLoading } = trpc.workflows.get.useQuery({
    id: workflowId,
  });

  // TODO: Fetch child workflow nodes from workflow definition
  // For now, we'll mock some data based on the workflow
  const childWorkflows: EnhancedWorkflowNode[] = [];

  if (isLoading) {
    return (
      <YStack f={1} ai="center" jc="center" gap="$4">
        <Spinner size="large" />
        <Text>Loading child workflows...</Text>
      </YStack>
    );
  }

  if (!workflowData?.workflow) {
    return (
      <YStack f={1} ai="center" jc="center" gap="$4">
        <Text fontSize="$6" color="$red10">Workflow not found</Text>
      </YStack>
    );
  }

  return (
    <YStack f={1} gap="$4" p="$4">
      {/* Header */}
      <XStack ai="center" jc="space-between">
        <XStack ai="center" gap="$3">
          <Button
            icon={ArrowLeft}
            size="$3"
            chromeless
            onPress={() => router.push(`/workflows/${workflowId}/builder`)}
          >
            Back to Builder
          </Button>
          <YStack gap="$1">
            <Heading size="$7">Child Workflows</Heading>
            <Text fontSize="$3" color="$gray11">
              {workflowData.workflow.display_name}
            </Text>
          </YStack>
        </XStack>

        <Button
          icon={Plus}
          size="$3"
          themeInverse
        >
          Add Child Workflow
        </Button>
      </XStack>

      {/* Child Workflows List */}
      {childWorkflows.length === 0 ? (
        <YStack
          f={1}
          ai="center"
          jc="center"
          gap="$4"
          p="$6"
          bg="$gray2"
          borderRadius="$4"
          borderWidth={1}
          borderColor="$borderColor"
        >
          <YStack w={80} h={80} bg="$blue5" borderRadius="$4" ai="center" jc="center">
            <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M12 2v6M12 18v4M6 8l-4 4 4 4M18 8l4 4-4 4" />
            </svg>
          </YStack>
          <YStack ai="center" gap="$2">
            <Text fontSize="$6" fontWeight="600">
              No Child Workflows Yet
            </Text>
            <Text fontSize="$3" color="$gray11" textAlign="center" maxWidth={400}>
              Child workflows allow you to break down complex processes into reusable,
              independently executing workflows that can communicate with the parent.
            </Text>
          </YStack>
          <Button icon={Plus} themeInverse>
            Add Your First Child Workflow
          </Button>
        </YStack>
      ) : (
        <YStack gap="$4">
          {childWorkflows.map((childWorkflow) => (
            <ChildWorkflowCard
              key={childWorkflow.id}
              childWorkflow={childWorkflow}
              onUpdate={() => {
                // TODO: Refetch child workflows
              }}
              onEdit={() => {
                // TODO: Open edit dialog
              }}
            />
          ))}
        </YStack>
      )}

      {/* Info Card */}
      <YStack
        p="$4"
        bg="$blue2"
        borderRadius="$4"
        borderWidth={1}
        borderColor="$blue6"
        gap="$2"
      >
        <Text fontSize="$4" fontWeight="600" color="$blue11">
          💡 About Child Workflows
        </Text>
        <Text fontSize="$3" color="$blue11">
          Child workflows can:
        </Text>
        <YStack gap="$1" ml="$3">
          <Text fontSize="$3" color="$blue11">
            • 📤 Signal back to parent to add work to queues
          </Text>
          <Text fontSize="$3" color="$blue11">
            • 🔍 Query parent for configuration or status
          </Text>
          <Text fontSize="$3" color="$blue11">
            • 🚫 Wait for dependencies before starting (blockUntil)
          </Text>
          <Text fontSize="$3" color="$blue11">
            • 🔄 Execute independently with their own lifecycle
          </Text>
        </YStack>
      </YStack>
    </YStack>
  );
}

