//! Shared test helpers for eaml-lexer integration tests.

use eaml_lexer::{lex, LexOutput, TokenKind};

/// Extracts token kinds from source, useful for simple assertions.
pub fn kinds(source: &str) -> Vec<TokenKind> {
    let output = lex(source);
    output.tokens.iter().map(|t| t.kind).collect()
}

/// Formats tokens from a LexOutput for snapshot comparison.
pub fn format_tokens(output: &LexOutput) -> String {
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
pub fn format_diagnostics(output: &LexOutput) -> String {
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
