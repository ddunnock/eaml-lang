//! Template string tokenization tests with insta snapshots.
//!
//! Tests interpolation, brace-depth tracking, escape handling, and error recovery.

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
fn template_plain_string() {
    let output = lex(r#""hello""#);
    insta::assert_snapshot!(format_tokens(&output), @r"
    TmplStart @ 0..1
    TmplText @ 1..6
    TmplEnd @ 6..7
    Eof @ 7..7
    ");
}

#[test]
fn template_simple_interpolation() {
    let output = lex(r#""Hello, {name}!""#);
    insta::assert_snapshot!(format_tokens(&output), @r"
    TmplStart @ 0..1
    TmplText @ 1..8
    TmplInterpStart @ 8..9
    Ident(name) @ 9..13
    TmplInterpEnd @ 13..14
    TmplText @ 14..15
    TmplEnd @ 15..16
    Eof @ 16..16
    ");
}

#[test]
fn template_nested_braces() {
    let output = lex(r#""Result: {fn({x})}""#);
    insta::assert_snapshot!(format_tokens(&output), @r"
    TmplStart @ 0..1
    TmplText @ 1..9
    TmplInterpStart @ 9..10
    Ident(fn) @ 10..12
    LParen @ 12..13
    LBrace @ 13..14
    Ident(x) @ 14..15
    RBrace @ 15..16
    RParen @ 16..17
    TmplInterpEnd @ 17..18
    TmplEnd @ 18..19
    Eof @ 19..19
    ");
}

#[test]
fn template_escaped_braces() {
    let output = lex(r#""Use {{braces}} here""#);
    insta::assert_snapshot!(format_tokens(&output), @r"
    TmplStart @ 0..1
    TmplText @ 1..20
    TmplEnd @ 20..21
    Eof @ 21..21
    ");
    assert!(output.diagnostics.is_empty());
}

#[test]
fn template_multiple_interpolations() {
    let output = lex(r#""a {x} b {y} c""#);
    insta::assert_snapshot!(format_tokens(&output), @r"
    TmplStart @ 0..1
    TmplText @ 1..3
    TmplInterpStart @ 3..4
    Ident(x) @ 4..5
    TmplInterpEnd @ 5..6
    TmplText @ 6..9
    TmplInterpStart @ 9..10
    Ident(y) @ 10..11
    TmplInterpEnd @ 11..12
    TmplText @ 12..14
    TmplEnd @ 14..15
    Eof @ 15..15
    ");
}

#[test]
fn template_empty_interpolation() {
    let output = lex(r#""empty {}""#);
    insta::assert_snapshot!(format_tokens(&output), @r"
    TmplStart @ 0..1
    TmplText @ 1..7
    TmplInterpStart @ 7..8
    TmplInterpEnd @ 8..9
    TmplEnd @ 9..10
    Eof @ 10..10
    ");
}

#[test]
fn template_escape_sequences() {
    let output = lex(r#""tab\there\nnewline""#);
    insta::assert_snapshot!(format_tokens(&output), @r"
    TmplStart @ 0..1
    TmplText @ 1..19
    TmplEnd @ 19..20
    Eof @ 20..20
    ");
    assert!(output.diagnostics.is_empty());
}

#[test]
fn template_adjacent_strings() {
    let output = lex(r#""first" "second""#);
    insta::assert_snapshot!(format_tokens(&output), @r"
    TmplStart @ 0..1
    TmplText @ 1..6
    TmplEnd @ 6..7
    TmplStart @ 8..9
    TmplText @ 9..15
    TmplEnd @ 15..16
    Eof @ 16..16
    ");
}

#[test]
fn template_unclosed_interpolation() {
    let output = lex(r#""text {expr"#);
    insta::assert_snapshot!(format_tokens(&output), @r"
    TmplStart @ 0..1
    TmplText @ 1..6
    TmplInterpStart @ 6..7
    Ident(expr) @ 7..11
    Eof @ 11..11
    ");
    insta::assert_snapshot!(format_diagnostics(&output), @"[SYN045] 11..11: unclosed template string interpolation");
}

#[test]
fn template_unterminated() {
    let output = lex(r#""unterminated"#);
    insta::assert_snapshot!(format_tokens(&output), @r"
    TmplStart @ 0..1
    TmplText @ 1..13
    TmplEnd @ 13..13
    Eof @ 13..13
    ");
    insta::assert_snapshot!(format_diagnostics(&output), @"[SYN002] 1..13: unterminated string literal");
}

#[test]
fn template_in_context() {
    let output = lex(r#"schema Foo { name: "default {x}" }"#);
    insta::assert_snapshot!(format_tokens(&output), @r#"
    KwSchema @ 0..6
    Ident(Foo) @ 7..10
    LBrace @ 11..12
    Ident(name) @ 13..17
    Colon @ 17..18
    TmplStart @ 19..20
    TmplText @ 20..28
    TmplInterpStart @ 28..29
    Ident(x) @ 29..30
    TmplInterpEnd @ 30..31
    TmplEnd @ 31..32
    RBrace @ 33..34
    Eof @ 34..34
    "#);
}
