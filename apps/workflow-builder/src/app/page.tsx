/**
 * Dashboard Page
 */

'use client';

import { YStack, XStack, Text, Card, H1, H2 } from 'tamagui';
import { AuthGuardWithLoading } from '@/components/shared/AuthGuard';
import { Header } from '@/components/shared/Header';
import { Sidebar } from '@/components/shared/Sidebar';
import { api } from '@/lib/trpc/client';

function DashboardContent() {
  console.log('🔍 [Dashboard] Initiating queries for user, workflows, components, agents');
  
  const { data: user, isLoading: userLoading, error: userError } = api.users.me.useQuery();
  const { data: workflows, isLoading: workflowsLoading, error: workflowsError } = api.workflows.list.useQuery({});
  const { data: components, isLoading: componentsLoading, error: componentsError } = api.components.list.useQuery({});
  const { data: agents, isLoading: agentsLoading, error: agentsError } = api.agentPrompts.list.useQuery({});
  
  console.log('📊 [Dashboard] Query states:', {
    user: { loading: userLoading, error: userError?.message, hasData: !!user },
    workflows: { loading: workflowsLoading, error: workflowsError?.message, count: workflows?.workflows?.length },
    components: { loading: componentsLoading, error: componentsError?.message, count: components?.components?.length },
    agents: { loading: agentsLoading, error: agentsError?.message, count: agents?.prompts?.length },
  });

  const activeWorkflowsCount = workflows?.workflows.filter(
    w => w.status.name === 'active'
  ).length || 0;

  return (
    <YStack flex={1}>
      <Header />
      <XStack flex={1}>
        <Sidebar />
        <YStack flex={1} padding="$6" gap="$6">
          <YStack gap="$2">
            <H1>Welcome, {user?.display_name || 'User'}!</H1>
            <Text color="$gray11">
              Start building services by composing reusable components
            </Text>
          </YStack>

          <XStack gap="$4" flexWrap="wrap">
            <Card flex={1} minWidth={280} padding="$4" elevate>
              <YStack gap="$2">
                <H2 fontSize="$6">Services</H2>
                <Text fontSize="$8" fontWeight="bold" color="$blue10">
                  {activeWorkflowsCount}
                </Text>
                <Text color="$gray11">
                  Active services ({workflows?.total || 0} total)
                </Text>
              </YStack>
            </Card>

            <Card flex={1} minWidth={280} padding="$4" elevate>
              <YStack gap="$2">
                <H2 fontSize="$6">Components</H2>
                <Text fontSize="$8" fontWeight="bold" color="$green10">
                  {components?.total || 0}
                </Text>
                <Text color="$gray11">Available components</Text>
              </YStack>
            </Card>

            <Card flex={1} minWidth={280} padding="$4" elevate>
              <YStack gap="$2">
                <H2 fontSize="$6">Agents</H2>
                <Text fontSize="$8" fontWeight="bold" color="$purple10">
                  {agents?.total || 0}
                </Text>
                <Text color="$gray11">Agent prompts</Text>
              </YStack>
            </Card>
          </XStack>

          <Card padding="$4" elevate>
            <YStack gap="$3">
              <H2 fontSize="$6">Getting Started</H2>
              <YStack gap="$2">
                <Text>• Create your first component from the Components page</Text>
                <Text>• Design a service using the visual builder</Text>
                <Text>• Deploy and monitor your service executions</Text>
              </YStack>
            </YStack>
          </Card>
        </YStack>
      </XStack>
    </YStack>
  );
}

export default function DashboardPage() {
  return (
    <AuthGuardWithLoading>
      <DashboardContent />
    </AuthGuardWithLoading>
  );
}

