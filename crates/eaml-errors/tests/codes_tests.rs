use eaml_errors::{Diagnostic, ErrorCode, Severity};

#[test]
fn error_code_syn042_display() {
    assert_eq!(ErrorCode::Syn042.to_string(), "SYN042");
}

#[test]
fn error_code_res001_display() {
    assert_eq!(ErrorCode::Res001.to_string(), "RES001");
}

#[test]
fn error_code_display_all_prefixes() {
    assert_eq!(ErrorCode::Syn001.to_string(), "SYN001");
    assert_eq!(ErrorCode::Sem010.to_string(), "SEM010");
    assert_eq!(ErrorCode::Typ001.to_string(), "TYP001");
    assert_eq!(ErrorCode::Cap001.to_string(), "CAP001");
    assert_eq!(ErrorCode::Pyb001.to_string(), "PYB001");
    assert_eq!(ErrorCode::Res001.to_string(), "RES001");
}

#[test]
fn error_code_prefix_method() {
    assert_eq!(ErrorCode::Syn042.prefix(), "SYN");
    assert_eq!(ErrorCode::Sem010.prefix(), "SEM");
    assert_eq!(ErrorCode::Typ001.prefix(), "TYP");
    assert_eq!(ErrorCode::Cap010.prefix(), "CAP");
    assert_eq!(ErrorCode::Pyb001.prefix(), "PYB");
    assert_eq!(ErrorCode::Res001.prefix(), "RES");
}

#[test]
fn error_code_number_method() {
    assert_eq!(ErrorCode::Syn042.number(), 42);
    assert_eq!(ErrorCode::Res001.number(), 1);
    assert_eq!(ErrorCode::Sem070.number(), 70);
    assert_eq!(ErrorCode::Typ040.number(), 40);
}

#[test]
fn all_42_error_codes_exist() {
    // 4 new lexer codes
    let _codes = [
        ErrorCode::Syn001,
        ErrorCode::Syn002,
        ErrorCode::Syn003,
        ErrorCode::Syn004,
        // 5 spec-defined SYN lexer codes
        ErrorCode::Syn042,
        ErrorCode::Syn043,
        ErrorCode::Syn044,
        ErrorCode::Syn045,
        ErrorCode::Syn046,
        // 8 SYN parser codes
        ErrorCode::Syn050,
        ErrorCode::Syn060,
        ErrorCode::Syn061,
        ErrorCode::Syn080,
        ErrorCode::Syn081,
        ErrorCode::Syn082,
        ErrorCode::Syn083,
        ErrorCode::Syn090,
        // 10 SEM codes
        ErrorCode::Sem010,
        ErrorCode::Sem020,
        ErrorCode::Sem025,
        ErrorCode::Sem030,
        ErrorCode::Sem035,
        ErrorCode::Sem040,
        ErrorCode::Sem050,
        ErrorCode::Sem060,
        ErrorCode::Sem061,
        ErrorCode::Sem070,
        // 8 TYP codes
        ErrorCode::Typ001,
        ErrorCode::Typ002,
        ErrorCode::Typ003,
        ErrorCode::Typ010,
        ErrorCode::Typ030,
        ErrorCode::Typ031,
        ErrorCode::Typ032,
        ErrorCode::Typ040,
        // 4 CAP codes
        ErrorCode::Cap001,
        ErrorCode::Cap002,
        ErrorCode::Cap010,
        ErrorCode::Cap020,
        // 2 PYB codes
        ErrorCode::Pyb001,
        ErrorCode::Pyb010,
        // 1 RES code
        ErrorCode::Res001,
    ];
    assert_eq!(_codes.len(), 42);
}

#[test]
fn error_code_derives() {
    // Debug
    let _debug = format!("{:?}", ErrorCode::Syn042);
    // Clone
    let a = ErrorCode::Syn042;
    let b = a;
    assert_eq!(a, b);
    // Copy (implicit in the line above)
    // PartialEq, Eq
    assert_eq!(ErrorCode::Syn042, ErrorCode::Syn042);
    assert_ne!(ErrorCode::Syn042, ErrorCode::Syn043);
    // Hash
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(ErrorCode::Syn042);
    assert!(set.contains(&ErrorCode::Syn042));
}

#[test]
fn severity_variants_exist() {
    let _fatal = Severity::Fatal;
    let _error = Severity::Error;
    let _warning = Severity::Warning;
}

#[test]
fn diagnostic_new_constructor() {
    let diag = Diagnostic::new(
        ErrorCode::Syn042,
        "unterminated string".to_string(),
        0..10,
        Severity::Error,
        "here".to_string(),
    );
    assert_eq!(diag.code, ErrorCode::Syn042);
    assert_eq!(diag.message, "unterminated string");
    assert_eq!(diag.span, 0..10);
    assert_eq!(diag.severity, Severity::Error);
    assert_eq!(diag.label, "here");
    assert!(diag.hints.is_empty());
}

#[test]
fn diagnostic_with_hint() {
    let diag = Diagnostic::new(
        ErrorCode::Syn042,
        "unterminated string".to_string(),
        0..10,
        Severity::Error,
        "here".to_string(),
    )
    .with_hint("try adding a closing quote");

    assert_eq!(diag.hints.len(), 1);
    assert_eq!(diag.hints[0], "try adding a closing quote");
}

#[test]
fn diagnostic_no_hints_has_empty_vec() {
    let diag = Diagnostic::new(
        ErrorCode::Syn042,
        "test".to_string(),
        0..1,
        Severity::Error,
        "label".to_string(),
    );
    assert_eq!(diag.hints, Vec::<String>::new());
}
