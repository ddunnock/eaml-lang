//! Logos-derived inner lexer for EAML.
//!
//! This module defines [`RawToken`], a DFA-generated token enum that handles
//! stateless tokenization of keywords, operators, literals, and whitespace/comment
//! skipping. The outer [`Lexer`](crate::lexer::Lexer) wraps this for mode switching.

use logos::Logos;

/// Raw token variants produced by the logos DFA.
///
/// These are internal to the lexer crate. The public API exposes
/// [`TokenKind`](crate::token::TokenKind) via the wrapper lexer.
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\r\n]+")]
// The `///` rule is kept separate for future doc-comment capture (post-MVP).
// Currently redundant with `//[^\n]*` but makes the intent explicit.
#[logos(skip r"///[^\n]*")]
#[logos(skip r"//[^\n]*")]
// Note: nested block comments (/* /* */ */) are intentionally unsupported.
#[logos(skip r"/\*[^*]*\*+(?:[^/*][^*]*\*+)*/")]
pub(crate) enum RawToken {
    // === Keywords (active v0.1) ===
    #[token("model")]
    KwModel,
    #[token("schema")]
    KwSchema,
    #[token("prompt")]
    KwPrompt,
    #[token("tool")]
    KwTool,
    #[token("agent")]
    KwAgent,
    #[token("import")]
    KwImport,
    #[token("let")]
    KwLet,
    #[token("if")]
    KwIf,
    #[token("else")]
    KwElse,
    #[token("return")]
    KwReturn,
    #[token("await")]
    KwAwait,
    #[token("true")]
    KwTrue,
    #[token("false")]
    KwFalse,
    #[token("null")]
    KwNull,
    #[token("python")]
    KwPython,

    // === Keywords (post-MVP reserved) ===
    #[token("pipeline")]
    KwPipeline,
    #[token("enum")]
    KwEnum,
    #[token("extends")]
    KwExtends,

    // === Keywords (future reserved) ===
    #[token("override")]
    KwOverride,
    #[token("interface")]
    KwInterface,
    #[token("type")]
    KwType,
    #[token("where")]
    KwWhere,
    #[token("for")]
    KwFor,
    #[token("while")]
    KwWhile,
    #[token("match")]
    KwMatch,
    #[token("async")]
    KwAsync,
    #[token("yield")]
    KwYield,

    // === Single-char operators ===
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token("<")]
    LAngle,
    #[token(">")]
    RAngle,
    #[token(":")]
    Colon,
    #[token(";")]
    Semicolon,
    #[token(",")]
    Comma,
    #[token(".")]
    Dot,
    #[token("=")]
    Eq,
    #[token("!")]
    Bang,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("|")]
    Pipe,
    #[token("&")]
    Ampersand,
    #[token("?")]
    Question,
    #[token("@")]
    At,

    // === Multi-char operators ===
    #[token("->")]
    Arrow,
    #[token("==")]
    EqEq,
    #[token("!=")]
    BangEq,
    #[token("<=")]
    LessEq,
    #[token(">=")]
    GreaterEq,
    #[token("&&")]
    AmpAmp,
    #[token("||")]
    PipePipe,
    #[token(">>")]
    PipelineOp,
    #[token("%{")]
    PercentLBrace,
    #[token("}%")]
    RBracePercent,

    // === Literals ===
    #[regex(r"0|[1-9][0-9]*", priority = 3)]
    IntLit,

    #[regex(r"(0|[1-9][0-9]*)\.[0-9]+", priority = 4)]
    FloatLit,

    #[token("\"")]
    DoubleQuote,

    // === Identifiers ===
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
    Ident,
}

#[cfg(test)]
mod tests {
    use super::*;
    use logos::Logos;

    /// Helper: collect all tokens from source.
    fn raw_tokens(source: &str) -> Vec<(RawToken, &str)> {
        let lex = RawToken::lexer(source);
        lex.spanned()
            .filter_map(|(result, span)| result.ok().map(|tok| (tok, &source[span])))
            .collect()
    }

    #[test]
    fn tokenizes_model_keyword() {
        let tokens = raw_tokens("model");
        assert_eq!(tokens, vec![(RawToken::KwModel, "model")]);
    }

    #[test]
    fn tokenizes_schema_keyword() {
        let tokens = raw_tokens("schema");
        assert_eq!(tokens, vec![(RawToken::KwSchema, "schema")]);
    }

    #[test]
    fn tokenizes_all_active_keywords() {
        let keywords = vec![
            ("model", RawToken::KwModel),
            ("schema", RawToken::KwSchema),
            ("prompt", RawToken::KwPrompt),
            ("tool", RawToken::KwTool),
            ("agent", RawToken::KwAgent),
            ("import", RawToken::KwImport),
            ("let", RawToken::KwLet),
            ("if", RawToken::KwIf),
            ("else", RawToken::KwElse),
            ("return", RawToken::KwReturn),
            ("await", RawToken::KwAwait),
            ("true", RawToken::KwTrue),
            ("false", RawToken::KwFalse),
            ("null", RawToken::KwNull),
            ("python", RawToken::KwPython),
        ];
        for (source, expected) in keywords {
            let tokens = raw_tokens(source);
            assert_eq!(tokens, vec![(expected, source)], "failed for: {source}");
        }
    }

    #[test]
    fn tokenizes_all_reserved_keywords() {
        let keywords = vec![
            ("pipeline", RawToken::KwPipeline),
            ("enum", RawToken::KwEnum),
            ("extends", RawToken::KwExtends),
            ("override", RawToken::KwOverride),
            ("interface", RawToken::KwInterface),
            ("type", RawToken::KwType),
            ("where", RawToken::KwWhere),
            ("for", RawToken::KwFor),
            ("while", RawToken::KwWhile),
            ("match", RawToken::KwMatch),
            ("async", RawToken::KwAsync),
            ("yield", RawToken::KwYield),
        ];
        for (source, expected) in keywords {
            let tokens = raw_tokens(source);
            assert_eq!(tokens, vec![(expected, source)], "failed for: {source}");
        }
    }

    #[test]
    fn tokenizes_integer_literal() {
        let tokens = raw_tokens("42");
        assert_eq!(tokens, vec![(RawToken::IntLit, "42")]);
    }

    #[test]
    fn tokenizes_float_literal() {
        let tokens = raw_tokens("3.14");
        assert_eq!(tokens, vec![(RawToken::FloatLit, "3.14")]);
    }

    #[test]
    fn tokenizes_braces() {
        let tokens = raw_tokens("{ }");
        assert_eq!(
            tokens,
            vec![(RawToken::LBrace, "{"), (RawToken::RBrace, "}"),]
        );
    }

    #[test]
    fn tokenizes_arrow_and_eqeq() {
        let tokens = raw_tokens("-> ==");
        assert_eq!(
            tokens,
            vec![(RawToken::Arrow, "->"), (RawToken::EqEq, "=="),]
        );
    }

    #[test]
    fn skips_whitespace() {
        let tokens = raw_tokens("model   schema");
        assert_eq!(
            tokens,
            vec![(RawToken::KwModel, "model"), (RawToken::KwSchema, "schema"),]
        );
    }

    #[test]
    fn skips_line_comment() {
        let tokens = raw_tokens("// this is a comment\nmodel");
        assert_eq!(tokens, vec![(RawToken::KwModel, "model")]);
    }

    #[test]
    fn skips_block_comment() {
        let tokens = raw_tokens("/* block */model");
        assert_eq!(tokens, vec![(RawToken::KwModel, "model")]);
    }

    #[test]
    fn skips_multiline_block_comment() {
        let tokens = raw_tokens("/* multi\nline\ncomment */model");
        assert_eq!(tokens, vec![(RawToken::KwModel, "model")]);
    }

    #[test]
    fn tokenizes_identifier() {
        let tokens = raw_tokens("myVar");
        assert_eq!(tokens, vec![(RawToken::Ident, "myVar")]);
    }

    #[test]
    fn keyword_takes_priority_over_ident() {
        // "model" should be KwModel, not Ident
        let tokens = raw_tokens("model");
        assert_eq!(tokens[0].0, RawToken::KwModel);
    }

    #[test]
    fn ident_prefix_of_keyword() {
        // "models" should be Ident, not KwModel
        let tokens = raw_tokens("models");
        assert_eq!(tokens, vec![(RawToken::Ident, "models")]);
    }

    #[test]
    fn tokenizes_double_quote() {
        let tokens = raw_tokens("\"");
        assert_eq!(tokens, vec![(RawToken::DoubleQuote, "\"")]);
    }

    #[test]
    fn skips_doc_comment() {
        let tokens = raw_tokens("/// doc comment\nmodel");
        assert_eq!(tokens, vec![(RawToken::KwModel, "model")]);
    }
}
