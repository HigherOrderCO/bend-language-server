use std::collections::HashMap;

use itertools::Itertools;
use ropey::Rope;
use tower_lsp::lsp_types::{Range, SemanticToken, SemanticTokenType};
use tree_sitter_bend::HIGHLIGHTS_QUERY;
use tree_sitter_highlight::{HighlightConfiguration, HighlightEvent};

use super::document::Document;
use crate::language::bend;

lazy_static::lazy_static! {
    /// Tree sitter capture names into LSP semantic token types.
    /// Changes to this table don't need to be added anywhere else due to the structures below.
    pub static ref NAME_TO_TOKEN_TYPE: HashMap<&'static str, SemanticTokenType> = {
        HashMap::from([
            ("variable", SemanticTokenType::VARIABLE),
            ("variable.parameter", SemanticTokenType::PARAMETER),
            ("variable.member", SemanticTokenType::ENUM_MEMBER),
            ("property", SemanticTokenType::TYPE),
            ("keyword", SemanticTokenType::KEYWORD),
            ("keyword.conditional", SemanticTokenType::KEYWORD),
            ("keyword.function", SemanticTokenType::KEYWORD),
            ("keyword.return", SemanticTokenType::KEYWORD),
            ("keyword.repeat", SemanticTokenType::KEYWORD),
            ("keyword.type", SemanticTokenType::KEYWORD),
            ("string", SemanticTokenType::STRING),
            ("function", SemanticTokenType::FUNCTION),
            ("function.call", SemanticTokenType::FUNCTION),
            ("type", SemanticTokenType::TYPE),
            // ("constructor", SemanticTokenType::?),
            ("character", SemanticTokenType::STRING),
            ("character.special", SemanticTokenType::STRING),
            ("number", SemanticTokenType::NUMBER),
            ("number.float", SemanticTokenType::NUMBER),
            ("comment", SemanticTokenType::COMMENT),
            // ("punctuation", SemanticTokenType::new("operator")),
            // ("punctuation.delimiter", SemanticTokenType::new("operator")),
            // ("punctuation.bracket", SemanticTokenType::new("operator")),
            ("operator", SemanticTokenType::OPERATOR),
        ])
    };

    /// Legend for token types.
    /// This is sent to the LSP client with the semantic tokens capabilities.
    pub static ref LEGEND_TOKEN_TYPE: Vec<SemanticTokenType> =
        NAME_TO_TOKEN_TYPE.values().cloned().unique().collect();

    /// Tree sitter highlighting names.
    /// This is used to perform syntax highlighting with tree sitter.
    pub static ref HIGHLIGHT_NAMES: Vec<&'static str> =
        NAME_TO_TOKEN_TYPE.keys().copied().collect();

    /// Translate indices from `HIGHLIGHT_NAMES` to indices from `LEGEND_TOKEN_TYPE`.
    pub static ref HIGHLIGHT_INDEX_TO_LSP_INDEX: HashMap<usize, usize> = {
        let token_type_index: HashMap<_, _> = LEGEND_TOKEN_TYPE.iter().enumerate().map(|(i, v)| (v.clone(), i)).collect();
        let highlight_index: HashMap<_, _> = HIGHLIGHT_NAMES.iter().enumerate().map(|(i, v)| (v, i)).collect();
        NAME_TO_TOKEN_TYPE.iter().map(|(key, val)| (highlight_index[key], token_type_index[val])).collect()
    };

    /// Global configuration for syntax highlighting.
    pub static ref HIGHLIGHTER_CONFIG: HighlightConfiguration = {
        let mut config = HighlightConfiguration::new(bend(), "bend", HIGHLIGHTS_QUERY, "", "").unwrap();
        config.configure(&HIGHLIGHT_NAMES);
        config
    };
}

/// Generate the semantic tokens of a document for syntax highlighting.
pub fn semantic_tokens(doc: &mut Document, range: Option<Range>) -> Vec<SemanticToken> {
    let code = doc.text.to_string(); // TODO: this is bad
    let highlights = doc
        .highlighter
        .highlight(&HIGHLIGHTER_CONFIG, code.as_bytes(), None, |_| None)
        .unwrap();

    let mut tokens = vec![]; // result vector
    let mut types = vec![]; // token type stack
    let mut pre_line = 0; // calculate line deltas between tokens
    let mut pre_start = 0; // calculate index deltas between tokens
    for event in highlights {
        match event {
            Result::Ok(HighlightEvent::HighlightStart(h)) => types.push(h.0),
            Result::Ok(HighlightEvent::HighlightEnd) => drop(types.pop()),
            Result::Ok(HighlightEvent::Source { mut start, end }) => {
                // Ranged or full semantic tokens call
                if let Some(range) = range {
                    let rstart = doc.text.line_to_byte(range.start.line as usize);
                    let rend = doc.text.line_to_byte(range.end.line as usize);
                    // If we still haven't gotten to the start of the range, continue.
                    if end < rstart {
                        continue;
                    }
                    // If we got past the end of the range, stop.
                    if rend < start {
                        break;
                    }
                }

                let token = types
                    .last()
                    .and_then(|curr| HIGHLIGHT_INDEX_TO_LSP_INDEX.get(curr))
                    .and_then(|type_index| {
                        // Prevents tokens from starting with new lines or other white space.
                        // New lines at the start of tokens may break the `make_semantic_token` function.
                        while start < end && char::from(doc.text.byte(start)).is_whitespace() {
                            start += 1;
                        }

                        // Translates the token ranges into the expected struct from LSP.
                        make_semantic_token(
                            &doc.text,
                            start..end,
                            *type_index as u32,
                            &mut pre_line,
                            &mut pre_start,
                        )
                    });

                if let Some(token) = token {
                    tokens.push(token);
                }
            }
            Err(_) => { /* log error? */ }
        }
    }

    tokens
}

/// Generates a specific semantic token within the guidelines of the LSP.
fn make_semantic_token(
    code: &Rope,
    range: std::ops::Range<usize>,
    token_type: u32,
    pre_line: &mut u32,
    pre_start: &mut u32,
) -> Option<SemanticToken> {
    let line = code.try_byte_to_line(range.start).ok()? as u32;
    let first = code.try_line_to_char(line as usize).ok()? as u32;
    let start = (code.try_byte_to_char(range.start).ok()? as u32).checked_sub(first)?;

    let delta_line = line.checked_sub(*pre_line)?;
    let delta_start = if delta_line == 0 {
        start.checked_sub(*pre_start)?
    } else {
        start
    };

    *pre_line = line;
    *pre_start = start;

    Some(SemanticToken {
        delta_line,
        delta_start,
        length: (range.end - range.start) as u32,
        token_type,
        token_modifiers_bitset: 0,
    })
}

/// Debugging test - tests steps from the semantic token generation algorithm.
#[test]
fn token_capture_test() {
    let code: Rope = r#"
def main():
  return "Hi!"
"#
    .into();
    let mut highlighter = tree_sitter_highlight::Highlighter::new();
    let config = &HIGHLIGHTER_CONFIG;

    let text = code.to_string(); // TODO: this is bad
    let highlights = highlighter
        .highlight(&config, text.as_bytes(), None, |_| None)
        .unwrap();

    let mut stack = vec![];
    for event in highlights {
        match event.unwrap() {
            HighlightEvent::HighlightStart(k) => {
                let name = HIGHLIGHT_NAMES[k.0];
                stack.push(name);
                println!("> start {}", name);
            }
            HighlightEvent::Source { start, end } => {
                println!("> {start}-{end}: {:?}", &text[start..end])
            }
            HighlightEvent::HighlightEnd => {
                println!("> end {}", stack.pop().unwrap());
            }
        }
    }
    println!();

    let highlights = highlighter
        .highlight(&config, text.as_bytes(), None, |_| None)
        .unwrap();

    let mut tokens = vec![];
    let mut stack = vec![];
    let mut pre_line = 0;
    let mut pre_start = 0;
    for event in highlights {
        match event {
            // if the highlight is nested, only save inner range
            Result::Ok(HighlightEvent::HighlightStart(h)) => stack.push(h.0),
            Result::Ok(HighlightEvent::HighlightEnd) => drop(stack.pop()),
            Result::Ok(HighlightEvent::Source { mut start, end }) => {
                stack
                    .last()
                    .and_then(|curr| HIGHLIGHT_INDEX_TO_LSP_INDEX.get(curr))
                    .and_then(|type_index| {
                        while start < end && char::from(code.byte(start)).is_whitespace() {
                            start += 1;
                        }

                        println!(
                            "{}-{} {:?}: {}",
                            start,
                            end,
                            &text[start..end],
                            LEGEND_TOKEN_TYPE[*type_index as usize].as_str()
                        );
                        make_semantic_token(
                            &code,
                            start..end,
                            *type_index as u32,
                            &mut pre_line,
                            &mut pre_start,
                        )
                    })
                    .map(|token| tokens.push(token));
            }
            Err(_) => { /* log error? */ }
        }
    }
    println!();

    println!("> got {} tokens", tokens.len());
    for token in tokens {
        println!("{:?}", token);
    }
}

// TODO: These are necessary for performant rope processing, but `tree_sitter_highlight`
// still does not work with them.
//
// pub struct TextProviderRope<'a>(pub &'a Rope);
//
// impl<'a> ts::TextProvider<&'a [u8]> for &'a TextProviderRope<'a> {
//     type I = ChunksBytes<'a>;
//     fn text(&mut self, node: tree_sitter::Node) -> Self::I {
//         ChunksBytes(self.0.byte_slice(node.byte_range()).chunks())
//     }
// }
//
// pub struct ChunksBytes<'a>(ropey::iter::Chunks<'a>);
//
// impl<'a> Iterator for ChunksBytes<'a> {
//     type Item = &'a [u8];
//
//     fn next(&mut self) -> Option<Self::Item> {
//         self.0.next().map(|s| s.as_bytes())
//     }
// }
