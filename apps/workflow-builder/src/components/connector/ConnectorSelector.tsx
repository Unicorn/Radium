'use client';

import { YStack, XStack, Text, Button, Select, Label, Spinner } from 'tamagui';
import { Plus, Check } from 'lucide-react';
import { useState } from 'react';
import { api } from '@/lib/trpc/client';
import { ConnectorCreationModal } from './ConnectorCreationModal';

type ConnectorClassification = 'redis' | 'http-log' | 'syslog' | 'file-log' | 'tcp-log' | 'udp-log';

interface ConnectorSelectorProps {
  projectId: string;
  connectorType?: 'email' | 'slack' | 'database' | 'api' | 'oauth';
  classification?: ConnectorClassification; // Filter by classification instead of connector type
  value?: string; // Selected connector ID
  onChange: (connectorId: string | undefined) => void;
  allowNewConnector?: boolean;
  disabled?: boolean;
}

/**
 * Reusable connector selector component for component configuration.
 * 
 * Supports selecting existing connectors or creating new ones.
 * Works with both the existing connections system (database) and
 * the new connectors system (email, slack, api, oauth).
 */
export function ConnectorSelector({
  projectId,
  connectorType,
  classification,
  value,
  onChange,
  allowNewConnector = true,
  disabled = false,
}: ConnectorSelectorProps) {
  const [showCreateModal, setShowCreateModal] = useState(false);
  const utils = api.useUtils();

  // For database connectors, use the existing connections API
  const isDatabaseConnector = connectorType === 'database';
  
  const { data: connectionsData, isLoading: connectionsLoading } = 
    api.connections.list.useQuery(
      { projectId },
      { enabled: isDatabaseConnector && !!projectId }
    );

  // Use classification-based query if classification is provided
  const { data: connectorsByClassification, isLoading: connectorsByClassificationLoading } =
    api.connectors.getByClassification.useQuery(
      { projectId, classification: classification! },
      { enabled: !!classification && !!projectId && !isDatabaseConnector }
    );

  // Use connector type-based query if connectorType is provided (and no classification)
  const { data: connectorsByType, isLoading: connectorsByTypeLoading } =
    api.connectors.list.useQuery(
      { projectId, connectorType },
      { enabled: !!connectorType && !classification && !isDatabaseConnector && !!projectId }
    );

  const connectors = isDatabaseConnector
    ? connectionsData?.connections.map(conn => ({
        id: conn.id,
        name: conn.name,
        type: conn.connectionType,
      })) || []
    : classification
    ? connectorsByClassification?.map(conn => ({
        id: conn.id,
        name: conn.display_name || conn.name,
        type: conn.connector_type,
      })) || []
    : connectorsByType?.map(conn => ({
        id: conn.id,
        name: conn.display_name || conn.name,
        type: conn.connector_type,
      })) || [];

  const isLoading = isDatabaseConnector 
    ? connectionsLoading 
    : classification
    ? connectorsByClassificationLoading
    : connectorsByTypeLoading;

  const handleCreateSuccess = (newConnectorId: string) => {
    // Invalidate queries to refresh the list
    if (isDatabaseConnector) {
      utils.connections.list.invalidate({ projectId });
    } else {
      if (classification) {
        utils.connectors.getByClassification.invalidate({ projectId, classification });
      } else if (connectorType) {
        utils.connectors.list.invalidate({ projectId, connectorType });
      }
    }
    
    // Select the newly created connector
    onChange(newConnectorId);
    setShowCreateModal(false);
  };

  return (
    <YStack gap="$2">
      <XStack ai="center" jc="space-between">
        <Label htmlFor="connector-select">
          {classification 
            ? `${classification.charAt(0).toUpperCase() + classification.slice(1).replace('-', ' ')} Connector`
            : connectorType === 'database' ? 'Database Connector' :
             connectorType === 'email' ? 'Email Connector' :
             connectorType === 'slack' ? 'Slack Connector' :
             connectorType === 'api' ? 'API Connector' :
             connectorType === 'oauth' ? 'OAuth Connector' :
             'Connector'}
        </Label>
        {allowNewConnector && (
          <Button
            size="$2"
            icon={Plus}
            onPress={() => setShowCreateModal(true)}
            disabled={disabled}
            chromeless
          >
            Create New
          </Button>
        )}
      </XStack>

      {isLoading ? (
        <XStack ai="center" gap="$2" p="$3">
          <Spinner size="small" />
          <Text fontSize="$2" color="$gray11">Loading connectors...</Text>
        </XStack>
      ) : (
        <Select
          value={value || ''}
          onValueChange={(v) => onChange(v || undefined)}
          disabled={disabled}
        >
          <Select.Trigger id="connector-select" width="100%">
            <Select.Value placeholder={
            classification 
              ? `Select ${classification.replace('-', ' ')} connector`
              : `Select ${connectorType || 'connector'}`
          } />
          </Select.Trigger>
          <Select.Content>
            <Select.Viewport>
              <Select.Item index={-1} value="">
                <Select.ItemText>None</Select.ItemText>
                {!value && (
                  <Select.ItemIndicator marginLeft="auto">
                    <Check size={16} />
                  </Select.ItemIndicator>
                )}
              </Select.Item>
              {connectors.map((connector, index) => (
                <Select.Item key={connector.id} index={index} value={connector.id}>
                  <Select.ItemText>{connector.name}</Select.ItemText>
                  {value === connector.id && (
                    <Select.ItemIndicator marginLeft="auto">
                      <Check size={16} />
                    </Select.ItemIndicator>
                  )}
                </Select.Item>
              ))}
            </Select.Viewport>
          </Select.Content>
        </Select>
      )}

      {value && (
        <Text fontSize="$2" color="$gray11">
          Selected: {connectors.find(c => c.id === value)?.name || 'Unknown'}
        </Text>
      )}

      {showCreateModal && connectorType && (
        <ConnectorCreationModal
          projectId={projectId}
          connectorType={connectorType}
          open={showCreateModal}
          onOpenChange={setShowCreateModal}
          onSuccess={handleCreateSuccess}
        />
      )}
    </YStack>
  );
}

