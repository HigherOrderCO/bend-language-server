use std::collections::HashMap;

use itertools::Itertools;
use ropey::Rope;
use tower_lsp::lsp_types::{SemanticToken, SemanticTokenType};
use tree_sitter as ts;
use tree_sitter_highlight::{HighlightConfiguration, HighlightEvent, Highlighter};

use crate::language::bend;

lazy_static::lazy_static! {
    pub static ref NAME_TO_TOKEN_TYPE: HashMap<&'static str, SemanticTokenType> = {
        HashMap::from([
            ("variable", SemanticTokenType::VARIABLE),
            ("variable.parameter", SemanticTokenType::PARAMETER),
            ("variable.member", SemanticTokenType::ENUM_MEMBER),
            ("property", SemanticTokenType::PROPERTY),
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
            // ("punctuation", SemanticTokenType::?),
            // ("punctuation.delimiter", SemanticTokenType::?),
            // ("punctuation.bracket", SemanticTokenType::?),
            ("operator", SemanticTokenType::OPERATOR),
        ])
    };

    pub static ref LEGEND_TOKEN_TYPE: Vec<SemanticTokenType> =
        NAME_TO_TOKEN_TYPE.values().map(|v| v.clone()).unique().collect();

    pub static ref HIGHLIGHT_NAMES: Vec<&'static str> =
        NAME_TO_TOKEN_TYPE.keys().map(|x| *x).collect();

    pub static ref NAME_TO_TYPE_INDEX: HashMap<&'static str, usize> = {
        let token_type_index: HashMap<_, _> = LEGEND_TOKEN_TYPE.iter().enumerate().map(|(i, v)| (v.clone(), i)).collect();
        NAME_TO_TOKEN_TYPE.iter().map(|(key, val)| (*key, token_type_index[val])).collect()
    };
}

pub fn semantic_tokens(text: &Rope) -> Vec<SemanticToken> {
    let mut highlighter = Highlighter::new();
    let mut config = HighlightConfiguration::new(bend(), "bend", &QUERY, "", "").unwrap();
    config.configure(&HIGHLIGHT_NAMES);

    let code = text.to_string(); // TODO: this is bad
    let highlights = highlighter
        .highlight(&config, code.as_bytes(), None, |_| None)
        .unwrap();

    let mut tokens = vec![];
    let mut curr = None;
    let mut pre_line = 0;
    let mut pre_start = 0;
    for event in highlights {
        match event {
            // if the highlight is nested, only save inner range
            Result::Ok(HighlightEvent::HighlightStart(h)) => curr = Some(h.0),
            Result::Ok(HighlightEvent::HighlightEnd) => curr = None,
            Result::Ok(HighlightEvent::Source { start, end }) => {
                curr.and_then(|curr| HIGHLIGHT_NAMES.get(curr))
                    .and_then(|name| NAME_TO_TYPE_INDEX.get(name))
                    .and_then(|type_index| {
                        make_semantic_token(
                            &text,
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

    tokens
}

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

#[test]
fn token_capture_test() {
    let code: Rope = r#"
def main(x):
  return "Hi!"
List/flatten (List/Cons x xs) = (List/concat x (List/flatten xs))
List/flatten (List/Nil)       = (List/Nil)
"#
    .into();
    let mut highlighter = Highlighter::new();
    let mut config = HighlightConfiguration::new(bend(), "bend", &QUERY, "", "").unwrap();
    config.configure(&HIGHLIGHT_NAMES);

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
                println!("> {start}-{end}: `{:?}`", &text[start..end])
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
    let mut curr = None;
    let mut pre_line = 0;
    let mut pre_start = 0;
    for event in highlights {
        match event {
            // if the highlight is nested, only save inner range
            Result::Ok(HighlightEvent::HighlightStart(h)) => curr = Some(h.0),
            Result::Ok(HighlightEvent::HighlightEnd) => curr = None,
            Result::Ok(HighlightEvent::Source { start, end }) => {
                curr.and_then(|curr| HIGHLIGHT_NAMES.get(curr))
                    .and_then(|name| NAME_TO_TYPE_INDEX.get(name))
                    .and_then(|type_index| {
                        println!(
                            "{}-{} `{}`: {}",
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
}

pub struct TextProviderRope<'a>(pub &'a Rope);

impl<'a> ts::TextProvider<&'a [u8]> for &'a TextProviderRope<'a> {
    type I = ChunksBytes<'a>;
    fn text(&mut self, node: tree_sitter::Node) -> Self::I {
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

const QUERY: &'static str = r#"
(identifier) @variable

; TODO: lots of repetitive queries because of this, how can we fix it?
(identifier
  (path) @property
  name: (_) @variable)

(import_name
  "import" @keyword
  (os_path) @string)

(import_from
  "from" @keyword
  (os_path) @string
  "import" @keyword
  (os_path) @string)

(fun_function_definition
  name: (identifier) @function)
(fun_function_definition
  name: (identifier (identifier) @function))

(imp_function_definition
  name: (identifier) @function)
(imp_function_definition
  name: (identifier (identifier) @function))
(parameters) @variable.parameter

(object_definition
  name: (identifier) @type)
(object_definition
  name: (identifier (identifier) @type))
(object_field
  (identifier) @variable.member)
(object_field
  (identifier (identifier) @variable.member))

(imp_type_definition
  name: (identifier) @type)
(imp_type_definition
  name: (identifier (identifier) @type))
(imp_type_constructor
  (identifier) @constructor)
(imp_type_constructor
  (identifier (identifier) @constructor))
(imp_type_constructor_field
  (identifier) @variable.member)
(imp_type_constructor_field
  (identifier (identifier) @variable.member))

(fun_type_definition
  name: (identifier) @type)
(fun_type_definition
  name: (identifier (identifier) @type))
(fun_type_constructor
  (identifier) @constructor)
(fun_type_constructor
  (identifier (identifier) @constructor))
(fun_type_constructor_fields
  (identifier) @variable.member)
(fun_type_constructor_fields
  (identifier (identifier) @variable.member))

(hvm_definition
  name: (identifier) @function)
(hvm_definition
  name: (identifier (identifier) @function))
(hvm_definition
  code: (hvm_code) @string)

(constructor
  (identifier) @constructor)
(constructor
  (identifier (identifier) @constructor))
(constructor
  field: (identifier) @property)

(call_expression
  (identifier) @function.call)
(call_expression
  (identifier (identifier) @function.call))

(switch_case
  (switch_pattern) @character.special
  (#eq? @character.special "_"))

(integer) @number
(float) @number.float

(character) @character
(comment) @comment
[
 (symbol)
 (string)
] @string

[
  ":"
  ","
  ";"
] @punctuation.delimiter

[
  "["
  "]"
  "("
  ")"
  "{"
  "}"
] @punctuation.bracket

[
  "-"
  "-="
  "!="
  "*"
  "**"
  "*="
  "/"
  "/="
  "&"
  "%"
  "^"
  "+"
  "+="
  "<"
  "<="
  "="
  "=="
  ">"
  ">="
  "|"
  "~"
  "&"
  "<-"
  "&="
  "|="
  "^="
  "@="
] @operator

[
  "if"
  "elif"
  "else"
] @keyword.conditional


[
 "def"
  "@"
  "λ"
  "lambda"
  "hvm"
] @keyword.function

"return" @keyword.return
"for" @keyword.repeat

[
  "object"
  "type"
] @keyword.type

[
  "with"
  "match"
  "case"
  "open"
  "use"
  "with"
  "bend"
  "when"
  "fold"
  "switch"
  "ask"
  "let"
  "in"
] @keyword
"#;
