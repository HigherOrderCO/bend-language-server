use tower_lsp::lsp_types::{self as lsp, SemanticTokenType};
use tree_sitter as ts;

use crate::language::{bend, bend_parser, conversion};

pub const LEGEND_TYPE: &[SemanticTokenType] = &[
    SemanticTokenType::FUNCTION,
    SemanticTokenType::VARIABLE,
    SemanticTokenType::STRING,
    SemanticTokenType::COMMENT,
    SemanticTokenType::NUMBER,
    SemanticTokenType::KEYWORD,
    SemanticTokenType::OPERATOR,
    SemanticTokenType::PARAMETER,
];

struct SemanticToken {
    pub range: conversion::Range,
    pub token_type: usize,
}

// pub fn semantic_tokens_from_tree(tree: ts::Tree) -> String /*Vec<SemanticToken>*/ {
    // let query = ts::Query::new(&bend(), QUERY);
    // query.unwrap().
// }

#[test]
fn test() {
    // this works!
    
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
