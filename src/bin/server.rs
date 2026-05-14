//! Gregorio LSP server (tower-lsp).

use std::sync::Mutex;

use serde_json::Value;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use gregorio_lsp::lint::{lint_gabc_text, LintOptions, LintSeverity};
use gregorio_lsp::parser::types::Severity as PSeverity;
use gregorio_lsp::parser::GabcParser;
#[cfg(feature = "tree-sitter")]
use gregorio_lsp::tree_sitter_integration::TreeSitterParser;
#[cfg(feature = "tree-sitter")]
use gregorio_lsp::Position as GPosition;
#[cfg(feature = "tree-sitter")]
use gregorio_lsp::Range as GRange;
#[cfg(feature = "tree-sitter")]
use tree_sitter::Tree;

#[derive(Debug, Clone)]
struct LintingConfig {
    enabled: bool,
    severity: LintSeverity,
    on_save: bool,
    ignore_rules: Vec<String>,
}

impl Default for LintingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            severity: LintSeverity::Warning,
            on_save: false,
            ignore_rules: Vec::new(),
        }
    }
}

struct Backend {
    client: Client,
    documents: Mutex<std::collections::HashMap<Url, String>>,
    config: Mutex<LintingConfig>,
    #[cfg(feature = "tree-sitter")]
    ts_parser: Mutex<Option<TreeSitterParser>>,
    #[cfg(feature = "tree-sitter")]
    ts_trees: Mutex<std::collections::HashMap<Url, Tree>>,
}

impl Backend {
    fn new(client: Client) -> Self {
        Self {
            client,
            documents: Mutex::new(std::collections::HashMap::new()),
            config: Mutex::new(LintingConfig::default()),
            #[cfg(feature = "tree-sitter")]
            ts_parser: Mutex::new(TreeSitterParser::new()),
            #[cfg(feature = "tree-sitter")]
            ts_trees: Mutex::new(std::collections::HashMap::new()),
        }
    }

    async fn validate(&self, uri: Url, text: &str) {
        let cfg = { self.config.lock().unwrap().clone() };
        if !cfg.enabled {
            self.client.publish_diagnostics(uri, Vec::new(), None).await;
            return;
        }
        let opts = LintOptions {
            min_severity: Some(cfg.severity),
            ignore_codes: cfg.ignore_rules.clone(),
        };
        let errors = lint_gabc_text(text, &opts);
        let diagnostics = errors.into_iter().map(to_diagnostic).collect();
        self.client.publish_diagnostics(uri, diagnostics, None).await;
    }
}

fn to_diagnostic(err: gregorio_lsp::parser::types::ParseError) -> Diagnostic {
    let severity = match err.severity {
        PSeverity::Error => DiagnosticSeverity::ERROR,
        PSeverity::Warning => DiagnosticSeverity::WARNING,
        PSeverity::Info => DiagnosticSeverity::INFORMATION,
    };
    let data = err.fix.map(|fix| {
        serde_json::json!({
            "fix": {
                "start_line": fix.range.start.line,
                "start_character": fix.range.start.character,
                "end_line": fix.range.end.line,
                "end_character": fix.range.end.character,
                "new_text": fix.new_text
            }
        })
    });
    Diagnostic {
        range: Range {
            start: Position::new(err.range.start.line as u32, err.range.start.character as u32),
            end: Position::new(err.range.end.line as u32, err.range.end.character as u32),
        },
        severity: Some(severity),
        code: err.code.map(NumberOrString::String),
        code_description: None,
        source: Some("gregorio-lsp".into()),
        message: err.message,
        related_information: None,
        tags: None,
        data,
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(true),
                    trigger_characters: Some(vec![
                        "(".into(),
                        "|".into(),
                        "<".into(),
                        "n".into(),
                        "a".into(),
                        "b".into(),
                        "c".into(),
                    ]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                code_action_provider: Some(CodeActionProviderCapability::Options(
                    CodeActionOptions {
                        code_action_kinds: Some(vec![CodeActionKind::QUICKFIX]),
                        resolve_provider: Some(false),
                        work_done_progress_options: Default::default(),
                    },
                )),
                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                        supported: Some(true),
                        change_notifications: Some(OneOf::Left(true)),
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "gregorio-lsp".into(),
                version: Some(env!("CARGO_PKG_VERSION").into()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Gregorio LSP Server initialized.")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_change_configuration(&self, params: DidChangeConfigurationParams) {
        if let Some(linting) = params.settings.get("linting") {
            let mut cfg = self.config.lock().unwrap();
            if let Some(b) = linting.get("enabled").and_then(Value::as_bool) {
                cfg.enabled = b;
            }
            if let Some(s) = linting.get("severity").and_then(Value::as_str) {
                if let Some(sev) = LintSeverity::parse(s) {
                    cfg.severity = sev;
                }
            }
            if let Some(b) = linting.get("onSave").and_then(Value::as_bool) {
                cfg.on_save = b;
            }
            if let Some(arr) = linting.get("ignoreRules").and_then(Value::as_array) {
                cfg.ignore_rules = arr
                    .iter()
                    .filter_map(|v| v.as_str().map(str::to_owned))
                    .collect();
            }
        }
        let docs = { self.documents.lock().unwrap().clone() };
        for (uri, text) in docs {
            self.validate(uri, &text).await;
        }
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let text = params.text_document.text;
        {
            let mut docs = self.documents.lock().unwrap();
            docs.insert(uri.clone(), text.clone());
        }
        #[cfg(feature = "tree-sitter")]
        {
            let mut parser = self.ts_parser.lock().unwrap();
            if let Some(parser) = parser.as_mut() {
                if let Some(tree) = parser.parse(&text) {
                    self.ts_trees.lock().unwrap().insert(uri.clone(), tree);
                }
            }
        }
        let cfg = { self.config.lock().unwrap().clone() };
        if cfg.enabled && !cfg.on_save {
            self.validate(uri, &text).await;
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let mut text = {
            self.documents
                .lock()
                .unwrap()
                .get(&uri)
                .cloned()
                .unwrap_or_default()
        };

        #[cfg(feature = "tree-sitter")]
        let mut tree = { self.ts_trees.lock().unwrap().remove(&uri) };

        for change in params.content_changes {
            if let Some(range) = change.range {
                #[cfg(feature = "tree-sitter")]
                if let Some(mut current_tree) = tree.take() {
                    let g_range = GRange {
                        start: GPosition {
                            line: range.start.line as usize,
                            character: range.start.character as usize,
                        },
                        end: GPosition {
                            line: range.end.line as usize,
                            character: range.end.character as usize,
                        },
                    };
                    if let Some((new_text, edit)) = TreeSitterParser::apply_incremental_edit(
                        &text,
                        g_range,
                        &change.text,
                    ) {
                        current_tree.edit(&edit);
                        let mut parser = self.ts_parser.lock().unwrap();
                        if let Some(parser) = parser.as_mut() {
                            tree = parser.parse_with_old(&new_text, &current_tree);
                        }
                        text = new_text;
                        continue;
                    }
                }

                if let Some(updated) = apply_lsp_change(&text, range, &change.text) {
                    text = updated;
                } else {
                    text = change.text;
                }
            } else {
                text = change.text;
            }

            #[cfg(feature = "tree-sitter")]
            {
                let mut parser = self.ts_parser.lock().unwrap();
                if let Some(parser) = parser.as_mut() {
                    tree = parser.parse(&text);
                }
            }
        }

        {
            let mut docs = self.documents.lock().unwrap();
            docs.insert(uri.clone(), text.clone());
        }

        #[cfg(feature = "tree-sitter")]
        {
            if let Some(tree) = tree {
                self.ts_trees.lock().unwrap().insert(uri.clone(), tree);
            }
        }

        let cfg = self.config.lock().unwrap().clone();
        if cfg.enabled && !cfg.on_save {
            self.validate(uri, &text).await;
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = { self.documents.lock().unwrap().get(&uri).cloned() };
        let cfg = { self.config.lock().unwrap().clone() };
        if cfg.enabled && cfg.on_save {
            if let Some(t) = text {
                self.validate(uri, &t).await;
            }
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        {
            let mut docs = self.documents.lock().unwrap();
            docs.remove(&params.text_document.uri);
        }
        #[cfg(feature = "tree-sitter")]
        {
            self.ts_trees.lock().unwrap().remove(&params.text_document.uri);
        }
        self.client
            .publish_diagnostics(params.text_document.uri, Vec::new(), None)
            .await;
    }

    async fn hover(&self, _params: HoverParams) -> Result<Option<Hover>> {
        Ok(None)
    }

    async fn completion(&self, _params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let items = vec![
            CompletionItem {
                label: "c4".into(),
                kind: Some(CompletionItemKind::TEXT),
                detail: Some("C clef on line 4".into()),
                documentation: Some(Documentation::String(
                    "Places a C clef on the 4th line of the staff".into(),
                )),
                ..Default::default()
            },
            CompletionItem {
                label: "f3".into(),
                kind: Some(CompletionItemKind::TEXT),
                detail: Some("F clef on line 3".into()),
                documentation: Some(Documentation::String(
                    "Places an F clef on the 3rd line of the staff".into(),
                )),
                ..Default::default()
            },
            CompletionItem {
                label: "::".into(),
                kind: Some(CompletionItemKind::TEXT),
                detail: Some("Divisio finalis".into()),
                documentation: Some(Documentation::String("Final bar (double bar)".into())),
                ..Default::default()
            },
            CompletionItem {
                label: "nabc-lines:".into(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("NABC lines header".into()),
                documentation: Some(Documentation::String(
                    "Declares the number of NABC lines".into(),
                )),
                ..Default::default()
            },
        ];
        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let text = {
            self.documents
                .lock()
                .unwrap()
                .get(&params.text_document.uri)
                .cloned()
        };
        let Some(text) = text else { return Ok(None) };
        let parsed = GabcParser::new(&text).parse();
        let mut symbols = Vec::new();
        for (name, value) in parsed.headers.iter() {
            let truncated: String = value.chars().take(30).collect();
            let suffix = if value.chars().count() > 30 { "..." } else { "" };
            #[allow(deprecated)]
            symbols.push(DocumentSymbol {
                name: format!("{name}: {truncated}{suffix}"),
                detail: None,
                kind: SymbolKind::STRING,
                tags: None,
                deprecated: None,
                range: Range {
                    start: Position::new(0, 0),
                    end: Position::new(0, 0),
                },
                selection_range: Range {
                    start: Position::new(0, 0),
                    end: Position::new(0, 0),
                },
                children: None,
            });
        }
        Ok(Some(DocumentSymbolResponse::Nested(symbols)))
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let mut actions: Vec<CodeActionOrCommand> = Vec::new();
        for diag in &params.context.diagnostics {
            let Some(data) = diag.data.as_ref() else {
                continue;
            };
            let Some(fix) = data.get("fix") else {
                continue;
            };
            let (Some(sl), Some(sc), Some(el), Some(ec), Some(new_text)) = (
                fix["start_line"].as_u64(),
                fix["start_character"].as_u64(),
                fix["end_line"].as_u64(),
                fix["end_character"].as_u64(),
                fix["new_text"].as_str(),
            ) else {
                continue;
            };
            let edit_range = Range {
                start: Position::new(sl as u32, sc as u32),
                end: Position::new(el as u32, ec as u32),
            };
            let mut changes = std::collections::HashMap::new();
            changes.insert(
                params.text_document.uri.clone(),
                vec![TextEdit {
                    range: edit_range,
                    new_text: new_text.to_string(),
                }],
            );
            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                title: "Split into individual note groups".into(),
                kind: Some(CodeActionKind::QUICKFIX),
                diagnostics: Some(vec![diag.clone()]),
                edit: Some(WorkspaceEdit {
                    changes: Some(changes),
                    ..Default::default()
                }),
                is_preferred: Some(true),
                ..Default::default()
            }));
        }
        Ok(if actions.is_empty() { None } else { Some(actions) })
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = LspService::new(Backend::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}

fn apply_lsp_change(text: &str, range: tower_lsp::lsp_types::Range, replacement: &str) -> Option<String> {
    let start = byte_offset(text, range.start)?;
    let end = byte_offset(text, range.end)?;
    let mut out = String::with_capacity(text.len() - (end - start) + replacement.len());
    out.push_str(&text[..start]);
    out.push_str(replacement);
    out.push_str(&text[end..]);
    Some(out)
}

fn byte_offset(text: &str, pos: tower_lsp::lsp_types::Position) -> Option<usize> {
    let mut line = 0usize;
    let mut col = 0usize;
    for (idx, ch) in text.char_indices() {
        if line == pos.line as usize && col == pos.character as usize {
            return Some(idx);
        }
        if ch == '\n' {
            line += 1;
            col = 0;
            if line > pos.line as usize {
                return Some(idx + 1);
            }
        } else if line == pos.line as usize {
            col += 1;
        }
    }
    if line == pos.line as usize && col == pos.character as usize {
        return Some(text.len());
    }
    None
}
