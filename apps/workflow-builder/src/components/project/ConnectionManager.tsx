'use client';

import { YStack, XStack, Text, Button, Card, Input, Label, Select, Adapt, Sheet, Separator, Spinner } from 'tamagui';
import { Plus, Trash2, TestTube, Edit, Check, X } from 'lucide-react';
import { useState } from 'react';
import { api } from '@/lib/trpc/client';

interface ConnectionManagerProps {
  projectId: string;
}

export function ConnectionManager({ projectId }: ConnectionManagerProps) {
  const utils = api.useUtils();
  const [editingId, setEditingId] = useState<string | null>(null);
  const [showAddForm, setShowAddForm] = useState(false);

  const { data, isLoading } = api.connections.list.useQuery({ projectId });
  const createMutation = api.connections.create.useMutation({
    onSuccess: () => {
      utils.connections.list.invalidate({ projectId });
      setShowAddForm(false);
    },
  });
  const updateMutation = api.connections.update.useMutation({
    onSuccess: () => {
      utils.connections.list.invalidate({ projectId });
      setEditingId(null);
    },
  });
  const deleteMutation = api.connections.delete.useMutation({
    onSuccess: () => {
      utils.connections.list.invalidate({ projectId });
    },
  });
  const testMutation = api.connections.test.useMutation();

  const connections = data?.connections || [];

  return (
    <YStack gap="$4">
      <XStack ai="center" jc="space-between">
        <Text fontSize="$5" fontWeight="600">Project Connections</Text>
        <Button
          size="$3"
          icon={Plus}
          onPress={() => setShowAddForm(true)}
        >
          Add Connection
        </Button>
      </XStack>

      {isLoading ? (
        <YStack ai="center" p="$6">
          <Spinner size="large" />
        </YStack>
      ) : connections.length === 0 ? (
        <Card p="$6" bg="$gray2" ai="center">
          <Text color="$gray11" textAlign="center">
            No connections yet. Add a connection to use PostgreSQL or Redis in your workflows.
          </Text>
        </Card>
      ) : (
        <YStack gap="$2">
          {connections.map((conn) => (
            <ConnectionCard
              key={conn.id}
              connection={conn}
              isEditing={editingId === conn.id}
              onEdit={() => setEditingId(conn.id)}
              onCancel={() => setEditingId(null)}
              onUpdate={(updates) => {
                updateMutation.mutate({
                  connectionId: conn.id,
                  ...updates,
                });
              }}
              onDelete={() => {
                if (confirm(`Delete connection "${conn.name}"?`)) {
                  deleteMutation.mutate({ connectionId: conn.id });
                }
              }}
              onTest={() => {
                testMutation.mutate({ connectionId: conn.id });
              }}
              testResult={testMutation.data}
              isTesting={testMutation.isLoading}
            />
          ))}
        </YStack>
      )}

      {showAddForm && (
        <AddConnectionForm
          projectId={projectId}
          onSave={(data) => {
            createMutation.mutate(data);
          }}
          onCancel={() => setShowAddForm(false)}
          isLoading={createMutation.isLoading}
        />
      )}
    </YStack>
  );
}

interface ConnectionCardProps {
  connection: {
    id: string;
    name: string;
    connectionType: 'postgresql' | 'redis';
    connectionUrl: string;
  };
  isEditing: boolean;
  onEdit: () => void;
  onCancel: () => void;
  onUpdate: (updates: { name?: string; connectionUrl?: string }) => void;
  onDelete: () => void;
  onTest: () => void;
  testResult?: { success: boolean; message: string; error?: string };
  isTesting: boolean;
}

function ConnectionCard({
  connection,
  isEditing,
  onEdit,
  onCancel,
  onUpdate,
  onDelete,
  onTest,
  testResult,
  isTesting,
}: ConnectionCardProps) {
  const [name, setName] = useState(connection.name);
  const [url, setUrl] = useState(connection.connectionUrl);

  if (isEditing) {
    return (
      <Card p="$4" bg="$gray2">
        <YStack gap="$3">
          <YStack gap="$2">
            <Label>Connection Name</Label>
            <Input
              value={name}
              onChangeText={setName}
              placeholder="e.g., Production DB"
            />
          </YStack>
          <YStack gap="$2">
            <Label>Connection URL</Label>
            <Input
              value={url}
              onChangeText={setUrl}
              placeholder="postgresql://user:pass@host:port/db"
              fontFamily="$mono"
            />
          </YStack>
          <XStack gap="$2" jc="flex-end">
            <Button size="$2" onPress={onCancel} chromeless>
              Cancel
            </Button>
            <Button
              size="$2"
              onPress={() => {
                onUpdate({ name, connectionUrl: url });
              }}
              disabled={!name || !url}
            >
              Save
            </Button>
          </XStack>
        </YStack>
      </Card>
    );
  }

  return (
    <Card p="$4" bg="$gray2">
      <YStack gap="$3">
        <XStack ai="center" jc="space-between">
          <YStack gap="$1">
            <XStack ai="center" gap="$2">
              <Text fontSize="$4" fontWeight="600">{connection.name}</Text>
              <Text fontSize="$2" color="$gray11" textTransform="uppercase">
                {connection.connectionType}
              </Text>
            </XStack>
            <Text fontSize="$2" color="$gray11" fontFamily="$mono" numberOfLines={1}>
              {connection.connectionUrl.replace(/:[^:@]+@/, ':****@')}
            </Text>
          </YStack>
          <XStack gap="$2">
            <Button
              size="$2"
              icon={TestTube}
              onPress={onTest}
              disabled={isTesting}
            >
              {isTesting ? 'Testing...' : 'Test'}
            </Button>
            <Button size="$2" icon={Edit} onPress={onEdit} chromeless />
            <Button size="$2" icon={Trash2} onPress={onDelete} chromeless />
          </XStack>
        </XStack>

        {testResult && (
          <Card
            p="$3"
            bg={testResult.success ? '$green2' : '$red2'}
            borderWidth={1}
            borderColor={testResult.success ? '$green6' : '$red6'}
          >
            <XStack ai="center" gap="$2">
              {testResult.success ? (
                <Check size={16} color="$green11" />
              ) : (
                <X size={16} color="$red11" />
              )}
              <Text
                fontSize="$2"
                color={testResult.success ? '$green11' : '$red11'}
              >
                {testResult.message}
              </Text>
            </XStack>
            {testResult.error && (
              <Text fontSize="$1" color="$red11" mt="$2">
                {testResult.error}
              </Text>
            )}
          </Card>
        )}
      </YStack>
    </Card>
  );
}

interface AddConnectionFormProps {
  projectId: string;
  onSave: (data: {
    projectId: string;
    connectionType: 'postgresql' | 'redis';
    name: string;
    connectionUrl: string;
  }) => void;
  onCancel: () => void;
  isLoading: boolean;
}

function AddConnectionForm({
  projectId,
  onSave,
  onCancel,
  isLoading,
}: AddConnectionFormProps) {
  const [connectionType, setConnectionType] = useState<'postgresql' | 'redis'>('postgresql');
  const [name, setName] = useState('');
  const [url, setUrl] = useState('');

  return (
    <Card p="$4" bg="$gray2">
      <YStack gap="$3">
        <Text fontSize="$4" fontWeight="600">Add Connection</Text>
        <Separator />
        
        <YStack gap="$2">
          <Label>Connection Type</Label>
          <Select value={connectionType} onValueChange={(v) => setConnectionType(v as 'postgresql' | 'redis')}>
            <Select.Trigger>
              <Select.Value />
            </Select.Trigger>
            <Select.Content>
              <Select.Viewport>
                <Select.Item index={0} value="postgresql">
                  <Select.ItemText>PostgreSQL</Select.ItemText>
                </Select.Item>
                <Select.Item index={1} value="redis">
                  <Select.ItemText>Redis</Select.ItemText>
                </Select.Item>
              </Select.Viewport>
            </Select.Content>
          </Select>
        </YStack>

        <YStack gap="$2">
          <Label>Connection Name</Label>
          <Input
            value={name}
            onChangeText={setName}
            placeholder="e.g., Production DB"
          />
        </YStack>

        <YStack gap="$2">
          <Label>Connection URL</Label>
          <Input
            value={url}
            onChangeText={setUrl}
            placeholder={
              connectionType === 'postgresql'
                ? 'postgresql://user:pass@host:port/db'
                : 'redis://user:pass@host:port'
            }
            fontFamily="$mono"
          />
        </YStack>

        <XStack gap="$2" jc="flex-end">
          <Button onPress={onCancel} chromeless disabled={isLoading}>
            Cancel
          </Button>
          <Button
            onPress={() => {
              onSave({ projectId, connectionType, name, connectionUrl: url });
            }}
            disabled={!name || !url || isLoading}
            themeInverse
          >
            {isLoading ? 'Creating...' : 'Create'}
          </Button>
        </XStack>
      </YStack>
    </Card>
  );
}

