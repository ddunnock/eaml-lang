# Project Research Summary

**Project:** EAML Compiler
**Domain:** Rust-based compiler for LLM DSL targeting Python/Pydantic
**Researched:** 2026-03-15
**Confidence:** HIGH

## Executive Summary

EAML is a compiler project with a well-defined specification (84 grammar productions, 38 error codes, full type system) and a clear target (Python 3.11+ / Pydantic v2). The implementation follows the standard compiler pipeline pattern: lexer → parser → semantic analysis → code generation, with each phase isolated in its own Rust crate. This is a proven architecture used by rustc, BAML, and other production compilers.

The recommended approach is bottom-up implementation following crate dependency order: errors → lexer → parser → semantic → codegen → runtime → CLI. Each phase is independently testable with snapshot tests (insta). The Python runtime can be developed in parallel with codegen since it's consumed by generated code, not linked to Rust.

Key risks are: Python indentation bugs in codegen (mitigated by CodeWriter pattern), template string brace depth tracking (mitigated by explicit depth counter + edge case tests), and Pydantic v2 API surface (mitigated by snapshot tests that verify generated Python imports and type-checks). The `}%` delimiter collision in python bridge blocks is a known edge case (EG-02) that needs documentation, not code changes.

## Key Findings

### Recommended Stack

Stack is already chosen and well-suited. logos 0.14 for lexing with manual mode switching for template strings and python blocks. lasso 0.7 for string interning. insta for snapshot testing. codespan-reporting for error display. No template engine for codegen — use CodeWriter builder pattern instead.

**Core technologies:**
- logos 0.14: Derive-based tokenizer — reduces boilerplate for simple tokens
- lasso 0.7: String interning — memory-efficient identifier deduplication
- codespan-reporting 0.11: Error display — rich diagnostics with source spans
- Pydantic v2: Generated code target — schema validation, JSON schema, model_rebuild

### Expected Features

**Must have (table stakes):**
- Correct compilation of all 84 grammar productions
- Meaningful error messages with source spans and hints
- Type checking (bounded types, literal unions, nullable)
- Capability validation (CAP010 = fatal mismatch)
- Multi-provider runtime (Anthropic, OpenAI, Ollama)

**Should have (competitive):**
- Compile-time capability checking (unique to EAML)
- Python bridge blocks for inline data processing
- Declarative agent composition
- Structured output guarantee (validate_or_retry)

**Defer (v2+):**
- LSP server / IDE support
- Import/module system
- Schema inheritance, enum types
- Pipeline operators

### Architecture Approach

Classic multi-stage pipeline with strict crate boundaries. Each crate has a single public API function (lex, parse, analyze, generate). Data flows forward only — no crate depends on a later-stage crate. The Python runtime is fully independent of the Rust compiler, connected only by the generated Python source code.

**Major components:**
1. eaml-errors — shared error codes and diagnostic types
2. eaml-lexer — three-mode tokenizer (normal, template, python block)
3. eaml-parser — recursive descent, one function per production
4. eaml-semantic — three-pass analysis (names, types, capabilities)
5. eaml-codegen — CodeWriter-based Python generation
6. eaml-runtime — provider adapters, validation, telemetry
7. eaml-cli — pipeline orchestration

### Critical Pitfalls

1. **Python indentation bugs** — Use CodeWriter with explicit indent/dedent; snapshot test all codegen paths
2. **Template string brace depth** — Explicit brace_depth counter; test `{dict[key]}` patterns
3. **`}%` delimiter collision** — Document workaround (str.format instead of f-strings); known edge case
4. **Pydantic v2 API surface** — Use v2 methods exclusively; snapshot tests catch v1 usage
5. **Span offset drift across lexer modes** — Single cursor as source of truth; verify `source[span]` matches

## Implications for Roadmap

### Phase 1: Error Foundation + Lexer
**Rationale:** Everything depends on error types; lexer is first pipeline stage
**Delivers:** TokenKind enum, three lexer modes, all tokens from grammar.ebnf
**Addresses:** Tokenization, template strings, python blocks
**Avoids:** Span drift (test from day one)

### Phase 2: Parser + AST
**Rationale:** Depends on lexer; parser is second pipeline stage
**Delivers:** AST types, recursive descent parser for all 84 productions
**Addresses:** Full grammar coverage
**Avoids:** Over-engineering (simple enum-based AST, no visitor pattern)

### Phase 3: Semantic Analysis
**Rationale:** Depends on parser; three passes build on each other
**Delivers:** Symbol table, type checking, capability validation
**Addresses:** Name resolution, type safety, CAP010

### Phase 4: Code Generation
**Rationale:** Depends on semantic analysis; produces target output
**Delivers:** CodeWriter, Python/Pydantic output for all declaration types
**Addresses:** Valid Python output, Pydantic v2 patterns
**Avoids:** Indentation bugs (CodeWriter pattern), Pydantic v1 API

### Phase 5: Python Runtime
**Rationale:** Independent of Rust crates; needed for generated code to run
**Delivers:** Provider adapters, validate_or_retry, telemetry hooks
**Addresses:** Multi-provider support, structured output guarantee

### Phase 6: CLI + Integration
**Rationale:** Wires everything together; needs all other phases complete
**Delivers:** eamlc compile/check/run, end-to-end tests
**Addresses:** Developer experience, example programs running

### Phase Ordering Rationale

- Strict dependency order: errors → lexer → parser → semantic → codegen
- Runtime is independent and can overlap with codegen development
- CLI is last because it orchestrates all crates
- Each phase is independently testable before moving to the next

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 1 (Lexer):** logos mode-switching for template strings needs investigation — may need hybrid approach
- **Phase 5 (Runtime):** Provider API differences (Anthropic requires max_tokens, OpenAI needs JSON instruction with json_mode)

Phases with standard patterns (skip research-phase):
- **Phase 2 (Parser):** Well-documented recursive descent patterns; grammar.ebnf is the blueprint
- **Phase 3 (Semantic):** Standard three-pass architecture; spec documents describe every check
- **Phase 6 (CLI):** Standard clap derive pattern; wiring only

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | All deps already chosen and version-pinned in workspace |
| Features | HIGH | Spec is frozen for v0.1; feature set is well-defined |
| Architecture | HIGH | Standard compiler pipeline; crate boundaries already established |
| Pitfalls | HIGH | Domain-specific issues well-documented in spec (EG-02, PYB-MAR rules) |

**Overall confidence:** HIGH

### Gaps to Address

- logos template string mode: May need fallback to hand-written mode switching if logos callbacks are insufficient
- Ollama provider: No official Python SDK; need to verify API compatibility with httpx
- Recursive schema codegen: `model_rebuild()` call placement needs testing with real Pydantic v2

## Sources

### Primary (HIGH confidence)
- EAML spec documents (spec/grammar.ebnf, TYPESYSTEM.md, CAPABILITIES.md, PYTHON_BRIDGE.md, ERRORS.md)
- Layer 5 design decisions (.claude/references/eaml-layer5-design-decisions.md)

### Secondary (MEDIUM confidence)
- logos crate documentation (docs.rs/logos)
- Pydantic v2 documentation (docs.pydantic.dev)
- BAML compiler architecture (prior art)

---
*Research completed: 2026-03-15*
*Ready for roadmap: yes*
