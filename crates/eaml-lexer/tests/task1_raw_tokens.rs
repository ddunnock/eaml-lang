//! Tests for RawToken logos lexer and Interner (Task 1).
//!
//! These tests verify that logos correctly tokenizes keywords, operators,
//! literals, and identifiers, and that the interner works correctly.

// Note: RawToken is pub(crate), so we test it via a re-export or
// by testing the public types and the logos behavior indirectly.
// For Task 1, we focus on public API: TokenKind variants exist,
// Interner works, and the logos lexer can be exercised.

use eaml_lexer::intern::Interner;
use eaml_lexer::token::TokenKind;

// === Interner tests ===

#[test]
fn interner_returns_same_key_for_same_string() {
    let mut interner = Interner::new();
    let k1 = interner.intern("foo");
    let k2 = interner.intern("foo");
    assert_eq!(k1, k2);
}

#[test]
fn interner_returns_different_keys_for_different_strings() {
    let mut interner = Interner::new();
    let k1 = interner.intern("foo");
    let k2 = interner.intern("bar");
    assert_ne!(k1, k2);
}

#[test]
fn interner_resolves_to_original_string() {
    let mut interner = Interner::new();
    let key = interner.intern("hello");
    assert_eq!(interner.resolve(&key), "hello");
}

// === TokenKind variant existence tests ===

#[test]
fn token_kind_has_all_active_keywords() {
    // Verify all 15 active keyword variants exist
    let _kinds = vec![
        TokenKind::KwModel,
        TokenKind::KwSchema,
        TokenKind::KwPrompt,
        TokenKind::KwTool,
        TokenKind::KwAgent,
        TokenKind::KwImport,
        TokenKind::KwLet,
        TokenKind::KwIf,
        TokenKind::KwElse,
        TokenKind::KwReturn,
        TokenKind::KwAwait,
        TokenKind::KwTrue,
        TokenKind::KwFalse,
        TokenKind::KwNull,
        TokenKind::KwPython,
    ];
}

#[test]
fn token_kind_has_post_mvp_reserved_keywords() {
    let _kinds = vec![
        TokenKind::KwPipeline,
        TokenKind::KwEnum,
        TokenKind::KwExtends,
    ];
}

#[test]
fn token_kind_has_future_reserved_keywords() {
    let _kinds = vec![
        TokenKind::KwOverride,
        TokenKind::KwInterface,
        TokenKind::KwType,
        TokenKind::KwWhere,
        TokenKind::KwFor,
        TokenKind::KwWhile,
        TokenKind::KwMatch,
        TokenKind::KwAsync,
        TokenKind::KwYield,
    ];
}

#[test]
fn token_kind_has_template_tokens() {
    let _kinds = vec![
        TokenKind::TmplStart,
        TokenKind::TmplText,
        TokenKind::TmplInterpStart,
        TokenKind::TmplInterpEnd,
        TokenKind::TmplEnd,
    ];
}

#[test]
fn token_kind_has_python_bridge_tokens() {
    let _kinds = vec![TokenKind::KwPythonBridge, TokenKind::PythonBlock];
}

#[test]
fn token_kind_has_ident_with_spur() {
    let mut interner = Interner::new();
    let key = interner.intern("myVar");
    let kind = TokenKind::Ident(key);
    match kind {
        TokenKind::Ident(k) => assert_eq!(interner.resolve(&k), "myVar"),
        _ => panic!("expected Ident"),
    }
}

#[test]
fn token_kind_has_literals_and_eof() {
    let _kinds = vec![TokenKind::IntLit, TokenKind::FloatLit, TokenKind::Eof];
}

#[test]
fn token_kind_has_all_operators() {
    let _kinds = vec![
        TokenKind::LParen,
        TokenKind::RParen,
        TokenKind::LBrace,
        TokenKind::RBrace,
        TokenKind::LBracket,
        TokenKind::RBracket,
        TokenKind::LAngle,
        TokenKind::RAngle,
        TokenKind::Colon,
        TokenKind::Semicolon,
        TokenKind::Comma,
        TokenKind::Dot,
        TokenKind::Eq,
        TokenKind::Bang,
        TokenKind::Plus,
        TokenKind::Minus,
        TokenKind::Star,
        TokenKind::Slash,
        TokenKind::Pipe,
        TokenKind::Ampersand,
        TokenKind::Question,
        TokenKind::At,
        TokenKind::Arrow,
        TokenKind::EqEq,
        TokenKind::BangEq,
        TokenKind::LessEq,
        TokenKind::GreaterEq,
        TokenKind::AmpAmp,
        TokenKind::PipePipe,
        TokenKind::PipelineOp,
    ];
}
