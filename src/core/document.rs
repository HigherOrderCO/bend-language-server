use std::{collections::btree_map::RangeMut, marker::PhantomData};

use tower_lsp::lsp_types as lsp;
use tree_sitter as ts;

use crate::language::bend_parser;

pub struct Document {
    pub url: lsp::Url,
    pub text: Rope,
    pub tree: Option<ts::Tree>,
    pub parser: ts::Parser,
    // pub components: HashMap<String, ComponentInfo>
}

impl Document {
    /// Create an empty document for `url`.
    pub fn new(url: lsp::Url) -> Self {
        Self {
            url,
            text: Rope(ropey::Rope::new()),
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
        self.text = Rope(ropey::Rope::from_str(text));
        self.tree = self.parser.parse(text, None);
    }
}

pub struct Rope(pub ropey::Rope);

impl<'a> ts::TextProvider<&'a [u8]> for &'a Rope {
    type I = ChunksBytes<'a>;
    // type I = impl Iterator<Item = &'a [u8]>;

    fn text(&mut self, node: tree_sitter::Node) -> Self::I {
        // let range = node.byte_range();
        // let (mut chunks, chunk_byte_idx, _, _) = self.0.chunks_at_byte(range.start);
        // let start = range.start - chunk_byte_idx;

        // let first_chunk = chunks.next().iter().map(|x| &x[start..]);
        // let rest = chunks.take_while(predicate)

        ChunksBytes(self.0.byte_slice(node.byte_range()).chunks())
    }
}

pub struct ChunksBytes<'a>(ropey::iter::Chunks<'a>);

impl<'a> Iterator for ChunksBytes<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|s| s.as_bytes())
    }
}

// struct RopeSection<'a> {
//     chunks: ropey::iter::Chunks<'a>,
//     remaining: usize,
// }

// impl<'a> RopeSection<'a> {
//     pub fn new(rope: &'a Rope, range: &std::ops::Range<usize>) -> Self {
//         RopeSection {
//             chunks: rope.0.chunks_at_byte(range.start).0,
//             remaining: range.end - range.start
//         }
//     }
// }

// impl<'a> Iterator for RopeSection<'a> {
//     type Item = &'a [u8];

//     fn next(&mut self) -> Option<Self::Item> {
//         todo!()
//     }
// }