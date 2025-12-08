import * as vscode from 'vscode';
import { sendSelection } from './commands/sendSelection';
import { applyCode } from './commands/applyCode';
import { chat } from './commands/chat';

export function activate(context: vscode.ExtensionContext) {
    console.log('Radium VS Code extension is now active');

    // Register commands
    const sendSelectionCommand = vscode.commands.registerCommand(
        'radium.sendSelection',
        sendSelection
    );

    const applyCodeCommand = vscode.commands.registerCommand(
        'radium.applyCode',
        applyCode
    );

    const chatCommand = vscode.commands.registerCommand(
        'radium.chat',
        chat
    );

    context.subscriptions.push(sendSelectionCommand);
    context.subscriptions.push(applyCodeCommand);
    context.subscriptions.push(chatCommand);
}

export function deactivate() {
    // Cleanup if needed
}

