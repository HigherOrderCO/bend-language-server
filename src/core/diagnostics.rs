use std::path::Path;

pub use bend::diagnostics::*;
use bend::{check_book, imports::DefaultLoader, CompileOpts};
use tower_lsp::lsp_types::{self as lsp, Position};

use super::document::Document;

/// Checks a Bend file and return its diagnostics.
pub fn check(doc: &Document) -> Diagnostics {
    let path = Path::new(doc.url.path());
    let diagnostics_config = DiagnosticsConfig::new(Severity::Warning, true);
    let compile_opts = CompileOpts::default();

    let package_loader = DefaultLoader::new(path);

    let diagnostics = bend::load_file_to_book(path, package_loader, diagnostics_config)
        .and_then(|mut book| check_book(&mut book, diagnostics_config, compile_opts));

    match diagnostics {
        Ok(d) | Err(d) => d,
    }
}

pub fn lsp_diagnostics(diagnostics: &Diagnostics) -> Vec<lsp::Diagnostic> {
    diagnostics
        .diagnostics
        .values()
        .flatten()
        .filter_map(treat_diagnostic)
        .collect()
}

fn treat_diagnostic(diag: &Diagnostic) -> Option<lsp::Diagnostic> {
    Some(lsp::Diagnostic {
        message: diag.message.clone(),
        severity: match diag.severity {
            Severity::Allow => Some(lsp::DiagnosticSeverity::HINT),
            Severity::Warning => Some(lsp::DiagnosticSeverity::WARNING),
            Severity::Error => Some(lsp::DiagnosticSeverity::ERROR),
        },
        range: diag
            .span
            .as_ref()
            .map(|s| span_to_range(&s.span))
            .unwrap_or_default(),
        code: None,
        code_description: None,
        source: Some("bend".into()),
        related_information: None,
        tags: None,
        data: None,
    })
}

fn span_to_range(span: &TextSpan) -> lsp::Range {
    lsp::Range {
        start: Position {
            line: span.0.line as u32,
            character: span.0.char as u32,
        },
        end: Position {
            line: span.1.line as u32,
            character: span.1.char as u32,
        },
    }
}
