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
        let uri = params.text_document.uri.to_string();
        let text = params.text_document.text;
        self.documents.insert(uri.clone(), text.clone());
        self.analyze_document(&uri, &text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if let Some(change) = params.content_changes.into_iter().next() {
            let uri = params.text_document.uri.to_string();
            self.documents.insert(uri.clone(), change.text.clone());
            self.analyze_document(&uri, &change.text).await;
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

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri.to_string();
        let position = params.text_document_position.position;
        let mut items = Vec::new();

        let content = if let Some(c) = self.documents.get(&uri) {
            c.clone()
        } else {
            return Ok(None);
        };

        // Calculate byte offset from (line, character) using raw bytes
        // This correctly handles both \n and \r\n line endings
        let mut byte_offset = 0usize;
        let mut current_line = 0u32;
        for (i, byte) in content.as_bytes().iter().enumerate() {
            if current_line == position.line {
                byte_offset = i + position.character as usize;
                break;
            }
            if *byte == b'\n' {
                current_line += 1;
            }
        }
        // If we're on the last line and didn't find enough newlines
        if current_line < position.line {
            byte_offset = content.len();
        }

        // Search backwards from cursor for a dot, but ONLY on the current line
        let mut dot_idx = None;
        let search_start = byte_offset.saturating_sub(1);
        for i in (0..=search_start).rev() {
            let b = content.as_bytes()[i];
            if b == b'.' {
                dot_idx = Some(i);
                break;
            }
            if b == b'\n' || b == b'\r' {
                break; // Stop at line boundary
            }
        }

        let mut target_type_key = None;

        if let Some(di) = dot_idx {
            // Shadow-compile: replace the dot with a space so the file parses
            let mut shadow = String::with_capacity(content.len());
            shadow.push_str(&content[..di]);
            shadow.push(' ');
            shadow.push_str(&content[di + 1..]);

            let lexed = match std::panic::catch_unwind(|| auwla_lexer::lex(&shadow)) {
                Ok(l) => l,
                Err(_) => {
                    // Lexer panicked, fall back to global completions
                    return Ok(Some(self.global_completions()));
                }
            };
            let token_byte_spans: Vec<std::ops::Range<usize>> =
                lexed.iter().map(|(_, s)| s.clone()).collect();
            let tokens: Vec<_> = lexed.into_iter().map(|(t, _)| t).collect();

            if let Ok(ast) = auwla_parser::parse(tokens) {
                let mut typechecker = auwla_typechecker::Typechecker::new();
                for entry in self.metadata.iter() {
                    typechecker
                        .extensions
                        .insert(entry.key().clone(), entry.value().clone());
                }
                let _ = typechecker.check_program(&ast);

                // node_types keys are token-index spans (e.g., 3..5 = tokens 3,4)
                // Convert them to byte spans using token_byte_spans for comparison
                let mut best_fit: Option<auwla_ast::Type> = None;
                let mut best_byte_end = 0usize;

                for (tok_span, ty) in typechecker.node_types.iter() {
                    // Convert token-index span to byte span
                    let byte_start = token_byte_spans
                        .get(tok_span.start)
                        .map(|r| r.start)
                        .unwrap_or(0);
                    let byte_end = token_byte_spans
                        .get(tok_span.end.saturating_sub(1))
                        .map(|r| r.end)
                        .unwrap_or(0);

                    // We want the expression whose byte_end is closest to (but <= ) the dot
                    if byte_end <= di && byte_end > best_byte_end {
                        best_byte_end = byte_end;
                        best_fit = Some(ty.clone());
                    }
                }

                if let Some(ty) = best_fit {
                    target_type_key = Some(typechecker.type_to_key(&ty));
                }
            }
        }

        // Also add struct fields if target_type_key matches a known struct
        // (struct fields aren't in metadata, they come from the typechecker)

        for entry in self.metadata.iter() {
            let type_key = entry.key();

            if let Some(ref target) = target_type_key {
                if target != type_key {
                    continue;
                }
            }

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

impl Backend {
    fn global_completions(&self) -> CompletionResponse {
        let mut items = Vec::new();
        for entry in self.metadata.iter() {
            let type_key = entry.key();
            for method in entry.value() {
                items.push(CompletionItem {
                    label: method.name.clone(),
                    detail: Some(format!("extension for {}", type_key)),
                    kind: Some(CompletionItemKind::METHOD),
                    ..Default::default()
                });
            }
        }
        items.sort_by(|a, b| a.label.cmp(&b.label));
        items.dedup_by(|a, b| a.label == b.label);
        CompletionResponse::Array(items)
    }

    async fn analyze_document(&self, uri: &str, content: &str) {
        let mut diagnostics = Vec::new();

        let lexed = match std::panic::catch_unwind(|| auwla_lexer::lex(content)) {
            Ok(l) => l,
            Err(_) => return, // Ignore panics during lexing and preserve previous diagnostics
        };
        let spans: Vec<std::ops::Range<usize>> = lexed.iter().map(|(_, s)| s.clone()).collect();
        let tokens: Vec<_> = lexed.into_iter().map(|(t, _)| t).collect();

        match auwla_parser::parse(tokens) {
            Ok(ast) => {
                let mut typechecker = auwla_typechecker::Typechecker::new();
                for entry in self.metadata.iter() {
                    typechecker
                        .extensions
                        .insert(entry.key().clone(), entry.value().clone());
                }

                if let Err(e) = typechecker.check_program(&ast) {
                    let byte_start = spans.get(e.span.start).map(|r| r.start).unwrap_or(0);
                    let byte_end = spans
                        .get(e.span.end.saturating_sub(1))
                        .map(|r| r.end)
                        .unwrap_or(content.len());

                    diagnostics.push(Diagnostic {
                        range: Range::new(
                            byte_to_position(content, byte_start),
                            byte_to_position(content, byte_end),
                        ),
                        severity: Some(DiagnosticSeverity::ERROR),
                        message: e.message,
                        source: Some("auwla".to_string()),
                        ..Default::default()
                    });
                }
            }
            Err(errs) => {
                for e in errs {
                    let span = e.span();
                    let byte_start = spans
                        .get(span.start)
                        .map(|r| r.start)
                        .unwrap_or_else(|| content.len());
                    let byte_end = spans
                        .get(span.end.saturating_sub(1))
                        .map(|r| r.end)
                        .unwrap_or_else(|| content.len());

                    let mut message = match e.reason() {
                        chumsky::error::SimpleReason::Unclosed { delimiter, .. } => {
                            format!("Unclosed delimiter '{}'", delimiter)
                        }
                        chumsky::error::SimpleReason::Unexpected => {
                            let expected_str = if e.expected().len() == 0 {
                                "something else".to_string()
                            } else {
                                let mut strings: Vec<String> = e
                                    .expected()
                                    .map(|ex| match ex {
                                        Some(token) => token.to_string(),
                                        None => "end of input".to_string(),
                                    })
                                    .collect();
                                strings.sort();
                                strings.dedup();
                                strings.join(", ")
                            };

                            if let Some(found) = e.found() {
                                format!("Unexpected token '{}', expected {}", found, expected_str)
                            } else {
                                format!("Unexpected end of input, expected {}", expected_str)
                            }
                        }
                        chumsky::error::SimpleReason::Custom(msg) => msg.to_string(),
                    };

                    use auwla_lexer::token::Token;
                    let mut is_missing_semi = false;
                    for expected_token in e.expected() {
                        if let Some(token) = expected_token {
                            if matches!(token, Token::Semicolon) {
                                message =
                                    "Missing semicolon ';' at the end of statement".to_string();
                                is_missing_semi = true;
                                break;
                            }
                        }
                    }

                    let mut b_start = byte_start;
                    let mut b_end = byte_end;

                    if is_missing_semi && span.start > 0 {
                        // Pin precisely to the end of the previous token
                        if let Some(prev_span) = spans.get(span.start - 1) {
                            b_start = prev_span.end;
                            b_end = prev_span.end;
                        }
                    } else {
                        // Highlight only the *first* offensive token to prevent a giant multi-line block
                        b_end = spans.get(span.start).map(|r| r.end).unwrap_or(b_start);
                    }

                    diagnostics.push(Diagnostic {
                        range: Range::new(
                            byte_to_position(content, b_start),
                            byte_to_position(content, b_end),
                        ),
                        severity: Some(DiagnosticSeverity::ERROR),
                        message,
                        source: Some("auwla".to_string()),
                        ..Default::default()
                    });
                }
            }
        }

        if let Ok(url) = url::Url::parse(uri) {
            self.client
                .publish_diagnostics(url, diagnostics, None)
                .await;
        }
    }
}

fn byte_to_position(source: &str, byte: usize) -> Position {
    let safe = byte.min(source.len());
    let prefix = &source[..safe];
    let line = prefix.lines().count().max(1) - 1;
    let col = prefix.rfind('\n').map(|i| safe - i - 1).unwrap_or(safe);
    Position::new(line as u32, col as u32)
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
