use tree_sitter::{Language, LanguageError, Parser};

pub fn bend() -> Language {
    tree_sitter_bend::language()
}

pub fn bend_parser() -> Result<Parser, LanguageError> {
    let mut parser = Parser::new();
    parser.set_language(&bend())?;
    Ok(parser)
}
