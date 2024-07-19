use tower_lsp::lsp_types as lsp;
use tree_sitter as ts;

use crate::language::bend_parser;

pub struct Document {
    pub url: lsp::Url,
    text: String,
    tree: Option<ts::Tree>,
    parser: ts::Parser,
    // pub components: HashMap<String, ComponentInfo>
}

impl Document {
    /// Create an empty document for `url`.
    pub fn new(url: lsp::Url) -> Self {
        Self {
            url,
            text: String::new(),
            tree: None,
            parser: bend_parser().unwrap(),
        }
    }

    /// Create a new document with text for `url`.
    pub fn new_with_text(url: lsp::Url, text: &str) -> Self {
        let mut doc = Self::new(url);
        doc.update_whole_text(text);
        doc
    }

    /// Update the document with entirely new text.
    pub fn update_whole_text(&mut self, text: &str) {
        self.text = text.to_string();
        self.tree = self.parser.parse(text, None);
    }
}
