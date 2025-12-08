import * as vscode from 'vscode';
import { getEditorContext } from '../utils/context';
import { executeRadiumCommand } from '../utils/cli';

const outputChannel = vscode.window.createOutputChannel('Radium');

// Store last agent output globally
let lastAgentOutput: string | undefined;

export function getLastAgentOutput(): string | undefined {
    return lastAgentOutput;
}

export function setLastAgentOutput(output: string): void {
    lastAgentOutput = output;
}

/**
 * Send selection command implementation
 */
export async function sendSelection(): Promise<void> {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
        vscode.window.showWarningMessage('No active editor');
        return;
    }

    const selection = editor.selection;
    if (selection.isEmpty) {
        vscode.window.showWarningMessage('No selection. Please select code first.');
        return;
    }

    try {
        // Get editor context
        const context = getEditorContext();

        // Get default agent ID from config
        const config = vscode.workspace.getConfiguration('radium');
        const agentId = config.get<string>('defaultAgent', 'code-agent');

        vscode.window.showInformationMessage('Sending selection to Radium...');
        outputChannel.show();

        // Execute rad step command
        const output = await executeRadiumCommand(`step ${agentId}`, context, outputChannel);

        // Store output for apply command
        setLastAgentOutput(output);

        // Show output in a new document
        const doc = await vscode.workspace.openTextDocument({
            content: output,
            language: 'markdown'
        });
        await vscode.window.showTextDocument(doc, vscode.ViewColumn.Beside);

        vscode.window.showInformationMessage(
            'Radium processing complete. Use "Radium: Apply Code" to apply changes.'
        );
    } catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        vscode.window.showErrorMessage(`Radium error: ${message}`);
        outputChannel.appendLine(`Error: ${message}`);
    }
}

