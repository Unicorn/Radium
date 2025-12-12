/**
 * Workflows List Page
 */

'use client';

import { YStack, XStack, H1, Button, Card, Text, Spinner } from 'tamagui';
import { Badge } from '@/components/shared/Badge';
import { useRouter } from 'next/navigation';
import { AuthGuardWithLoading } from '@/components/shared/AuthGuard';
import { Header } from '@/components/shared/Header';
import { Sidebar } from '@/components/shared/Sidebar';
import { api } from '@/lib/trpc/client';
import { formatDistanceToNow } from 'date-fns';

function WorkflowsContent() {
  const router = useRouter();
  const { data, isLoading } = api.workflows.list.useQuery({});

  return (
    <YStack flex={1}>
      <Header />
      <XStack flex={1}>
        <Sidebar />
        <YStack flex={1} padding="$6" gap="$4" role="main" id="main-content">
            <XStack justifyContent="space-between" alignItems="center">
              <H1>Services</H1>
              <Button
                theme="blue"
                onPress={() => router.push('/workflows/new')}
              >
                New Service
              </Button>
            </XStack>

            {isLoading ? (
              <YStack flex={1} alignItems="center" justifyContent="center">
                <Spinner size="large" />
                <Text marginTop="$4">Loading services...</Text>
              </YStack>
            ) : (
              <YStack gap="$3">
                {data?.workflows.length === 0 ? (
                  <YStack padding="$6" alignItems="center">
                    <Text color="$gray11">No services yet</Text>
                    <Button
                      marginTop="$4"
                      onPress={() => router.push('/workflows/new')}
                    >
                      Create Your First Service
                    </Button>
                  </YStack>
                ) : (
                  data?.workflows.map((workflow) => (
                    <Card
                      key={workflow.id}
                      padding="$4"
                      pressStyle={{ scale: 0.98 }}
                      hoverStyle={{ backgroundColor: '$gray3' }}
                      cursor="pointer"
                      onPress={() => router.push(`/workflows/${workflow.id}`)}
                    >
                      <YStack gap="$3">
                        <XStack justifyContent="space-between" alignItems="center">
                          <Text fontSize="$6" fontWeight="600">
                            {workflow.display_name}
                          </Text>
                          <Badge 
                            backgroundColor={workflow.status.color || '$gray10'}
                            size="2"
                          >
                            <Text fontSize="$2" color="white">
                              {workflow.status.name}
                            </Text>
                          </Badge>
                        </XStack>

                        {workflow.description && (
                          <Text fontSize="$3" color="$gray11">
                            {workflow.description}
                          </Text>
                        )}

                        <XStack justifyContent="space-between" alignItems="center">
                          <XStack gap="$3" alignItems="center">
                            <Text fontSize="$2" color="$gray10">
                              Channel: {workflow.task_queue.name}
                            </Text>
                            <Text fontSize="$2" color="$gray10">
                              v{workflow.version}
                            </Text>
                          </XStack>
                          <Text fontSize="$2" color="$gray10">
                            Updated {formatDistanceToNow(new Date(workflow.updated_at), { addSuffix: true })}
                          </Text>
                        </XStack>
                      </YStack>
                    </Card>
                  ))
                )}
            </YStack>
          )}
        </YStack>
      </XStack>
    </YStack>
  );
}

export default function WorkflowsPage() {
  return (
    <AuthGuardWithLoading>
      <WorkflowsContent />
    </AuthGuardWithLoading>
  );
}

