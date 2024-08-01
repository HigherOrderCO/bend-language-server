use std::fs;

// use bend::diagnostics::Diagnostic;
use dashmap::DashMap;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{self as lsp, SemanticTokensRangeResult};
use tower_lsp::{Client, LanguageServer};

use crate::core::diagnostic;
use crate::core::document::{self, Document};
use crate::core::semantic_token;
use crate::utils::lsp_log;

pub struct Backend {
    /// Connection to the client, used to send data
    pub client: Client,
    /// Currently open documents
    pub open_docs: DashMap<lsp::Url, document::Document>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    // All of these represent messages the server may receive from the client.
    // See the automatic documentation generated by `tower_lsp` to understand what each method does.

    async fn initialize(&self, _: lsp::InitializeParams) -> Result<lsp::InitializeResult> {
        let capabilities = Self::capabilities();

        Ok(lsp::InitializeResult {
            server_info: Some(lsp::ServerInfo {
                name: "Bend Language Server".into(),
                version: Some(env!("CARGO_PKG_VERSION").into()),
            }),
            offset_encoding: None,
            capabilities,
        })
    }

    async fn initialized(&self, _: lsp::InitializedParams) {
        let values = self
            .client
            .configuration(vec![lsp::ConfigurationItem {
                scope_uri: Some(lsp::Url::parse("file:///libraryPaths").unwrap()),
                section: Some("bend-lsp".to_string()),
            }])
            .await;

        if let Ok(_val) = values {
            // TODO: configuration
        }

        self.publish_all_diagnostics().await;

        lsp_log::info!(self.client, "bend-lsp initialized");
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: lsp::DidOpenTextDocumentParams) {
        // This is called when the client opens a new document.
        // We get the entire text of the document; let's store it.
        lsp_log::info!(self.client, "opening file at {}", params.text_document.uri);

        self.open_doc(params.text_document.uri.clone(), params.text_document.text);
        self.publish_diagnostics(&params.text_document.uri).await;
    }

    async fn did_change_configuration(&self, _params: lsp::DidChangeConfigurationParams) {
        lsp_log::info!(self.client, "changing language server configurations");

        // TODO: configuration

        self.publish_all_diagnostics().await;
    }

    async fn did_change(&self, params: lsp::DidChangeTextDocumentParams) {
        lsp_log::log!(
            self.client,
            "getting new text from {}",
            params.text_document.uri
        );

        self.update_document(&params.text_document.uri, |doc| {
            for event in &params.content_changes {
                doc.update_whole_text(&event.text);
            }
        });
    }

    async fn did_save(&self, params: lsp::DidSaveTextDocumentParams) {
        // Called when document is saved.
        // Update diagnostics (when we have them!)
        lsp_log::log!(
            self.client,
            "document saved at {}",
            params.text_document.uri
        );

        let url = &params.text_document.uri;
        self.publish_diagnostics(url).await;
    }

    async fn semantic_tokens_full(
        &self,
        params: lsp::SemanticTokensParams,
    ) -> Result<Option<lsp::SemanticTokensResult>> {
        lsp_log::info!(self.client, "generating full semantic tokens");

        let uri = params.text_document.uri;
        let semantic_tokens =
            self.read_document_mut(&uri, |doc| Some(semantic_token::semantic_tokens(doc, None)));

        let token_amount = semantic_tokens.as_ref().map(|ts| ts.len()).unwrap_or(0);
        lsp_log::info!(self.client, "got {} tokens", token_amount);

        Ok(semantic_tokens.map(|tokens| {
            lsp::SemanticTokensResult::Tokens(lsp::SemanticTokens {
                result_id: None,
                data: tokens,
            })
        }))
    }

    async fn semantic_tokens_range(
        &self,
        params: lsp::SemanticTokensRangeParams,
    ) -> Result<Option<SemanticTokensRangeResult>> {
        let range = params.range;
        lsp_log::info!(
            self.client,
            "generating range {}:{}-{}:{} semantic tokens",
            range.start.line,
            range.start.character,
            range.end.line,
            range.end.character
        );

        let uri = params.text_document.uri;
        let semantic_tokens = self.read_document_mut(&uri, |doc| {
            Some(semantic_token::semantic_tokens(doc, Some(range)))
        });

        let token_amount = semantic_tokens.as_ref().map(|ts| ts.len()).unwrap_or(0);
        lsp_log::info!(self.client, "got {} tokens", token_amount);

        Ok(semantic_tokens.map(|tokens| {
            lsp::SemanticTokensRangeResult::Tokens(lsp::SemanticTokens {
                result_id: None,
                data: tokens,
            })
        }))
    }

    async fn completion(
        &self,
        _: lsp::CompletionParams,
    ) -> Result<Option<lsp::CompletionResponse>> {
        Ok(Some(lsp::CompletionResponse::Array(vec![
            lsp::CompletionItem::new_simple("Hello".to_string(), "Some detail".to_string()),
            lsp::CompletionItem::new_simple("Bye/Bye".to_string(), "More detail".to_string()),
        ])))
    }

    async fn hover(&self, _: lsp::HoverParams) -> Result<Option<lsp::Hover>> {
        Ok(Some(lsp::Hover {
            contents: lsp::HoverContents::Scalar(lsp::MarkedString::String(
                "You're hovering!".to_string(),
            )),
            range: None,
        }))
    }
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            open_docs: DashMap::new(),
        }
    }

    fn capabilities() -> lsp::ServerCapabilities {
        lsp::ServerCapabilities {
            text_document_sync: Some(lsp::TextDocumentSyncCapability::Options(
                lsp::TextDocumentSyncOptions {
                    open_close: Some(true),
                    change: Some(lsp::TextDocumentSyncKind::FULL),
                    will_save: None,
                    will_save_wait_until: None,
                    save: Some(lsp::TextDocumentSyncSaveOptions::Supported(true)),
                },
            )),
            semantic_tokens_provider: Some(
                lsp::SemanticTokensServerCapabilities::SemanticTokensRegistrationOptions(
                    lsp::SemanticTokensRegistrationOptions {
                        text_document_registration_options: {
                            lsp::TextDocumentRegistrationOptions {
                                document_selector: Some(vec![lsp::DocumentFilter {
                                    language: Some("bend".into()),
                                    scheme: Some("file".into()),
                                    pattern: None,
                                }]),
                            }
                        },
                        semantic_tokens_options: lsp::SemanticTokensOptions {
                            work_done_progress_options: lsp::WorkDoneProgressOptions::default(),
                            legend: lsp::SemanticTokensLegend {
                                token_types: semantic_token::LEGEND_TOKEN_TYPE.to_vec(),
                                token_modifiers: vec![],
                            },
                            range: Some(true),
                            full: Some(lsp::SemanticTokensFullOptions::Bool(true)),
                        },
                        static_registration_options: lsp::StaticRegistrationOptions::default(),
                    },
                ),
            ),
            // definition_provider: Some(lsp::OneOf::Left(true)),
            // completion_provider: Some(lsp::CompletionOptions {
            //     resolve_provider: Some(false),
            //     trigger_characters: Some(vec!["/".into()]),
            //     all_commit_characters: None,
            //     work_done_progress_options: Default::default(),
            //     completion_item: None,
            // }),
            // hover_provider: Some(lsp::HoverProviderCapability::Simple(false)),
            ..Default::default()
        }
    }

    /// Publish diagnostics for every open file.
    async fn publish_all_diagnostics(&self) {
        for refer in self.open_docs.iter() {
            self.publish_diagnostics(refer.key()).await;
        }
    }

    /// Publish diagnostics for document `url`.
    async fn publish_diagnostics(&self, url: &lsp::Url) {
        let diags = self
            .read_document(url, |doc| {
                Some(diagnostic::lsp_diagnostic(&diagnostic::check(doc)))
            })
            .unwrap_or_default();

        self.client
            .publish_diagnostics(url.clone(), diags, None)
            .await;
    }

    /// Update the document at `url` using function `updater`.
    fn update_document<F>(&self, url: &lsp::Url, mut updater: F)
    where
        F: FnMut(&mut Document),
    {
        if let Some(mut doc) = self.open_docs.get_mut(url) {
            updater(doc.value_mut());
        }
    }

    /// Read the contents of `url` using function `reader`.
    fn read_document<F, T>(&self, url: &lsp::Url, reader: F) -> Option<T>
    where
        F: Fn(&document::Document) -> Option<T>,
    {
        self.open_docs
            .get(url)
            .and_then(|refer| reader(refer.value()))
    }

    /// Read the contents of `url` using function `reader`, possibly changing the document.
    fn read_document_mut<F, T>(&self, url: &lsp::Url, mut updater: F) -> Option<T>
    where
        F: FnMut(&mut Document) -> Option<T>,
    {
        self.open_docs
            .get_mut(url)
            .and_then(|mut refer| updater(refer.value_mut()))
    }

    /// Open a new document at `url` with its contents as a parameter.
    fn open_doc(&self, url: lsp::Url, text: String) {
        self.open_docs
            .insert(url.clone(), Document::new_with_text(url, &text));
    }

    /// Open a new document at `url` with its contents from the file system.
    fn open_doc_from_path(&self, url: lsp::Url) {
        if let Ok(text) = fs::read_to_string(url.to_file_path().unwrap()) {
            self.open_doc(url, text);
        }
    }
}
