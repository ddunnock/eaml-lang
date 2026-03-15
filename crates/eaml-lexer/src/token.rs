//! Token types for the EAML lexer.
//!
//! Defines the [`Token`] struct and [`TokenKind`] enum that represent
//! the output of lexical analysis.

use lasso::Spur;

/// A byte-offset range in source text, shared with eaml-errors.
pub type Span = eaml_errors::Span;

/// A single token produced by the lexer.
#[derive(Debug, Clone)]
pub struct Token {
    /// The kind of token.
    pub kind: TokenKind,
    /// Byte-offset span in the source text.
    pub span: Span,
}

impl Token {
    /// Creates a new token with the given kind and span.
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

/// All token kinds produced by the EAML lexer.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // === Keywords (active v0.1, 15) ===
    /// `model`
    KwModel,
    /// `schema`
    KwSchema,
    /// `prompt`
    KwPrompt,
    /// `tool`
    KwTool,
    /// `agent`
    KwAgent,
    /// `import`
    KwImport,
    /// `let`
    KwLet,
    /// `if`
    KwIf,
    /// `else`
    KwElse,
    /// `return`
    KwReturn,
    /// `await`
    KwAwait,
    /// `true`
    KwTrue,
    /// `false`
    KwFalse,
    /// `null`
    KwNull,
    /// `python`
    KwPython,

    // === Keywords (post-MVP reserved, 3) ===
    /// `pipeline` (reserved)
    KwPipeline,
    /// `enum` (reserved)
    KwEnum,
    /// `extends` (reserved)
    KwExtends,

    // === Keywords (future reserved, 9) ===
    /// `override` (reserved)
    KwOverride,
    /// `interface` (reserved)
    KwInterface,
    /// `type` (reserved)
    KwType,
    /// `where` (reserved)
    KwWhere,
    /// `for` (reserved)
    KwFor,
    /// `while` (reserved)
    KwWhile,
    /// `match` (reserved)
    KwMatch,
    /// `async` (reserved)
    KwAsync,
    /// `yield` (reserved)
    KwYield,

    // === Identifiers ===
    /// An identifier, with its interned key.
    Ident(Spur),

    // === Literals ===
    /// Integer literal (value extracted from source span).
    IntLit,
    /// Float literal (value extracted from source span).
    FloatLit,

    // === Template string tokens ===
    /// Opening `"` of a template string.
    TmplStart,
    /// Text fragment within a template string.
    TmplText,
    /// `{` opening an interpolation within a template string.
    TmplInterpStart,
    /// `}` closing an interpolation within a template string.
    TmplInterpEnd,
    /// Closing `"` of a template string.
    TmplEnd,

    // === Python bridge ===
    /// The `python` keyword when followed by `%{`.
    KwPythonBridge,
    /// Opaque content between `%{` and `}%`.
    PythonBlock,

    // === Single-char operators ===
    /// `(`
    LParen,
    /// `)`
    RParen,
    /// `{`
    LBrace,
    /// `}`
    RBrace,
    /// `[`
    LBracket,
    /// `]`
    RBracket,
    /// `<`
    LAngle,
    /// `>`
    RAngle,
    /// `:`
    Colon,
    /// `;`
    Semicolon,
    /// `,`
    Comma,
    /// `.`
    Dot,
    /// `=`
    Eq,
    /// `!`
    Bang,
    /// `+`
    Plus,
    /// `-`
    Minus,
    /// `*`
    Star,
    /// `/`
    Slash,
    /// `|`
    Pipe,
    /// `&`
    Ampersand,
    /// `?`
    Question,
    /// `@`
    At,

    // === Multi-char operators ===
    /// `->`
    Arrow,
    /// `==`
    EqEq,
    /// `!=`
    BangEq,
    /// `<=`
    LessEq,
    /// `>=`
    GreaterEq,
    /// `&&`
    AmpAmp,
    /// `||`
    PipePipe,
    /// `>>` (pipeline operator, reserved)
    PipelineOp,

    // === Special ===
    /// End of file.
    Eof,
}
