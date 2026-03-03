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
    // Find the bundled release binary inside the extension's `bin` folder
    let serverPath = context.asAbsolutePath(path.join('bin', 'auwla-lsp.exe'));

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
