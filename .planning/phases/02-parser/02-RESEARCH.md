# Phase 2: Parser - Research

**Researched:** 2026-03-16
**Domain:** Recursive descent parsing, typed arena AST, Pratt expression parsing, error recovery
**Confidence:** HIGH

## Summary

Phase 2 builds a hand-written recursive descent parser for the EAML language. The parser consumes `LexOutput` from eaml-lexer (tokens, diagnostics, interner) and produces a typed-arena AST covering all 84 grammar productions defined in `spec/grammar.ebnf`. The grammar is verified LL(1) at all decision points except two documented LL(2) points (import disambiguation and argument naming), both trivially resolved with 2-token lookahead.

The EAML grammar has already been thoroughly verified for LL(1) compliance (see grammar.ebnf verification report). The key engineering challenges are: (1) designing typed arena allocation with newtype index IDs for each AST node kind, (2) implementing a Pratt parser for expressions with the binding power table from Layer 4, (3) implementing declaration-level error recovery that synchronizes at top-level keywords or depth-0 closing braces, and (4) parsing template strings whose structure is already pre-tokenized by the lexer (TmplStart/TmplText/TmplInterpStart/TmplInterpEnd/TmplEnd).

**Primary recommendation:** Structure implementation bottom-up: AST types first, then parser infrastructure (token cursor, expect/peek helpers, error recovery), then leaf parsers (type expressions, template strings, expressions via Pratt), then declaration parsers, and finally the top-level Program parser with integration tests against all 7 example files.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **AST Node Design:** Typed arena allocation with per-kind Vec (models, schemas, prompts, tools, agents, exprs, type_exprs, etc.). Typed index newtypes per kind: ModelId(u32), SchemaId(u32), ExprId(u32), TypeExprId(u32), etc. Spans stored inline on each node struct. Top-level Program holds Vec<DeclId> preserving source order. Expression children reference via ExprId into expression arena. Type expressions use nested wrapper enum: Optional(TypeExprId), Array(TypeExprId), Bounded { base, params }. Template strings: TemplateString { span, parts: Vec<TemplatePart> }. Every AST enum has an Error variant for parser recovery.
- **Error Recovery Strategy:** Declaration-level synchronization (skip to next top-level keyword or depth-0 closing brace). Skip body on header error. Specific Post-MVP error codes: SYN080 (pipeline), SYN081 (>>), SYN082 (enum), SYN083 (extends), SYN090 (@annotations). Error limit check after each declaration (20 errors).
- **Expression Precedence:** Pratt parser with C-family precedence (primary, postfix, unary, multiplicative, additive, comparison, AND, OR). Chained comparisons parse normally then semantic analysis rejects. Field access + function call compose independently (no special method-call node).

### Claude's Discretion
- Internal parser state machine structure and helper method organization
- Exact Pratt parser binding power numeric values
- Token peek/advance API design
- Arena growth strategy and initial capacity hints

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| PAR-01 | Parser produces AST nodes for all 7 top-level declaration types | Grammar productions [25]-[41] define all 7 declarations; AST arena design with typed IDs covers each |
| PAR-02 | Parser handles type expressions: primitives, named, arrays, optionals, bounded, literal unions | Grammar productions [42]-[50] fully specified; left-factored typeModifiers [42a] is LL(1) |
| PAR-03 | Parser handles expressions via Pratt parsing | Grammar productions [54]-[65]; binding power table from Layer 4 Section 4.3 |
| PAR-04 | Parser handles prompt body with message sections and template strings | Grammar productions [31]-[33] and [51]-[53]; lexer pre-tokenizes template structure |
| PAR-05 | Parser handles `requires` clauses | Grammar production [76]; LL(1) verified (FIRST/FOLLOW disjoint) |
| PAR-06 | Parser handles tool declarations with params, return types, python bridge | Grammar productions [34]-[37]; toolBody left-factored to LL(1) |
| PAR-07 | Parser handles agent declarations | Grammar productions [38]-[40] |
| PAR-08 | Parser recovers from syntax errors via synchronization points | Declaration-level sync at top-level keywords/depth-0 braces; Error variants in AST enums |
| PAR-09 | Every AST node carries source span information | Spans stored inline on each node struct per CONTEXT.md decision |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| eaml-lexer | workspace | Token stream input (LexOutput, TokenKind, Span, Interner) | Phase 1 output, crate boundary contract |
| eaml-errors | workspace | Diagnostic, DiagnosticCollector, ErrorCode, Severity | Shared error infrastructure |
| lasso | 0.7 (workspace) | Spur type for interned identifiers in AST | Already used by lexer, tokens carry Spur keys |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| insta | 1 (workspace) | Snapshot testing for AST output | All parser tests use insta for golden-file AST snapshots |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Hand-written parser | Parser generator (pest, lalrpop) | Hand-written gives full control over error recovery and diagnostics -- critical for EAML's error quality requirements |
| Typed arena per kind | Single Vec<AstNode> with enum | Typed arenas give type-safe indexing (ExprId cannot index into model arena), better cache locality per node kind |
| Pratt for expressions | Recursive descent stratified hierarchy | Pratt is more compact (single function vs 9), same semantics -- standard practice for expression-heavy grammars |

**Installation:**
No new dependencies required. Parser crate already has correct Cargo.toml:
```toml
[dependencies]
eaml-errors = { path = "../eaml-errors" }
eaml-lexer  = { path = "../eaml-lexer" }

[dev-dependencies]
insta = { workspace = true }
```

## Architecture Patterns

### Recommended Project Structure
```
crates/eaml-parser/
├── Cargo.toml
├── src/
│   ├── lib.rs           # Public API: parse(), ParseOutput, Ast types re-exports
│   ├── ast.rs           # AST node types, arena containers, typed IDs
│   ├── parser.rs        # Parser struct, token cursor, expect/peek, error recovery
│   ├── decl.rs          # Declaration parsers (model, schema, prompt, tool, agent, import, let)
│   ├── type_expr.rs     # Type expression parser (baseType, modifiers, bounded, literal union)
│   ├── expr.rs          # Pratt expression parser
│   └── template.rs      # Template string parser
└── tests/
    ├── declarations.rs  # Declaration parsing tests
    ├── type_exprs.rs    # Type expression tests
    ├── expressions.rs   # Expression parsing tests
    ├── templates.rs     # Template string tests
    ├── recovery.rs      # Error recovery tests
    └── examples.rs      # Integration tests against examples/*.eaml
```

### Pattern 1: Typed Arena AST
**What:** Each AST node kind gets its own `Vec` in a central `Ast` struct, with typed newtype index wrappers.
**When to use:** Always -- this is the locked decision from CONTEXT.md.
**Example:**
```rust
// Typed index newtypes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExprId(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeExprId(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModelDeclId(u32);

// Similar for SchemaId, PromptId, ToolId, AgentId, ImportId, LetId

/// Central AST storage with typed arenas
pub struct Ast {
    pub models: Vec<ModelDecl>,
    pub schemas: Vec<SchemaDecl>,
    pub prompts: Vec<PromptDecl>,
    pub tools: Vec<ToolDecl>,
    pub agents: Vec<AgentDecl>,
    pub imports: Vec<ImportDecl>,
    pub lets: Vec<LetDecl>,
    pub exprs: Vec<Expr>,
    pub type_exprs: Vec<TypeExpr>,
}

impl Ast {
    pub fn alloc_expr(&mut self, expr: Expr) -> ExprId {
        let id = ExprId(self.exprs.len() as u32);
        self.exprs.push(expr);
        id
    }
    // Similar alloc methods for each arena
}
```

### Pattern 2: Token Cursor with Peek/Advance
**What:** A `Parser` struct wrapping the token slice with position tracking, providing `peek()`, `advance()`, `expect()`, `at()`, and `eat()` methods.
**When to use:** All parser methods use the cursor to consume tokens.
**Example:**
```rust
pub struct Parser<'src> {
    tokens: Vec<Token>,
    pos: usize,
    source: &'src str,
    ast: Ast,
    diagnostics: Vec<Diagnostic>,
    interner: Interner,
}

impl<'src> Parser<'src> {
    fn peek(&self) -> TokenKind { self.tokens[self.pos].kind }
    fn peek_span(&self) -> Span { self.tokens[self.pos].span.clone() }
    fn advance(&mut self) -> &Token { let t = &self.tokens[self.pos]; self.pos += 1; t }
    fn at(&self, kind: TokenKind) -> bool { /* discriminant match, ignoring Ident payload */ }
    fn eat(&mut self, kind: TokenKind) -> bool { if self.at(kind) { self.advance(); true } else { false } }
    fn expect(&mut self, kind: TokenKind) -> Result<&Token, ()> { /* emit SYN050 on mismatch */ }
}
```

Note: `at()` must compare by discriminant for `Ident(Spur)` since the Spur payload varies. Use `std::mem::discriminant` or a matches! macro.

### Pattern 3: Pratt Expression Parser
**What:** Single `parse_expr(min_bp)` function using binding power table.
**When to use:** All expression parsing -- called from interpolation slots, let RHS, if conditions, etc.
**Example:**
```rust
// Binding power table from Layer 4 Section 4.3
fn prefix_bp(kind: TokenKind) -> Option<u8> {
    match kind {
        TokenKind::Bang | TokenKind::Minus => Some(70),
        TokenKind::KwAwait => Some(65),
        _ => None,
    }
}

fn infix_bp(kind: TokenKind) -> Option<(u8, u8)> {
    match kind {
        TokenKind::PipePipe => Some((10, 11)),       // || left-assoc
        TokenKind::AmpAmp => Some((20, 21)),          // && left-assoc
        TokenKind::EqEq | TokenKind::BangEq => Some((30, 31)),  // == != left-assoc
        TokenKind::LAngle | TokenKind::RAngle
        | TokenKind::LessEq | TokenKind::GreaterEq => Some((35, 36)),
        TokenKind::Plus | TokenKind::Minus => Some((40, 41)),
        TokenKind::Star | TokenKind::Slash => Some((50, 51)),
        TokenKind::Dot => Some((80, 81)),             // member access
        TokenKind::LParen => Some((80, 81)),          // call
        TokenKind::LBracket => Some((80, 81)),        // index
        _ => None,
    }
}
```

**IMPORTANT NOTE on non-associativity:** Layer 5 and Layer 4 specify comparisons as non-associative (BP 30,30 and 35,35). However, the CONTEXT.md decision says "Chained comparisons parse normally via left-associativity, then semantic analysis rejects with SEM060." This means the parser should use left-associative BPs (30,31 and 35,36) and let Phase 3 semantic analysis handle the error. This simplifies the parser.

### Pattern 4: Declaration-Level Error Recovery
**What:** On parse error inside a declaration, skip tokens until a synchronization point.
**When to use:** When any `expect()` or `parse_*()` fails inside a declaration body.
**Example:**
```rust
fn synchronize(&mut self) {
    loop {
        match self.peek() {
            TokenKind::KwModel | TokenKind::KwSchema | TokenKind::KwPrompt
            | TokenKind::KwTool | TokenKind::KwAgent | TokenKind::KwImport
            | TokenKind::KwLet | TokenKind::KwPipeline | TokenKind::KwEnum
            | TokenKind::Eof => break,
            TokenKind::RBrace => {
                // Only sync on depth-0 closing brace
                // (need brace depth tracking during recovery)
                self.advance();
                break;
            }
            _ => { self.advance(); }
        }
    }
}
```

### Anti-Patterns to Avoid
- **Box<Expr> for child expressions:** Use ExprId indices into the arena instead. Box causes pointer chasing and heap fragmentation. Arena gives contiguous memory and O(1) allocation.
- **Separate error list per parser function:** Use a single `diagnostics: Vec<Diagnostic>` on the Parser struct. All parser functions push to the same list.
- **Panicking on unexpected tokens:** Never panic. Always emit a diagnostic and either recover or produce an Error AST node. The parser must be resilient.
- **Re-lexing or modifying tokens:** The parser consumes tokens as-is from the lexer. No token modification, insertion, or re-lexing.
- **Comparing Ident by value in parser:** Field names like "user", "system", "temperature" in prompt bodies are contextual keywords. Compare by resolving the Spur through the interner, not by pattern-matching TokenKind. Use a helper: `fn at_contextual(&self, name: &str) -> bool`.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| String interning | Custom hash map | lasso::Rodeo (already in lexer) | Parser receives Interner from LexOutput, just use resolve() |
| Error display | Custom formatting | codespan-reporting (in eaml-errors) | Already integrated in Phase 1, render module handles it |
| Snapshot testing | Custom golden files | insta crate | Already in workspace, used by lexer tests, review with `cargo insta review` |
| Token kind display | Manual Display impl | Derive or manual but MUST match lexer | Token kind names must be consistent across crates |

**Key insight:** The parser crate depends only on eaml-errors and eaml-lexer. All infrastructure for error reporting and string interning is already built. The parser's job is purely: tokens in, AST + diagnostics out.

## Common Pitfalls

### Pitfall 1: TokenKind::Ident Comparison
**What goes wrong:** `self.at(TokenKind::Ident(some_spur))` fails because Spur values differ even for the same string.
**Why it happens:** TokenKind::Ident carries a Spur payload. Direct equality comparison requires the exact Spur value.
**How to avoid:** Use `matches!(self.peek(), TokenKind::Ident(_))` for "is this any identifier?" checks. For contextual keyword matching, resolve the Spur through the interner: `if let TokenKind::Ident(spur) = self.peek() { self.interner.resolve(&spur) == "user" }`.
**Warning signs:** Tests pass for first identifier but fail for subsequent occurrences of the same name.

### Pitfall 2: Template String Already Tokenized
**What goes wrong:** Attempting to scan for `{` or `}` characters when parsing template strings.
**Why it happens:** The lexer has already decomposed template strings into TmplStart, TmplText, TmplInterpStart, (expression tokens), TmplInterpEnd, TmplEnd sequences.
**How to avoid:** The parser just matches the token sequence: expect TmplStart, loop on TmplText/TmplInterpStart, parse expression after TmplInterpStart, expect TmplInterpEnd, until TmplEnd.
**Warning signs:** Seeing raw `{` or `}` tokens where TmplInterpStart/End are expected.

### Pitfall 3: Span Construction for Compound Nodes
**What goes wrong:** AST nodes get incorrect spans that don't cover the full syntax.
**Why it happens:** Forgetting to capture the start span before parsing child nodes, or using the wrong end span.
**How to avoid:** Always capture `let start = self.peek_span().start` before parsing, then `let end = self.previous_span().end` after. Construct span as `start..end`.
**Warning signs:** Error reporting points to wrong locations in source.

### Pitfall 4: Semicolon Consumption
**What goes wrong:** Parser fails on valid input because it expects a semicolon or chokes on a semicolon it doesn't expect.
**Why it happens:** Semicolons are optional everywhere (Layer 5 Section 2.2). Every declaration and statement must have `self.eat(TokenKind::Semicolon)` at the end.
**How to avoid:** After every declaration and statement parse function, call `self.eat(TokenKind::Semicolon)` (fire-and-forget, don't check result).
**Warning signs:** Tests fail when semicolons are present or absent in different combinations.

### Pitfall 5: Left-Factored Tool Body
**What goes wrong:** Parser enters wrong branch when parsing tool body.
**Why it happens:** Both `python %{ }%` and native body start with `{`. The grammar is left-factored: consume `{`, then peek.
**How to avoid:** After consuming `{` in tool body, check if next token is `KwPythonBridge` or a contextual "description" identifier. The lexer emits `KwPythonBridge` (not `KwPython`) when `python` is followed by `%{`. Also check for `"description"` contextual keyword per production [36].
**Warning signs:** Python bridge tools parse incorrectly, or native tool bodies don't produce SYN050.

### Pitfall 6: Literal Union vs String in Type Position
**What goes wrong:** A single string literal in type position is incorrectly parsed as a literal union.
**Why it happens:** Grammar production [50] requires minimum TWO members: `STRING ( "|" STRING )+`.
**How to avoid:** After parsing the first STRING in a type expression, check for `|`. If no pipe follows, this is just a string type (which semantic analysis will handle). If pipe follows, continue parsing the union.
**Warning signs:** Single-string type annotations produce union AST nodes.

### Pitfall 7: `<` Disambiguation (Type vs Comparison)
**What goes wrong:** `float<0.0, 1.0>` in type position is parsed as comparison expression.
**Why it happens:** `<` can be type parameter opener or comparison operator.
**How to avoid:** The parser knows whether it's in a type expression context or an expression context based on the call stack. Type expressions are parsed by `parse_type_expr()` which explicitly handles `<` as bounded suffix. Expressions are parsed by `parse_expr()` which treats `<` as comparison. These are separate code paths -- no flag needed.
**Warning signs:** Bounded type parameters cause parse errors.

### Pitfall 8: Error Recovery Consuming Too Much
**What goes wrong:** After a parse error, synchronization skips past valid declarations.
**Why it happens:** The synchronize function doesn't respect brace depth, or stops at a keyword inside a nested block.
**How to avoid:** Track brace depth during recovery. Only synchronize at depth 0. When encountering `{`, increment depth. When encountering `}`, decrement. Only consider keywords at depth 0 as sync points.
**Warning signs:** First error causes all subsequent declarations to be skipped.

## Code Examples

### Parser Public API
```rust
// Source: designed from CONTEXT.md decisions and existing crate patterns

/// Output of the parse function.
pub struct ParseOutput {
    /// The abstract syntax tree.
    pub ast: Ast,
    /// The program root with declarations in source order.
    pub program: Program,
    /// All diagnostics (from both lexer and parser).
    pub diagnostics: Vec<Diagnostic>,
    /// The string interner (passed through from lexer).
    pub interner: Interner,
}

/// Top-level program node.
pub struct Program {
    pub declarations: Vec<DeclId>,
    pub span: Span,
}

/// A declaration identifier pointing into the appropriate typed arena.
#[derive(Debug, Clone, Copy)]
pub enum DeclId {
    Model(ModelDeclId),
    Schema(SchemaDeclId),
    Prompt(PromptDeclId),
    Tool(ToolDeclId),
    Agent(AgentDeclId),
    Import(ImportDeclId),
    Let(LetDeclId),
    Error(Span),  // Error recovery placeholder
}

/// Parses EAML source text into an AST.
pub fn parse(source: &str) -> ParseOutput {
    let lex_output = eaml_lexer::lex(source);
    let mut parser = Parser::new(
        source,
        lex_output.tokens,
        lex_output.interner,
        lex_output.diagnostics,
    );
    parser.parse_program()
}
```

### Type Expression Parsing
```rust
// Source: grammar.ebnf productions [42]-[50]

fn parse_type_expr(&mut self) -> TypeExprId {
    let start = self.peek_span().start;

    // Try literal union first: STRING ("|" STRING)+
    if self.at_tmpl_start_or_string() {
        // Check if this is a literal union by looking ahead for "|"
        // Actually: literal unions use STRING tokens not template strings
        // The lexer tokenizes all strings as templates, so we need to
        // handle this carefully
    }

    let base = self.parse_base_type();

    // typeModifiers: arraySuffix then optional?, or optionalSuffix then optional[]?
    match self.peek() {
        TokenKind::LBracket => {
            // [] first, then optional ?
            self.advance(); // [
            self.expect(TokenKind::RBracket); // ]
            // Check for SYN042 multi-dimensional array
            if self.at(TokenKind::LBracket) {
                self.emit_error(/* SYN042 */);
            }
            let array_id = self.ast.alloc_type_expr(TypeExpr::Array(base, span));
            if self.eat(TokenKind::Question) {
                self.ast.alloc_type_expr(TypeExpr::Optional(array_id, span))
            } else {
                array_id
            }
        }
        TokenKind::Question => {
            // ? first, then optional []
            self.advance(); // ?
            let opt_id = self.ast.alloc_type_expr(TypeExpr::Optional(base, span));
            if self.at(TokenKind::LBracket) {
                self.advance(); // [
                self.expect(TokenKind::RBracket); // ]
                let array_id = self.ast.alloc_type_expr(TypeExpr::Array(opt_id, span));
                if self.eat(TokenKind::Question) {
                    self.ast.alloc_type_expr(TypeExpr::Optional(array_id, span))
                } else {
                    array_id
                }
            } else {
                opt_id
            }
        }
        _ => base, // bare type, no modifiers
    }
}
```

### Model Declaration Parsing
```rust
// Source: grammar.ebnf production [27]

fn parse_model_decl(&mut self) -> DeclId {
    let start = self.peek_span().start;
    self.expect(TokenKind::KwModel);  // already checked by dispatch
    let name = self.expect_ident();   // IDENT
    self.expect(TokenKind::Eq);       // =
    self.expect_contextual("Model");  // "Model" (predeclared identifier)
    self.expect(TokenKind::LParen);   // (

    // Fixed order: id, provider, caps
    self.expect_contextual("id");
    self.expect(TokenKind::Colon);
    let id_str = self.parse_template_string(); // STRING as template
    self.expect(TokenKind::Comma);

    self.expect_contextual("provider");
    self.expect(TokenKind::Colon);
    let provider = self.parse_template_string();
    self.expect(TokenKind::Comma);

    self.expect_contextual("caps");
    self.expect(TokenKind::Colon);
    self.expect(TokenKind::LBracket);
    let caps = self.parse_cap_list();
    self.expect(TokenKind::RBracket);

    self.expect(TokenKind::RParen);
    self.eat(TokenKind::Semicolon);

    let end = self.previous_span().end;
    let id = self.ast.alloc_model(ModelDecl {
        name, id: id_str, provider, caps, span: start..end,
    });
    DeclId::Model(id)
}
```

### Template String Parsing
```rust
// Source: grammar.ebnf productions [51]-[53]

fn parse_template_string(&mut self) -> TemplateString {
    let start = self.peek_span().start;
    self.expect(TokenKind::TmplStart);  // opening "

    let mut parts = Vec::new();
    loop {
        match self.peek() {
            TokenKind::TmplText => {
                let span = self.peek_span();
                self.advance();
                parts.push(TemplatePart::Text(span));
            }
            TokenKind::TmplInterpStart => {
                let interp_start = self.peek_span().start;
                self.advance(); // {
                let expr_id = self.parse_expr(0); // full expression
                let interp_end = self.peek_span().end;
                self.expect(TokenKind::TmplInterpEnd); // }
                parts.push(TemplatePart::Interpolation(expr_id, interp_start..interp_end));
            }
            TokenKind::TmplEnd => {
                self.advance(); // closing "
                break;
            }
            TokenKind::Eof => {
                // Unterminated -- lexer already emitted SYN002
                break;
            }
            _ => {
                // Unexpected token inside template string
                self.emit_error(/* SYN050: expected template content */);
                break;
            }
        }
    }

    let end = self.previous_span().end;
    TemplateString { span: start..end, parts }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Box<Expr> tree allocation | Typed arena with index IDs | Standard in modern Rust parsers (rust-analyzer, 2020+) | Better cache locality, no pointer chasing, cheaper cloning |
| Visitor pattern for AST traversal | Index-based side tables | rust-analyzer pattern | Downstream passes (semantic, codegen) use separate side tables keyed by NodeId, not mutable AST |
| Stratified recursive descent for expressions | Pratt parsing | Pratt 1973, popularized by matklad 2020 | Single function replaces N precedence-level functions |
| String allocation per identifier | String interning (lasso) | Standard practice | Memory efficient, O(1) comparison via Spur |

**Deprecated/outdated:**
- Nothing specific -- hand-written recursive descent with Pratt is the current gold standard for language parsers (used by Rust compiler, TypeScript, V8, etc.)

## Open Questions

1. **String literal values in type position (literal unions)**
   - What we know: The lexer tokenizes ALL strings as template strings (TmplStart...TmplEnd). Literal union parsing in type expressions needs to consume these template string tokens and extract the literal text.
   - What's unclear: Should the parser reconstruct the string value from TmplText tokens, or should it just reference the span and let downstream passes extract the value?
   - Recommendation: Store the span and resolve from source text. Template strings in type position should be simple (no interpolation). If TmplInterpStart is found inside a type-position string, emit a parse error.

2. **Interner ownership transfer**
   - What we know: LexOutput owns the Interner. The parser needs it for contextual keyword resolution.
   - What's unclear: Should the parser take ownership or borrow?
   - Recommendation: Take ownership (move) since ParseOutput needs to carry it forward for downstream phases. The parser consumes LexOutput.

3. **`requires` keyword: reserved or contextual?**
   - What we know: Grammar production [76] shows `"requires"` in quotes as a literal. But `requires` is NOT in the keyword list (Section 2.5 of Layer 5). It is listed under contextual keywords in the grammar header.
   - What's unclear: The lexer would emit `Ident(spur)` for `requires`, not a keyword token.
   - Recommendation: Treat `requires` as a contextual keyword. In `parse_prompt_decl()`, after the parameter list closing `)`, check if the next token is `Ident` resolving to "requires". This is consistent with how other contextual keywords (user, system, id, provider, caps) work.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | insta 1.x + cargo test |
| Config file | crates/eaml-parser/Cargo.toml (insta in dev-dependencies) |
| Quick run command | `cargo test -p eaml-parser` |
| Full suite command | `cargo test --workspace` |

### Phase Requirements to Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| PAR-01 | All 7 declaration types produce AST nodes | unit + snapshot | `cargo test -p eaml-parser -- declarations` | Wave 0 |
| PAR-02 | Type expressions parse correctly | unit + snapshot | `cargo test -p eaml-parser -- type_exprs` | Wave 0 |
| PAR-03 | Pratt expression parsing | unit + snapshot | `cargo test -p eaml-parser -- expressions` | Wave 0 |
| PAR-04 | Prompt body with template strings | unit + snapshot | `cargo test -p eaml-parser -- templates` | Wave 0 |
| PAR-05 | Requires clauses | unit | `cargo test -p eaml-parser -- requires` | Wave 0 |
| PAR-06 | Tool declarations | unit + snapshot | `cargo test -p eaml-parser -- tool` | Wave 0 |
| PAR-07 | Agent declarations | unit + snapshot | `cargo test -p eaml-parser -- agent` | Wave 0 |
| PAR-08 | Error recovery | unit | `cargo test -p eaml-parser -- recovery` | Wave 0 |
| PAR-09 | Span information on all nodes | unit | `cargo test -p eaml-parser -- spans` | Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p eaml-parser`
- **Per wave merge:** `cargo test --workspace && make check`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `crates/eaml-parser/tests/declarations.rs` -- covers PAR-01
- [ ] `crates/eaml-parser/tests/type_exprs.rs` -- covers PAR-02
- [ ] `crates/eaml-parser/tests/expressions.rs` -- covers PAR-03
- [ ] `crates/eaml-parser/tests/templates.rs` -- covers PAR-04, PAR-05
- [ ] `crates/eaml-parser/tests/recovery.rs` -- covers PAR-08
- [ ] `crates/eaml-parser/tests/examples.rs` -- integration tests against examples/*.eaml

## Sources

### Primary (HIGH confidence)
- `spec/grammar.ebnf` -- All 84 grammar productions, verification report, FIRST/FOLLOW analysis
- `.claude/references/eaml-layer5-design-decisions.md` -- All CLOSED design decisions, ambiguity resolutions (Section 12), Post-MVP reserved syntax (Section 11), EG-rules (Section 14)
- `.claude/references/eaml-layer4-compiler-theory.md` -- LL(1) analysis, Pratt parsing theory, binding power table (Section 4)
- `spec/TYPESYSTEM.md` -- Type expression rules, modifier ordering, bounded types
- `spec/ERRORS.md` -- Error code catalog, SYN-prefix codes relevant to parser
- `crates/eaml-lexer/src/token.rs` -- TokenKind enum (actual token types parser will consume)
- `crates/eaml-lexer/src/lexer.rs` -- LexOutput structure, Interner API
- `crates/eaml-errors/src/diagnostic.rs` -- Diagnostic, DiagnosticCollector API
- `crates/eaml-errors/src/codes.rs` -- ErrorCode enum (parser error codes: Syn050, Syn060, Syn061, Syn080-083, Syn090)

### Secondary (MEDIUM confidence)
- `examples/01-minimal/*.eaml` through `examples/07-all-type-variants/*.eaml` -- Integration test inputs (4 populated, 3 gitkeep stubs)

### Tertiary (LOW confidence)
- None -- all findings verified from authoritative project sources

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all libraries already in workspace, no new dependencies needed
- Architecture: HIGH -- typed arena pattern well-understood, grammar fully verified LL(1), decisions locked in CONTEXT.md
- Pitfalls: HIGH -- grammar verification report identifies all conflict points, lexer token structure is fully documented
- Code examples: MEDIUM -- examples show intended patterns but exact API will evolve during implementation

**Research date:** 2026-03-16
**Valid until:** 2026-04-16 (stable domain -- compiler internals, no external dependencies)
