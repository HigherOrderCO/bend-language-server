use std::sync::Arc;

pub struct Session {
    pub language: tree_sitter::Language,
    client: Option<tower_lsp::Client>,
}

impl Session {
    pub fn new(client: Option<tower_lsp::Client>, language: tree_sitter::Language) -> Arc<Self> {
        Arc::new(Session { client, language })
    }
}
