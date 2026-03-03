mod completion;
mod diagnostics;
mod hover;
mod utils;

use dashmap::DashMap;
use std::sync::Arc;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
pub struct Backend {
    pub client: Client,
    /// uri -> content
    pub documents: DashMap<String, String>,
    /// type_key -> [Extensions]
    pub metadata: Arc<DashMap<String, Vec<auwla_ast::ExtensionMethod>>>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        // Try to find and load auwla_metadata.json from the workspace root
        if let Some(root_uri) = params.root_uri {
            if let Ok(path) = root_uri.to_file_path() {
                let metadata_path = path.join("output").join("auwla_metadata.json");
                if metadata_path.exists() {
                    if let Ok(content) = std::fs::read_to_string(&metadata_path) {
                        if let Ok(map) = serde_json::from_str::<
                            std::collections::HashMap<String, Vec<auwla_ast::ExtensionMethod>>,
                        >(&content)
                        {
                            for (k, v) in map {
                                self.metadata.insert(k, v);
                            }
                            self.client
                                .log_message(
                                    MessageType::INFO,
                                    format!("Loaded {} metadata entries", self.metadata.len()),
                                )
                                .await;
                        }
                    }
                }
            }
        }

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![".".to_string()]),
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Auwla Language Server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        let text = params.text_document.text;
        self.documents.insert(uri.clone(), text.clone());
        diagnostics::analyze_document(self, &uri, &text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if let Some(change) = params.content_changes.into_iter().next() {
            let uri = params.text_document.uri.to_string();
            self.documents.insert(uri.clone(), change.text.clone());
            diagnostics::analyze_document(self, &uri, &change.text).await;
        }
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        hover::handle_hover(self, params).await
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        completion::handle_completion(self, params).await
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend {
        client,
        documents: DashMap::new(),
        metadata: Arc::new(DashMap::new()),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
