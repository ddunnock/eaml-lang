//! Tests for comment skipping and span accuracy.

mod common;

use common::kinds;
use eaml_lexer::{lex, TokenKind};

#[test]
fn lex_line_comment_skipped() {
    let ks = kinds("// comment\nmodel");
    assert_eq!(ks, vec![TokenKind::KwModel, TokenKind::Eof]);
}

#[test]
fn lex_block_comment_skipped() {
    let ks = kinds("/* block */model");
    assert_eq!(ks, vec![TokenKind::KwModel, TokenKind::Eof]);
}

#[test]
fn lex_multiline_block_comment_skipped() {
    let ks = kinds("/* multi\nline */model");
    assert_eq!(ks, vec![TokenKind::KwModel, TokenKind::Eof]);
}

#[test]
fn lex_doc_comment_skipped() {
    let ks = kinds("/// doc comment\nmodel");
    assert_eq!(ks, vec![TokenKind::KwModel, TokenKind::Eof]);
}

#[test]
fn lex_code_after_line_comment() {
    let ks = kinds("// first\nschema // second\nmodel");
    assert_eq!(
        ks,
        vec![TokenKind::KwSchema, TokenKind::KwModel, TokenKind::Eof]
    );
}

#[test]
fn lex_span_after_comment_is_correct() {
    let output = lex("// comment\nmodel");
    // "model" starts at byte 11 (after "// comment\n")
    assert_eq!(output.tokens[0].kind, TokenKind::KwModel);
    assert_eq!(output.tokens[0].span, 11..16);
}

#[test]
fn lex_span_after_block_comment_is_correct() {
    let output = lex("/* xx */model");
    // "model" starts at byte 8 (after "/* xx */")
    assert_eq!(output.tokens[0].kind, TokenKind::KwModel);
    assert_eq!(output.tokens[0].span, 8..13);
}

#[test]
fn lex_multiple_comments_mixed() {
    let ks = kinds("// line\n/* block */\n/// doc\nmodel");
    assert_eq!(ks, vec![TokenKind::KwModel, TokenKind::Eof]);
}

#[test]
fn lex_comment_does_not_appear_in_tokens() {
    let output = lex("model // this is a comment\nschema");
    let ks: Vec<_> = output.tokens.iter().map(|t| t.kind).collect();
    // Should only have KwModel, KwSchema, Eof -- no comment tokens
    assert_eq!(
        ks,
        vec![TokenKind::KwModel, TokenKind::KwSchema, TokenKind::Eof]
    );
}

#[test]
fn lex_nested_block_comments_not_supported() {
    // Nested block comments are intentionally unsupported. The outer `*/` closes the
    // comment, and the remaining `outer */` is tokenized (producing errors or tokens).
    let output = lex("/* /* inner */ outer */model");
    // "outer" becomes an Ident, "*/" produces tokens, then "model"
    let has_model = output.tokens.iter().any(|t| t.kind == TokenKind::KwModel);
    assert!(
        has_model,
        "model should still be tokenized after nested comment residue"
    );
    // The key assertion: there should be more than just KwModel + Eof,
    // proving the inner `*/` closed the comment and `outer */` leaked out.
    assert!(
        output.tokens.len() > 2,
        "nested block comments should NOT be consumed as one comment"
    );
}

#[test]
fn lex_unexpected_char_produces_error_and_continues() {
    let output = lex("model\x01schema");
    let ks: Vec<_> = output.tokens.iter().map(|t| t.kind).collect();
    // Should have model, schema, eof (the \x01 is skipped with error)
    assert!(ks.contains(&TokenKind::KwModel));
    assert!(ks.contains(&TokenKind::KwSchema));
    assert_eq!(*ks.last().unwrap(), TokenKind::Eof);
    // Should have SYN001 diagnostic
    assert!(
        output
            .diagnostics
            .iter()
            .any(|d| d.code == eaml_errors::ErrorCode::Syn001),
        "expected SYN001 for unexpected character"
    );
}
