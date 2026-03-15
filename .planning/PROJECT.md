# EAML Compiler

## What This Is

A complete compiler for EAML (Engineering AI Markup Language), a declarative DSL for LLM integrations. The compiler reads `.eaml` source files and emits Python 3.11+ / Pydantic v2 code that calls LLM providers (Anthropic, OpenAI, Ollama). The project includes the Rust compiler pipeline, a Python runtime library consumed by generated code, and a CLI binary (`eamlc`).

## Core Value

The compiler must correctly translate all v0.1 EAML constructs (model, schema, prompt, tool, agent) into runnable Python that type-checks, imports cleanly, and actually calls LLM APIs when executed.

## Requirements

### Validated

<!-- Shipped and confirmed valuable. -->

- ✓ Formal grammar specification (spec/grammar.ebnf) — 84 productions, W3C EBNF
- ✓ Type system specification (spec/TYPESYSTEM.md) — nominal typing, bounded types, literal unions
- ✓ Capability system specification (spec/CAPABILITIES.md) — subset checking, registry
- ✓ Python bridge specification (spec/PYTHON_BRIDGE.md) — `python %{ }%` blocks
- ✓ Error code catalog (spec/ERRORS.md) — 38 compiler error codes across SYN/SEM/CAP/TYP/PYB/RES
- ✓ Example programs (examples/01-07) — minimal, sentiment, bad_model, types
- ✓ Rust workspace scaffolding — 6 crates with dependency graph
- ✓ Python runtime scaffolding — eaml-runtime package structure
- ✓ Layer 1-5 AI reference documents — notation, patterns, prior art, theory, design decisions

### Active

<!-- Current scope. Building toward these. -->

- [ ] Lexer tokenizes all EAML constructs including template strings and python blocks
- [ ] Parser produces correct AST for all 84 grammar productions
- [ ] Semantic analyzer performs name resolution, type checking, and capability checking
- [ ] Codegen emits valid Python/Pydantic code for model, schema, prompt, tool, and agent
- [ ] Python runtime supports validate_or_retry, tool dispatch, and telemetry hooks
- [ ] Provider adapters work for Anthropic, OpenAI, and Ollama
- [ ] CLI binary (eamlc) supports compile, check, and run commands
- [ ] All 7 example programs compile to valid Python
- [ ] Compiled Python actually runs and calls LLM APIs
- [ ] All error codes from spec/ERRORS.md are emitted correctly

### Out of Scope

<!-- Explicit boundaries. Includes reasoning to prevent re-adding. -->

- LSP server (tower-lsp) — deferred to post-v0.1, needs stable compiler first
- VS Code extension — depends on LSP server
- Import/module system — spec marks as post-MVP
- Schema inheritance — post-MVP per TYPESYSTEM.md
- Enum types — post-MVP per TYPESYSTEM.md
- Pipeline operators — post-MVP per grammar.ebnf
- Async python bridge blocks — closed as unsupported in v0.1 (OQ-01)
- `unsafe` Rust code — zero unsafe blocks in the compiler

## Context

- Phase 1 (Specification) is complete — all spec documents are finalized with Layer 5 design decisions closed
- All 6 Rust crates exist as stubs (~6 lines each) with correct dependency wiring
- Python runtime has stub `__init__.py` files only
- The formal grammar (grammar.ebnf) is the source of truth for parser implementation
- EAML uses logos for lexing, lasso for string interning, insta for snapshot testing
- The compiler follows strict crate boundaries: lexer → parser → semantic → codegen → cli
- Target output is Python 3.11+ with Pydantic v2 models
- Three LLM providers: Anthropic (Claude), OpenAI (GPT), Ollama (local)

## Constraints

- **Rust version**: 1.75+ (Edition 2021) — pinned in workspace manifest
- **Python version**: 3.11+ for runtime, 3.12+ for development tooling
- **No unsafe code**: Zero `unsafe` blocks in the compiler codebase
- **Spec compliance**: Grammar, type system, and error codes must match spec documents exactly
- **TDD**: All code follows test-driven development — tests before implementation
- **Dependencies**: All shared deps pinned in root Cargo.toml with `workspace = true`

## Key Decisions

<!-- Decisions that constrain future work. Add throughout project lifecycle. -->

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Logos for lexer | Workspace dep already configured, derive-based tokenizer reduces boilerplate | — Pending |
| Hand-written recursive descent parser | LL(1) grammar with one LL(2) point, no parser generator needed | — Pending |
| Insta snapshot tests for AST/codegen | Golden-file testing catches regressions in output format | — Pending |
| Pydantic v2 only (no v1) | Simplifies codegen, v2 is current standard | — Pending |
| Three-pass semantic analysis | Name resolution → type checking → capability checking, matches spec | — Pending |
| No unsafe code | Compiler correctness is paramount, no performance-critical paths need unsafe | — Pending |

---
*Last updated: 2026-03-15 after initialization*