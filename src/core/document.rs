use ropey::Rope;
use tower_lsp::lsp_types as lsp;
use tree_sitter as ts;
use tree_sitter_highlight::{self as hg, Highlighter};

use crate::language::bend_parser;

pub struct Document {
    pub url: lsp::Url,
    pub text: Rope,
    pub tree: Option<ts::Tree>,
    pub parser: ts::Parser,
    pub highlighter: hg::Highlighter,
    // pub components: HashMap<String, ComponentInfo>
}

impl Document {
    /// Create an empty document for `url`.
    pub fn new(url: lsp::Url) -> Self {
        Self {
            url,
            text: Rope::new(),
            tree: None,
            parser: bend_parser().unwrap(),
            highlighter: Highlighter::new(),
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
        self.text = Rope::from_str(text);
        self.tree = self.parser.parse(text, None);
    }
}
