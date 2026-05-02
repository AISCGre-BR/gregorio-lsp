//! Gregorio LSP server (tower-lsp).

use std::sync::Mutex;

use serde_json::Value;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use gregorio_lsp::lint::{lint_gabc_text, LintOptions, LintSeverity};
use gregorio_lsp::parser::types::Severity as PSeverity;
use gregorio_lsp::parser::GabcParser;

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
}

impl Backend {
    fn new(client: Client) -> Self {
        Self {
            client,
            documents: Mutex::new(std::collections::HashMap::new()),
            config: Mutex::new(LintingConfig::default()),
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
        data: None,
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
        let cfg = { self.config.lock().unwrap().clone() };
        if cfg.enabled && !cfg.on_save {
            self.validate(uri, &text).await;
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let text = {
            let mut docs = self.documents.lock().unwrap();
            let entry = docs.entry(uri.clone()).or_default();
            for change in params.content_changes {
                // Simplified update: tower-lsp surfaces full text for Full sync; for Incremental
                // edits we still use the new chunk as best-effort full replacement.
                *entry = change.text;
            }
            entry.clone()
        };

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
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = LspService::new(Backend::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}
