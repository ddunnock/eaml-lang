# Research Summary: EAML Compiler

**Domain:** Rust compiler for LLM integration DSL targeting Python/Pydantic
**Researched:** 2026-03-15
**Overall confidence:** HIGH

## Executive Summary

The EAML compiler architecture is a well-defined multi-stage pipeline (lexer -> parser -> semantic -> codegen -> cli) with strict crate boundaries already established in the workspace. The research confirms that the chosen approach -- logos for lexing, hand-written recursive descent parsing, multi-pass semantic analysis, and builder-pattern code generation -- aligns with best practices used by production Rust compilers including ruff, rust-analyzer, and oxc.

The stack is fully specified and pinned in Cargo.toml / pyproject.toml. No new dependencies are needed. The key architectural decision is to use typed enums (not trait objects) for the AST, NodeId-based side tables (not mutable AST) for semantic annotations, and a hand-written CodeWriter (not genco or templates) for Python code generation. genco was evaluated and rejected because it requires Rust 1.88+ while EAML targets Rust 1.75+.

The Python runtime (eaml-runtime) should follow a provider-adapter pattern where generated code calls eaml_runtime.call_prompt() rather than provider SDKs directly. This provides a stable API surface, enables provider-agnostic code generation, and centralizes retry/validation logic.

The primary risks are in lexer mode switching (template strings require brace-depth counting outside logos's stateless derive system), forward reference resolution (requires two-sub-pass name resolution), and Python indentation correctness in codegen (requires a structured CodeWriter). All risks have known mitigations documented in PITFALLS.md.

## Key Findings

**Stack:** All dependencies already pinned. No new crates needed. genco rejected (Rust 1.88+ requirement).
**Architecture:** Classic compiler pipeline with enum-based AST, NodeId side tables, three-pass semantic analysis, CodeWriter-based codegen.
**Critical pitfall:** Lexer mode switching for template strings -- logos is stateless, so brace-depth counting must be implemented as a wrapper layer.

## Implications for Roadmap

Based on research, suggested phase structure:

1. **Foundation (eaml-errors)** - Define Diagnostic, Span, Severity, ErrorCode types
   - Addresses: Error display infrastructure
   - Avoids: Downstream crates blocked by missing error types

2. **Lexer (eaml-lexer)** - Token enum, logos derive, string interning, mode switching
   - Addresses: Tokenization of all EAML constructs
   - Avoids: Template string mode switching pitfall (test early)

3. **Parser (eaml-parser)** - AST types, recursive descent, error recovery
   - Addresses: AST construction for all 84 productions
   - Avoids: Mutable AST coupling pitfall (use NodeId from start)

4. **Semantic Analysis (eaml-semantic)** - Name resolution, type checking, capability checking
   - Addresses: Core value proposition (compile-time validation)
   - Avoids: Forward reference pitfall (two-sub-pass name resolution)

5. **Code Generation (eaml-codegen)** - CodeWriter, Python/Pydantic emission
   - Addresses: Generating runnable Python code
   - Avoids: Indentation bugs (structured CodeWriter)

6. **CLI (eaml-cli)** - Pipeline orchestration, compile/check commands
   - Addresses: User-facing tool
   - Avoids: N/A (simple orchestration)

7. **Python Runtime (eaml-runtime)** - Provider adapters, validation, retry
   - Addresses: Making generated code actually run
   - Avoids: Provider API drift (adapter abstraction)
   - Note: Can be developed in parallel with phases 4-5

**Phase ordering rationale:**
- Strict dependency chain: each crate depends on previous crates
- errors must come first (all crates depend on it)
- lexer before parser (parser consumes tokens)
- parser before semantic (semantic walks AST)
- semantic before codegen (codegen needs type info)
- Runtime is parallel -- its API contract is defined by codegen output shape

**Research flags for phases:**
- Phase 2 (Lexer): Template string mode switching needs careful design and extensive testing
- Phase 3 (Parser): Error recovery strategy should follow matklad's synchronization point pattern
- Phase 4 (Semantic): Forward references require two-sub-pass name resolution
- Phase 5 (Codegen): CodeWriter design is straightforward but Python indentation must be verified with snapshot tests
- Phase 7 (Runtime): Standard patterns, unlikely to need deeper research

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | All dependencies already pinned and validated in workspace |
| Features | HIGH | Derived directly from authoritative spec documents |
| Architecture | HIGH | Follows established patterns from rustc, ruff, oxc, rust-analyzer |
| Pitfalls | HIGH | Most pitfalls derived from spec (EG-02, bool subclass) or well-known compiler engineering issues |

## Gaps to Address

- **Lexer mode switching implementation**: The exact boundary between logos-derived tokens and manual mode switching (for template strings and python blocks) needs design work during Phase 2. This is an implementation design question, not a research gap.
- **Runtime API contract finalization**: The exact signature of eaml_runtime.call_prompt() and eaml_runtime.call_tool() should be designed when codegen and runtime phases begin. The shape in ARCHITECTURE.md is a starting point.
- **Integration testing strategy**: How to test the full pipeline (compile + run + call LLM) needs a plan. Mock providers? Test against real APIs? This is a testing strategy question for phase planning.
