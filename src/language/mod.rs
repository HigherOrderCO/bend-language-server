//! Language module.
//!
//! Deals with processing directly related to the Bend language.
//! Right now it only returns the parser from tree sitter, but in the future we
//! might do additional processing from this module.

use tree_sitter::{Language, LanguageError, Parser};

/// Tree sitter representation for the Bend language.
pub fn bend() -> Language {
    tree_sitter_bend::language()
}
/// Returns a new tree sitter parser for Bend.
pub fn bend_parser() -> Result<Parser, LanguageError> {
    let mut parser = Parser::new();
    parser.set_language(&bend())?;
    Ok(parser)
}
