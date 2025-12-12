'use client';

import { YStack, XStack, Text, Button, Card, Spinner, Separator, Select, Label, Input, TextArea } from 'tamagui';
import { Badge } from '@/components/shared/Badge';
import { Plus, Radio, Search, Edit, Trash, Globe, Send, ArrowRight } from 'lucide-react';
import { useState } from 'react';
import { api } from '@/lib/trpc/client';

interface ServiceInterfacesProps {
  serviceId: string; // workflowId
}

/**
 * Service Interfaces management component.
 * 
 * Manages service-to-service communication interfaces:
 * - Send Action (signal)
 * - Get State (query)
 * - Modify State (update)
 * - Start Service (startChild)
 * 
 * Also manages public interfaces exposed via Kong API.
 */
export function ServiceInterfaces({ serviceId }: ServiceInterfacesProps) {
  const utils = api.useUtils();
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);

  // TODO: When service interfaces API is implemented, use it
  // const { data, isLoading } = api.serviceInterfaces.list.useQuery({ serviceId });
  // const interfaces = data?.interfaces || [];
  const isLoading = false;
  const interfaces: any[] = []; // Placeholder until API is ready

  const getInterfaceTypeIcon = (type: string) => {
    switch (type) {
      case 'signal':
        return Send;
      case 'query':
        return Search;
      case 'update':
        return Edit;
      case 'start_child':
        return ArrowRight;
      default:
        return Radio;
    }
  };

  const getInterfaceTypeLabel = (type: string) => {
    switch (type) {
      case 'signal':
        return 'Send Action';
      case 'query':
        return 'Get State';
      case 'update':
        return 'Modify State';
      case 'start_child':
        return 'Start Service';
      default:
        return type;
    }
  };

  const getInterfaceTypeColor = (type: string) => {
    switch (type) {
      case 'signal':
        return '$blue10';
      case 'query':
        return '$green10';
      case 'update':
        return '$orange10';
      case 'start_child':
        return '$purple10';
      default:
        return '$gray10';
    }
  };

  // Group interfaces by type
  const interfacesByType = {
    signal: interfaces.filter(i => i.interface_type === 'signal'),
    query: interfaces.filter(i => i.interface_type === 'query'),
    update: interfaces.filter(i => i.interface_type === 'update'),
    start_child: interfaces.filter(i => i.interface_type === 'start_child'),
  };

  return (
    <YStack gap="$4">
      <XStack ai="center" jc="space-between">
        <YStack gap="$1">
          <Text fontSize="$5" fontWeight="600">Service Interfaces</Text>
          <Text fontSize="$2" color="$gray11">
            Define how this service communicates with other services
          </Text>
        </YStack>
        <Button
          size="$3"
          icon={Plus}
          onPress={() => setShowCreateModal(true)}
        >
          Add Interface
        </Button>
      </XStack>

      {isLoading ? (
        <YStack ai="center" p="$6">
          <Spinner size="large" />
        </YStack>
      ) : interfaces.length === 0 ? (
        <Card p="$6" bg="$gray2" ai="center">
          <Text color="$gray11" textAlign="center" mb="$4">
            No interfaces defined yet. Add interfaces to enable service-to-service communication.
          </Text>
          <Button
            size="$3"
            icon={Plus}
            onPress={() => setShowCreateModal(true)}
          >
            Create Your First Interface
          </Button>
        </Card>
      ) : (
        <YStack gap="$4">
          {/* Send Action Interfaces (Signals) */}
          {interfacesByType.signal.length > 0 && (
            <YStack gap="$2">
              <XStack ai="center" gap="$2">
                <Send size={18} color="$blue11" />
                <Text fontSize="$4" fontWeight="600">
                  Send Action ({interfacesByType.signal.length})
                </Text>
              </XStack>
              <YStack gap="$2">
                {interfacesByType.signal.map((iface) => (
                  <InterfaceCard
                    key={iface.id}
                    interface={iface}
                    isEditing={editingId === iface.id}
                    onEdit={() => setEditingId(iface.id)}
                    onCancel={() => setEditingId(null)}
                    onDelete={() => {
                      // TODO: Implement delete
                    }}
                    getTypeIcon={getInterfaceTypeIcon}
                    getTypeLabel={getInterfaceTypeLabel}
                    getTypeColor={getInterfaceTypeColor}
                  />
                ))}
              </YStack>
            </YStack>
          )}

          {/* Get State Interfaces (Queries) */}
          {interfacesByType.query.length > 0 && (
            <YStack gap="$2">
              <XStack ai="center" gap="$2">
                <Search size={18} color="$green11" />
                <Text fontSize="$4" fontWeight="600">
                  Get State ({interfacesByType.query.length})
                </Text>
              </XStack>
              <YStack gap="$2">
                {interfacesByType.query.map((iface) => (
                  <InterfaceCard
                    key={iface.id}
                    interface={iface}
                    isEditing={editingId === iface.id}
                    onEdit={() => setEditingId(iface.id)}
                    onCancel={() => setEditingId(null)}
                    onDelete={() => {
                      // TODO: Implement delete
                    }}
                    getTypeIcon={getInterfaceTypeIcon}
                    getTypeLabel={getInterfaceTypeLabel}
                    getTypeColor={getInterfaceTypeColor}
                  />
                ))}
              </YStack>
            </YStack>
          )}

          {/* Modify State Interfaces (Updates) */}
          {interfacesByType.update.length > 0 && (
            <YStack gap="$2">
              <XStack ai="center" gap="$2">
                <Edit size={18} color="$orange11" />
                <Text fontSize="$4" fontWeight="600">
                  Modify State ({interfacesByType.update.length})
                </Text>
              </XStack>
              <YStack gap="$2">
                {interfacesByType.update.map((iface) => (
                  <InterfaceCard
                    key={iface.id}
                    interface={iface}
                    isEditing={editingId === iface.id}
                    onEdit={() => setEditingId(iface.id)}
                    onCancel={() => setEditingId(null)}
                    onDelete={() => {
                      // TODO: Implement delete
                    }}
                    getTypeIcon={getInterfaceTypeIcon}
                    getTypeLabel={getInterfaceTypeLabel}
                    getTypeColor={getInterfaceTypeColor}
                  />
                ))}
              </YStack>
            </YStack>
          )}

          {/* Start Service Interfaces (Child Workflows) */}
          {interfacesByType.start_child.length > 0 && (
            <YStack gap="$2">
              <XStack ai="center" gap="$2">
                <ArrowRight size={18} color="$purple11" />
                <Text fontSize="$4" fontWeight="600">
                  Start Service ({interfacesByType.start_child.length})
                </Text>
              </XStack>
              <YStack gap="$2">
                {interfacesByType.start_child.map((iface) => (
                  <InterfaceCard
                    key={iface.id}
                    interface={iface}
                    isEditing={editingId === iface.id}
                    onEdit={() => setEditingId(iface.id)}
                    onCancel={() => setEditingId(null)}
                    onDelete={() => {
                      // TODO: Implement delete
                    }}
                    getTypeIcon={getInterfaceTypeIcon}
                    getTypeLabel={getInterfaceTypeLabel}
                    getTypeColor={getInterfaceTypeColor}
                  />
                ))}
              </YStack>
            </YStack>
          )}
        </YStack>
      )}

      {showCreateModal && (
        <CreateInterfaceModal
          serviceId={serviceId}
          open={showCreateModal}
          onOpenChange={setShowCreateModal}
          onSuccess={() => {
            // TODO: utils.serviceInterfaces.list.invalidate({ serviceId });
            setShowCreateModal(false);
          }}
        />
      )}
    </YStack>
  );
}

interface InterfaceCardProps {
  interface: {
    id: string;
    name: string;
    display_name: string;
    description?: string;
    interface_type: string;
    temporal_callable_name: string;
    is_public?: boolean;
    public_interface?: {
      http_method: string;
      http_path: string;
      kong_route_id?: string;
    };
  };
  isEditing: boolean;
  onEdit: () => void;
  onCancel: () => void;
  onDelete: () => void;
  getTypeIcon: (type: string) => any;
  getTypeLabel: (type: string) => string;
  getTypeColor: (type: string) => string;
}

function InterfaceCard({
  interface: iface,
  isEditing,
  onEdit,
  onCancel,
  onDelete,
  getTypeIcon,
  getTypeLabel,
  getTypeColor,
}: InterfaceCardProps) {
  const Icon = getTypeIcon(iface.interface_type);

  if (isEditing) {
    // TODO: Implement edit form
    return (
      <Card p="$4" bg="$gray2">
        <Text>Edit form coming soon</Text>
        <Button onPress={onCancel}>Cancel</Button>
      </Card>
    );
  }

  return (
    <Card p="$4" bg="$gray2">
      <YStack gap="$3">
        <XStack ai="center" jc="space-between">
          <XStack ai="center" gap="$2">
            {Icon && <Icon size={18} color={getTypeColor(iface.interface_type)} />}
            <Text fontSize="$4" fontWeight="600">
              {iface.display_name}
            </Text>
            <Badge backgroundColor={getTypeColor(iface.interface_type)} size="2">
              <Text fontSize="$1" color="white">
                {getTypeLabel(iface.interface_type)}
              </Text>
            </Badge>
            {iface.is_public && (
              <Badge backgroundColor="$green10" size="2">
                <XStack ai="center" gap="$1">
                  <Globe size={12} />
                  <Text fontSize="$1" color="white">Public</Text>
                </XStack>
              </Badge>
            )}
          </XStack>
          <XStack gap="$2">
            <Button size="$2" icon={Edit} onPress={onEdit} chromeless />
            <Button size="$2" icon={Trash} onPress={onDelete} chromeless />
          </XStack>
        </XStack>

        {iface.description && (
          <Text fontSize="$2" color="$gray11">
            {iface.description}
          </Text>
        )}

        <YStack gap="$1">
          <Text fontSize="$1" color="$gray10" fontFamily="$mono">
            Temporal: {iface.temporal_callable_name}
          </Text>
          {iface.public_interface && (
            <Text fontSize="$1" color="$gray10" fontFamily="$mono">
              API: {iface.public_interface.http_method} {iface.public_interface.http_path}
            </Text>
          )}
        </YStack>
      </YStack>
    </Card>
  );
}

interface CreateInterfaceModalProps {
  serviceId: string;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSuccess: () => void;
}

function CreateInterfaceModal({
  serviceId,
  open,
  onOpenChange,
  onSuccess,
}: CreateInterfaceModalProps) {
  const [interfaceType, setInterfaceType] = useState<'signal' | 'query' | 'update' | 'start_child'>('signal');
  const [name, setName] = useState('');
  const [displayName, setDisplayName] = useState('');
  const [description, setDescription] = useState('');
  const [temporalCallableName, setTemporalCallableName] = useState('');
  const [payloadSchema, setPayloadSchema] = useState('{}');
  const [returnSchema, setReturnSchema] = useState('');
  const [isPublic, setIsPublic] = useState(false);

  const handleSave = () => {
    // TODO: When API is ready
    // createMutation.mutate({
    //   serviceId,
    //   interfaceType,
    //   name,
    //   displayName,
    //   description,
    //   temporalCallableName,
    //   payloadSchema: JSON.parse(payloadSchema),
    //   returnSchema: returnSchema ? JSON.parse(returnSchema) : undefined,
    //   isPublic,
    // });
    alert('Service interfaces API not yet implemented. This will create the interface when the backend is ready.');
    onSuccess();
  };

  return (
    <Card p="$4" bg="$gray2" maxWidth={600}>
      <YStack gap="$3">
        <Text fontSize="$4" fontWeight="600">Create Service Interface</Text>
        <Separator />

        <YStack gap="$2">
          <Label>Interface Type *</Label>
          <Select value={interfaceType} onValueChange={(v) => setInterfaceType(v as any)}>
            <Select.Trigger>
              <Select.Value />
            </Select.Trigger>
            <Select.Content>
              <Select.Viewport>
                <Select.Item index={0} value="signal">
                  <Select.ItemText>Send Action (Signal)</Select.ItemText>
                </Select.Item>
                <Select.Item index={1} value="query">
                  <Select.ItemText>Get State (Query)</Select.ItemText>
                </Select.Item>
                <Select.Item index={2} value="update">
                  <Select.ItemText>Modify State (Update)</Select.ItemText>
                </Select.Item>
                <Select.Item index={3} value="start_child">
                  <Select.ItemText>Start Service (Child Workflow)</Select.ItemText>
                </Select.Item>
              </Select.Viewport>
            </Select.Content>
          </Select>
        </YStack>

        <YStack gap="$2">
          <Label>Name *</Label>
          <Input
            value={name}
            onChangeText={setName}
            placeholder="e.g., updateStatus"
            fontFamily="$mono"
          />
        </YStack>

        <YStack gap="$2">
          <Label>Display Name *</Label>
          <Input
            value={displayName}
            onChangeText={setDisplayName}
            placeholder="e.g., Update Status"
          />
        </YStack>

        <YStack gap="$2">
          <Label>Description</Label>
          <TextArea
            value={description}
            onChangeText={setDescription}
            placeholder="Optional description"
            numberOfLines={2}
          />
        </YStack>

        <YStack gap="$2">
          <Label>Temporal Callable Name *</Label>
          <Input
            value={temporalCallableName}
            onChangeText={setTemporalCallableName}
            placeholder="e.g., updateStatusSignal"
            fontFamily="$mono"
          />
          <Text fontSize="$1" color="$gray11">
            The actual Temporal signal/query/update name
          </Text>
        </YStack>

        <YStack gap="$2">
          <Label>Payload Schema (JSON) *</Label>
          <TextArea
            value={payloadSchema}
            onChangeText={setPayloadSchema}
            placeholder='{"type": "object", "properties": {...}}'
            fontFamily="$mono"
            numberOfLines={4}
          />
        </YStack>

        {(interfaceType === 'query' || interfaceType === 'update') && (
          <YStack gap="$2">
            <Label>Return Schema (JSON)</Label>
            <TextArea
              value={returnSchema}
              onChangeText={setReturnSchema}
              placeholder='{"type": "object", "properties": {...}}'
              fontFamily="$mono"
              numberOfLines={4}
            />
          </YStack>
        )}

        <XStack ai="center" gap="$3">
          <input
            type="checkbox"
            checked={isPublic}
            onChange={(e) => setIsPublic(e.target.checked)}
          />
          <Label>Expose as public API (via Kong)</Label>
        </XStack>

        <XStack gap="$2" jc="flex-end">
          <Button onPress={() => onOpenChange(false)} chromeless>
            Cancel
          </Button>
          <Button
            themeInverse
            onPress={handleSave}
            disabled={!name || !displayName || !temporalCallableName}
          >
            Create Interface
          </Button>
        </XStack>
      </YStack>
    </Card>
  );
}

