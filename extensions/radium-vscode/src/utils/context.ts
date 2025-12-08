import * as vscode from 'vscode';

export interface EditorContext {
    filePath: string;
    language: string;
    selection: string;
    surroundingLines: string;
    workspace?: string;
}

/**
 * Get editor context for Radium requests
 */
export function getEditorContext(): EditorContext {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
        throw new Error('No active editor');
    }

    const document = editor.document;
    const selection = editor.selection;
    const filePath = document.fileName;
    const language = document.languageId;

    // Get selected text
    const selectedText = document.getText(selection);

    // Get surrounding lines for context
    const contextLines = 3;
    const startLine = Math.max(0, selection.start.line - contextLines);
    const endLine = Math.min(document.lineCount - 1, selection.end.line + contextLines);
    
    const beforeLines: string[] = [];
    const afterLines: string[] = [];
    
    for (let i = startLine; i < selection.start.line; i++) {
        beforeLines.push(document.lineAt(i).text);
    }
    
    for (let i = selection.end.line + 1; i <= endLine; i++) {
        afterLines.push(document.lineAt(i).text);
    }
    
    const surroundingLines = beforeLines.join('\n') + '\n---\n' + afterLines.join('\n');

    // Get workspace folder
    const workspaceFolder = vscode.workspace.getWorkspaceFolder(document.uri);
    const workspace = workspaceFolder?.uri.fsPath;

    return {
        filePath,
        language,
        selection: selectedText,
        surroundingLines,
        workspace
    };
}

/**
 * Get selection text from active editor
 */
export function getSelection(): string {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
        return '';
    }
    return editor.document.getText(editor.selection);
}

