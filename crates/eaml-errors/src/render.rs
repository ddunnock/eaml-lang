//! codespan-reporting integration for EAML diagnostics.
//!
//! Converts EAML [`Diagnostic`] values into `codespan_reporting` diagnostics
//! and renders them to the terminal or to strings (for testing).

use codespan_reporting::diagnostic::{Diagnostic as CSDiagnostic, Label, Severity as CSSeverity};
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::{Buffer, ColorChoice, StandardStream};

use crate::diagnostic::Diagnostic;
use crate::severity::Severity;

/// Converts an EAML diagnostic to a codespan-reporting diagnostic.
///
/// Severity mapping:
/// - Fatal -> Bug
/// - Error -> Error
/// - Warning -> Warning
pub fn to_codespan(diag: &Diagnostic, file_id: usize) -> CSDiagnostic<usize> {
    let severity = match diag.severity {
        Severity::Fatal => CSSeverity::Bug,
        Severity::Error => CSSeverity::Error,
        Severity::Warning => CSSeverity::Warning,
    };

    let mut cs_diag = CSDiagnostic::new(severity)
        .with_code(diag.code.to_string())
        .with_message(&diag.message)
        .with_labels(vec![
            Label::primary(file_id, diag.span.clone()).with_message(&diag.label)
        ]);

    if !diag.hints.is_empty() {
        cs_diag = cs_diag.with_notes(diag.hints.clone());
    }

    cs_diag
}

/// Renders diagnostics to stderr with colored output.
pub fn render_diagnostics(files: &SimpleFiles<&str, &str>, diagnostics: &[Diagnostic]) {
    let writer = StandardStream::stderr(ColorChoice::Auto);
    let config = term::Config::default();
    for diag in diagnostics {
        let cs_diag = to_codespan(diag, 0);
        term::emit(&mut writer.lock(), &config, files, &cs_diag)
            .expect("writing to stderr should not fail");
    }
}

/// Renders diagnostics to a string (no color) for testing purposes.
pub fn render_to_string(files: &SimpleFiles<&str, &str>, diagnostics: &[Diagnostic]) -> String {
    let mut buffer = Buffer::no_color();
    let config = term::Config::default();
    for diag in diagnostics {
        let cs_diag = to_codespan(diag, 0);
        term::emit(&mut buffer, &config, files, &cs_diag)
            .expect("writing to buffer should not fail");
    }
    String::from_utf8(buffer.into_inner()).expect("codespan-reporting should produce valid UTF-8")
}
