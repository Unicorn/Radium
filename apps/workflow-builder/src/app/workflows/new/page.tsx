/**
 * Create New Workflow Page
 */

'use client';

import { YStack, XStack, H1, Button, Input, TextArea, Select, Adapt, Sheet, Card, Text } from 'tamagui';
import { useState, useEffect } from 'react';
import { useRouter } from 'next/navigation';
import { AuthGuardWithLoading } from '@/components/shared/AuthGuard';
import { Header } from '@/components/shared/Header';
import { Sidebar } from '@/components/shared/Sidebar';
import { api } from '@/lib/trpc/client';
import { Check, ChevronDown } from 'lucide-react';

// Helper function to convert string to kebab-case
function toKebabCase(str: string): string {
  return str
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-') // Replace non-alphanumeric with hyphens
    .replace(/^-+|-+$/g, '');     // Remove leading/trailing hyphens
}

function NewWorkflowContent() {
  const router = useRouter();
  const [displayName, setDisplayName] = useState('');
  const [kebabName, setKebabName] = useState('');
  const [description, setDescription] = useState('');
  const [selectedProjectId, setSelectedProjectId] = useState('');
  const [showNewProjectInput, setShowNewProjectInput] = useState(false);
  const [newProjectName, setNewProjectName] = useState('');
  const [visibility, setVisibility] = useState<string>('private');
  const [error, setError] = useState('');

  const { data: projectsData, isLoading: loadingProjects } = api.projects.list.useQuery();
  const projects = projectsData?.projects || [];
  
  // Find default project
  const defaultProject = projects.find(p => p.is_default);
  
  // Set default project on mount if available
  useEffect(() => {
    if (defaultProject && !selectedProjectId) {
      setSelectedProjectId(defaultProject.id);
    }
  }, [defaultProject, selectedProjectId]);
  
  const createProjectMutation = api.projects.create.useMutation();
  
  const createMutation = api.workflows.create.useMutation({
    onSuccess: (workflow) => {
      console.log('✅ [createMutation] Success! Workflow created:', workflow);
      router.push(`/workflows/${workflow.id}/edit`);
    },
    onError: (err) => {
      console.error('❌ [createMutation] Error:', err);
      setError(err.message);
    },
  });

  // Auto-generate kebab-case name from display name
  const handleDisplayNameChange = (value: string) => {
    setDisplayName(value);
    setKebabName(toKebabCase(value));
  };

  const handleSubmit = async () => {
    console.log('🔵 [handleSubmit] Called with:', { displayName, kebabName, selectedProjectId, showNewProjectInput, newProjectName });
    setError('');

    // Validation
    if (!displayName || !kebabName) {
      const errorMsg = 'Workflow name and identifier are required';
      console.error('❌ [handleSubmit] Validation failed:', errorMsg);
      setError(errorMsg);
      return;
    }

    // Validate project selection/creation
    if (showNewProjectInput) {
      if (!newProjectName.trim()) {
        setError('Project name is required');
        return;
      }
    } else if (!selectedProjectId && projects.length > 0) {
      setError('Please select a project');
      return;
    }

    // Validate kebab name format
    if (!/^[a-z0-9]+(-[a-z0-9]+)*$/.test(kebabName)) {
      const errorMsg = 'Kebab name must contain only lowercase letters, numbers, and hyphens';
      console.error('❌ [handleSubmit] Kebab name validation failed:', kebabName);
      setError(errorMsg);
      return;
    }

    try {
      let projectId = selectedProjectId;
      let taskQueueId = '';

      // Create new project if needed
      if (showNewProjectInput || projects.length === 0) {
        const projectNameToUse = showNewProjectInput ? newProjectName : newProjectName || 'Default Project';
        console.log('📦 [handleSubmit] Creating new project:', projectNameToUse);
        
        const result = await createProjectMutation.mutateAsync({
          name: projectNameToUse,
          description: undefined,
        });
        
        projectId = result.project.id;
        taskQueueId = result.taskQueue?.id || '';
        console.log('✅ [handleSubmit] Project created:', { projectId, taskQueueId });
      } else {
        // Find the default task queue for the selected project
        // For now, we'll let the backend handle this
        projectId = selectedProjectId;
      }

      const payload = {
        kebabName: kebabName.trim(),
        displayName: displayName.trim(),
        description: description.trim() || undefined,
        visibility: visibility as any,
        projectId,
        taskQueueId: taskQueueId || undefined, // Pass task queue ID if we have it
      };
      
      console.log('✅ [handleSubmit] Creating workflow with:', payload);
      createMutation.mutate(payload);
    } catch (err: any) {
      console.error('❌ [handleSubmit] Error:', err);
      setError(err.message || 'Failed to create workflow');
    }
  };

  return (
    <YStack flex={1}>
      <Header />
      <XStack flex={1}>
        <Sidebar />
        <YStack flex={1} padding="$6" gap="$4">
          <H1>Create Workflow</H1>

          <Card padding="$4" elevate maxWidth={600}>
            <YStack gap="$4">
                {error && (
                  <YStack padding="$3" backgroundColor="$red3" borderRadius="$4">
                    <Text color="$red11">{error}</Text>
                  </YStack>
                )}

                <YStack gap="$2">
                  <Text fontSize="$3" fontWeight="600" htmlFor="displayName" tag="label">
                    Workflow Name *
                  </Text>
                  <Input
                    id="displayName"
                    size="$4"
                    placeholder="Customer Onboarding"
                    value={displayName}
                    onChangeText={handleDisplayNameChange}
                    disabled={createMutation.isLoading}
                  />
                  <Text fontSize="$2" color="$gray11">
                    Human-friendly name for your workflow
                  </Text>
                </YStack>

                <YStack gap="$2">
                  <Text fontSize="$3" fontWeight="600" htmlFor="kebabName" tag="label">
                    Workflow Identifier *
                  </Text>
                  <Input
                    id="kebabName"
                    size="$4"
                    placeholder="customer-onboarding"
                    value={kebabName}
                    onChangeText={setKebabName}
                    disabled={createMutation.isLoading}
                  />
                  <Text fontSize="$2" color="$gray11">
                    Kebab-case identifier (auto-generated, editable before save, locked after)
                  </Text>
                </YStack>

                <YStack gap="$2">
                  <Text fontSize="$3" fontWeight="600" htmlFor="description" tag="label">
                    Description
                  </Text>
                  <TextArea
                    id="description"
                    size="$4"
                    placeholder="What does this workflow do?"
                    value={description}
                    onChangeText={setDescription}
                    disabled={createMutation.isLoading}
                    numberOfLines={3}
                  />
                </YStack>

                <YStack gap="$2">
                  <Text fontSize="$3" fontWeight="600" htmlFor="project" tag="label">
                    Project *
                  </Text>
                  
                  {loadingProjects ? (
                    <Text fontSize="$3" color="$gray11">Loading projects...</Text>
                  ) : projects.length === 0 || showNewProjectInput ? (
                    <YStack gap="$2">
                      <Input
                        id="newProjectName"
                        size="$4"
                        placeholder="Enter project name"
                        value={newProjectName}
                        onChangeText={setNewProjectName}
                        disabled={createMutation.isLoading || createProjectMutation.isLoading}
                      />
                      <Text fontSize="$2" color="$gray11">
                        {projects.length === 0 
                          ? 'Create your first project to organize workflows'
                          : 'Creating a new project...'}
                      </Text>
                      {projects.length > 0 && (
                        <Button
                          size="$2"
                          variant="outlined"
                          onPress={() => {
                            setShowNewProjectInput(false);
                            setNewProjectName('');
                          }}
                          disabled={createMutation.isLoading || createProjectMutation.isLoading}
                        >
                          Cancel
                        </Button>
                      )}
                    </YStack>
                  ) : (
                    <YStack gap="$2">
                      <Select
                        value={selectedProjectId}
                        onValueChange={(value) => {
                          if (value === '__new__') {
                            setShowNewProjectInput(true);
                            setSelectedProjectId('');
                          } else {
                            setSelectedProjectId(value);
                          }
                        }}
                      >
                        <Select.Trigger width="100%" iconAfter={ChevronDown}>
                          <Select.Value placeholder="Select project" />
                        </Select.Trigger>

                        <Adapt when="sm" platform="touch">
                          <Sheet modal dismissOnSnapToBottom>
                            <Sheet.Frame>
                              <Sheet.ScrollView>
                                <Adapt.Contents />
                              </Sheet.ScrollView>
                            </Sheet.Frame>
                            <Sheet.Overlay />
                          </Sheet>
                        </Adapt>

                        <Select.Content zIndex={200000}>
                          <Select.ScrollUpButton />
                          <Select.Viewport>
                            {projects.map((project, idx) => (
                              <Select.Item key={project.id} index={idx} value={project.id}>
                                <Select.ItemText>{project.name}</Select.ItemText>
                                <Select.ItemIndicator marginLeft="auto">
                                  <Check size={16} />
                                </Select.ItemIndicator>
                              </Select.Item>
                            ))}
                            <Select.Item index={projects.length} value="__new__">
                              <Select.ItemText>+ Create New Project</Select.ItemText>
                            </Select.Item>
                          </Select.Viewport>
                          <Select.ScrollDownButton />
                        </Select.Content>
                      </Select>
                      <Text fontSize="$2" color="$gray11">
                        Each project has its own task queue for workflow execution
                      </Text>
                    </YStack>
                  )}
                </YStack>

                <YStack gap="$2">
                  <Text fontSize="$3" fontWeight="600" htmlFor="visibility" tag="label">
                    Visibility
                  </Text>
                  <Select
                    value={visibility}
                    onValueChange={setVisibility}
                  >
                    <Select.Trigger width="100%" iconAfter={ChevronDown}>
                      <Select.Value />
                    </Select.Trigger>

                    <Adapt when="sm" platform="touch">
                      <Sheet modal dismissOnSnapToBottom>
                        <Sheet.Frame>
                          <Sheet.ScrollView>
                            <Adapt.Contents />
                          </Sheet.ScrollView>
                        </Sheet.Frame>
                        <Sheet.Overlay />
                      </Sheet>
                    </Adapt>

                    <Select.Content zIndex={200000}>
                      <Select.Viewport>
                        <Select.Item index={0} value="public">
                          <Select.ItemText>Public</Select.ItemText>
                          <Select.ItemIndicator marginLeft="auto">
                            <Check size={16} />
                          </Select.ItemIndicator>
                        </Select.Item>
                        <Select.Item index={1} value="private">
                          <Select.ItemText>Private</Select.ItemText>
                          <Select.ItemIndicator marginLeft="auto">
                            <Check size={16} />
                          </Select.ItemIndicator>
                        </Select.Item>
                        <Select.Item index={2} value="organization">
                          <Select.ItemText>Organization</Select.ItemText>
                          <Select.ItemIndicator marginLeft="auto">
                            <Check size={16} />
                          </Select.ItemIndicator>
                        </Select.Item>
                      </Select.Viewport>
                    </Select.Content>
                  </Select>
                </YStack>

                <XStack gap="$3" marginTop="$4">
                  <Button
                    size="$4"
                    theme="blue"
                    onPress={handleSubmit}
                    disabled={createMutation.isLoading}
                    flex={1}
                  >
                    {createMutation.isLoading ? 'Creating...' : 'Create & Edit'}
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

export default function NewWorkflowPage() {
  return (
    <AuthGuardWithLoading>
      <NewWorkflowContent />
    </AuthGuardWithLoading>
  );
}

