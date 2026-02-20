/**
 * Edit Workflow Page - Visual workflow editor
 */

'use client';

import { YStack, Text, Spinner } from 'tamagui';
import { AuthGuardWithLoading } from '@/components/shared/AuthGuard';
import { WorkflowCanvas } from '@/components/workflow/WorkflowCanvas';
import { api } from '@/lib/trpc/client';
import { useParams } from 'next/navigation';

function EditWorkflowContent() {
  const params = useParams();
  const workflowId = params.id as string;

  const { data: workflowData, isLoading, error } = api.workflows.get.useQuery({ 
    id: workflowId 
  });
  
  const workflow = workflowData?.workflow;

  if (isLoading) {
    return (
      <YStack flex={1} alignItems="center" justifyContent="center">
        <Spinner size="large" />
        <Text marginTop="$4">Loading workflow...</Text>
      </YStack>
    );
  }

  if (error || !workflow) {
    return (
      <YStack flex={1} alignItems="center" justifyContent="center">
        <Text color="$red10">
          Error: {error?.message || 'Workflow not found'}
        </Text>
      </YStack>
    );
  }

  return (
    <WorkflowCanvas
      workflowId={workflowId}
      initialDefinition={workflow.definition as any}
      readOnly={workflow.status?.name === 'active'}
    />
  );
}

export default function EditWorkflowPage() {
  return (
    <AuthGuardWithLoading>
      <EditWorkflowContent />
    </AuthGuardWithLoading>
  );
}

