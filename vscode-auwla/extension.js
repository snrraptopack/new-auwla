const path = require('path');
const { workspace, ExtensionContext } = require('vscode');
const {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind
} = require('vscode-languageclient/node');

let client;

function activate(context) {
    // The server is implemented in Rust
    // Find the binary in the target directory relative to the workspace root
    // For development, we'll assume it's at ../target/debug/auwla-lsp.exe
    let serverPath = context.asAbsolutePath(path.join('..', 'target', 'debug', 'auwla-lsp.exe'));

    // Server options
    let serverOptions = {
        run: { command: serverPath, transport: TransportKind.stdio },
        debug: { command: serverPath, transport: TransportKind.stdio }
    };

    // Client options
    let clientOptions = {
        documentSelector: [{ scheme: 'file', language: 'auwla' }],
        synchronize: {
            fileEvents: workspace.createFileSystemWatcher('**/auwla_metadata.json')
        }
    };

    // Create the language client and start the client.
    client = new LanguageClient(
        'auwlaLsp',
        'Auwla Language Server',
        serverOptions,
        clientOptions
    );

    client.start();
}

function deactivate() {
    if (!client) {
        return undefined;
    }
    return client.stop();
}

module.exports = {
    activate,
    deactivate
};
