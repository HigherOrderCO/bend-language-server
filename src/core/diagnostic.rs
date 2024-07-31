use std::path::Path;

pub use bend::diagnostics::*;
use bend::{check_book, imports::DefaultLoader, CompileOpts};
use tower_lsp::lsp_types as lsp;

use super::document::Document;

/// Checks a Bend file its diagnostics.
///
/// The `Ok` variant represents diagnostics from the code, the `Err` variant
/// represents compilation error diagnostics.
pub fn check(doc: &Document) -> Result<Diagnostics, Diagnostics> {
    let path = Path::new(doc.url.path());
    let diagnostics_config = DiagnosticsConfig::new(Severity::Warning, true);
    let compile_opts = CompileOpts::default();

    let package_loader = DefaultLoader::new(path);
    let mut book = bend::load_file_to_book(path, package_loader, diagnostics_config)?;

    check_book(&mut book, diagnostics_config, compile_opts)
}

pub fn lsp_diagnostic(diagnostics: &Diagnostics) -> Vec<lsp::Diagnostic> {
    diagnostics
        .diagnostics
        .values()
        .flatten()
        .map(|diag| lsp::Diagnostic {
            range: lsp::Range::default(),
            severity: Some(lsp::DiagnosticSeverity::WARNING),
            code: todo!(),
            code_description: todo!(),
            source: todo!(),
            message: todo!(),
            related_information: todo!(),
            tags: todo!(),
            data: todo!(),
        })
        .collect()
}
