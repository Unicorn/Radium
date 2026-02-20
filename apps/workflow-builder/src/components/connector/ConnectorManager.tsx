'use client';

import { YStack, XStack, Text, Button, Card, Spinner, Separator } from 'tamagui';
import { Plus, Mail, MessageSquare, Globe, Key } from 'lucide-react';
import { useState } from 'react';
import { api } from '@/lib/trpc/client';
import { ConnectorCreationModal } from './ConnectorCreationModal';

interface ConnectorManagerProps {
  projectId: string;
}

/**
 * Connector management component for project page.
 * 
 * Manages third-party connectors (email, slack, api, oauth) for a project.
 * This is separate from database connections which are managed via ConnectionManager.
 */
export function ConnectorManager({ projectId }: ConnectorManagerProps) {
  const utils = api.useUtils();
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [selectedConnectorType, setSelectedConnectorType] = useState<'email' | 'slack' | 'database' | 'api' | 'oauth' | null>(null);

  // TODO: When connectors API is implemented, use it
  // const { data, isLoading } = api.connectors.list.useQuery({ projectId });
  // const connectors = data?.connectors || [];
  const isLoading = false;
  const connectors: any[] = []; // Placeholder until API is ready

  const handleCreateSuccess = () => {
    // TODO: utils.connectors.list.invalidate({ projectId });
    setShowCreateModal(false);
    setSelectedConnectorType(null);
  };

  // Group connectors by type
  const connectorsByType = {
    email: connectors.filter(c => c.connectorType === 'email'),
    slack: connectors.filter(c => c.connectorType === 'slack'),
    database: connectors.filter(c => c.connectorType === 'database'),
    api: connectors.filter(c => c.connectorType === 'api'),
    oauth: connectors.filter(c => c.connectorType === 'oauth'),
  };

  const getConnectorTypeIcon = (type: string) => {
    switch (type) {
      case 'email':
        return Mail;
      case 'slack':
        return MessageSquare;
      case 'api':
        return Globe;
      case 'oauth':
        return Key;
      default:
        return null;
    }
  };

  const getConnectorTypeLabel = (type: string) => {
    switch (type) {
      case 'email':
        return 'Email Connectors';
      case 'slack':
        return 'Slack Connectors';
      case 'database':
        return 'Database Connectors';
      case 'api':
        return 'API Connectors';
      case 'oauth':
        return 'OAuth Connectors';
      default:
        return 'Connectors';
    }
  };

  return (
    <YStack gap="$4">
      <XStack ai="center" jc="space-between">
        <Text fontSize="$5" fontWeight="600">Project Connectors</Text>
        <Button
          size="$3"
          icon={Plus}
          onPress={() => {
            setSelectedConnectorType('email'); // Default to email
            setShowCreateModal(true);
          }}
        >
          Add Connector
        </Button>
      </XStack>

      {isLoading ? (
        <YStack ai="center" p="$6">
          <Spinner size="large" />
        </YStack>
      ) : connectors.length === 0 ? (
        <Card p="$6" bg="$gray2" ai="center">
          <Text color="$gray11" textAlign="center" mb="$4">
            No connectors yet. Add connectors to integrate with external services like SendGrid, Slack, or APIs.
          </Text>
          <Button
            size="$3"
            icon={Plus}
            onPress={() => {
              setSelectedConnectorType('email');
              setShowCreateModal(true);
            }}
          >
            Create Your First Connector
          </Button>
        </Card>
      ) : (
        <YStack gap="$4">
          {/* Email Connectors */}
          {connectorsByType.email.length > 0 && (
            <YStack gap="$2">
              <XStack ai="center" gap="$2">
                {getConnectorTypeIcon('email') && (
                  <Mail size={18} color="$gray11" />
                )}
                <Text fontSize="$4" fontWeight="600">
                  {getConnectorTypeLabel('email')} ({connectorsByType.email.length})
                </Text>
              </XStack>
              <YStack gap="$2">
                {connectorsByType.email.map((connector) => (
                  <ConnectorCard key={connector.id} connector={connector} />
                ))}
              </YStack>
            </YStack>
          )}

          {/* Slack Connectors */}
          {connectorsByType.slack.length > 0 && (
            <YStack gap="$2">
              <XStack ai="center" gap="$2">
                {getConnectorTypeIcon('slack') && (
                  <MessageSquare size={18} color="$gray11" />
                )}
                <Text fontSize="$4" fontWeight="600">
                  {getConnectorTypeLabel('slack')} ({connectorsByType.slack.length})
                </Text>
              </XStack>
              <YStack gap="$2">
                {connectorsByType.slack.map((connector) => (
                  <ConnectorCard key={connector.id} connector={connector} />
                ))}
              </YStack>
            </YStack>
          )}

          {/* API Connectors */}
          {connectorsByType.api.length > 0 && (
            <YStack gap="$2">
              <XStack ai="center" gap="$2">
                {getConnectorTypeIcon('api') && (
                  <Globe size={18} color="$gray11" />
                )}
                <Text fontSize="$4" fontWeight="600">
                  {getConnectorTypeLabel('api')} ({connectorsByType.api.length})
                </Text>
              </XStack>
              <YStack gap="$2">
                {connectorsByType.api.map((connector) => (
                  <ConnectorCard key={connector.id} connector={connector} />
                ))}
              </YStack>
            </YStack>
          )}

          {/* OAuth Connectors */}
          {connectorsByType.oauth.length > 0 && (
            <YStack gap="$2">
              <XStack ai="center" gap="$2">
                {getConnectorTypeIcon('oauth') && (
                  <Key size={18} color="$gray11" />
                )}
                <Text fontSize="$4" fontWeight="600">
                  {getConnectorTypeLabel('oauth')} ({connectorsByType.oauth.length})
                </Text>
              </XStack>
              <YStack gap="$2">
                {connectorsByType.oauth.map((connector) => (
                  <ConnectorCard key={connector.id} connector={connector} />
                ))}
              </YStack>
            </YStack>
          )}
        </YStack>
      )}

      {showCreateModal && selectedConnectorType && (
        <ConnectorCreationModal
          projectId={projectId}
          connectorType={selectedConnectorType}
          open={showCreateModal}
          onOpenChange={setShowCreateModal}
          onSuccess={handleCreateSuccess}
        />
      )}
    </YStack>
  );
}

interface ConnectorCardProps {
  connector: {
    id: string;
    name: string;
    display_name?: string;
    description?: string;
    connectorType: string;
    is_active?: boolean;
  };
}

function ConnectorCard({ connector }: ConnectorCardProps) {
  // TODO: Add edit, delete, test functionality when connectors API is ready
  return (
    <Card p="$4" bg="$gray2">
      <YStack gap="$2">
        <XStack ai="center" jc="space-between">
          <Text fontSize="$4" fontWeight="600">
            {connector.display_name || connector.name}
          </Text>
          {connector.is_active !== false && (
            <Text fontSize="$2" color="$green11" textTransform="uppercase">
              Active
            </Text>
          )}
        </XStack>
        {connector.description && (
          <Text fontSize="$2" color="$gray11">
            {connector.description}
          </Text>
        )}
        <Text fontSize="$1" color="$gray10" textTransform="uppercase">
          {connector.connectorType}
        </Text>
      </YStack>
    </Card>
  );
}

