//! Tests for keyword tokenization via the public lex() API.

mod common;

use common::kinds;
use eaml_lexer::{lex, TokenKind};

#[test]
fn lex_model_keyword() {
    let kinds = kinds("model");
    assert_eq!(kinds, vec![TokenKind::KwModel, TokenKind::Eof]);
}

#[test]
fn lex_schema_keyword() {
    let kinds = kinds("schema");
    assert_eq!(kinds, vec![TokenKind::KwSchema, TokenKind::Eof]);
}

#[test]
fn lex_all_active_keywords() {
    let expected = vec![
        ("model", TokenKind::KwModel),
        ("schema", TokenKind::KwSchema),
        ("prompt", TokenKind::KwPrompt),
        ("tool", TokenKind::KwTool),
        ("agent", TokenKind::KwAgent),
        ("import", TokenKind::KwImport),
        ("let", TokenKind::KwLet),
        ("if", TokenKind::KwIf),
        ("else", TokenKind::KwElse),
        ("return", TokenKind::KwReturn),
        ("await", TokenKind::KwAwait),
        ("true", TokenKind::KwTrue),
        ("false", TokenKind::KwFalse),
        ("null", TokenKind::KwNull),
        ("python", TokenKind::KwPython),
    ];
    for (source, kind) in expected {
        let ks = kinds(source);
        assert_eq!(ks, vec![kind, TokenKind::Eof], "failed for: {source}");
    }
}

#[test]
fn lex_post_mvp_reserved_keywords() {
    let expected = vec![
        ("pipeline", TokenKind::KwPipeline),
        ("enum", TokenKind::KwEnum),
        ("extends", TokenKind::KwExtends),
    ];
    for (source, kind) in expected {
        let ks = kinds(source);
        assert_eq!(ks, vec![kind, TokenKind::Eof], "failed for: {source}");
    }
}

#[test]
fn lex_future_reserved_keywords() {
    let expected = vec![
        ("override", TokenKind::KwOverride),
        ("interface", TokenKind::KwInterface),
        ("type", TokenKind::KwType),
        ("where", TokenKind::KwWhere),
        ("for", TokenKind::KwFor),
        ("while", TokenKind::KwWhile),
        ("match", TokenKind::KwMatch),
        ("async", TokenKind::KwAsync),
        ("yield", TokenKind::KwYield),
    ];
    for (source, kind) in expected {
        let ks = kinds(source);
        assert_eq!(ks, vec![kind, TokenKind::Eof], "failed for: {source}");
    }
}

#[test]
fn lex_all_27_keywords_in_one_string() {
    let source = "model schema prompt tool agent import let if else return await true false null python pipeline enum extends override interface type where for while match async yield";
    let ks = kinds(source);
    assert_eq!(ks.len(), 28); // 27 keywords + Eof
    assert_eq!(*ks.last().unwrap(), TokenKind::Eof);
}

#[test]
fn lex_schema_foo_braces() {
    let output = lex("schema Foo { }");
    let kinds: Vec<_> = output.tokens.iter().map(|t| t.kind).collect();
    assert_eq!(kinds[0], TokenKind::KwSchema);
    // Foo is an identifier
    match &kinds[1] {
        TokenKind::Ident(spur) => {
            assert_eq!(output.interner.resolve(spur), "Foo");
        }
        other => panic!("expected Ident, got {:?}", other),
    }
    assert_eq!(kinds[2], TokenKind::LBrace);
    assert_eq!(kinds[3], TokenKind::RBrace);
    assert_eq!(kinds[4], TokenKind::Eof);
}
