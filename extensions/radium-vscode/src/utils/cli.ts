import * as vscode from 'vscode';
import * as child_process from 'child_process';
import { EditorContext } from './context';

/**
 * Execute Radium CLI command with context
 */
export function executeRadiumCommand(
    command: string,
    context: EditorContext,
    outputChannel: vscode.OutputChannel
): Promise<string> {
    return new Promise((resolve, reject) => {
        // Set environment variables for hook
        const env = {
            ...process.env,
            RADIUM_EDITOR_FILE_PATH: context.filePath,
            RADIUM_EDITOR_LANGUAGE: context.language,
            RADIUM_EDITOR_SELECTION: context.selection,
            RADIUM_EDITOR_SURROUNDING_LINES: context.surroundingLines
        };

        // Format context as JSON for stdin
        const contextJson = JSON.stringify(context);
        const fullCommand = `rad ${command}`;

        outputChannel.appendLine(`Executing: ${fullCommand}`);
        outputChannel.appendLine(`Context: ${contextJson}`);

        // Spawn process with context in stdin
        const process = child_process.spawn('rad', command.split(' '), {
            env,
            stdio: ['pipe', 'pipe', 'pipe']
        });

        // Send context to stdin
        process.stdin.write(contextJson);
        process.stdin.end();

        let stdout = '';
        let stderr = '';

        process.stdout.on('data', (data) => {
            const text = data.toString();
            stdout += text;
            outputChannel.append(text);
        });

        process.stderr.on('data', (data) => {
            const text = data.toString();
            stderr += text;
            outputChannel.append(text);
        });

        process.on('close', (code) => {
            if (code === 0) {
                resolve(stdout);
            } else {
                reject(new Error(`Command failed with code ${code}: ${stderr}`));
            }
        });

        process.on('error', (error) => {
            reject(new Error(`Failed to execute command: ${error.message}`));
        });
    });
}

/**
 * Parse JSON output from CLI
 */
export function parseOutput(output: string): any {
    try {
        return JSON.parse(output);
    } catch {
        return output;
    }
}

