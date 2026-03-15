//! Wrapper lexer with mode switching for template strings and python bridge.
//!
//! The [`Lexer`] struct drives the logos-based [`RawToken`](crate::logos_lexer::RawToken)
//! DFA in Normal mode, and hand-scans in TemplateString and Interpolation modes.

use crate::intern::Interner;
use crate::logos_lexer::RawToken;
use crate::token::{Span, Token, TokenKind};
use eaml_errors::{Diagnostic, ErrorCode, Severity};
use logos::Logos;

/// The mode the lexer is currently operating in.
#[derive(Debug, Clone, Copy, PartialEq)]
enum LexerMode {
    /// Standard tokenization via logos.
    Normal,
    /// Inside a double-quoted string, hand-scanning text and interpolations.
    TemplateString,
    /// Inside `{...}` within a template string, using logos but tracking brace depth.
    Interpolation { brace_depth: u32 },
    /// Inside `python %{ ... }%`, capturing raw content (stub for Plan 03).
    PythonBridge,
}

/// Output of the [`lex`] function.
pub struct LexOutput {
    /// The token stream.
    pub tokens: Vec<Token>,
    /// Any diagnostics emitted during lexing.
    pub diagnostics: Vec<Diagnostic>,
    /// The string interner used for identifiers.
    pub interner: Interner,
}

/// Stateful wrapper lexer that drives logos and handles mode switching.
pub struct Lexer<'src> {
    /// The (LF-normalized) source text.
    source: &'src str,
    /// Current byte position in the source.
    pos: usize,
    /// Current lexer mode.
    mode: LexerMode,
    /// String interner for identifiers.
    interner: Interner,
    /// Accumulated diagnostics.
    diagnostics: Vec<Diagnostic>,
    /// Accumulated tokens.
    tokens: Vec<Token>,
}

impl<'src> Lexer<'src> {
    /// Creates a new lexer for the given source text.
    ///
    /// The source is normalized to LF line endings before tokenization.
    fn new(source: &'src str) -> Self {
        Self {
            source,
            pos: 0,
            mode: LexerMode::Normal,
            interner: Interner::new(),
            diagnostics: Vec::new(),
            tokens: Vec::new(),
        }
    }

    /// Emits a SYN001 diagnostic for an unexpected character at the given span.
    fn emit_unexpected_char(&mut self, span: Span) {
        self.diagnostics.push(Diagnostic::new(
            ErrorCode::Syn001,
            format!(
                "unexpected character `{}`",
                self.source[span.clone()].escape_debug()
            ),
            span,
            Severity::Error,
            "unexpected character".to_string(),
        ));
    }

    /// Runs the lexer to completion, populating `self.tokens` and `self.diagnostics`.
    fn tokenize(&mut self) {
        loop {
            match self.mode {
                LexerMode::Normal => {
                    if !self.scan_normal() {
                        break;
                    }
                }
                LexerMode::TemplateString => {
                    self.scan_template_string();
                }
                LexerMode::Interpolation { .. } => {
                    if !self.scan_interpolation() {
                        break;
                    }
                }
                LexerMode::PythonBridge => {
                    self.scan_python_bridge();
                }
            }
        }
        self.tokens
            .push(Token::new(TokenKind::Eof, self.pos..self.pos));
    }

    /// Scans tokens in Normal mode using logos.
    /// Returns false when input is exhausted.
    fn scan_normal(&mut self) -> bool {
        let base = self.pos;
        let remaining = &self.source[base..];
        if remaining.is_empty() {
            return false;
        }

        let mut logos_lex = RawToken::lexer(remaining);

        while let Some(result) = logos_lex.next() {
            let span = logos_lex.span();
            let abs_start = base + span.start;
            let abs_end = base + span.end;
            let abs_span: Span = abs_start..abs_end;

            match result {
                Ok(raw) => {
                    match raw {
                        // Double quote: switch to template string mode
                        RawToken::DoubleQuote => {
                            self.tokens.push(Token::new(TokenKind::TmplStart, abs_span));
                            self.pos = abs_end;
                            self.mode = LexerMode::TemplateString;
                            return true;
                        }
                        // PercentLBrace: check if previous token was KwPython
                        RawToken::PercentLBrace => {
                            let is_python_bridge = self
                                .tokens
                                .last()
                                .is_some_and(|t| t.kind == TokenKind::KwPython);
                            if is_python_bridge {
                                // Upgrade the previous KwPython to KwPythonBridge
                                if let Some(last) = self.tokens.last_mut() {
                                    last.kind = TokenKind::KwPythonBridge;
                                }
                                self.pos = abs_end;
                                self.mode = LexerMode::PythonBridge;
                                return true;
                            }
                            // Not after python keyword -- emit as two separate tokens
                            // This is an unusual case; emit error
                            self.diagnostics.push(Diagnostic::new(
                                ErrorCode::Syn001,
                                "unexpected `%{` outside python bridge".to_string(),
                                abs_span,
                                Severity::Error,
                                "not preceded by `python` keyword".to_string(),
                            ));
                            self.pos = abs_end;
                        }
                        // RBracePercent outside python bridge
                        RawToken::RBracePercent => {
                            self.diagnostics.push(Diagnostic::new(
                                ErrorCode::Syn001,
                                "unexpected `}%` outside python bridge".to_string(),
                                abs_span,
                                Severity::Error,
                                "no matching `python %{`".to_string(),
                            ));
                            self.pos = abs_end;
                        }
                        // Map all other raw tokens to token kinds
                        _ => {
                            let kind = self.map_raw_token(raw, &self.source[abs_start..abs_end]);
                            self.tokens.push(Token::new(kind, abs_span));
                            self.pos = abs_end;
                        }
                    }
                }
                Err(()) => {
                    self.emit_unexpected_char(abs_span);
                    self.pos = abs_end;
                }
            }
        }

        // logos consumed everything
        self.pos = self.source.len();
        false
    }

    /// Maps a RawToken to a TokenKind.
    fn map_raw_token(&mut self, raw: RawToken, slice: &str) -> TokenKind {
        match raw {
            // Keywords (active)
            RawToken::KwModel => TokenKind::KwModel,
            RawToken::KwSchema => TokenKind::KwSchema,
            RawToken::KwPrompt => TokenKind::KwPrompt,
            RawToken::KwTool => TokenKind::KwTool,
            RawToken::KwAgent => TokenKind::KwAgent,
            RawToken::KwImport => TokenKind::KwImport,
            RawToken::KwLet => TokenKind::KwLet,
            RawToken::KwIf => TokenKind::KwIf,
            RawToken::KwElse => TokenKind::KwElse,
            RawToken::KwReturn => TokenKind::KwReturn,
            RawToken::KwAwait => TokenKind::KwAwait,
            RawToken::KwTrue => TokenKind::KwTrue,
            RawToken::KwFalse => TokenKind::KwFalse,
            RawToken::KwNull => TokenKind::KwNull,
            RawToken::KwPython => TokenKind::KwPython,
            // Keywords (post-MVP reserved)
            RawToken::KwPipeline => TokenKind::KwPipeline,
            RawToken::KwEnum => TokenKind::KwEnum,
            RawToken::KwExtends => TokenKind::KwExtends,
            // Keywords (future reserved)
            RawToken::KwOverride => TokenKind::KwOverride,
            RawToken::KwInterface => TokenKind::KwInterface,
            RawToken::KwType => TokenKind::KwType,
            RawToken::KwWhere => TokenKind::KwWhere,
            RawToken::KwFor => TokenKind::KwFor,
            RawToken::KwWhile => TokenKind::KwWhile,
            RawToken::KwMatch => TokenKind::KwMatch,
            RawToken::KwAsync => TokenKind::KwAsync,
            RawToken::KwYield => TokenKind::KwYield,
            // Identifiers
            RawToken::Ident => {
                let spur = self.interner.intern(slice);
                TokenKind::Ident(spur)
            }
            // Literals
            RawToken::IntLit => TokenKind::IntLit,
            RawToken::FloatLit => TokenKind::FloatLit,
            // Single-char operators
            RawToken::LParen => TokenKind::LParen,
            RawToken::RParen => TokenKind::RParen,
            RawToken::LBrace => TokenKind::LBrace,
            RawToken::RBrace => TokenKind::RBrace,
            RawToken::LBracket => TokenKind::LBracket,
            RawToken::RBracket => TokenKind::RBracket,
            RawToken::LAngle => TokenKind::LAngle,
            RawToken::RAngle => TokenKind::RAngle,
            RawToken::Colon => TokenKind::Colon,
            RawToken::Semicolon => TokenKind::Semicolon,
            RawToken::Comma => TokenKind::Comma,
            RawToken::Dot => TokenKind::Dot,
            RawToken::Eq => TokenKind::Eq,
            RawToken::Bang => TokenKind::Bang,
            RawToken::Plus => TokenKind::Plus,
            RawToken::Minus => TokenKind::Minus,
            RawToken::Star => TokenKind::Star,
            RawToken::Slash => TokenKind::Slash,
            RawToken::Pipe => TokenKind::Pipe,
            RawToken::Ampersand => TokenKind::Ampersand,
            RawToken::Question => TokenKind::Question,
            RawToken::At => TokenKind::At,
            // Multi-char operators
            RawToken::Arrow => TokenKind::Arrow,
            RawToken::EqEq => TokenKind::EqEq,
            RawToken::BangEq => TokenKind::BangEq,
            RawToken::LessEq => TokenKind::LessEq,
            RawToken::GreaterEq => TokenKind::GreaterEq,
            RawToken::AmpAmp => TokenKind::AmpAmp,
            RawToken::PipePipe => TokenKind::PipePipe,
            RawToken::PipelineOp => TokenKind::PipelineOp,
            // These are handled specially in scan_normal, should not reach here
            RawToken::DoubleQuote | RawToken::PercentLBrace | RawToken::RBracePercent => {
                unreachable!("handled in scan_normal")
            }
        }
    }

    /// Scans text inside a template string (between `"` delimiters).
    fn scan_template_string(&mut self) {
        let bytes = self.source.as_bytes();
        let text_start = self.pos;
        let mut text_has_content = false;

        while self.pos < bytes.len() {
            match bytes[self.pos] {
                b'"' => {
                    // End of template string
                    if text_has_content {
                        self.tokens
                            .push(Token::new(TokenKind::TmplText, text_start..self.pos));
                    }
                    let close_span = self.pos..self.pos + 1;
                    self.pos += 1;
                    self.tokens.push(Token::new(TokenKind::TmplEnd, close_span));
                    self.mode = LexerMode::Normal;
                    return;
                }
                b'\\' => {
                    // Escape sequence
                    text_has_content = true;
                    if self.pos + 1 >= bytes.len() {
                        // Backslash at end of input
                        self.diagnostics.push(Diagnostic::new(
                            ErrorCode::Syn004,
                            "invalid escape sequence at end of input".to_string(),
                            self.pos..self.pos + 1,
                            Severity::Error,
                            "incomplete escape".to_string(),
                        ));
                        self.pos += 1;
                    } else {
                        let next = bytes[self.pos + 1];
                        match next {
                            b'n' | b't' | b'r' | b'"' | b'\\' => {
                                // Valid escape: skip both characters
                                self.pos += 2;
                            }
                            _ => {
                                // Invalid escape: emit SYN004
                                let escape_char = self.source[self.pos + 1..]
                                    .chars()
                                    .next()
                                    .expect("pos+1 is within bounds");
                                let char_len = escape_char.len_utf8();
                                self.diagnostics.push(Diagnostic::new(
                                    ErrorCode::Syn004,
                                    format!(
                                        "invalid escape sequence `\\{}`",
                                        escape_char.escape_debug()
                                    ),
                                    self.pos..self.pos + 1 + char_len,
                                    Severity::Error,
                                    "invalid escape".to_string(),
                                ));
                                self.pos += 1 + char_len;
                            }
                        }
                    }
                }
                b'{' => {
                    // Check for escaped brace `{{`
                    if self.pos + 1 < bytes.len() && bytes[self.pos + 1] == b'{' {
                        // Escaped brace: treat as text content
                        text_has_content = true;
                        self.pos += 2;
                    } else {
                        // Interpolation start
                        if text_has_content {
                            self.tokens
                                .push(Token::new(TokenKind::TmplText, text_start..self.pos));
                        }
                        let interp_span = self.pos..self.pos + 1;
                        self.pos += 1;
                        self.tokens
                            .push(Token::new(TokenKind::TmplInterpStart, interp_span));
                        self.mode = LexerMode::Interpolation { brace_depth: 1 };
                        return;
                    }
                }
                b'}' => {
                    // Check for escaped brace `}}`
                    if self.pos + 1 < bytes.len() && bytes[self.pos + 1] == b'}' {
                        text_has_content = true;
                        self.pos += 2;
                    } else {
                        // Stray `}` in template text -- include as text
                        text_has_content = true;
                        self.pos += 1;
                    }
                }
                b'\n' => {
                    // Newline in string -- include in text
                    text_has_content = true;
                    self.pos += 1;
                }
                _ => {
                    // Regular character
                    text_has_content = true;
                    // Advance by one UTF-8 character
                    let ch = self.source[self.pos..]
                        .chars()
                        .next()
                        .expect("pos is within bounds");
                    self.pos += ch.len_utf8();
                }
            }
        }

        // EOF without closing quote: emit SYN002
        if text_has_content {
            self.tokens
                .push(Token::new(TokenKind::TmplText, text_start..self.pos));
        }
        self.diagnostics.push(Diagnostic::new(
            ErrorCode::Syn002,
            "unterminated string literal".to_string(),
            text_start..self.pos,
            Severity::Error,
            "string started here but never closed".to_string(),
        ));
        self.tokens
            .push(Token::new(TokenKind::TmplEnd, self.pos..self.pos));
        self.mode = LexerMode::Normal;
    }

    /// Scans tokens inside an interpolation `{...}` within a template string.
    /// Uses logos but tracks brace depth.
    /// Returns false when input is exhausted.
    fn scan_interpolation(&mut self) -> bool {
        let base = self.pos;
        let remaining = &self.source[base..];
        if remaining.is_empty() {
            // EOF inside interpolation: emit SYN045
            self.diagnostics.push(Diagnostic::new(
                ErrorCode::Syn045,
                "unclosed template string interpolation".to_string(),
                self.pos..self.pos,
                Severity::Error,
                "interpolation opened but never closed".to_string(),
            ));
            self.mode = LexerMode::Normal;
            return false;
        }

        let mut logos_lex = RawToken::lexer(remaining);

        while let Some(result) = logos_lex.next() {
            let span = logos_lex.span();
            let abs_start = base + span.start;
            let abs_end = base + span.end;
            let abs_span: Span = abs_start..abs_end;

            match result {
                Ok(raw) => {
                    match raw {
                        RawToken::LBrace => {
                            if let LexerMode::Interpolation {
                                ref mut brace_depth,
                            } = self.mode
                            {
                                *brace_depth += 1;
                            }
                            self.tokens.push(Token::new(TokenKind::LBrace, abs_span));
                            self.pos = abs_end;
                        }
                        RawToken::RBrace => {
                            if let LexerMode::Interpolation {
                                ref mut brace_depth,
                            } = self.mode
                            {
                                *brace_depth -= 1;
                                if *brace_depth == 0 {
                                    // End of interpolation
                                    self.tokens
                                        .push(Token::new(TokenKind::TmplInterpEnd, abs_span));
                                    self.pos = abs_end;
                                    self.mode = LexerMode::TemplateString;
                                    return true;
                                }
                            }
                            self.tokens.push(Token::new(TokenKind::RBrace, abs_span));
                            self.pos = abs_end;
                        }
                        RawToken::DoubleQuote => {
                            // Nested string inside interpolation: start template
                            self.tokens.push(Token::new(TokenKind::TmplStart, abs_span));
                            self.pos = abs_end;
                            // We need to save the current interpolation state and
                            // scan the nested string. For simplicity, we hand-scan
                            // the nested string inline (recursive template strings
                            // are not expected in v0.1, but we handle the basic case).
                            self.scan_template_string();
                            // After the nested string closes, we're back to scanning
                            // the interpolation. Return true to re-enter the loop.
                            return true;
                        }
                        _ => {
                            let kind = self.map_raw_token(raw, &self.source[abs_start..abs_end]);
                            self.tokens.push(Token::new(kind, abs_span));
                            self.pos = abs_end;
                        }
                    }
                }
                Err(()) => {
                    self.emit_unexpected_char(abs_span);
                    self.pos = abs_end;
                }
            }
        }

        // logos consumed everything without closing brace
        self.pos = self.source.len();
        self.diagnostics.push(Diagnostic::new(
            ErrorCode::Syn045,
            "unclosed template string interpolation".to_string(),
            self.pos..self.pos,
            Severity::Error,
            "interpolation opened but never closed".to_string(),
        ));
        self.mode = LexerMode::Normal;
        false
    }

    /// Scans a python bridge block (stub for Plan 03).
    fn scan_python_bridge(&mut self) {
        // Stub: scan to `}%` at start of line or EOF
        let start = self.pos;
        let bytes = self.source.as_bytes();

        while self.pos < bytes.len() {
            // Check for `}%` at start of line (with optional leading whitespace)
            if bytes[self.pos] == b'\n' {
                self.pos += 1;
                // Skip leading whitespace
                while self.pos < bytes.len()
                    && (bytes[self.pos] == b' ' || bytes[self.pos] == b'\t')
                {
                    self.pos += 1;
                }
                // Check for `}%`
                if self.pos + 1 < bytes.len()
                    && bytes[self.pos] == b'}'
                    && bytes[self.pos + 1] == b'%'
                {
                    let content_span = start..self.pos;
                    self.tokens
                        .push(Token::new(TokenKind::PythonBlock, content_span));
                    self.pos += 2; // skip `}%`
                    self.mode = LexerMode::Normal;
                    return;
                }
            } else {
                self.pos += 1;
            }
        }

        // EOF without closing `}%`
        self.tokens
            .push(Token::new(TokenKind::PythonBlock, start..self.pos));
        self.diagnostics.push(Diagnostic::new(
            ErrorCode::Syn046,
            "unterminated python bridge block".to_string(),
            start..self.pos,
            Severity::Error,
            "python bridge opened but `}%` never found".to_string(),
        ));
        self.mode = LexerMode::Normal;
    }

    /// Collapses adjacent SYN001 diagnostics into a single diagnostic spanning the range.
    ///
    /// Per CONTEXT.md: "Consecutive identical errors at adjacent positions: collapse into
    /// one diagnostic spanning the range."
    fn collapse_adjacent_errors(diagnostics: Vec<Diagnostic>) -> Vec<Diagnostic> {
        if diagnostics.is_empty() {
            return diagnostics;
        }

        let mut result: Vec<Diagnostic> = Vec::with_capacity(diagnostics.len());

        for diag in diagnostics {
            if diag.code == ErrorCode::Syn001 {
                if let Some(last) = result.last_mut() {
                    if last.code == ErrorCode::Syn001 && last.span.end == diag.span.start {
                        // Extend the previous diagnostic to cover this one
                        last.span.end = diag.span.end;
                        last.message = "unexpected characters".to_string();
                        continue;
                    }
                }
            }
            result.push(diag);
        }

        result
    }

    /// Consumes the lexer and returns the output.
    fn into_output(self) -> LexOutput {
        LexOutput {
            tokens: self.tokens,
            diagnostics: Self::collapse_adjacent_errors(self.diagnostics),
            interner: self.interner,
        }
    }
}

/// Tokenizes EAML source text into a token stream.
///
/// Returns the tokens, any diagnostics, and the string interner.
pub fn lex(source: &str) -> LexOutput {
    // Normalize CRLF to LF
    let normalized;
    let source = if source.contains('\r') {
        normalized = source.replace("\r\n", "\n").replace('\r', "\n");
        normalized.as_str()
    } else {
        source
    };

    let mut lexer = Lexer::new(source);
    lexer.tokenize();
    lexer.into_output()
}
