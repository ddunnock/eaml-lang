# Feature Landscape

**Domain:** Compiled DSL for LLM integrations (Rust compiler -> Python/Pydantic)
**Researched:** 2026-03-15

## Table Stakes

Features users expect. Missing = product feels incomplete.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Compile .eaml to valid Python | Core value proposition | High | 84 grammar productions, all declaration types |
| Colored error messages with source snippets | Standard in modern compilers (rustc, ruff) | Medium | codespan-reporting handles display |
| All 38 error codes emitted correctly | Spec compliance | Medium | Each code maps to specific condition |
| Schema -> Pydantic BaseModel | Core type system feature | Medium | Includes bounded types, literal unions, optionals |
| Prompt -> async Python function | Core execution feature | Medium | Template string interpolation, system/user messages |
| Model declaration validation | Prevents runtime errors | Low | id, provider, caps fields |
| Tool -> Python function with bridge block | Python bridge is a key differentiator | High | Opaque code passthrough, marshaling rules |
| Agent -> orchestration function | Multi-tool coordination | High | Tool dispatch, error policies, max_turns |
| Type checking (all TYP errors) | Compile-time safety is the value prop | High | Nominal typing, bounded types, composites |
| Capability checking (CAP010) | Prevents "model doesn't support X" at runtime | Medium | Subset check: prompt requires <= model caps |
| CLI with compile/check commands | Standard compiler interface | Low | clap-derived, file I/O |

## Differentiators

Features that set product apart. Not expected, but valued.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Compile-time capability checking | No other LLM DSL does this statically | Medium | CAP010 catches json_mode/streaming mismatches before runtime |
| Bounded types (float<0,1>, int<1,10>) | Express constraints in the type system, validated by Pydantic at runtime | Medium | Unique to EAML among LLM DSLs |
| Literal union types ("a" | "b" | "c") | Constrained string outputs without external enums | Low | Maps to Pydantic Literal |
| Python bridge blocks | Escape hatch for custom logic without leaving the language | Medium | python %{ }% with type-safe marshaling |
| validate_or_retry runtime loop | Automatic retry when LLM output fails Pydantic validation | Medium | Runtime feature, configurable max_retries |
| Provider-agnostic code | Switch providers by changing model declaration only | Medium | Runtime adapter pattern |

## Anti-Features

Features to explicitly NOT build.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| Multi-file imports (v0.1) | Spec marks as post-MVP. Adds module resolution complexity | Single-file compilation only |
| Schema inheritance / extends | Post-MVP per TYPESYSTEM.md. Complicates type checking significantly | Use composition (schema fields of schema type) |
| Enum types | Post-MVP per TYPESYSTEM.md. Literal unions cover the common case | Use literal union types |
| Pipeline operators | Post-MVP per grammar.ebnf. Complex data flow semantics | Chain prompts in Python code |
| Async python bridge blocks | Closed as unsupported in v0.1 (OQ-01). Adds runtime complexity | Use sync code in bridge blocks |
| LSP server | Needs stable compiler first. tower-lsp reserved for Phase 7 | Focus on CLI compiler |
| Type inference | Post-MVP. All type annotations are explicit in v0.1 | Require explicit annotations |

## Feature Dependencies

```
eaml-errors (Diagnostic, Span) -> ALL other features
Token types (lexer) -> Parser (needs tokens to parse)
AST types (parser) -> Semantic analysis (walks AST)
Symbol table (semantic pass 1) -> Type checking (semantic pass 2)
Type checking (semantic pass 2) -> Capability checking (semantic pass 3)
AnalyzedProgram (semantic) -> Code generation (needs type info)
CodeWriter (codegen) -> Python emission (indentation tracking)
Provider adapters (runtime) -> Generated code execution
Pydantic validation (runtime) -> validate_or_retry loop
```

## MVP Recommendation

Prioritize:
1. Error types and diagnostics (eaml-errors) -- foundation for all error reporting
2. Lexer with all token types -- unblocks parser work
3. Parser with AST for model + schema + prompt -- the three most common declarations
4. Semantic analysis (name resolution + type checking) -- core value proposition
5. Codegen for schema (BaseModel) and prompt (async function) -- minimum runnable output
6. Python runtime with one provider (Anthropic) -- proves end-to-end compilation works

Defer: Tool/Agent declarations to second pass (more complex, depend on Python bridge)
Defer: Ollama provider (httpx adapter is simpler but lower priority than Anthropic/OpenAI)

## Sources

- spec/grammar.ebnf (84 productions)
- spec/ERRORS.md (38 error codes)
- spec/TYPESYSTEM.md (type checking rules)
- spec/CAPABILITIES.md (capability checking rules)
- spec/PYTHON_BRIDGE.md (python bridge rules)
- Layer 5 design decisions (post-MVP deferrals)
- [BAML](https://github.com/BoundaryML/baml) -- closest prior art for feature comparison
