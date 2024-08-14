use std::path::Path;

pub use bend::diagnostics::*;
use bend::{check_book, imports::DefaultLoader, CompileOpts};
use tower_lsp::lsp_types::{self as lsp, Position};
use tree_sitter as ts;

use super::document::Document;
use crate::utils::color_wrapper::treat_colors;

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

pub fn lsp_diagnostics(doc: &Document, diagnostics: &Diagnostics) -> Vec<lsp::Diagnostic> {
    diagnostics
        .diagnostics
        // Iter<(DiagnosticOrigin, Vec<Diagnostic>)>
        .iter()
        // -> Iter<(DiagnosticOrigin, Diagnostic)>
        .flat_map(|(key, vals)| vals.iter().map(move |val| (key, val)))
        // Ignore unwanted diagnostics
        .filter_map(|(origin, diagnostic)| treat_diagnostic(doc, origin, diagnostic))
        .collect()
}

fn treat_diagnostic(
    doc: &Document,
    origin: &DiagnosticOrigin,
    diag: &Diagnostic,
) -> Option<lsp::Diagnostic> {
    Some(lsp::Diagnostic {
        message: treat_colors(&diag.display_with_origin(origin).to_string()),
        severity: match diag.severity {
            Severity::Allow => Some(lsp::DiagnosticSeverity::HINT),
            Severity::Warning => Some(lsp::DiagnosticSeverity::WARNING),
            Severity::Error => Some(lsp::DiagnosticSeverity::ERROR),
        },
        range: match origin {
            DiagnosticOrigin::Rule(name) => find_def(doc, name.as_ref())?,
            DiagnosticOrigin::Function(name) => find_def(doc, name.as_ref())?,
            DiagnosticOrigin::Inet(name) => find_def(doc, name.as_ref())?,
            _ => span_to_range(&diag.span),
        },
        code: None,
        code_description: None,
        source: Some("bend".into()),
        related_information: None,
        tags: None,
        data: None,
    })
}

/// Diagnostics with `Rule`, `Function` or `Inet` origins may have their
/// spans including entire definitions, while we would only like to
/// highlight their names.
fn find_def(doc: &Document, name: &str) -> Option<lsp::Range> {
    let query = format!(
        r#"
        (fun_function_definition
            name: (identifier) @name
            (#eq? @name "{name}"))
        (imp_function_definition
            name: (identifier) @name
            (#eq? @name "{name}"))
    "#
    );

    doc.find_one(&query)
        .map(|node| ts_range_to_lsp(node.range()))
}

fn span_to_range(span: &Option<FileSpan>) -> lsp::Range {
    span.as_ref()
        .map(|span| lsp::Range {
            start: Position {
                line: span.span.start.line as u32,
                character: span.span.start.char as u32,
            },
            end: Position {
                line: span.span.end.line as u32,
                character: span.span.end.char as u32,
            },
        })
        .unwrap_or_default()
}

pub fn ts_range_to_lsp(range: ts::Range) -> lsp::Range {
    lsp::Range {
        start: lsp::Position {
            line: range.start_point.row as u32,
            character: range.start_point.column as u32,
        },
        end: lsp::Position {
            line: range.end_point.row as u32,
            character: range.end_point.column as u32,
        },
    }
}
