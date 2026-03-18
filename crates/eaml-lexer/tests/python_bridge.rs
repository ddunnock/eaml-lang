//! Python bridge block tokenization tests with insta snapshots.
//!
//! Tests opaque content capture, }% line-start detection, and error recovery.

mod common;

use common::format_diagnostics;
use eaml_lexer::{lex, LexOutput, TokenKind};

/// Formats both tokens and the source content of PythonBlock tokens.
fn format_tokens_with_content<'a>(output: &'a LexOutput, source: &'a str) -> String {
    output
        .tokens
        .iter()
        .map(|t| {
            let kind_str = match &t.kind {
                TokenKind::Ident(spur) => {
                    format!("Ident({})", output.interner.resolve(*spur))
                }
                TokenKind::PythonBlock => {
                    let content = &source[t.span.clone()];
                    format!("PythonBlock({:?})", content)
                }
                other => format!("{:?}", other),
            };
            format!("{} @ {}..{}", kind_str, t.span.start, t.span.end)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn python_bridge_basic() {
    let source = "python %{\ndef foo():\n    return 42\n}%";
    let output = lex(source);
    insta::assert_snapshot!(format_tokens_with_content(&output, source), @r#"
    KwPythonBridge @ 0..6
    PythonBlock("\ndef foo():\n    return 42\n") @ 9..35
    Eof @ 37..37
    "#);
    insta::assert_snapshot!(format_diagnostics(&output), @"no diagnostics");
}

#[test]
fn python_bridge_fstring_premature_close() {
    // Per spec (PYB-SYN-01 errata): `}%` inside f-strings prematurely closes the block.
    // This is a known v0.1 limitation — `}%` is a simple two-character scan, no nesting.
    let source = "python %{\nx = f\"{value}% done\"\n}%";
    let output = lex(source);
    // The `}%` inside the f-string closes the block early at "value}%",
    // then the remainder is tokenized in Normal mode.
    insta::assert_snapshot!(format_tokens_with_content(&output, source), @r#"
    KwPythonBridge @ 0..6
    PythonBlock("\nx = f\"{value") @ 9..22
    Ident(done) @ 25..29
    TmplStart @ 29..30
    TmplText @ 30..33
    TmplEnd @ 33..33
    Eof @ 33..33
    "#);
}

#[test]
fn python_bridge_whitespace_close() {
    let source = "python %{\n  code\n  }%";
    let output = lex(source);
    insta::assert_snapshot!(format_tokens_with_content(&output, source), @r#"
    KwPythonBridge @ 0..6
    PythonBlock("\n  code\n  ") @ 9..19
    Eof @ 21..21
    "#);
    insta::assert_snapshot!(format_diagnostics(&output), @"no diagnostics");
}

#[test]
fn python_bridge_unterminated() {
    let source = "python %{\nunterminated";
    let output = lex(source);
    insta::assert_snapshot!(format_tokens_with_content(&output, source), @r#"
    KwPythonBridge @ 0..6
    PythonBlock("\nunterminated") @ 9..22
    Eof @ 22..22
    "#);
    insta::assert_snapshot!(format_diagnostics(&output), @"[SYN046] 9..22: unterminated python bridge block");
}

#[test]
fn python_bridge_followed_by_code() {
    let source = "python %{\npass\n}%\nmodel Foo {}";
    let output = lex(source);
    insta::assert_snapshot!(format_tokens_with_content(&output, source), @r#"
    KwPythonBridge @ 0..6
    PythonBlock("\npass\n") @ 9..15
    KwModel @ 18..23
    Ident(Foo) @ 24..27
    LBrace @ 28..29
    RBrace @ 29..30
    Eof @ 30..30
    "#);
    insta::assert_snapshot!(format_diagnostics(&output), @"no diagnostics");
}

#[test]
fn python_bridge_empty() {
    let source = "python %{\n}%";
    let output = lex(source);
    insta::assert_snapshot!(format_tokens_with_content(&output, source), @r#"
    KwPythonBridge @ 0..6
    PythonBlock("\n") @ 9..10
    Eof @ 12..12
    "#);
    insta::assert_snapshot!(format_diagnostics(&output), @"no diagnostics");
}

#[test]
fn python_bridge_multiline() {
    let source = "python %{\nimport os\nimport sys\n\ndef process(data):\n    result = []\n    for item in data:\n        result.append(item * 2)\n    return result\n}%";
    let output = lex(source);
    // Just verify it tokenizes correctly and captures all content
    assert_eq!(output.tokens[0].kind, TokenKind::KwPythonBridge);
    assert_eq!(output.tokens[1].kind, TokenKind::PythonBlock);
    let content = &source[output.tokens[1].span.clone()];
    assert!(content.contains("import os"));
    assert!(content.contains("return result"));
    assert!(output.diagnostics.is_empty());
}
