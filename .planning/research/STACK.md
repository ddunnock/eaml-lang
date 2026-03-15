# Technology Stack

**Project:** EAML Compiler
**Researched:** 2026-03-15

## Recommended Stack

### Core Framework (Rust Compiler)

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| Rust | 1.75+ (Edition 2021) | Compiler implementation language | Already decided. Strong type system, exhaustive matching, zero-cost abstractions |
| logos | 0.14 | Lexer generator | Already in workspace. Derive-based, extremely fast (GB/s), custom error types |
| lasso | 0.7 | String interning | Already in workspace. ThreadedRodeo for safe concurrent interning, Spur keys |
| codespan-reporting | 0.11 | Error display | Already in workspace. Colored terminal output, source snippets, annotations |
| clap | 4 (derive) | CLI argument parsing | Already in workspace. Derive API for declarative CLI definitions |
| thiserror | 1 | Error derive macros | Already in workspace. Ergonomic error type definitions |
| serde / serde_json | 1 | Serialization | Already in workspace. AST serialization for debugging and snapshot tests |
| insta | 1 | Snapshot testing | Already in workspace. Golden-file tests for AST and codegen output |

### Python Runtime

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| Python | 3.11+ | Runtime execution | Already decided. Minimum version for generated code |
| Pydantic | v2.0+ | Schema validation | Already decided. BaseModel for schema types, Field for bounded types |
| anthropic | 0.43+ | Anthropic provider | Already in pyproject.toml. Official SDK |
| openai | 1.0+ | OpenAI provider | Already in pyproject.toml. Official SDK |
| httpx | 0.25+ | Ollama provider | Already in pyproject.toml. Async HTTP for Ollama REST API |
| hatchling | latest | Build system | Already decided. Python package building |
| uv | latest | Package manager | Already decided. Fast dependency management |

### Development and Testing

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| pytest | latest | Python testing | Already in dev deps |
| pytest-asyncio | latest | Async test support | Already in dev deps. Required for testing async provider calls |
| ruff | latest | Python linting/formatting | Already in dev deps. Fast, replaces flake8 + black |
| mypy | latest | Python type checking | Already in dev deps. Strict mode enabled |

## Alternatives Considered

| Category | Recommended | Alternative | Why Not |
|----------|-------------|-------------|---------|
| Lexer | logos 0.14 | hand-written | logos is fast enough, derive-based reduces boilerplate |
| Parser | Hand-written recursive descent | pest, lalrpop, tree-sitter | LL(1) grammar is simple enough. Generators limit error recovery |
| String interning | lasso 0.7 | string-interner, manual HashMap | lasso has ThreadedRodeo, well-maintained, good API |
| Code generation | Hand-written CodeWriter | genco, tera/handlebars | genco requires Rust 1.88+. Templates add complexity |
| Error display | codespan-reporting | ariadne, miette | codespan-reporting is stable, version already pinned |
| Python HTTP | httpx | aiohttp, requests | httpx supports async natively, already in deps |

## Installation



No additional dependencies needed. Stack fully defined in Cargo.toml and python/pyproject.toml.

## Sources

- Workspace Cargo.toml (pinned versions)
- python/pyproject.toml (pinned versions)
- [Logos crate](https://crates.io/crates/logos)
- [Lasso crate](https://crates.io/crates/lasso)
- [codespan-reporting](https://crates.io/crates/codespan-reporting)
- [genco crate](https://crates.io/crates/genco) -- evaluated, rejected due to Rust 1.88 requirement
