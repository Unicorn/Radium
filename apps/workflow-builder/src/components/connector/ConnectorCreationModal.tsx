'use client';

import { YStack, XStack, Text, Button, Input, Label, Select, Separator, Dialog, TextArea } from 'tamagui';
import { api } from '@/lib/trpc/client';
import { useState } from 'react';

interface ConnectorCreationModalProps {
  projectId: string;
  connectorType: 'email' | 'slack' | 'database' | 'api' | 'oauth';
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSuccess: (connectorId: string) => void;
}

/**
 * Modal for creating new connectors.
 * 
 * Supports different connector types with type-specific configuration forms.
 */
export function ConnectorCreationModal({
  projectId,
  connectorType,
  open,
  onOpenChange,
  onSuccess,
}: ConnectorCreationModalProps) {
  const utils = api.useUtils();
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  
  // Type-specific state
  const [emailProvider, setEmailProvider] = useState<'sendgrid' | 'smtp'>('sendgrid');
  const [sendgridApiKey, setSendgridApiKey] = useState('');
  const [smtpHost, setSmtpHost] = useState('');
  const [smtpPort, setSmtpPort] = useState('');
  const [smtpUser, setSmtpUser] = useState('');
  const [smtpPassword, setSmtpPassword] = useState('');
  
  const [slackWebhookUrl, setSlackWebhookUrl] = useState('');
  const [slackBotToken, setSlackBotToken] = useState('');
  
  const [databaseType, setDatabaseType] = useState<'postgresql' | 'mysql' | 'redis'>('postgresql');
  const [databaseUrl, setDatabaseUrl] = useState('');
  
  const [apiBaseUrl, setApiBaseUrl] = useState('');
  const [apiKey, setApiKey] = useState('');
  const [apiHeaders, setApiHeaders] = useState('{}');
  
  const [oauthProvider, setOauthProvider] = useState<'google' | 'github' | 'custom'>('google');
  const [oauthClientId, setOauthClientId] = useState('');
  const [oauthClientSecret, setOauthClientSecret] = useState('');
  const [oauthAuthUrl, setOauthAuthUrl] = useState('');
  const [oauthTokenUrl, setOauthTokenUrl] = useState('');

  // For database connectors, use existing connections API
  const isDatabaseConnector = connectorType === 'database';
  
  const createConnectionMutation = api.connections.create.useMutation({
    onSuccess: (data) => {
      utils.connections.list.invalidate({ projectId });
      onSuccess(data.id);
      resetForm();
    },
  });

  // TODO: When connectors API is implemented, use it for non-database connectors
  // const createConnectorMutation = api.connectors.create.useMutation({
  //   onSuccess: (data) => {
  //     utils.connectors.list.invalidate({ projectId, connectorType });
  //     onSuccess(data.id);
  //     resetForm();
  //   },
  // });

  const resetForm = () => {
    setName('');
    setDescription('');
    setEmailProvider('sendgrid');
    setSendgridApiKey('');
    setSmtpHost('');
    setSmtpPort('');
    setSmtpUser('');
    setSmtpPassword('');
    setSlackWebhookUrl('');
    setSlackBotToken('');
    setDatabaseType('postgresql');
    setDatabaseUrl('');
    setApiBaseUrl('');
    setApiKey('');
    setApiHeaders('{}');
    setOauthProvider('google');
    setOauthClientId('');
    setOauthClientSecret('');
    setOauthAuthUrl('');
    setOauthTokenUrl('');
  };

  const handleSave = () => {
    if (!name.trim()) return;

    if (isDatabaseConnector) {
      // Use existing connections API for database connectors
      const connectionType = databaseType === 'postgresql' ? 'postgresql' : 
                           databaseType === 'mysql' ? 'postgresql' : // TODO: Add mysql support
                           'redis';
      
      createConnectionMutation.mutate({
        projectId,
        connectionType: connectionType as 'postgresql' | 'redis',
        name,
        connectionUrl: databaseUrl,
      });
    } else {
      // TODO: When connectors API is implemented
      // const config: Record<string, any> = {};
      // 
      // if (connectorType === 'email') {
      //   if (emailProvider === 'sendgrid') {
      //     config.apiKey = sendgridApiKey;
      //   } else {
      //     config.host = smtpHost;
      //     config.port = parseInt(smtpPort);
      //     config.user = smtpUser;
      //     config.password = smtpPassword;
      //   }
      // } else if (connectorType === 'slack') {
      //   config.webhookUrl = slackWebhookUrl;
      //   config.botToken = slackBotToken;
      // } else if (connectorType === 'api') {
      //   config.baseUrl = apiBaseUrl;
      //   config.apiKey = apiKey;
      //   try {
      //     config.headers = JSON.parse(apiHeaders);
      //   } catch {
      //     // Invalid JSON, ignore
      //   }
      // } else if (connectorType === 'oauth') {
      //   config.provider = oauthProvider;
      //   config.clientId = oauthClientId;
      //   config.clientSecret = oauthClientSecret;
      //   if (oauthProvider === 'custom') {
      //     config.authUrl = oauthAuthUrl;
      //     config.tokenUrl = oauthTokenUrl;
      //   }
      // }
      // 
      // createConnectorMutation.mutate({
      //   projectId,
      //   connectorType,
      //   name,
      //   description,
      //   config,
      // });
      
      // For now, show a message that this is not yet implemented
      alert(`Connector creation for ${connectorType} is not yet implemented. The connectors API needs to be created first.`);
    }
  };

  const isLoading = createConnectionMutation.isLoading; // TODO: || createConnectorMutation.isLoading

  return (
    <Dialog modal open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay
          key="overlay"
          animation="quick"
          opacity={0.5}
          enterStyle={{ opacity: 0 }}
          exitStyle={{ opacity: 0 }}
        />
        <Dialog.Content
          bordered
          elevate
          key="content"
          animateOnly={['transform', 'opacity']}
          animation={[
            'quick',
            {
              opacity: {
                overshootClamping: true,
              },
            },
          ]}
          enterStyle={{ x: 0, y: -20, opacity: 0, scale: 0.9 }}
          exitStyle={{ x: 0, y: 10, opacity: 0, scale: 0.95 }}
          gap="$4"
          maxWidth={600}
        >
          <Dialog.Title>Create {connectorType === 'database' ? 'Database' :
                           connectorType === 'email' ? 'Email' :
                           connectorType === 'slack' ? 'Slack' :
                           connectorType === 'api' ? 'API' :
                           'OAuth'} Connector</Dialog.Title>

          <YStack gap="$3" maxHeight="70vh" overflow="scroll">
            {/* Common fields */}
            <YStack gap="$2">
              <Label>Name *</Label>
              <Input
                value={name}
                onChangeText={setName}
                placeholder={`e.g., ${connectorType === 'email' ? 'SendGrid Production' :
                             connectorType === 'slack' ? 'Slack Notifications' :
                             connectorType === 'database' ? 'Main PostgreSQL' :
                             connectorType === 'api' ? 'External API' :
                             'OAuth Provider'}`}
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

            <Separator />

            {/* Type-specific configuration */}
            {connectorType === 'database' && (
              <>
                <YStack gap="$2">
                  <Label>Database Type</Label>
                  <Select value={databaseType} onValueChange={(v) => setDatabaseType(v as 'postgresql' | 'mysql' | 'redis')}>
                    <Select.Trigger>
                      <Select.Value />
                    </Select.Trigger>
                    <Select.Content>
                      <Select.Viewport>
                        <Select.Item index={0} value="postgresql">
                          <Select.ItemText>PostgreSQL</Select.ItemText>
                        </Select.Item>
                        <Select.Item index={1} value="mysql">
                          <Select.ItemText>MySQL</Select.ItemText>
                        </Select.Item>
                        <Select.Item index={2} value="redis">
                          <Select.ItemText>Redis</Select.ItemText>
                        </Select.Item>
                      </Select.Viewport>
                    </Select.Content>
                  </Select>
                </YStack>

                <YStack gap="$2">
                  <Label>Connection URL *</Label>
                  <Input
                    value={databaseUrl}
                    onChangeText={setDatabaseUrl}
                    placeholder="postgresql://user:pass@host:port/db"
                    fontFamily="$mono"
                  />
                </YStack>
              </>
            )}

            {connectorType === 'email' && (
              <>
                <YStack gap="$2">
                  <Label>Email Provider</Label>
                  <Select value={emailProvider} onValueChange={(v) => setEmailProvider(v as 'sendgrid' | 'smtp')}>
                    <Select.Trigger>
                      <Select.Value />
                    </Select.Trigger>
                    <Select.Content>
                      <Select.Viewport>
                        <Select.Item index={0} value="sendgrid">
                          <Select.ItemText>SendGrid</Select.ItemText>
                        </Select.Item>
                        <Select.Item index={1} value="smtp">
                          <Select.ItemText>SMTP</Select.ItemText>
                        </Select.Item>
                      </Select.Viewport>
                    </Select.Content>
                  </Select>
                </YStack>

                {emailProvider === 'sendgrid' ? (
                  <YStack gap="$2">
                    <Label>SendGrid API Key *</Label>
                    <Input
                      value={sendgridApiKey}
                      onChangeText={setSendgridApiKey}
                      placeholder="SG.xxxxxxxxxxxxx"
                      secureTextEntry
                    />
                  </YStack>
                ) : (
                  <>
                    <YStack gap="$2">
                      <Label>SMTP Host *</Label>
                      <Input value={smtpHost} onChangeText={setSmtpHost} placeholder="smtp.example.com" />
                    </YStack>
                    <XStack gap="$2">
                      <YStack gap="$2" flex={1}>
                        <Label>Port *</Label>
                        <Input value={smtpPort} onChangeText={setSmtpPort} placeholder="587" keyboardType="numeric" />
                      </YStack>
                      <YStack gap="$2" flex={1}>
                        <Label>Username</Label>
                        <Input value={smtpUser} onChangeText={setSmtpUser} />
                      </YStack>
                    </XStack>
                    <YStack gap="$2">
                      <Label>Password</Label>
                      <Input value={smtpPassword} onChangeText={setSmtpPassword} secureTextEntry />
                    </YStack>
                  </>
                )}
              </>
            )}

            {connectorType === 'slack' && (
              <>
                <YStack gap="$2">
                  <Label>Webhook URL</Label>
                  <Input
                    value={slackWebhookUrl}
                    onChangeText={setSlackWebhookUrl}
                    placeholder="https://hooks.slack.com/services/..."
                    fontFamily="$mono"
                  />
                </YStack>
                <YStack gap="$2">
                  <Label>Bot Token (alternative)</Label>
                  <Input
                    value={slackBotToken}
                    onChangeText={setSlackBotToken}
                    placeholder="xoxb-..."
                    secureTextEntry
                  />
                  <Text fontSize="$2" color="$gray11">
                    Use either webhook URL or bot token
                  </Text>
                </YStack>
              </>
            )}

            {connectorType === 'api' && (
              <>
                <YStack gap="$2">
                  <Label>Base URL *</Label>
                  <Input
                    value={apiBaseUrl}
                    onChangeText={setApiBaseUrl}
                    placeholder="https://api.example.com"
                    fontFamily="$mono"
                  />
                </YStack>
                <YStack gap="$2">
                  <Label>API Key</Label>
                  <Input
                    value={apiKey}
                    onChangeText={setApiKey}
                    placeholder="Optional API key"
                    secureTextEntry
                  />
                </YStack>
                <YStack gap="$2">
                  <Label>Custom Headers (JSON)</Label>
                  <TextArea
                    value={apiHeaders}
                    onChangeText={setApiHeaders}
                    placeholder='{"Authorization": "Bearer token"}'
                    fontFamily="$mono"
                    numberOfLines={3}
                  />
                </YStack>
              </>
            )}

            {connectorType === 'oauth' && (
              <>
                <YStack gap="$2">
                  <Label>OAuth Provider</Label>
                  <Select value={oauthProvider} onValueChange={(v) => setOauthProvider(v as 'google' | 'github' | 'custom')}>
                    <Select.Trigger>
                      <Select.Value />
                    </Select.Trigger>
                    <Select.Content>
                      <Select.Viewport>
                        <Select.Item index={0} value="google">
                          <Select.ItemText>Google</Select.ItemText>
                        </Select.Item>
                        <Select.Item index={1} value="github">
                          <Select.ItemText>GitHub</Select.ItemText>
                        </Select.Item>
                        <Select.Item index={2} value="custom">
                          <Select.ItemText>Custom</Select.ItemText>
                        </Select.Item>
                      </Select.Viewport>
                    </Select.Content>
                  </Select>
                </YStack>
                <YStack gap="$2">
                  <Label>Client ID *</Label>
                  <Input value={oauthClientId} onChangeText={setOauthClientId} />
                </YStack>
                <YStack gap="$2">
                  <Label>Client Secret *</Label>
                  <Input value={oauthClientSecret} onChangeText={setOauthClientSecret} secureTextEntry />
                </YStack>
                {oauthProvider === 'custom' && (
                  <>
                    <YStack gap="$2">
                      <Label>Authorization URL *</Label>
                      <Input value={oauthAuthUrl} onChangeText={setOauthAuthUrl} placeholder="https://..." />
                    </YStack>
                    <YStack gap="$2">
                      <Label>Token URL *</Label>
                      <Input value={oauthTokenUrl} onChangeText={setOauthTokenUrl} placeholder="https://..." />
                    </YStack>
                  </>
                )}
              </>
            )}
          </YStack>

          <XStack gap="$3" justifyContent="flex-end" marginTop="$4">
            <Dialog.Close displayWhenAdapted asChild>
              <Button variant="outlined" onPress={resetForm}>
                Cancel
              </Button>
            </Dialog.Close>
            <Button
              themeInverse
              onPress={handleSave}
              disabled={isLoading || !name.trim() || 
                       (isDatabaseConnector && !databaseUrl.trim()) ||
                       (!isDatabaseConnector && !name.trim())} // TODO: Add validation for other types when connectors API is ready
            >
              {isLoading ? 'Creating...' : 'Create Connector'}
            </Button>
          </XStack>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog>
  );
}

