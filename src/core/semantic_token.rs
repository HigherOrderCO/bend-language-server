use std::collections::HashMap;

use itertools::Itertools;
use tower_lsp::lsp_types::{self as lsp, SemanticTokenType};
use tree_sitter as ts;

use crate::language::{bend, bend_parser, conversion};

use super::document::Document;

lazy_static::lazy_static! {
    static ref NAME_TO_TOKEN_TYPE: HashMap<&'static str, SemanticTokenType> = {
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

    static ref LEGEND_TOKEN_TYPE: Vec<SemanticTokenType> =
        NAME_TO_TOKEN_TYPE.values().map(|v| v.clone()).unique().collect();

    static ref TOKEN_TYPE_INDEX: HashMap<SemanticTokenType, usize> =
        LEGEND_TOKEN_TYPE.iter().enumerate().map(|(i, v)| (v.clone(), i)).collect();
}

// pub const LEGEND_TYPE: &[SemanticTokenType] = &[
//     SemanticTokenType::FUNCTION,
//     SemanticTokenType::VARIABLE,
//     SemanticTokenType::STRING,
//     SemanticTokenType::COMMENT,
//     SemanticTokenType::NUMBER,
//     SemanticTokenType::KEYWORD,
//     SemanticTokenType::OPERATOR,
//     SemanticTokenType::PARAMETER,
// ];

pub struct SemanticToken {
    pub range: conversion::Range,
    pub token_type: usize,
}

pub fn semantic_tokens(doc: Document) -> Vec<SemanticToken> {
    let mut cursor = ts::QueryCursor::new();
    let query = ts::Query::new(&bend(), QUERY).unwrap();
    let names = query.capture_names();
    let root = doc.tree.as_ref().unwrap().root_node();

    for matche in cursor.matches(&query, root, doc.text.as_bytes()) {
        for capture in matche.captures {
            
        }
    }

    todo!()
}

#[test]
fn test() {
    let code = "main = (f \"hi!\")";
    let mut parser = bend_parser().unwrap();
    let tree = parser.parse(code, None).unwrap();

    let query = ts::Query::new(&bend(), &QUERY).unwrap();
    println!("{:?}", query.capture_names());

    for qmatch in ts::QueryCursor::new().matches(&query, tree.root_node(), code.as_bytes()) {
        for capture in qmatch.captures {
            println!("{:?}", capture);
        }
        println!();
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
