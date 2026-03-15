# Phase 1: Error Foundation and Lexer - Context

**Gathered:** 2026-03-15
**Status:** Ready for planning

<domain>
## Phase Boundary

The compiler can tokenize any EAML source file into a stream of typed tokens with accurate source positions, emitting structured diagnostics for malformed input. This includes shared error infrastructure (eaml-errors crate) and complete lexer (eaml-lexer crate). Parser, semantic analysis, codegen, runtime, and CLI are separate phases.

</domain>

<decisions>
## Implementation Decisions

### Error Diagnostic Experience
- Rustc-style diagnostics: colored source snippets with underlines, error codes, primary message, and optional help/hint lines via codespan-reporting
- Error codes always displayed in output: `error[SYN042]: unterminated string` format
- Accumulate up to 20 errors before stopping (overridable with `--max-errors N`)
- Phase 1 implements ERROR severity only; FATAL and WARNING severity levels deferred to later phases when semantic analysis introduces them

### Template String Tokenization
- Multi-token approach: TemplateStart, StringFragment, InterpolationStart, <expr tokens>, InterpolationEnd, StringFragment, TemplateEnd
- Parser sees structure directly without re-lexing — lexer does all the mode switching
- Newlines normalized to LF inside template text fragments (matches grammar spec's NL handling)
- Lexer handles brace escaping: `{{` and `}}` are converted to text fragments containing literal `{` and `}` — parser never sees escape sequences
- Unterminated interpolation (missing closing brace): emit SYN error, recover at end of line, continue lexing

### Python Bridge Handling
- Closing `}%` delimiter must appear at the start of a line (possibly with leading whitespace only) — avoids false-close from Python f-strings like `f"{value}% done"`
- Bridge block content is completely opaque — lexer captures raw bytes, no Python validation
- Unterminated python block (missing `}%`): scan to EOF, emit SYN error pointing back to opening `python %{`

### Lexer Error Recovery
- Unrecognizable character: skip one byte, emit SYN error, continue lexing
- Unterminated string literal: recover at end of line, emit SYN error
- Consecutive identical errors at adjacent positions: collapse into one diagnostic spanning the range (e.g., "unexpected characters at 5:1-5:10" instead of 10 separate errors)
- No 'did you mean?' suggestions in Phase 1 — deferred to parser/semantic phases where context is available

### Claude's Discretion
- Exact codespan-reporting configuration and color scheme
- Internal error accumulation data structure design
- Logos wrapper layer architecture for mode switching
- Token type enum naming and organization
- Test fixture organization for snapshot tests

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Grammar and Language
- `spec/grammar.ebnf` -- Formal grammar (84 productions), token definitions, whitespace handling policy, lexer annotations
- `.claude/references/eaml-layer5-design-decisions.md` -- Authoritative design decisions (all [CLOSED] entries are final)

### Error System
- `spec/ERRORS.md` -- All 38 error codes, severity levels, error code format (PREFIX + NNN), SYN-prefix codes relevant to lexer

### Python Bridge
- `spec/PYTHON_BRIDGE.md` -- `python %{ }%` specification, bridge block semantics

### Type System (for token design awareness)
- `spec/TYPESYSTEM.md` -- Bounded types, literal unions — affects what tokens the lexer must produce

### Reference Layers (load in order for grammar work)
- `.claude/references/eaml-layer1-notation-reference.md` -- W3C EBNF operators
- `.claude/references/eaml-layer4-compiler-theory.md` -- FIRST/FOLLOW, Pratt, LL(1) theory

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- None — both `eaml-errors/src/lib.rs` and `eaml-lexer/src/lib.rs` are stubs (doc comments only)
- Workspace Cargo.toml already pins logos 0.14, lasso 0.7, codespan-reporting 0.11, thiserror 1, insta 1

### Established Patterns
- Crate boundary convention: each crate has single `src/lib.rs` as public API surface
- Workspace dep references: `{ workspace = true }` in per-crate Cargo.toml
- Snapshot testing with insta for AST and codegen golden tests
- Clippy with `-D warnings` (warnings are errors)

### Integration Points
- `eaml-errors` is the dependency root — all other crates depend on it
- `eaml-lexer` depends only on `eaml-errors` — strict boundary
- Parser (Phase 2) will consume the token stream produced by this phase

</code_context>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches. All decisions followed recommended defaults aligned with spec documents and industry conventions (Rust compiler-style diagnostics, multi-token template approach).

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 01-error-foundation-and-lexer*
*Context gathered: 2026-03-15*
