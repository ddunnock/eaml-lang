use eaml_errors::render::{render_to_string, to_codespan};
use eaml_errors::{Diagnostic, DiagnosticCollector, ErrorCode, Severity};

fn make_error(code: ErrorCode, msg: &str, span: std::ops::Range<usize>) -> Diagnostic {
    Diagnostic::new(
        code,
        msg.to_string(),
        span,
        Severity::Error,
        "here".to_string(),
    )
}

fn make_warning(code: ErrorCode, msg: &str, span: std::ops::Range<usize>) -> Diagnostic {
    Diagnostic::new(
        code,
        msg.to_string(),
        span,
        Severity::Warning,
        "here".to_string(),
    )
}

// --- DiagnosticCollector tests ---

#[test]
fn collector_new_sets_max_errors() {
    let collector = DiagnosticCollector::new(20);
    assert_eq!(collector.error_count(), 0);
    assert!(!collector.has_errors());
    assert!(!collector.overflow());
}

#[test]
fn collector_emit_adds_diagnostic() {
    let mut collector = DiagnosticCollector::new(20);
    collector.emit(make_error(ErrorCode::Syn042, "test", 0..5));
    assert_eq!(collector.diagnostics().len(), 1);
}

#[test]
fn collector_has_errors_true_after_error() {
    let mut collector = DiagnosticCollector::new(20);
    collector.emit(make_error(ErrorCode::Syn042, "test", 0..5));
    assert!(collector.has_errors());
}

#[test]
fn collector_has_errors_false_for_warnings_only() {
    let mut collector = DiagnosticCollector::new(20);
    collector.emit(make_warning(ErrorCode::Syn083, "optional semicolon", 0..5));
    assert!(!collector.has_errors());
}

#[test]
fn collector_overflow_after_max_errors() {
    let mut collector = DiagnosticCollector::new(20);
    for i in 0..21 {
        collector.emit(make_error(ErrorCode::Syn042, "test", i..i + 1));
    }
    assert!(collector.overflow());
    // Only 20 should be stored
    assert_eq!(collector.diagnostics().len(), 20);
}

#[test]
fn collector_error_count_tracks_errors() {
    let mut collector = DiagnosticCollector::new(20);
    collector.emit(make_error(ErrorCode::Syn042, "err1", 0..1));
    collector.emit(make_warning(ErrorCode::Syn083, "warn", 1..2));
    collector.emit(make_error(ErrorCode::Syn043, "err2", 2..3));
    assert_eq!(collector.error_count(), 2);
    // All 3 diagnostics stored (warnings don't count toward limit)
    assert_eq!(collector.diagnostics().len(), 3);
}

#[test]
fn collector_diagnostics_returns_slice() {
    let mut collector = DiagnosticCollector::new(20);
    collector.emit(make_error(ErrorCode::Syn042, "first", 0..1));
    collector.emit(make_error(ErrorCode::Syn043, "second", 1..2));
    let diags = collector.diagnostics();
    assert_eq!(diags.len(), 2);
    assert_eq!(diags[0].code, ErrorCode::Syn042);
    assert_eq!(diags[1].code, ErrorCode::Syn043);
}

#[test]
fn collector_fatal_counts_as_error() {
    let mut collector = DiagnosticCollector::new(20);
    collector.emit(Diagnostic::new(
        ErrorCode::Syn001,
        "fatal".to_string(),
        0..1,
        Severity::Fatal,
        "here".to_string(),
    ));
    assert!(collector.has_errors());
    assert_eq!(collector.error_count(), 1);
}

// --- codespan-reporting integration tests ---

#[test]
fn to_codespan_maps_severity() {
    use codespan_reporting::diagnostic::Severity as CSSeverity;

    let fatal_diag = Diagnostic::new(
        ErrorCode::Syn001,
        "fatal error".to_string(),
        0..1,
        Severity::Fatal,
        "here".to_string(),
    );
    let cs = to_codespan(&fatal_diag, 0);
    assert_eq!(cs.severity, CSSeverity::Bug);

    let error_diag = make_error(ErrorCode::Syn042, "error", 0..1);
    let cs = to_codespan(&error_diag, 0);
    assert_eq!(cs.severity, CSSeverity::Error);

    let warning_diag = make_warning(ErrorCode::Syn083, "warning", 0..1);
    let cs = to_codespan(&warning_diag, 0);
    assert_eq!(cs.severity, CSSeverity::Warning);
}

#[test]
fn to_codespan_includes_error_code() {
    let diag = make_error(ErrorCode::Syn042, "test", 0..1);
    let cs = to_codespan(&diag, 0);
    assert_eq!(cs.code, Some("SYN042".to_string()));
}

#[test]
fn to_codespan_includes_primary_label() {
    let diag = Diagnostic::new(
        ErrorCode::Syn042,
        "test message".to_string(),
        5..10,
        Severity::Error,
        "this is the label".to_string(),
    );
    let cs = to_codespan(&diag, 0);
    assert_eq!(cs.labels.len(), 1);
    assert_eq!(cs.labels[0].range, 5..10);
}

#[test]
fn to_codespan_includes_hints_as_notes() {
    let diag = make_error(ErrorCode::Syn042, "test", 0..1)
        .with_hint("hint one")
        .with_hint("hint two");
    let cs = to_codespan(&diag, 0);
    assert_eq!(cs.notes.len(), 2);
    assert_eq!(cs.notes[0], "hint one");
    assert_eq!(cs.notes[1], "hint two");
}

#[test]
fn render_to_string_contains_error_code_and_message() {
    use codespan_reporting::files::SimpleFiles;

    let mut files = SimpleFiles::new();
    let _file_id = files.add("test.eaml", "let x = 42;");

    let diag = make_error(ErrorCode::Syn042, "unterminated string", 4..5);
    let output = render_to_string(&files, &[diag]);
    assert!(
        output.contains("SYN042"),
        "output should contain error code: {output}"
    );
    assert!(
        output.contains("unterminated string"),
        "output should contain message: {output}"
    );
}
