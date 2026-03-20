# EAML Compiler

## What This Is

A complete compiler for EAML (Engineering AI Markup Language), a declarative DSL for LLM integrations. The compiler reads `.eaml` source files and emits Python 3.11+ / Pydantic v2 code that calls LLM providers (Anthropic, OpenAI, Ollama). The project includes a 6-crate Rust compiler pipeline (15,048 LOC), a Python runtime library (639 LOC), and a CLI binary (`eamlc`) with compile, check, and run commands.

## Core Value

The compiler must correctly translate all v0.1 EAML constructs (model, schema, prompt, tool, agent) into runnable Python that type-checks, imports cleanly, and actually calls LLM APIs when executed.

## Current State

**Shipped:** v1.0 (2026-03-20)
**Codebase:** 15,048 lines Rust + 639 lines Python, 6 Rust crates + 1 Python package
**Tests:** ~460 tests (all green), clippy/ruff/mypy clean
**Examples:** 7 example programs (01-minimal through 07-all-type-variants), all compiling

The v1.0 compiler covers the complete pipeline: lexing (logos + template strings + python bridge), parsing (hand-written recursive descent, 84 productions), semantic analysis (name resolution, type checking, capability subset checking), code generation (Pydantic v2 models, async prompt functions, agent classes), and a Python runtime with Anthropic/OpenAI/Ollama provider adapters, validate_or_retry, and telemetry hooks.

## Requirements

### Validated

- ✓ Formal grammar specification (spec/grammar.ebnf) — 84 productions, W3C EBNF
- ✓ Type system specification (spec/TYPESYSTEM.md) — nominal typing, bounded types, literal unions
- ✓ Capability system specification (spec/CAPABILITIES.md) — subset checking, registry
- ✓ Python bridge specification (spec/PYTHON_BRIDGE.md) — `python %{ }%` blocks
- ✓ Error code catalog (spec/ERRORS.md) — 42 compiler error codes across SYN/SEM/CAP/TYP/PYB/RES
- ✓ Example programs (examples/01-07) — v1.0
- ✓ Rust workspace scaffolding — 6 crates with dependency graph — v1.0
- ✓ Python runtime scaffolding — eaml-runtime package — v1.0
- ✓ Layer 1-5 AI reference documents — notation, patterns, prior art, theory, design decisions
- ✓ Lexer tokenizes all EAML constructs including template strings and python blocks — v1.0
- ✓ Parser produces correct AST for all 84 grammar productions — v1.0
- ✓ Semantic analyzer performs name resolution, type checking, and capability checking — v1.0
- ✓ Codegen emits valid Python/Pydantic code for model, schema, prompt, tool, and agent — v1.0
- ✓ Python runtime supports validate_or_retry, tool dispatch, and telemetry hooks — v1.0
- ✓ Provider adapters work for Anthropic, OpenAI, and Ollama — v1.0
- ✓ CLI binary (eamlc) supports compile, check, and run commands — v1.0
- ✓ All 7 example programs compile to valid Python — v1.0
- ✓ Compiled Python actually runs and calls LLM APIs — v1.0
- ✓ All error codes from spec/ERRORS.md are emitted correctly — v1.0

### Active

<!-- Next milestone requirements will go here -->

### Out of Scope

- LSP server (tower-lsp) — deferred to post-v0.1, needs stable compiler first
- VS Code extension — depends on LSP server
- Import/module system — spec marks as post-MVP
- Schema inheritance — post-MVP per TYPESYSTEM.md
- Enum types — post-MVP per TYPESYSTEM.md
- Pipeline operators — post-MVP per grammar.ebnf
- Async python bridge blocks — closed as unsupported in v0.1 (OQ-01)
- `unsafe` Rust code — zero unsafe blocks in the compiler
- Multi-file compilation — module system is post-MVP

## Context

- v1.0 shipped with full compiler pipeline operational
- Tech stack: Rust (logos, lasso, insta, clap, codespan-reporting) + Python (pydantic v2, anthropic, openai, httpx)
- Strict crate boundaries: eaml-errors → eaml-lexer → eaml-parser → eaml-semantic → eaml-codegen → eaml-cli
- Three LLM providers: Anthropic (Claude), OpenAI (GPT), Ollama (local)
- Known tech debt: live LLM API roundtrip tests are `#[ignore]` (require API keys), Phase 6 Nyquist validation incomplete

## Constraints

- **Rust version**: 1.75+ (Edition 2021) — pinned in workspace manifest
- **Python version**: 3.11+ for runtime, 3.12+ for development tooling
- **No unsafe code**: Zero `unsafe` blocks in the compiler codebase
- **Spec compliance**: Grammar, type system, and error codes must match spec documents exactly
- **TDD**: All code follows test-driven development — tests before implementation
- **Dependencies**: All shared deps pinned in root Cargo.toml with `workspace = true`

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Logos for lexer | Workspace dep already configured, derive-based tokenizer reduces boilerplate | ✓ Good — fast DFA, minimal code |
| Hand-written recursive descent parser | LL(1) grammar with one LL(2) point, no parser generator needed | ✓ Good — full control, clear error recovery |
| Insta snapshot tests for AST/codegen | Golden-file testing catches regressions in output format | ✓ Good — caught many codegen regressions |
| Pydantic v2 only (no v1) | Simplifies codegen, v2 is current standard | ✓ Good — cleaner Field() API |
| Three-pass semantic analysis | Name resolution → type checking → capability checking, matches spec | ✓ Good — clean separation, forward refs work |
| No unsafe code | Compiler correctness is paramount, no performance-critical paths need unsafe | ✓ Good — zero unsafe blocks maintained |
| All strings as template strings at lexer level | Avoids context-sensitivity in lexer | ✓ Good — simplified parser |
| Kahn's algorithm for schema toposort | Handles forward references in generated Python | ✓ Good — prevents NameError in output |
| BTreeSet for import deduplication | Deterministic sorted import output | ✓ Good — consistent codegen |
| Provider _client typed as Any | Avoids requiring SDK type stubs at import time | ✓ Good — flexible for all 3 providers |

---
*Last updated: 2026-03-20 after v1.0 milestone*
