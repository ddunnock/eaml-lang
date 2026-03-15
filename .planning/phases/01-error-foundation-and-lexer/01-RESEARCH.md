# Phase 1: Error Foundation and Lexer - Research

**Researched:** 2026-03-15
**Domain:** Rust compiler infrastructure -- error diagnostics (codespan-reporting) and lexical analysis (logos + lasso)
**Confidence:** HIGH

## Summary

Phase 1 implements two crates: `eaml-errors` (shared diagnostic infrastructure) and `eaml-lexer` (tokenization). The error crate defines the complete error code enum from spec/ERRORS.md (38 compiler codes across 6 prefixes), a `Diagnostic` struct carrying code/message/span/severity/hints, and rendering via `codespan-reporting`. The lexer crate tokenizes all EAML grammar terminals using `logos` for the core DFA with a manual wrapper layer for mode switching (template strings, python bridge blocks), and `lasso` for identifier interning.

The primary challenge is that logos is stateless -- it generates a DFA that cannot track brace depth or lexer modes. The solution is a wrapper struct that drives logos in multiple modes: NORMAL (standard tokens), TEMPLATE_STRING (text fragments and interpolation delimiters with brace-depth tracking), and PYTHON_BRIDGE (raw capture until `}%`). The wrapper consumes logos tokens and emits the public token stream.

**Primary recommendation:** Build eaml-errors first (dependency root), then eaml-lexer. Use logos for the fast path (keywords, operators, literals, whitespace/comment skipping) and hand-write the mode-switching wrapper for template strings and python bridge blocks. Test with insta snapshot tests for token streams.

<user_constraints>

## User Constraints (from CONTEXT.md)

### Locked Decisions
- Rustc-style diagnostics: colored source snippets with underlines, error codes, primary message, and optional help/hint lines via codespan-reporting
- Error codes always displayed in output: `error[SYN042]: unterminated string` format
- Accumulate up to 20 errors before stopping (overridable with `--max-errors N`)
- Phase 1 implements ERROR severity only; FATAL and WARNING severity levels deferred to later phases when semantic analysis introduces them
- Multi-token template approach: TemplateStart, StringFragment, InterpolationStart, <expr tokens>, InterpolationEnd, StringFragment, TemplateEnd
- Parser sees structure directly without re-lexing -- lexer does all the mode switching
- Newlines normalized to LF inside template text fragments
- Lexer handles brace escaping: `{{` and `}}` converted to text fragments containing literal `{` and `}`
- Unterminated interpolation (missing closing brace): emit SYN error, recover at end of line, continue lexing
- Python bridge closing `}%` delimiter must appear at the start of a line (possibly with leading whitespace only)
- Bridge block content is completely opaque -- lexer captures raw bytes, no Python validation
- Unterminated python block (missing `}%`): scan to EOF, emit SYN error pointing back to opening `python %{`
- Unrecognizable character: skip one byte, emit SYN error, continue lexing
- Unterminated string literal: recover at end of line, emit SYN error
- Consecutive identical errors at adjacent positions: collapse into one diagnostic spanning the range
- No 'did you mean?' suggestions in Phase 1

### Claude's Discretion
- Exact codespan-reporting configuration and color scheme
- Internal error accumulation data structure design
- Logos wrapper layer architecture for mode switching
- Token type enum naming and organization
- Test fixture organization for snapshot tests

### Deferred Ideas (OUT OF SCOPE)
- None -- discussion stayed within phase scope

</user_constraints>

<phase_requirements>

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| ERR-01 | Compiler defines all error codes from spec/ERRORS.md as a Rust enum (SYN, SEM, CAP, TYP, PYB, RES) | Error code enum design with 38 codes across 6 prefixes; see Architecture Patterns |
| ERR-02 | Diagnostic struct carries error code, message, source span, severity, and optional hints | Diagnostic struct pattern with codespan-reporting Label/Diagnostic mapping |
| ERR-03 | Errors display with codespan-reporting showing colored source snippets and underlines | codespan-reporting SimpleFiles + term::emit pattern; see Code Examples |
| ERR-04 | Multiple errors accumulate per compilation (not abort-on-first) | DiagnosticCollector pattern with max_errors limit; see Architecture Patterns |
| LEX-01 | Lexer tokenizes all keywords from grammar.ebnf | 27 keywords (7 active + 3 post-MVP + 8 stmt/expr + 9 future-reserved); logos `#[token]` |
| LEX-02 | Lexer tokenizes all operators and delimiters from grammar.ebnf | ~30 operators/delimiters extracted from grammar; logos `#[token]` |
| LEX-03 | Lexer tokenizes string literals with escape sequences | 5 escape sequences: `\"`, `\\`, `\n`, `\r`, `\t`; logos `#[regex]` |
| LEX-04 | Lexer tokenizes numeric literals (integers and floats) | INT: `0 \| [1-9][0-9]*`, FLOAT: `(0\|[1-9][0-9]*)\.[0-9]+`; logos `#[regex]` |
| LEX-05 | Lexer tokenizes template strings with `{expr}` interpolation tracking brace depth | Multi-token approach with wrapper layer mode switching; see Architecture Patterns |
| LEX-06 | Lexer captures python bridge blocks `python %{...}%` as opaque content | Manual scan in wrapper layer; `}%` at line-start only; see Architecture Patterns |
| LEX-07 | Lexer interns identifiers via lasso for memory-efficient deduplication | `lasso::Rodeo` with `Spur` keys stored in Token; see Code Examples |
| LEX-08 | Lexer skips comments while preserving accurate byte-offset spans | `//`, `/* */`, `///` as logos skip patterns; see Code Examples |
| LEX-09 | Lexer emits SYN error codes for malformed tokens with accurate source positions | SYN045 (unclosed interpolation), SYN046 (unclosed bridge), plus new SYN codes for unterminated strings, unexpected chars |

</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| logos | 0.14 | DFA-based lexer generator | Industry standard for Rust lexers; derives token enum with regex patterns; zero-copy |
| lasso | 0.7 | String interning | Thread-safe string interner; `Rodeo` for single-threaded, `Spur` handles are `Copy` |
| codespan-reporting | 0.11 | Diagnostic rendering | Rustc-style colored error output with source snippets; used by many Rust compilers |
| thiserror | 1 | Error derive macros | Standard for deriving `std::error::Error` on enums |
| insta | 1 | Snapshot testing | Golden-file testing for token streams and error output |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| codespan | 0.11 | Source file database | `SimpleFiles` type for storing source text with file IDs; paired with codespan-reporting |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| logos | Hand-written lexer | More control over modes but significantly more code; logos handles the 90% case (keywords, operators, literals) efficiently |
| lasso | string-interner | lasso is more widely used, better API; string-interner is fine but less popular |

**Installation:**
Already configured in workspace Cargo.toml. Per-crate Cargo.toml files reference with `{ workspace = true }`.

## Architecture Patterns

### Recommended Project Structure
```
crates/eaml-errors/src/
├── lib.rs           # Public API: re-exports
├── codes.rs         # ErrorCode enum (38 codes + Display impl)
├── diagnostic.rs    # Diagnostic struct, DiagnosticCollector
├── severity.rs      # Severity enum (Fatal, Error, Warning)
└── render.rs        # codespan-reporting integration (to_codespan_diagnostic)

crates/eaml-lexer/src/
├── lib.rs           # Public API: lex() function, re-exports
├── token.rs         # Token struct, TokenKind enum
├── logos_lexer.rs   # Logos-derived inner lexer (raw DFA tokens)
├── lexer.rs         # Wrapper lexer with mode switching
└── intern.rs        # Lasso interner wrapper (Rodeo + Spur)

crates/eaml-lexer/tests/
├── keywords.rs      # Keyword tokenization tests
├── literals.rs      # String, int, float literal tests
├── operators.rs     # Operator/delimiter tests
├── template.rs      # Template string tests (snapshot)
├── python_bridge.rs # Python bridge block tests (snapshot)
├── errors.rs        # Error recovery tests (snapshot)
└── snapshots/       # insta snapshot files
```

### Pattern 1: Logos Inner Lexer + Manual Wrapper

**What:** Use logos to define a `RawToken` enum that handles the stateless DFA portion (keywords, operators, simple literals, whitespace/comment skipping). Wrap it in a `Lexer` struct that manages mode state and emits the public `Token` stream.

**When to use:** Whenever the grammar has context-sensitive lexing (template strings, heredocs, string interpolation).

**Architecture:**

```rust
// logos_lexer.rs -- DFA-generated tokens
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\r\n]+")]  // whitespace
#[logos(skip r"//[^\n]*")]     // line comments
#[logos(skip r"///[^\n]*")]    // doc comments (silently discard in v0.1)
#[logos(skip r"/\*[^*]*\*+(?:[^/*][^*]*\*+)*/")]  // block comments
enum RawToken {
    // Keywords
    #[token("model")]   KwModel,
    #[token("schema")]  KwSchema,
    // ... all 27 keywords

    // Operators
    #[token("(")]  LParen,
    #[token(")")]  RParen,
    // ... all operators

    // Literals
    #[regex(r"0|[1-9][0-9]*", priority = 2)]  IntLit,
    #[regex(r"(0|[1-9][0-9]*)\.[0-9]+")]      FloatLit,
    #[token("\"")]  DoubleQuote,  // triggers mode switch in wrapper

    // Identifiers
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]  Ident,

    // Special delimiters (detected by wrapper)
    #[token("%{")]  PercentLBrace,
    #[token("}%")]  PercentRBrace,
    #[token("@")]   At,  // SYN090 post-MVP
}

// lexer.rs -- stateful wrapper
enum LexerMode {
    Normal,
    TemplateString { brace_depth: u32 },
    PythonBridge { start_span: Span },
}

struct Lexer<'src> {
    source: &'src str,
    mode: LexerMode,
    inner: logos::Lexer<'src, RawToken>,
    interner: Rodeo,
    diagnostics: Vec<Diagnostic>,
    tokens: Vec<Token>,
}
```

### Pattern 2: Error Code Enum with Display

**What:** Define all 38 error codes as enum variants with a `Display` impl that formats as `PREFIX NNN`.

**Architecture:**

```rust
// codes.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    // Syntax errors -- lexer (001-039 reserved, 042-046 defined)
    Syn042, Syn043, Syn044, Syn045, Syn046,
    // Syntax errors -- parser (050-099)
    Syn050, Syn060, Syn061,
    Syn080, Syn081, Syn082, Syn083, Syn090,
    // Semantic errors
    Sem010, Sem020, Sem025, Sem030, Sem035, Sem040, Sem050, Sem060, Sem061, Sem070,
    // Type errors
    Typ001, Typ002, Typ003, Typ010, Typ030, Typ031, Typ032, Typ040,
    // Capability errors
    Cap001, Cap002, Cap010, Cap020,
    // Python bridge errors
    Pyb001, Pyb010,
    // Resolution errors
    Res001,
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Syn042 => write!(f, "SYN042"),
            // ...
        }
    }
}
```

### Pattern 3: Diagnostic Collector with Max Errors

**What:** Accumulate diagnostics up to a configurable limit. After limit, stop adding but track overflow count.

```rust
pub struct DiagnosticCollector {
    diagnostics: Vec<Diagnostic>,
    max_errors: usize,  // default 20
    error_count: usize,
    overflow: bool,
}

impl DiagnosticCollector {
    pub fn emit(&mut self, diag: Diagnostic) {
        if diag.severity == Severity::Error {
            self.error_count += 1;
        }
        if self.error_count > self.max_errors {
            self.overflow = true;
            return;
        }
        self.diagnostics.push(diag);
    }

    pub fn has_errors(&self) -> bool {
        self.error_count > 0
    }
}
```

### Pattern 4: Template String Mode Switching

**What:** When the wrapper encounters a `"` in a context where template strings are valid, it switches to TEMPLATE_STRING mode. In this mode, it hand-scans characters:
- Regular text -> accumulate into TMPL_TEXT token
- `{{` -> emit TMPL_TEXT with literal `{`
- `}}` -> emit TMPL_TEXT with literal `}`
- `{` (single) -> emit TMPL_INTERP_START, push brace_depth=1, switch to Normal mode for expr tokens
- `}` at depth 0 in interpolation -> emit TMPL_INTERP_END, return to TEMPLATE_STRING mode
- `"` (not in interpolation) -> emit TMPL_END, return to Normal mode

**Critical detail from CONTEXT.md:** The lexer determines whether a `"` is a template string or a plain string. Since the lexer is context-free, the simplest correct approach is: ALL strings in EAML are template strings at the lexer level. A plain string `"hello"` becomes TMPL_START, TMPL_TEXT("hello"), TMPL_END. The parser can distinguish (no interpolation parts = simple string) if needed, but the lexer always uses template mode for `"`. This avoids needing parser feedback.

**Alternative:** Track whether we're inside a prompt/agent body to decide template vs plain string. This requires parser-like context in the lexer, which is fragile. The "everything is a template" approach is simpler and correct -- the parser and later phases handle semantics.

**Recommendation (Claude's discretion):** Treat all double-quoted strings as template strings at the lexer level. The overhead is negligible (3 tokens instead of 1), and it eliminates a class of context-sensitivity bugs.

### Pattern 5: Python Bridge Capture

**What:** When the wrapper sees `python` keyword followed by `%{`, it enters PYTHON_BRIDGE mode. In this mode, it scans character-by-character looking for `}%` at the start of a line (per CONTEXT.md decision). All content between delimiters is captured as a single PYTHON_BLOCK token.

**Key constraint:** The closing `}%` must appear at line start (with optional leading whitespace). This prevents false closes from Python f-strings like `f"{value}% done"`.

```rust
fn scan_python_bridge(&mut self, start: usize) -> Token {
    let content_start = self.pos; // after %{
    loop {
        if self.at_eof() {
            self.emit_error(ErrorCode::Syn046, start..self.pos);
            return Token::new(TokenKind::PythonBlock, start..self.pos);
        }
        if self.at_line_start_with_optional_ws() && self.looking_at("}%") {
            let content = &self.source[content_start..self.pos];
            self.advance(2); // skip }%
            return Token::new(TokenKind::PythonBlock(content), start..self.pos);
        }
        self.advance(1);
    }
}
```

### Anti-Patterns to Avoid
- **Parsing in the lexer:** The lexer should NOT try to validate expressions inside template interpolations. It only tracks brace depth for `{`/`}` matching. The parser validates the expression grammar.
- **Using logos for template strings:** Logos cannot track state/depth. Template string internals MUST be hand-scanned in the wrapper.
- **Storing string content in tokens:** Store byte-offset spans only. The source text is accessed via the span when needed. Exception: interned identifiers store `Spur` keys.
- **Separate string vs template token paths:** Having two code paths (one for "plain strings", one for "template strings") creates subtle bugs. Use one path (template) for all.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| DFA-based keyword/operator matching | Custom state machine | logos `#[derive(Logos)]` | Logos generates optimized DFA at compile time; handles priority, skip patterns |
| Colored terminal error output | ANSI escape code formatting | codespan-reporting `term::emit` | Handles terminal detection, color support, multi-line spans, label formatting |
| String interning | HashMap<String, usize> | lasso `Rodeo` | Lasso handles thread safety, compact handles, efficient lookup |
| Error enum boilerplate | Manual Display impls | thiserror `#[derive(Error)]` for wrapper errors | Standard pattern; though ErrorCode Display is simple enough to hand-write |

**Key insight:** The lexer's complexity is in mode switching, not in token recognition. Let logos handle the boring part (DFA) so implementation effort focuses on the interesting part (template strings, python bridge, error recovery).

## Common Pitfalls

### Pitfall 1: Logos Skip Pattern for Block Comments
**What goes wrong:** The regex `r"/\*.*?\*/"` for block comments doesn't work with logos because `.` doesn't match newlines by default. Block comments spanning multiple lines aren't skipped.
**Why it happens:** Logos regex uses the `regex` crate's default mode where `.` doesn't match `\n`.
**How to avoid:** Use a character class instead: `r"/\*[^*]*\*+(?:[^/*][^*]*\*+)*/"` -- this is the classic regex for C-style block comments that handles `*` inside comments and multi-line content without needing `(?s)` mode.
**Warning signs:** Block comments cause unexpected token errors on subsequent lines.

### Pitfall 2: Logos Priority Conflicts (Keywords vs Identifiers)
**What goes wrong:** `model` matches both the keyword token and the identifier regex. Without explicit priority, logos may pick the wrong one.
**Why it happens:** Logos resolves ambiguity by: (1) longest match wins, (2) if equal length, `#[token]` beats `#[regex]`, (3) explicit `priority` attribute.
**How to avoid:** Keywords defined with `#[token("keyword")]` naturally take priority over `#[regex]` identifiers when the match length is equal. This is correct out-of-the-box with logos. Verify with tests.
**Warning signs:** Keywords being tokenized as identifiers.

### Pitfall 3: Byte Offsets vs Character Offsets
**What goes wrong:** codespan-reporting expects byte offsets for spans, but hand-scanning code uses character indexing.
**Why it happens:** Rust strings are UTF-8; `.chars().nth(n)` gives character position, not byte position.
**How to avoid:** Always work with byte offsets (`&source[start..end]`). logos provides byte-offset spans via `lexer.span()`. In the manual scanning parts (template strings, python bridge), advance by bytes, not characters.
**Warning signs:** Off-by-one errors in error underlines for files with non-ASCII content.

### Pitfall 4: Template String Brace Depth Off-By-One
**What goes wrong:** Nested braces in expressions like `{obj.method({inner})}` cause the lexer to close the interpolation too early or too late.
**Why it happens:** Not correctly incrementing on `{` and decrementing on `}` in all contexts, or handling `{{`/`}}` escapes inside interpolation mode.
**How to avoid:** In interpolation mode (after TMPL_INTERP_START), the wrapper delegates to logos for normal tokens. Every `{` (LBrace) increments depth, every `}` (RBrace) decrements. When depth reaches 0, emit TMPL_INTERP_END. The `{{`/`}}` escapes are only meaningful in TEMPLATE_STRING text mode, not inside interpolation expressions.
**Warning signs:** Test with nested braces: `"outer {fn({x})} end"`.

### Pitfall 5: Newline Normalization Timing
**What goes wrong:** CRLF (`\r\n`) is 2 bytes but represents 1 newline. If normalization happens after span recording, spans point to wrong byte offsets.
**Why it happens:** Normalizing source text before lexing changes byte positions of everything after the first `\r\n`.
**How to avoid:** Two options: (a) normalize entire source to LF before lexing (simplest -- all spans are post-normalization), or (b) handle `\r\n` as single newline in scanning without modifying source. Option (a) is recommended -- do it once upfront. Grammar spec says "normalized to LF by the lexer before tokenization."
**Warning signs:** Error underlines shifted right on Windows-originated source files.

### Pitfall 6: Logos Error Token Handling
**What goes wrong:** When logos encounters a character it can't match, it returns `Err(())`. If the wrapper doesn't handle this, the lexer panics or loops.
**Why it happens:** Default logos iterator yields `Result<Token, ()>` where `Err` means unrecognized input.
**How to avoid:** Handle `Err(())` by: recording the span, emitting a SYN diagnostic for "unexpected character", advancing past the bad byte, and continuing. This matches the CONTEXT.md decision: "skip one byte, emit SYN error, continue lexing."
**Warning signs:** Lexer hangs or panics on binary content or unusual Unicode.

## Code Examples

### codespan-reporting Integration

```rust
// Source: codespan-reporting 0.11 API
use codespan_reporting::diagnostic::{Diagnostic as CSDiagnostic, Label};
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};

// Convert our Diagnostic to codespan's Diagnostic
impl Diagnostic {
    pub fn to_codespan(&self, file_id: usize) -> CSDiagnostic<usize> {
        let severity = match self.severity {
            Severity::Fatal => codespan_reporting::diagnostic::Severity::Bug,
            Severity::Error => codespan_reporting::diagnostic::Severity::Error,
            Severity::Warning => codespan_reporting::diagnostic::Severity::Warning,
        };

        let mut diag = CSDiagnostic::new(severity)
            .with_code(self.code.to_string())
            .with_message(&self.message);

        diag = diag.with_labels(vec![
            Label::primary(file_id, self.span.clone())
                .with_message(&self.label),
        ]);

        if !self.hints.is_empty() {
            diag = diag.with_notes(self.hints.clone());
        }

        diag
    }
}

// Rendering errors
pub fn render_diagnostics(
    files: &SimpleFiles<&str, &str>,
    diagnostics: &[Diagnostic],
) {
    let writer = StandardStream::stderr(ColorChoice::Auto);
    let config = term::Config::default();
    for diag in diagnostics {
        let cs_diag = diag.to_codespan(0);
        term::emit(&mut writer.lock(), &config, files, &cs_diag).unwrap();
    }
}
```

### Lasso String Interning

```rust
// Source: lasso 0.7 API
use lasso::{Rodeo, Spur};

pub struct Interner {
    rodeo: Rodeo,
}

impl Interner {
    pub fn new() -> Self {
        Self { rodeo: Rodeo::default() }
    }

    pub fn intern(&mut self, s: &str) -> Spur {
        self.rodeo.get_or_intern(s)
    }

    pub fn resolve(&self, key: Spur) -> &str {
        self.rodeo.resolve(&key)
    }
}

// Token with interned identifier
#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,  // byte offsets: start..end
}

pub type Span = std::ops::Range<usize>;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Identifiers (interned)
    Ident(Spur),

    // Keywords
    KwModel, KwSchema, KwPrompt, KwTool, KwAgent,
    KwImport, KwLet, KwIf, KwElse, KwReturn,
    KwAwait, KwTrue, KwFalse, KwNull, KwPython,
    // Post-MVP reserved keywords
    KwPipeline, KwEnum, KwExtends,
    // Future-reserved keywords
    KwOverride, KwInterface, KwType, KwWhere,
    KwFor, KwWhile, KwMatch, KwAsync, KwYield,

    // Literals
    IntLit,      // value extracted from source span
    FloatLit,    // value extracted from source span
    StringLit,   // plain string (if not using universal template approach)

    // Template string tokens
    TmplStart,        // opening "
    TmplText,         // text fragment (including resolved escapes)
    TmplInterpStart,  // { opening interpolation
    TmplInterpEnd,    // } closing interpolation
    TmplEnd,          // closing "

    // Python bridge
    PythonBlock,      // opaque content between %{ and }%

    // Operators (single char)
    LParen, RParen, LBrace, RBrace, LBracket, RBracket,
    LAngle, RAngle,  // < > (also comparison operators)
    Colon, Semicolon, Comma, Dot,
    Eq, Bang, Plus, Minus, Star, Slash, Pipe, Ampersand, Question,
    At,  // @ reserved for annotations (SYN090)

    // Multi-char operators
    Arrow,      // ->
    FatArrow,   // => (if needed)
    EqEq,       // ==
    BangEq,     // !=
    LessEq,     // <=
    GreaterEq,  // >=
    AmpAmp,     // &&
    PipePipe,   // ||
    PercentLBrace,  // %{
    RBracePercent,  // }%
    PipelinePipe,   // >> (reserved, SYN081)

    // Special
    Eof,
}
```

### Logos DFA Definition (Inner Lexer)

```rust
use logos::Logos;

#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\r\n]+")]
#[logos(skip r"//[^\n]*")]
#[logos(skip r"///[^\n]*")]
#[logos(skip r"/\*[^*]*\*+(?:[^/*][^*]*\*+)*/")]
pub(crate) enum RawToken {
    // === Keywords (active v0.1) ===
    #[token("model")]    KwModel,
    #[token("schema")]   KwSchema,
    #[token("prompt")]   KwPrompt,
    #[token("tool")]     KwTool,
    #[token("agent")]    KwAgent,
    #[token("import")]   KwImport,
    #[token("let")]      KwLet,
    #[token("if")]       KwIf,
    #[token("else")]     KwElse,
    #[token("return")]   KwReturn,
    #[token("await")]    KwAwait,
    #[token("true")]     KwTrue,
    #[token("false")]    KwFalse,
    #[token("null")]     KwNull,
    #[token("python")]   KwPython,

    // === Keywords (Post-MVP reserved) ===
    #[token("pipeline")] KwPipeline,
    #[token("enum")]     KwEnum,
    #[token("extends")]  KwExtends,

    // === Keywords (future reserved) ===
    #[token("override")]  KwOverride,
    #[token("interface")] KwInterface,
    #[token("type")]      KwType,
    #[token("where")]     KwWhere,
    #[token("for")]       KwFor,
    #[token("while")]     KwWhile,
    #[token("match")]     KwMatch,
    #[token("async")]     KwAsync,
    #[token("yield")]     KwYield,

    // === Operators ===
    #[token("(")]   LParen,
    #[token(")")]   RParen,
    #[token("{")]   LBrace,
    #[token("}")]   RBrace,
    #[token("[")]   LBracket,
    #[token("]")]   RBracket,
    #[token("<")]   LAngle,
    #[token(">")]   RAngle,
    #[token(":")]   Colon,
    #[token(";")]   Semicolon,
    #[token(",")]   Comma,
    #[token(".")]   Dot,
    #[token("=")]   Eq,
    #[token("!")]   Bang,
    #[token("+")]   Plus,
    #[token("-")]   Minus,
    #[token("*")]   Star,
    #[token("/")]   Slash,
    #[token("|")]   Pipe,
    #[token("&")]   Ampersand,
    #[token("?")]   Question,
    #[token("@")]   At,

    // Multi-char operators
    #[token("->")]  Arrow,
    #[token("==")]  EqEq,
    #[token("!=")]  BangEq,
    #[token("<=")]  LessEq,
    #[token(">=")]  GreaterEq,
    #[token("&&")]  AmpAmp,
    #[token("||")]  PipePipe,
    #[token(">>")]  PipelineOp,
    #[token("%{")]  PercentLBrace,
    #[token("}%")]  RBracePercent,

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
```

### Snapshot Test Pattern

```rust
// tests/template.rs
use eaml_lexer::lex;
use insta::assert_snapshot;

#[test]
fn template_simple_interpolation() {
    let tokens = lex(r#""Hello, {name}!""#);
    assert_snapshot!(format_tokens(&tokens));
}

#[test]
fn template_nested_braces() {
    let tokens = lex(r#""Result: {fn({x})}""#);
    assert_snapshot!(format_tokens(&tokens));
}

#[test]
fn template_escaped_braces() {
    let tokens = lex(r#""Use {{braces}} here""#);
    assert_snapshot!(format_tokens(&tokens));
}

fn format_tokens(tokens: &[Token]) -> String {
    tokens.iter()
        .map(|t| format!("{:?} @ {}..{}", t.kind, t.span.start, t.span.end))
        .collect::<Vec<_>>()
        .join("\n")
}
```

## Complete Token Inventory (from grammar.ebnf)

### Keywords (27 total)

**Active declaration (7):** `model`, `schema`, `prompt`, `tool`, `agent`, `import`, `let`

**Statement/expression (8):** `if`, `else`, `return`, `await`, `true`, `false`, `null`, `python`

**Post-MVP reserved (3):** `pipeline`, `enum`, `extends`

**Future reserved (9):** `override`, `interface`, `type`, `where`, `for`, `while`, `match`, `async`, `yield`

### Operators and Delimiters

**Grouping:** `(` `)` `{` `}` `[` `]` `<` `>`

**Punctuation:** `:` `;` `,` `.` `?` `@`

**Assignment/arrow:** `=` `->`

**Comparison:** `==` `!=` `<` `>` `<=` `>=`

**Logical:** `&&` `||` `!`

**Arithmetic:** `+` `-` `*` `/`

**Special:** `|` (type union only) `&` (expression guardrail only) `>>` (reserved, SYN081)

**Bridge delimiters:** `%{` `}%`

### Lexer Error Codes

| Code | Condition | Phase 1? |
|------|-----------|----------|
| SYN045 | Unclosed template string interpolation | YES |
| SYN046 | Unclosed Python bridge block | YES |
| (new, SYN001-039 range) | Unterminated string literal | YES - must define |
| (new, SYN001-039 range) | Unterminated block comment | YES - must define |
| (new, SYN001-039 range) | Unexpected/unrecognized character | YES - must define |
| (new, SYN001-039 range) | Invalid escape sequence in string | YES - must define |

**Note:** spec/ERRORS.md reserves SYN001-039 for lexer errors but does not define specific codes. Phase 1 implementation should assign stable codes from this range for the above conditions. Recommended assignments:
- SYN001: Unexpected character
- SYN002: Unterminated string literal
- SYN003: Unterminated block comment
- SYN004: Invalid escape sequence in string literal

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| logos 0.12 single-mode | logos 0.14 with wrapper for modes | logos 0.14 (2023) | `#[logos(skip)]` attribute simplifies whitespace/comment handling |
| Manual string interning | lasso 0.7 Rodeo | lasso 0.7 (2023) | `Spur` is `Copy`, 4 bytes; efficient for compiler use |
| Manual error formatting | codespan-reporting 0.11 | Stable since 2021 | De facto standard for Rust compiler error UX |

**Deprecated/outdated:**
- logos 0.12 API differences: `Lexer::new()` signature changed in 0.13+; 0.14 is current
- `Extras` type in logos: used for passing state; in 0.14, extras are set via `#[logos(extras = Type)]`

## Open Questions

1. **New SYN error codes for lexer (SYN001-039 range)**
   - What we know: ERRORS.md reserves SYN001-039 for future lexer errors but defines none
   - What's unclear: Whether specific code assignments should be coordinated with spec maintainer
   - Recommendation: Assign SYN001-004 as recommended above; document in ERRORS.md as part of implementation

2. **Template string vs plain string at lexer level**
   - What we know: CONTEXT.md says multi-token template approach; grammar has both STRING and template productions
   - What's unclear: Whether the lexer should distinguish "plain string" from "template string" or treat all as template
   - Recommendation: Treat all `"..."` as template strings at lexer level (3 tokens for plain strings: TmplStart, TmplText, TmplEnd). Simpler, eliminates context-sensitivity. Parser can optimize.

3. **Logos extras for span tracking**
   - What we know: logos provides `.span()` returning `Range<usize>` for current match
   - What's unclear: Whether logos extras should carry the interner or if it should be external
   - Recommendation: Keep interner external to the logos lexer (in the wrapper). Logos extras add complexity with minimal benefit here.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | insta 1.x (snapshot) + standard #[test] |
| Config file | None needed -- insta auto-discovers `snapshots/` dirs |
| Quick run command | `cargo test -p eaml-errors -p eaml-lexer` |
| Full suite command | `cargo test --workspace` |

### Phase Requirements to Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| ERR-01 | All error codes from spec defined as enum | unit | `cargo test -p eaml-errors -- codes` | No - Wave 0 |
| ERR-02 | Diagnostic struct with code/message/span/severity/hints | unit | `cargo test -p eaml-errors -- diagnostic` | No - Wave 0 |
| ERR-03 | codespan-reporting colored output | unit | `cargo test -p eaml-errors -- render` | No - Wave 0 |
| ERR-04 | Multiple error accumulation | unit | `cargo test -p eaml-errors -- collector` | No - Wave 0 |
| LEX-01 | All keywords tokenized | unit | `cargo test -p eaml-lexer -- keywords` | No - Wave 0 |
| LEX-02 | All operators/delimiters tokenized | unit | `cargo test -p eaml-lexer -- operators` | No - Wave 0 |
| LEX-03 | String literals with escapes | unit+snapshot | `cargo test -p eaml-lexer -- literals` | No - Wave 0 |
| LEX-04 | Numeric literals (int/float) | unit | `cargo test -p eaml-lexer -- literals` | No - Wave 0 |
| LEX-05 | Template strings with interpolation | snapshot | `cargo test -p eaml-lexer -- template` | No - Wave 0 |
| LEX-06 | Python bridge blocks | snapshot | `cargo test -p eaml-lexer -- python_bridge` | No - Wave 0 |
| LEX-07 | Identifier interning via lasso | unit | `cargo test -p eaml-lexer -- intern` | No - Wave 0 |
| LEX-08 | Comments skipped, spans accurate | unit | `cargo test -p eaml-lexer -- comments` | No - Wave 0 |
| LEX-09 | SYN errors for malformed tokens | snapshot | `cargo test -p eaml-lexer -- errors` | No - Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p eaml-errors -p eaml-lexer`
- **Per wave merge:** `cargo test --workspace && cargo clippy --workspace -- -D warnings`
- **Phase gate:** Full suite green + `make check` before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `crates/eaml-errors/src/codes.rs` -- ErrorCode enum with all 38 codes
- [ ] `crates/eaml-errors/src/diagnostic.rs` -- Diagnostic struct, DiagnosticCollector
- [ ] `crates/eaml-errors/src/severity.rs` -- Severity enum
- [ ] `crates/eaml-errors/src/render.rs` -- codespan-reporting bridge
- [ ] `crates/eaml-lexer/src/token.rs` -- Token, TokenKind, Span types
- [ ] `crates/eaml-lexer/src/logos_lexer.rs` -- RawToken with logos derive
- [ ] `crates/eaml-lexer/src/lexer.rs` -- Wrapper lexer with mode switching
- [ ] `crates/eaml-lexer/tests/` -- All test files listed above

## Sources

### Primary (HIGH confidence)
- `spec/grammar.ebnf` -- Complete token inventory (27 keywords, all operators), template string mode transitions, python bridge capture algorithm
- `spec/ERRORS.md` -- All 38 error codes, severity levels, SYN045/SYN046 definitions, SYN001-039 reserved range
- `spec/PYTHON_BRIDGE.md` -- Bridge block delimiter specification, `}%` line-start constraint
- `Cargo.toml` (workspace) -- Pinned dependency versions: logos 0.14, lasso 0.7, codespan-reporting 0.11, thiserror 1, insta 1
- `crates/eaml-errors/Cargo.toml` -- Existing dep declarations (thiserror, codespan-reporting)
- `crates/eaml-lexer/Cargo.toml` -- Existing dep declarations (logos, lasso, eaml-errors, insta)

### Secondary (MEDIUM confidence)
- logos 0.14 API patterns -- Based on training data knowledge of logos API; skip attribute, token/regex attributes, priority system are well-established
- lasso 0.7 API patterns -- Rodeo/Spur API is stable and well-documented
- codespan-reporting 0.11 API -- SimpleFiles, Diagnostic, Label, term::emit pattern is well-established

### Tertiary (LOW confidence)
- New SYN error code assignments (SYN001-004) -- Recommended by research, not yet validated against spec maintainer intent

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all libraries already pinned in workspace Cargo.toml, well-established Rust ecosystem choices
- Architecture: HIGH -- logos wrapper pattern is well-known for stateful lexers; grammar spec provides exact token definitions
- Pitfalls: HIGH -- common issues with logos, byte offsets, and mode switching are well-documented in Rust compiler community
- Error codes: MEDIUM -- SYN045/046 are spec-defined; new SYN001-004 are reasonable assignments from reserved range but not spec-confirmed

**Research date:** 2026-03-15
**Valid until:** 2026-04-15 (stable ecosystem, no breaking changes expected)
