//! Tests for operator and delimiter tokenization.

use eaml_lexer::{lex, TokenKind};

fn kinds(source: &str) -> Vec<TokenKind> {
    let output = lex(source);
    output.tokens.iter().map(|t| t.kind.clone()).collect()
}

#[test]
fn lex_single_char_operators() {
    let tests = vec![
        ("(", TokenKind::LParen),
        (")", TokenKind::RParen),
        ("{", TokenKind::LBrace),
        ("}", TokenKind::RBrace),
        ("[", TokenKind::LBracket),
        ("]", TokenKind::RBracket),
        ("<", TokenKind::LAngle),
        (">", TokenKind::RAngle),
        (":", TokenKind::Colon),
        (";", TokenKind::Semicolon),
        (",", TokenKind::Comma),
        (".", TokenKind::Dot),
        ("=", TokenKind::Eq),
        ("!", TokenKind::Bang),
        ("+", TokenKind::Plus),
        ("-", TokenKind::Minus),
        ("*", TokenKind::Star),
        ("/", TokenKind::Slash),
        ("|", TokenKind::Pipe),
        ("&", TokenKind::Ampersand),
        ("?", TokenKind::Question),
        ("@", TokenKind::At),
    ];
    for (source, expected) in tests {
        let ks = kinds(source);
        assert_eq!(ks, vec![expected, TokenKind::Eof], "failed for: {source}");
    }
}

#[test]
fn lex_multi_char_operators() {
    let ks = kinds("-> == != <= >= && || >>");
    assert_eq!(
        ks,
        vec![
            TokenKind::Arrow,
            TokenKind::EqEq,
            TokenKind::BangEq,
            TokenKind::LessEq,
            TokenKind::GreaterEq,
            TokenKind::AmpAmp,
            TokenKind::PipePipe,
            TokenKind::PipelineOp,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lex_arrow_operator() {
    let ks = kinds("->");
    assert_eq!(ks, vec![TokenKind::Arrow, TokenKind::Eof]);
}

#[test]
fn lex_eqeq_operator() {
    let ks = kinds("==");
    assert_eq!(ks, vec![TokenKind::EqEq, TokenKind::Eof]);
}

#[test]
fn lex_mixed_operators_and_keywords() {
    let ks = kinds("if x == 42");
    assert_eq!(ks[0], TokenKind::KwIf);
    assert!(matches!(ks[1], TokenKind::Ident(_)));
    assert_eq!(ks[2], TokenKind::EqEq);
    assert_eq!(ks[3], TokenKind::IntLit);
    assert_eq!(ks[4], TokenKind::Eof);
}
