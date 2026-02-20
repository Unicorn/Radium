'use client';

import { YStack, XStack, H1, H2, Button, Input, TextArea, Card, Text, Spinner, Separator, Tabs } from 'tamagui';
import { Badge } from '@/components/shared/Badge';
import { useState } from 'react';
import { useRouter, useParams } from 'next/navigation';
import { AuthGuardWithLoading } from '@/components/shared/AuthGuard';
import { Header } from '@/components/shared/Header';
import { Sidebar } from '@/components/shared/Sidebar';
import { api } from '@/lib/trpc/client';
import { formatDistanceToNow } from 'date-fns';
import { Edit, Trash, Save, X, ExternalLink, BarChart3, Database, Globe } from 'lucide-react';
import { ProjectStatisticsPanel } from '@/components/execution/ProjectStatisticsPanel';
import { ConnectionManager } from '@/components/project/ConnectionManager';
import { ConnectorManager } from '@/components/connector/ConnectorManager';
import { ProjectView } from '@/components/service/ProjectView';
import { ProjectEndpointsList } from '@/components/project/ProjectEndpointsList';

function ProjectDetailContent() {
  const router = useRouter();
  const params = useParams();
  const projectId = params.id as string;
  const utils = api.useUtils();

  const [isEditing, setIsEditing] = useState(false);
  const [editName, setEditName] = useState('');
  const [editDescription, setEditDescription] = useState('');
  const [error, setError] = useState('');

  const { data: projectData, isLoading, error: fetchError } = api.projects.get.useQuery({ id: projectId });
  const project = projectData?.project;

  const updateMutation = api.projects.update.useMutation({
    onSuccess: () => {
      utils.projects.get.invalidate({ id: projectId });
      utils.projects.list.invalidate();
      setIsEditing(false);
      setError('');
    },
    onError: (err) => {
      setError(err.message);
    },
  });

  const deleteMutation = api.projects.archive.useMutation({
    onSuccess: () => {
      router.push('/projects');
    },
  });

  const handleEdit = () => {
    setEditName(project?.name || '');
    setEditDescription(project?.description || '');
    setIsEditing(true);
  };

  const handleSave = () => {
    if (!editName.trim()) {
      setError('Project name is required');
      return;
    }

    updateMutation.mutate({
      id: projectId,
      name: editName.trim(),
      description: editDescription.trim() || undefined,
    });
  };

  const handleCancel = () => {
    setIsEditing(false);
    setError('');
  };

  const handleDelete = () => {
    if (
      confirm(
        `Are you sure you want to delete "${project?.name}"?\n\nThis will delete all workflows in this project. This action cannot be undone.`
      )
    ) {
      deleteMutation.mutate({ id: projectId });
    }
  };

  if (isLoading) {
    return (
      <YStack flex={1}>
        <Header />
        <XStack flex={1}>
          <Sidebar />
          <YStack flex={1} alignItems="center" justifyContent="center" gap="$3">
            <Spinner size="large" />
            <Text>Loading project...</Text>
          </YStack>
        </XStack>
      </YStack>
    );
  }

  if (fetchError || !project) {
    return (
      <YStack flex={1}>
        <Header />
        <XStack flex={1}>
          <Sidebar />
          <YStack flex={1} alignItems="center" justifyContent="center" gap="$3">
            <Text color="$red10" fontSize="$5">
              {fetchError?.message || 'Project not found'}
            </Text>
            <Button onPress={() => router.push('/projects')}>Back to Projects</Button>
          </YStack>
        </XStack>
      </YStack>
    );
  }

  const workflowsInProject = project.workflows || [];
  const worker = project.workers?.[0];
  const workerStatus = worker?.status || 'stopped';

  const getWorkerStatusColor = () => {
    if (workerStatus === 'running') return '$green10';
    if (workerStatus === 'starting') return '$blue10';
    if (workerStatus === 'error') return '$red10';
    return '$gray10';
  };

  const getWorkerStatusText = () => {
    if (workerStatus === 'running') return 'Running';
    if (workerStatus === 'starting') return 'Starting';
    if (workerStatus === 'error') return 'Error';
    return 'Stopped';
  };

  return (
    <YStack flex={1}>
      <Header />
      <XStack flex={1}>
        <Sidebar />
        <YStack flex={1} padding="$6" gap="$4">
          {/* Header */}
          <XStack justifyContent="space-between" alignItems="flex-start" flexWrap="wrap" gap="$3">
            <YStack gap="$2">
              {isEditing ? (
                <Input
                  size="$6"
                  value={editName}
                  onChangeText={setEditName}
                  placeholder="Project name"
                  autoFocus
                />
              ) : (
                <H1>{project.name}</H1>
              )}
              {isEditing ? (
                <TextArea
                  value={editDescription}
                  onChangeText={setEditDescription}
                  placeholder="Project description"
                  numberOfLines={2}
                />
              ) : (
                project.description && <Text color="$gray11">{project.description}</Text>
              )}
            </YStack>

            <XStack gap="$2">
              {isEditing ? (
                <>
                  <Button
                    size="$3"
                    theme="blue"
                    icon={Save}
                    onPress={handleSave}
                    disabled={updateMutation.isLoading}
                  >
                    {updateMutation.isLoading ? 'Saving...' : 'Save'}
                  </Button>
                  <Button
                    size="$3"
                    variant="outlined"
                    icon={X}
                    onPress={handleCancel}
                    disabled={updateMutation.isLoading}
                  >
                    Cancel
                  </Button>
                </>
              ) : (
                <>
                  <Button size="$3" icon={Edit} variant="outlined" onPress={handleEdit}>
                    Edit
                  </Button>
                  <Button
                    size="$3"
                    theme="red"
                    icon={Trash}
                    variant="outlined"
                    onPress={handleDelete}
                    disabled={deleteMutation.isLoading}
                  >
                    Delete
                  </Button>
                </>
              )}
            </XStack>
          </XStack>

          {error && (
            <YStack padding="$3" backgroundColor="$red3" borderRadius="$4">
              <Text color="$red11">{error}</Text>
            </YStack>
          )}

          {/* Project Details Card */}
          <Card padding="$4" elevate>
            <YStack gap="$3">
              <H2 fontSize="$5">Project Details</H2>
              <Separator />
              
              {/* Endpoints Section */}
              <YStack gap="$3">
                <H2 fontSize="$4">API Endpoints</H2>
                <ProjectEndpointsList projectId={projectId} />
              </YStack>
              
              <Separator />

              <XStack justifyContent="space-between" flexWrap="wrap">
                <Text color="$gray11">Project ID</Text>
                <Text fontFamily="$mono" fontSize="$2">{project.id}</Text>
              </XStack>

              <XStack justifyContent="space-between" flexWrap="wrap">
                <Text color="$gray11">Channel Name</Text>
                <Text fontFamily="$mono">{project.task_queue_name}</Text>
              </XStack>

              <XStack justifyContent="space-between" flexWrap="wrap">
                <Text color="$gray11">Worker Status</Text>
                <Badge backgroundColor={getWorkerStatusColor()} paddingHorizontal="$3" paddingVertical="$1">
                  <Text fontSize="$2" color="white">
                    {getWorkerStatusText()}
                  </Text>
                </Badge>
              </XStack>

              {worker && (
                <>
                  <XStack justifyContent="space-between" flexWrap="wrap">
                    <Text color="$gray11">Worker ID</Text>
                    <Text fontFamily="$mono" fontSize="$2">{worker.worker_id}</Text>
                  </XStack>

                  {worker.last_heartbeat && (
                    <XStack justifyContent="space-between" flexWrap="wrap">
                      <Text color="$gray11">Last Heartbeat</Text>
                      <Text>
                        {formatDistanceToNow(new Date(worker.last_heartbeat), { addSuffix: true })}
                      </Text>
                    </XStack>
                  )}
                </>
              )}

              <XStack justifyContent="space-between" flexWrap="wrap">
                <Text color="$gray11">Created</Text>
                <Text>
                  {formatDistanceToNow(new Date(project.created_at), { addSuffix: true })}
                </Text>
              </XStack>

              <XStack justifyContent="space-between" flexWrap="wrap">
                <Text color="$gray11">Updated</Text>
                <Text>
                  {formatDistanceToNow(new Date(project.updated_at), { addSuffix: true })}
                </Text>
              </XStack>
            </YStack>
          </Card>

          {/* Tabs */}
          <Tabs defaultValue="workflows" flex={1}>
            <Tabs.List>
              <Tabs.Tab value="workflows">
                <Text>Services</Text>
              </Tabs.Tab>
              <Tabs.Tab value="statistics">
                <XStack gap="$2" ai="center">
                  <BarChart3 size={16} />
                  <Text>Statistics</Text>
                </XStack>
              </Tabs.Tab>
              <Tabs.Tab value="connections">
                <XStack gap="$2" ai="center">
                  <Database size={16} />
                  <Text>Database Connections</Text>
                </XStack>
              </Tabs.Tab>
              <Tabs.Tab value="connectors">
                <XStack gap="$2" ai="center">
                  <Globe size={16} />
                  <Text>Connectors</Text>
                </XStack>
              </Tabs.Tab>
              <Tabs.Tab value="visualization">
                <XStack gap="$2" ai="center">
                  <Globe size={16} />
                  <Text>Visualization</Text>
                </XStack>
              </Tabs.Tab>
            </Tabs.List>

            <Tabs.Content value="workflows">
              {/* Workflows in Project */}
              <Card padding="$4" elevate>
                <YStack gap="$3">
                  <XStack justifyContent="space-between" alignItems="center">
                    <H2 fontSize="$5">Services ({workflowsInProject.length})</H2>
                    <Button
                      size="$3"
                      theme="blue"
                      onPress={() => router.push(`/workflows/new?projectId=${projectId}`)}
                    >
                      New Service
                    </Button>
                  </XStack>
                  <Separator />

                  {workflowsInProject.length === 0 ? (
                    <YStack padding="$4" alignItems="center" gap="$2">
                      <Text color="$gray11">No services in this project yet</Text>
                      <Button
                        size="$3"
                        onPress={() => router.push(`/workflows/new?projectId=${projectId}`)}
                      >
                        Create First Service
                      </Button>
                    </YStack>
                  ) : (
                    <YStack gap="$2">
                      {workflowsInProject.map((workflow: any) => (
                        <XStack
                          key={workflow.id}
                          padding="$3"
                          backgroundColor="$gray2"
                          borderRadius="$4"
                          justifyContent="space-between"
                          alignItems="center"
                          pressStyle={{ backgroundColor: '$gray3' }}
                          cursor="pointer"
                          onPress={() => router.push(`/workflows/${workflow.id}`)}
                        >
                          <YStack gap="$1">
                            <Text fontSize="$4" fontWeight="600">
                              {workflow.display_name || workflow.name}
                            </Text>
                            <Text fontSize="$2" color="$gray11" fontFamily="$mono">
                              {workflow.kebab_name || workflow.name}
                            </Text>
                          </YStack>
                          <XStack gap="$2" alignItems="center">
                            <Badge backgroundColor="$blue10" paddingHorizontal="$2" paddingVertical="$1">
                              <Text fontSize="$1" color="white">
                                {workflow.status_id}
                              </Text>
                            </Badge>
                            <ExternalLink size={16} />
                          </XStack>
                        </XStack>
                      ))}
                    </YStack>
                  )}
                </YStack>
              </Card>
            </Tabs.Content>

            <Tabs.Content value="statistics">
              <ProjectStatisticsPanel projectId={projectId} />
            </Tabs.Content>

            <Tabs.Content value="connections">
              <ConnectionManager projectId={projectId} />
            </Tabs.Content>

            <Tabs.Content value="connectors">
              <ConnectorManager projectId={projectId} />
            </Tabs.Content>

            <Tabs.Content value="visualization">
              <Card padding="$4" elevate>
                <YStack gap="$3" minHeight="600px">
                  <H2 fontSize="$5">Project Visualization</H2>
                  <Text color="$gray11">
                    View all services and their connections in this project
                  </Text>
                  <Separator />
                  <ProjectView projectId={projectId} readOnly={false} />
                </YStack>
              </Card>
            </Tabs.Content>
          </Tabs>
        </YStack>
      </XStack>
    </YStack>
  );
}

export default function ProjectDetailPage() {
  return (
    <AuthGuardWithLoading>
      <ProjectDetailContent />
    </AuthGuardWithLoading>
  );
}

