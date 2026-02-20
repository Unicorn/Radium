'use client';

import { YStack, XStack, H1, Button, Input, TextArea, Card, Text } from 'tamagui';
import { useState } from 'react';
import { useRouter } from 'next/navigation';
import { AuthGuardWithLoading } from '@/components/shared/AuthGuard';
import { Header } from '@/components/shared/Header';
import { Sidebar } from '@/components/shared/Sidebar';
import { api } from '@/lib/trpc/client';

// Helper function to convert string to kebab-case
function toKebabCase(str: string): string {
  return str
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '');
}

function NewProjectContent() {
  const router = useRouter();
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [error, setError] = useState('');

  const { data: user } = api.users.me.useQuery();
  const userIdPrefix = user?.id?.split('-')[0] || 'user';

  const createMutation = api.projects.create.useMutation({
    onSuccess: (data) => {
      // Use full page navigation to ensure auth cookies are properly sent
      // Client-side navigation can lose auth session in Next.js App Router
      window.location.href = `/projects/${data.project.id}`;
    },
    onError: (err) => {
      setError(err.message);
    },
  });

  const handleSubmit = () => {
    setError('');

    if (!name.trim()) {
      setError('Project name is required');
      return;
    }

    createMutation.mutate({
      name: name.trim(),
      description: description.trim() || undefined,
    });
  };

  // Generate preview of queue name
  const projectKebab = toKebabCase(name);
  const queueName = projectKebab ? `${userIdPrefix}-${projectKebab}-queue` : '';
  const queueDisplayName = name ? `${name} Task Queue` : '';

  return (
    <YStack flex={1}>
      <Header />
      <XStack flex={1}>
        <Sidebar />
        <YStack flex={1} padding="$6" gap="$4">
          <H1>Create Project</H1>

          <Card padding="$4" elevate maxWidth={600}>
            <YStack gap="$4">
              {error && (
                <YStack padding="$3" backgroundColor="$red3" borderRadius="$4">
                  <Text color="$red11">{error}</Text>
                </YStack>
              )}

              {/* Project Name */}
              <YStack gap="$2">
                <Text fontSize="$3" fontWeight="600" htmlFor="projectName" tag="label">
                  Project Name *
                </Text>
                <Input
                  id="projectName"
                  size="$4"
                  placeholder="My Analytics Project"
                  value={name}
                  onChangeText={setName}
                  disabled={createMutation.isLoading}
                  autoFocus
                />
                <Text fontSize="$2" color="$gray11">
                  A descriptive name for your project
                </Text>
              </YStack>

              {/* Description */}
              <YStack gap="$2">
                <Text fontSize="$3" fontWeight="600" htmlFor="description" tag="label">
                  Description
                </Text>
                <TextArea
                  id="description"
                  size="$4"
                  placeholder="What is this project for?"
                  value={description}
                  onChangeText={setDescription}
                  disabled={createMutation.isLoading}
                  numberOfLines={3}
                />
                <Text fontSize="$2" color="$gray11">
                  Optional description of the project's purpose
                </Text>
              </YStack>

              {/* Task Queue Preview */}
              {queueName && (
                <YStack gap="$2" padding="$3" backgroundColor="$gray3" borderRadius="$4">
                  <Text fontSize="$3" fontWeight="600">
                    Task Queue (Auto-Generated)
                  </Text>
                  <YStack gap="$1">
                    <XStack gap="$2" alignItems="center">
                      <Text fontSize="$2" color="$gray11" width={80}>
                        Name:
                      </Text>
                      <Text fontSize="$3" fontFamily="$mono">
                        {queueName}
                      </Text>
                    </XStack>
                    <XStack gap="$2" alignItems="center">
                      <Text fontSize="$2" color="$gray11" width={80}>
                        Display:
                      </Text>
                      <Text fontSize="$3">{queueDisplayName}</Text>
                    </XStack>
                  </YStack>
                  <Text fontSize="$2" color="$gray11">
                    This task queue will be automatically created for your project
                  </Text>
                </YStack>
              )}

              {/* Actions */}
              <XStack gap="$3" marginTop="$2">
                <Button
                  size="$4"
                  theme="blue"
                  flex={1}
                  onPress={handleSubmit}
                  disabled={createMutation.isLoading || !name.trim()}
                >
                  {createMutation.isLoading ? 'Creating...' : 'Create Project'}
                </Button>
                <Button
                  size="$4"
                  variant="outlined"
                  onPress={() => router.back()}
                  disabled={createMutation.isLoading}
                >
                  Cancel
                </Button>
              </XStack>
            </YStack>
          </Card>
        </YStack>
      </XStack>
    </YStack>
  );
}

export default function NewProjectPage() {
  return (
    <AuthGuardWithLoading>
      <NewProjectContent />
    </AuthGuardWithLoading>
  );
}

