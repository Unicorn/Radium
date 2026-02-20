/**
 * Workflow Toolbar - Save, deploy, and other actions
 */

'use client';

import { XStack, YStack, Button, Text, Separator } from 'tamagui';
import { Save, Play, Edit, AlertCircle, Undo, Redo, ArrowLeft, Code } from 'lucide-react';
import { useRouter } from 'next/navigation';

interface WorkflowToolbarProps {
  onSave: () => void;
  onCompile?: () => void;
  onDeploy: () => void;
  onEnterEditMode?: () => void;
  onUndo?: () => void;
  onRedo?: () => void;
  canUndo?: boolean;
  canRedo?: boolean;
  isSaving?: boolean;
  isCompiling?: boolean;
  isDeploying?: boolean;
  readOnly?: boolean;
}

export function WorkflowToolbar({
  onSave,
  onCompile,
  onDeploy,
  onEnterEditMode,
  onUndo,
  onRedo,
  canUndo = false,
  canRedo = false,
  isSaving = false,
  isCompiling = false,
  isDeploying = false,
  readOnly = false,
}: WorkflowToolbarProps) {
  const router = useRouter();

  return (
    <YStack>
      {/* Main Toolbar */}
      <XStack
        paddingHorizontal="$4"
        paddingVertical="$3"
        backgroundColor="$background"
        borderBottomWidth={1}
        borderColor="$borderColor"
        alignItems="center"
        gap="$3"
      >
        {/* Back Button */}
        <Button
          size="$3"
          icon={ArrowLeft}
          onPress={() => router.push('/workflows')}
          chromeless
        />

        <Separator vertical />

        {!readOnly && (
          <>
            <Button
              size="$3"
              icon={Save}
              onPress={onSave}
              disabled={isSaving || isCompiling || isDeploying}
              data-testid="save-workflow-button"
            >
              {isSaving ? 'Saving...' : 'Save'}
            </Button>

            <Separator vertical />

            {onCompile && (
              <>
                <Button
                  size="$3"
                  theme="purple"
                  icon={Code}
                  onPress={onCompile}
                  disabled={isSaving || isCompiling || isDeploying}
                  data-testid="compile-workflow-button"
                >
                  {isCompiling ? 'Compiling...' : 'Compile'}
                </Button>

                <Separator vertical />
              </>
            )}

            <Button
              size="$3"
              theme="blue"
              icon={Play}
              onPress={onDeploy}
              disabled={isSaving || isCompiling || isDeploying}
              data-testid="deploy-workflow-button"
            >
              {isDeploying ? 'Deploying...' : 'Deploy'}
            </Button>

            <Separator vertical />

            <Button
              size="$3"
              icon={Undo}
              onPress={onUndo}
              disabled={!canUndo || isSaving || isCompiling || isDeploying}
              opacity={canUndo ? 1 : 0.5}
              data-testid="undo-button"
              aria-label="Undo"
            />

            <Button
              size="$3"
              icon={Redo}
              onPress={onRedo}
              disabled={!canRedo || isSaving || isCompiling || isDeploying}
              opacity={canRedo ? 1 : 0.5}
              data-testid="redo-button"
              aria-label="Redo"
            />
          </>
        )}

        {readOnly && onEnterEditMode && (
          <Button
            size="$3"
            theme="orange"
            icon={Edit}
            onPress={onEnterEditMode}
          >
            Pause to Edit
          </Button>
        )}

        <XStack flex={1} />

        <Text fontSize="$2" color={readOnly ? "$orange10" : "$gray11"} fontWeight={readOnly ? "600" : "400"}>
          {readOnly ? '🔒 Read-only (Active Workflow)' : '✏️ Editing'}
        </Text>
      </XStack>

      {/* Read-only Warning Banner */}
      {readOnly && (
        <XStack
          paddingHorizontal="$4"
          paddingVertical="$3"
          backgroundColor="$orange2"
          borderBottomWidth={1}
          borderColor="$orange6"
          alignItems="center"
          gap="$3"
        >
          <AlertCircle size={20} color="$orange10" />
          <YStack flex={1}>
            <Text fontSize="$3" fontWeight="600" color="$orange11">
              Workflow is Active
            </Text>
            <Text fontSize="$2" color="$orange10">
              This workflow is currently deployed and running. Pause it to make changes.
            </Text>
          </YStack>
          {onEnterEditMode && (
            <Button
              size="$2"
              theme="orange"
              icon={Edit}
              onPress={onEnterEditMode}
            >
              Pause to Edit
            </Button>
          )}
        </XStack>
      )}
    </YStack>
  );
}

