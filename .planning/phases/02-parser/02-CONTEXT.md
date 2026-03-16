# Phase 2: Parser - Context

**Gathered:** 2026-03-16
**Status:** Ready for planning

<domain>
## Phase Boundary

Hand-written recursive descent parser that consumes the Phase 1 token stream (TokenKind, Span, Interner) and produces a complete AST representing all EAML language constructs. Covers all 84 grammar productions including declarations, type expressions, expressions (Pratt), template strings, and error recovery. Semantic analysis, codegen, runtime, and CLI are separate phases.

</domain>

<decisions>
## Implementation Decisions

### AST Node Design
- Typed arena allocation: each node kind gets its own Vec (models, schemas, prompts, tools, agents, exprs, type_exprs, etc.)
- Typed index newtypes per kind: ModelId(u32), SchemaId(u32), ExprId(u32), TypeExprId(u32), etc.
- Spans stored inline on each node struct (not in a parallel side-table)
- Top-level Program holds a single Vec<DeclId> preserving source order, where DeclId is an enum pointing to typed arenas
- Expression children reference via ExprId indices into the expression arena (no Box<Expr>)
- Type expressions use nested wrapper enum: Optional(TypeExprId), Array(TypeExprId), Bounded { base, params } — directly encodes Layer 5 modifier ordering (T?[] vs T[]?)
- Template strings represented as TemplateString { span, parts: Vec<TemplatePart> } where TemplatePart is Text(String, Span) or Interpolation(ExprId, Span)
- Every AST enum (Expr, TypeExpr, Decl) has an Error variant for parser recovery points — enables ErrorNode propagation in downstream passes

### Error Recovery Strategy
- Declaration-level synchronization: on error inside a declaration body, skip tokens until the next top-level keyword (model, schema, prompt, tool, agent, import, let) or closing brace at depth 0
- Skip body on header error: if the declaration header fails (missing name, etc.), skip to matching '}' or next declaration keyword without attempting to parse body contents
- Specific Post-MVP error codes for reserved keywords used as declarations: SYN080 (pipeline), SYN081 (>>), SYN082 (enum), SYN083 (extends), SYN090 (@annotations) — per Layer 5 Section 11
- Check error limit after each declaration: if DiagnosticCollector has hit overflow (20 errors), stop parsing and return collected AST

### Expression Precedence
- Pratt parser with standard C-family precedence (highest to lowest):
  1. Primary: literals, identifiers, parenthesized expressions
  2. Postfix: field access `.`, function call `()`
  3. Unary prefix: `-`, `!`
  4. Multiplicative: `*`, `/`
  5. Additive: `+`, `-`
  6. Comparison: `<`, `>`, `<=`, `>=`, `==`, `!=`
  7. Logical AND: `&&`
  8. Logical OR: `||`
- Chained comparisons (`a == b == c`) parse normally via left-associativity, then semantic analysis (Phase 3) rejects with SEM060 per Layer 5 Rule EG-06
- Field access and function call compose independently: `obj.method(args)` parses as Call(FieldAccess(obj, method), args) — no special method-call node

### Claude's Discretion
- Internal parser state machine structure and helper method organization
- Exact Pratt parser binding power numeric values
- Token peek/advance API design
- Arena growth strategy and initial capacity hints

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Grammar and Language
- `spec/grammar.ebnf` -- Formal grammar (84 productions), all parser productions defined here
- `.claude/references/eaml-layer5-design-decisions.md` -- Authoritative design decisions, all ambiguity resolutions (Section 12), Post-MVP error codes (Section 11)

### Compiler Theory
- `.claude/references/eaml-layer4-compiler-theory.md` -- FIRST/FOLLOW sets, Pratt parsing theory, LL(1)/LL(2) analysis

### Error System
- `spec/ERRORS.md` -- SYN-prefix error codes relevant to parser (SYN042, SYN050, SYN060-061, SYN080-083, SYN090)

### Type System
- `spec/TYPESYSTEM.md` -- Type expression rules: bounded types, literal unions, modifier ordering (T?[] vs T[]?)

### Python Bridge
- `spec/PYTHON_BRIDGE.md` -- Import declaration syntax, tool body requirements

### Example Programs
- `examples/01-minimal` through `examples/07-all-type-variants` -- Integration test inputs that must parse successfully

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `eaml-lexer` crate: Token, TokenKind (29 keywords + operators + template tokens + python bridge), Span (byte-offset range), Interner (lasso), lex() function returning LexOutput
- `eaml-errors` crate: Diagnostic, DiagnosticCollector (with has_overflow()), ErrorCode enum, Severity, codespan-reporting rendering
- Multi-token template string tokens: TmplStart, TmplText, TmplInterpStart, TmplInterpEnd, TmplEnd — parser sees structure directly
- Python bridge: KwPythonBridge + PythonBlock tokens — parser treats as opaque

### Established Patterns
- Crate boundary: parser depends only on eaml-errors and eaml-lexer
- Snapshot testing with insta crate (used extensively in Phase 1 lexer tests)
- Clippy with -D warnings, cargo fmt
- TokenKind derives Copy — cheap to pass around
- Workspace dep references via { workspace = true }

### Integration Points
- Parser consumes: LexOutput { tokens: Vec<Token>, diagnostics: Vec<Diagnostic>, interner: Interner }
- Parser produces: AST (Program + typed arenas) + accumulated Diagnostic list
- Semantic analysis (Phase 3) will consume the AST and use NodeId-based side tables for annotations

</code_context>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches. All decisions followed recommended defaults aligned with Rust compiler conventions (typed arenas, Pratt parsing, declaration-level sync recovery).

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 02-parser*
*Context gathered: 2026-03-16*
