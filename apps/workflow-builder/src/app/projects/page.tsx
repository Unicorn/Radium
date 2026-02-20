'use client';

import { YStack, XStack, H1, Button, Input, Spinner, Text } from 'tamagui';
import { useState } from 'react';
import { useRouter } from 'next/navigation';
import { AuthGuardWithLoading } from '@/components/shared/AuthGuard';
import { Header } from '@/components/shared/Header';
import { Sidebar } from '@/components/shared/Sidebar';
import { ProjectCard } from '@/components/project/ProjectCard';
import { api } from '@/lib/trpc/client';
import { Plus, Search } from 'lucide-react';

function ProjectsContent() {
  const router = useRouter();
  const [searchQuery, setSearchQuery] = useState('');
  const utils = api.useUtils();

  const { data, isLoading, error } = api.projects.list.useQuery();

  const archiveMutation = api.projects.archive.useMutation({
    onSuccess: () => {
      utils.projects.list.invalidate();
    },
  });

  const handleArchive = (projectId: string, projectName: string, isDefault: boolean) => {
    if (isDefault) {
      alert('Cannot archive default project. You can rename it, but not archive it.');
      return;
    }
    
    if (
      confirm(
        `Are you sure you want to archive "${projectName}"?\n\nThis will also archive all workflows in this project. You can unarchive them later.`
      )
    ) {
      archiveMutation.mutate({ id: projectId });
    }
  };

  // Filter projects by search query
  const filteredProjects = data?.projects?.filter((project) => {
    if (!searchQuery) return true;
    const query = searchQuery.toLowerCase();
    return (
      project.name.toLowerCase().includes(query) ||
      project.description?.toLowerCase().includes(query) ||
      project.task_queue_name.toLowerCase().includes(query)
    );
  });

  return (
    <YStack flex={1}>
      <Header />
      <XStack flex={1}>
        <Sidebar />
        <YStack flex={1} padding="$6" gap="$4">
          {/* Header */}
          <XStack justifyContent="space-between" alignItems="center" flexWrap="wrap" gap="$3">
            <H1>Projects</H1>
            <Button
              size="$4"
              theme="blue"
              icon={Plus}
              onPress={() => router.push('/projects/new')}
            >
              New Project
            </Button>
          </XStack>

          {/* Search */}
          <XStack gap="$2" alignItems="center" maxWidth={600}>
            <XStack flex={1} alignItems="center" gap="$2" borderWidth={1} borderColor="$gray5" borderRadius="$4" paddingHorizontal="$3" paddingVertical="$2">
              <Input
                flex={1}
                size="$4"
                placeholder="Search projects..."
                value={searchQuery}
                onChangeText={setSearchQuery}
                borderWidth={0}
                padding={0}
              />
              <Search size={16} color="$gray10" />
            </XStack>
          </XStack>

          {/* Loading State */}
          {isLoading && (
            <YStack flex={1} alignItems="center" justifyContent="center" gap="$3">
              <Spinner size="large" />
              <Text>Loading projects...</Text>
            </YStack>
          )}

          {/* Error State */}
          {error && (
            <YStack flex={1} alignItems="center" justifyContent="center">
              <Text color="$red10" fontSize="$5">
                Error loading projects: {error.message}
              </Text>
            </YStack>
          )}

          {/* Empty State */}
          {!isLoading && !error && filteredProjects?.length === 0 && !searchQuery && (
            <YStack flex={1} alignItems="center" justifyContent="center" gap="$4">
              <Text fontSize="$6" color="$gray11">
                No projects yet
              </Text>
              <Text fontSize="$3" color="$gray11" textAlign="center">
                Create your first project to organize workflows and manage task queues
              </Text>
              <Button
                size="$4"
                theme="blue"
                icon={Plus}
                onPress={() => router.push('/projects/new')}
              >
                Create Your First Project
              </Button>
            </YStack>
          )}

          {/* No Search Results */}
          {!isLoading && !error && filteredProjects?.length === 0 && searchQuery && (
            <YStack flex={1} alignItems="center" justifyContent="center" gap="$3">
              <Text fontSize="$5" color="$gray11">
                No projects found matching "{searchQuery}"
              </Text>
              <Button variant="outlined" onPress={() => setSearchQuery('')}>
                Clear Search
              </Button>
            </YStack>
          )}

          {/* Projects Grid */}
          {!isLoading && !error && filteredProjects && filteredProjects.length > 0 && (
            <YStack gap="$3">
              <Text fontSize="$3" color="$gray11">
                {filteredProjects.length} project{filteredProjects.length !== 1 ? 's' : ''} found
              </Text>

              <YStack gap="$3">
                {filteredProjects.map((project) => (
                  <ProjectCard
                    key={project.id}
                    project={project}
                    onClick={() => router.push(`/projects/${project.id}`)}
                    onEdit={() => router.push(`/projects/${project.id}`)}
                    onArchive={() => handleArchive(project.id, project.name, project.is_default || false)}
                    showActions
                  />
                ))}
              </YStack>
            </YStack>
          )}
        </YStack>
      </XStack>
    </YStack>
  );
}

export default function ProjectsPage() {
  return (
    <AuthGuardWithLoading>
      <ProjectsContent />
    </AuthGuardWithLoading>
  );
}

