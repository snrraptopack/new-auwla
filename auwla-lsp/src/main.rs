use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct Backend {
    client: Client,
    /// uri -> content
    documents: DashMap<String, String>,
    /// type_key -> [Extensions]
    metadata: Arc<DashMap<String, Vec<auwla_ast::ExtensionMethod>>>,
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
                            HashMap<String, Vec<auwla_ast::ExtensionMethod>>,
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
        self.documents.insert(
            params.text_document.uri.to_string(),
            params.text_document.text,
        );
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if let Some(change) = params.content_changes.into_iter().next() {
            self.documents
                .insert(params.text_document.uri.to_string(), change.text);
        }
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params
            .text_document_position_params
            .text_document
            .uri
            .to_string();
        let position = params.text_document_position_params.position;

        if let Some(content) = self.documents.get(&uri) {
            let lines: Vec<&str> = content.lines().collect();
            if let Some(line) = lines.get(position.line as usize) {
                // Extremely naive: find word under cursor
                let word = get_word_at_offset(line, position.character as usize);

                // Search in metadata for this method name
                for entry in self.metadata.iter() {
                    let type_key = entry.key();
                    let methods = entry.value();
                    if let Some(method) = methods.iter().find(|m| m.name == word) {
                        let mut markdown =
                            format!("### Extension: `{}.{}`\n\n", type_key, method.name);
                        if !method.attributes.is_empty() {
                            markdown.push_str("**Attributes:**\n");
                            for attr in &method.attributes {
                                markdown
                                    .push_str(&format!("- `@{}({:?})`\n", attr.name, attr.args));
                            }
                            markdown.push_str("\n");
                        }

                        let params_str: Vec<String> = method
                            .params
                            .iter()
                            .map(|(p, t)| format!("{}: {:?}", p, t))
                            .collect();
                        markdown.push_str(&format!(
                            "```auwla\nfn {}({}) -> {:?}\n```\n\n",
                            method.name,
                            params_str.join(", "),
                            method.return_ty
                        ));
                        markdown.push_str(&format!("Defined in: `{}`", method.file));

                        return Ok(Some(Hover {
                            contents: HoverContents::Markup(MarkupContent {
                                kind: MarkupKind::Markdown,
                                value: markdown,
                            }),
                            range: None,
                        }));
                    }
                }
            }
        }

        Ok(None)
    }

    async fn completion(&self, _params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let mut items = Vec::new();

        // Naively suggest ALL extension methods across the project
        for entry in self.metadata.iter() {
            let type_key = entry.key();
            for method in entry.value() {
                items.push(CompletionItem {
                    label: method.name.clone(),
                    detail: Some(format!("extension for {}", type_key)),
                    documentation: Some(Documentation::MarkupContent(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: format!("```auwla\nfn {}(...)\n```", method.name),
                    })),
                    kind: Some(CompletionItemKind::METHOD),
                    ..Default::default()
                });
            }
        }

        // De-duplicate by name for now to avoid noise
        items.sort_by(|a, b| a.label.cmp(&b.label));
        items.dedup_by(|a, b| a.label == b.label);

        Ok(Some(CompletionResponse::Array(items)))
    }
}

fn get_word_at_offset(line: &str, char_idx: usize) -> &str {
    let mut start = char_idx;
    while start > 0 && line.as_bytes()[start - 1].is_ascii_alphanumeric()
        || line.as_bytes()[start - 1] == b'_'
    {
        start -= 1;
    }
    let mut end = char_idx;
    while end < line.len() && line.as_bytes()[end].is_ascii_alphanumeric()
        || line.as_bytes()[end] == b'_'
    {
        end += 1;
    }
    &line[start..end]
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
