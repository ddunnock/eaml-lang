//! Comprehensive error recovery and diagnostic tests with insta snapshots.

use eaml_lexer::{lex, LexOutput, TokenKind};

/// Formats tokens from a LexOutput for snapshot comparison.
fn format_tokens(output: &LexOutput) -> String {
    output
        .tokens
        .iter()
        .map(|t| {
            let kind_str = match &t.kind {
                TokenKind::Ident(spur) => {
                    format!("Ident({})", output.interner.resolve(spur))
                }
                other => format!("{:?}", other),
            };
            format!("{} @ {}..{}", kind_str, t.span.start, t.span.end)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Formats diagnostics from a LexOutput for snapshot comparison.
fn format_diagnostics(output: &LexOutput) -> String {
    if output.diagnostics.is_empty() {
        return "no diagnostics".to_string();
    }
    output
        .diagnostics
        .iter()
        .map(|d| {
            format!(
                "[{}] {}..{}: {}",
                d.code, d.span.start, d.span.end, d.message
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn error_unexpected_char() {
    let output = lex("~");
    insta::assert_snapshot!(format_diagnostics(&output), @"[SYN001] 0..1: unexpected character `~`");
    // Should still have Eof token
    assert_eq!(output.tokens.last().unwrap().kind, TokenKind::Eof);
}

#[test]
fn error_adjacent_unexpected_chars() {
    // Multiple adjacent unexpected chars should collapse into one diagnostic
    let output = lex("~~~");
    // After error collapsing, adjacent SYN001 diagnostics merge into one
    insta::assert_snapshot!(format_diagnostics(&output), @"[SYN001] 0..3: unexpected characters");
}

#[test]
fn error_recovery_continues() {
    // After an error, valid tokens should still be produced
    let output = lex("~ model");
    insta::assert_snapshot!(format_tokens(&output), @r"
    KwModel @ 2..7
    Eof @ 7..7
    ");
    assert_eq!(output.diagnostics.len(), 1);
}

#[test]
fn error_mixed_valid_invalid() {
    let output = lex("model ~ schema");
    insta::assert_snapshot!(format_tokens(&output), @r"
    KwModel @ 0..5
    KwSchema @ 8..14
    Eof @ 14..14
    ");
    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(output.diagnostics[0].code, eaml_errors::ErrorCode::Syn001);
}

#[test]
fn error_unterminated_string_recovery() {
    // Unterminated string -- the lexer includes all content as template text
    // Note: { and } inside the string trigger interpolation mode since the lexer
    // treats all strings as templates
    let output = lex("\"unterminated\nmodel Foo {}");
    // Should have SYN002 for the unterminated string
    assert!(
        output
            .diagnostics
            .iter()
            .any(|d| d.code == eaml_errors::ErrorCode::Syn002),
        "expected SYN002"
    );
    // The { triggers interpolation, then } closes it, then unterminated
    insta::assert_snapshot!(format_tokens(&output), @r"
    TmplStart @ 0..1
    TmplText @ 1..24
    TmplInterpStart @ 24..25
    TmplInterpEnd @ 25..26
    TmplEnd @ 26..26
    Eof @ 26..26
    ");
}

#[test]
fn error_multiple_types() {
    // Source triggering different error types: SYN001 (unexpected char) and SYN004 (bad escape)
    let output = lex(r#"~ "bad\qescape""#);
    let codes: Vec<_> = output.diagnostics.iter().map(|d| d.code).collect();
    assert!(codes.contains(&eaml_errors::ErrorCode::Syn001));
    assert!(codes.contains(&eaml_errors::ErrorCode::Syn004));
}

#[test]
fn error_full_eaml_with_errors() {
    let source = r#"model Greeter {
    provider: "anthropic"
    prompt: "Hello, {name}! ~ Welcome"
}
~ extra
schema Output {
    greeting: "default {val}"
}"#;
    let output = lex(source);
    insta::assert_snapshot!(format_tokens(&output), @r#"
    KwModel @ 0..5
    Ident(Greeter) @ 6..13
    LBrace @ 14..15
    Ident(provider) @ 20..28
    Colon @ 28..29
    TmplStart @ 30..31
    TmplText @ 31..40
    TmplEnd @ 40..41
    KwPrompt @ 46..52
    Colon @ 52..53
    TmplStart @ 54..55
    TmplText @ 55..62
    TmplInterpStart @ 62..63
    Ident(name) @ 63..67
    TmplInterpEnd @ 67..68
    TmplText @ 68..79
    TmplEnd @ 79..80
    RBrace @ 81..82
    Ident(extra) @ 85..90
    KwSchema @ 91..97
    Ident(Output) @ 98..104
    LBrace @ 105..106
    Ident(greeting) @ 111..119
    Colon @ 119..120
    TmplStart @ 121..122
    TmplText @ 122..130
    TmplInterpStart @ 130..131
    Ident(val) @ 131..134
    TmplInterpEnd @ 134..135
    TmplEnd @ 135..136
    RBrace @ 137..138
    Eof @ 138..138
    "#);
    // Should have errors for the ~ characters
    assert!(
        !output.diagnostics.is_empty(),
        "should have diagnostics for ~ chars"
    );
}
