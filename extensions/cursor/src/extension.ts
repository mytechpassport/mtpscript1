import * as path from 'path';
import { workspace, ExtensionContext, commands } from 'vscode';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind
} from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: ExtensionContext) {
    // The server is implemented in C and runs as a separate process
    const serverPath = workspace.getConfiguration('mtpscript').get('server.path', 'mtpsc');

    // Server options - run the MTPScript CLI with lsp command
    const serverOptions: ServerOptions = {
        command: serverPath,
        args: ['lsp'],
        options: {
            cwd: workspace.workspaceFolders?.[0]?.uri.fsPath
        }
    };

    // Options to control the language client
    const clientOptions: LanguageClientOptions = {
        // Register the server for MTPScript documents
        documentSelector: [{ scheme: 'file', language: 'mtpscript' }],
        synchronize: {
            // Notify the server about file changes to '.mtp' files
            fileEvents: workspace.createFileSystemWatcher('**/*.mtp')
        },
    };

    // Create the language client and start the client.
    client = new LanguageClient(
        'mtpscriptLanguageServer',
        'MTPScript Language Server',
        serverOptions,
        clientOptions
    );

    // Start the client. This will also launch the server
    client.start();

    // Register Cursor-specific commands
    context.subscriptions.push(
        commands.registerCommand('mtpscript.compile', async () => {
            const activeEditor = workspace.activeTextEditor;
            if (activeEditor && activeEditor.document.languageId === 'mtpscript') {
                // Run mtpsc compile on the current file
                const terminal = workspace.createTerminal('MTPScript');
                terminal.show();
                terminal.sendText(`mtpsc compile ${activeEditor.document.fileName}`);
            }
        }),

        commands.registerCommand('mtpscript.run', async () => {
            const activeEditor = workspace.activeTextEditor;
            if (activeEditor && activeEditor.document.languageId === 'mtpscript') {
                // Run mtpsc run on the current file
                const terminal = workspace.createTerminal('MTPScript');
                terminal.show();
                terminal.sendText(`mtpsc run ${activeEditor.document.fileName}`);
            }
        }),

        commands.registerCommand('mtpscript.serve', async () => {
            const activeEditor = workspace.activeTextEditor;
            if (activeEditor && activeEditor.document.languageId === 'mtpscript') {
                // Run mtpsc serve on the current file
                const terminal = workspace.createTerminal('MTPScript Server');
                terminal.show();
                terminal.sendText(`mtpsc serve ${activeEditor.document.fileName}`);
            }
        }),

        client,
    );
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
