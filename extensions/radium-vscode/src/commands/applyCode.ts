import * as vscode from 'vscode';
import { getLastAgentOutput } from './sendSelection';

/**
 * Parse code blocks from markdown
 */
function parseCodeBlocks(markdown: string): Array<{ language?: string; content: string }> {
    const codeBlocks: Array<{ language?: string; content: string }> = [];
    const regex = /```(\w*)\n([\s\S]*?)```/g;
    
    let match;
    while ((match = regex.exec(markdown)) !== null) {
        codeBlocks.push({
            language: match[1] || undefined,
            content: match[2].trim()
        });
    }
    
    return codeBlocks;
}

/**
 * Apply code command implementation
 */
export async function applyCode(): Promise<void> {
    const output = getLastAgentOutput();
    
    if (!output) {
        vscode.window.showWarningMessage(
            'No agent output found. Run "Radium: Send Selection" first.'
        );
        return;
    }

    const editor = vscode.window.activeTextEditor;
    if (!editor) {
        vscode.window.showWarningMessage('No active editor');
        return;
    }

    try {
        // Parse code blocks
        const codeBlocks = parseCodeBlocks(output);
        
        if (codeBlocks.length === 0) {
            vscode.window.showWarningMessage('No code blocks found in agent output.');
            return;
        }

        // If multiple blocks, let user choose
        let selectedCode: string;
        if (codeBlocks.length > 1) {
            const items = codeBlocks.map((block, index) => ({
                label: `Code block ${index + 1}`,
                description: block.language || 'text',
                detail: block.content.substring(0, 100) + '...',
                block
            }));

            const selection = await vscode.window.showQuickPick(items, {
                placeHolder: 'Select code block to apply'
            });

            if (!selection) {
                return; // User cancelled
            }

            selectedCode = selection.block.content;
        } else {
            selectedCode = codeBlocks[0].content;
        }

        // Get current selection or cursor position
        const selection = editor.selection;
        const document = editor.document;

        // Prepare diff preview
        const currentText = selection.isEmpty 
            ? document.getText() 
            : document.getText(selection);

        // Show diff using VS Code's built-in diff viewer
        const uri = document.uri;
        const tempUri = uri.with({ scheme: 'radium-temp' });
        
        // Create temporary document with new content
        const tempDoc = await vscode.workspace.openTextDocument({
            content: selectedCode,
            language: document.languageId
        });

        // Show diff view
        await vscode.commands.executeCommand(
            'vscode.diff',
            uri,
            tempUri,
            'Original ↔ Modified'
        );

        // Prompt for confirmation
        const action = await vscode.window.showInformationMessage(
            'Apply changes?',
            'Apply',
            'Cancel'
        );

        if (action === 'Apply') {
            // Apply changes
            if (selection.isEmpty) {
                // Replace entire document
                const edit = new vscode.WorkspaceEdit();
                edit.replace(uri, new vscode.Range(0, 0, document.lineCount, 0), selectedCode);
                await vscode.workspace.applyEdit(edit);
            } else {
                // Replace selection
                await editor.edit((editBuilder) => {
                    editBuilder.replace(selection, selectedCode);
                });
            }

            vscode.window.showInformationMessage('Code applied successfully.');
        }
    } catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        vscode.window.showErrorMessage(`Error applying code: ${message}`);
    }
}

