import * as vscode from 'vscode';
import { getEditorContext } from '../utils/context';

/**
 * Chat command implementation
 */
export async function chat(): Promise<void> {
    try {
        // Get default agent ID from config
        const config = vscode.workspace.getConfiguration('radium');
        const agentId = config.get<string>('defaultAgent', 'code-agent');

        // Get current editor context for environment variables
        let context: any = {};
        try {
            context = getEditorContext();
        } catch {
            // No active editor, continue without context
        }

        // Create terminal for chat session
        const terminal = vscode.window.createTerminal({
            name: 'Radium Chat',
            env: {
                ...process.env,
                RADIUM_EDITOR_FILE_PATH: context.filePath || '',
                RADIUM_EDITOR_LANGUAGE: context.language || ''
            }
        });

        terminal.show();
        terminal.sendText(`rad chat ${agentId}`);

        vscode.window.showInformationMessage(
            'Radium chat session started. Type your messages in the terminal.'
        );
    } catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        vscode.window.showErrorMessage(`Error starting chat: ${message}`);
    }
}

