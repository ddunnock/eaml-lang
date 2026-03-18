//! Tests for literal tokenization (integers, floats, strings).

mod common;

use common::kinds;
use eaml_lexer::{lex, TokenKind};

// === Integer literals ===

#[test]
fn lex_integer_zero() {
    let ks = kinds("0");
    assert_eq!(ks, vec![TokenKind::IntLit, TokenKind::Eof]);
}

#[test]
fn lex_integer_42() {
    let output = lex("42");
    assert_eq!(output.tokens[0].kind, TokenKind::IntLit);
    assert_eq!(output.tokens[0].span, 0..2);
}

#[test]
fn lex_integer_1000() {
    let ks = kinds("1000");
    assert_eq!(ks, vec![TokenKind::IntLit, TokenKind::Eof]);
}

// === Float literals ===

#[test]
fn lex_float_zero_point_zero() {
    let ks = kinds("0.0");
    assert_eq!(ks, vec![TokenKind::FloatLit, TokenKind::Eof]);
}

#[test]
fn lex_float_pi() {
    let output = lex("3.14");
    assert_eq!(output.tokens[0].kind, TokenKind::FloatLit);
    assert_eq!(output.tokens[0].span, 0..4);
}

#[test]
fn lex_float_hundred() {
    let ks = kinds("100.5");
    assert_eq!(ks, vec![TokenKind::FloatLit, TokenKind::Eof]);
}

#[test]
fn lex_int_and_float() {
    let output = lex("42 3.14");
    assert_eq!(output.tokens[0].kind, TokenKind::IntLit);
    assert_eq!(output.tokens[0].span, 0..2);
    assert_eq!(output.tokens[1].kind, TokenKind::FloatLit);
    assert_eq!(output.tokens[1].span, 3..7);
    assert_eq!(output.tokens[2].kind, TokenKind::Eof);
}

// === String literals (as template tokens) ===

#[test]
fn lex_simple_string() {
    let ks = kinds(r#""hello""#);
    assert_eq!(
        ks,
        vec![
            TokenKind::TmplStart,
            TokenKind::TmplText,
            TokenKind::TmplEnd,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lex_empty_string() {
    let ks = kinds(r#""""#);
    assert_eq!(
        ks,
        vec![TokenKind::TmplStart, TokenKind::TmplEnd, TokenKind::Eof,]
    );
}

#[test]
fn lex_string_with_newline_escape() {
    let output = lex(r#""he\nllo""#);
    let ks: Vec<_> = output.tokens.iter().map(|t| t.kind).collect();
    assert_eq!(
        ks,
        vec![
            TokenKind::TmplStart,
            TokenKind::TmplText,
            TokenKind::TmplEnd,
            TokenKind::Eof,
        ]
    );
    assert!(
        output.diagnostics.is_empty(),
        "no errors expected for valid escapes"
    );
}

#[test]
fn lex_string_with_all_valid_escapes() {
    let output = lex(r#""a\nb\tc\rd\"e\\f""#);
    assert!(
        output.diagnostics.is_empty(),
        "all escapes are valid: {:?}",
        output.diagnostics
    );
}

#[test]
fn lex_unterminated_string() {
    let output = lex(r#""hello"#);
    let ks: Vec<_> = output.tokens.iter().map(|t| t.kind).collect();
    // Should still produce TmplStart, TmplText, TmplEnd (recovery), Eof
    assert_eq!(ks[0], TokenKind::TmplStart);
    assert!(ks.contains(&TokenKind::TmplEnd));
    assert_eq!(*ks.last().unwrap(), TokenKind::Eof);
    // Should have SYN002 diagnostic
    assert!(
        output
            .diagnostics
            .iter()
            .any(|d| d.code == eaml_errors::ErrorCode::Syn002),
        "expected SYN002 for unterminated string"
    );
}

#[test]
fn lex_string_with_invalid_escape() {
    let output = lex(r#""he\qllo""#);
    // Should have SYN004 diagnostic
    assert!(
        output
            .diagnostics
            .iter()
            .any(|d| d.code == eaml_errors::ErrorCode::Syn004),
        "expected SYN004 for invalid escape"
    );
}

// === Identifier interning ===

#[test]
fn lex_identifier() {
    let output = lex("myVar");
    match &output.tokens[0].kind {
        TokenKind::Ident(spur) => {
            assert_eq!(output.interner.resolve(*spur), "myVar");
        }
        other => panic!("expected Ident, got {:?}", other),
    }
    assert_eq!(output.tokens[1].kind, TokenKind::Eof);
}

#[test]
fn lex_same_identifier_twice_produces_same_spur() {
    let output = lex("foo foo");
    match (&output.tokens[0].kind, &output.tokens[1].kind) {
        (TokenKind::Ident(s1), TokenKind::Ident(s2)) => {
            assert_eq!(s1, s2, "same identifier should produce same Spur key");
        }
        _ => panic!("expected two Ident tokens"),
    }
}
