     [1mSTDIN[0m
[38;5;247m   1[0m [38;5;254m# Technology Stack[0m
[38;5;247m   2[0m 
[38;5;247m   3[0m [38;5;254m**Project:** EAML Compiler[0m
[38;5;247m   4[0m [38;5;254m**Researched:** 2026-03-15[0m
[38;5;247m   5[0m 
[38;5;247m   6[0m [38;5;254m## Recommended Stack[0m
[38;5;247m   7[0m 
[38;5;247m   8[0m [38;5;254m### Core Framework (Rust Compiler)[0m
[38;5;247m   9[0m 
[38;5;247m  10[0m [38;5;254m| Technology | Version | Purpose | Why |[0m
[38;5;247m  11[0m [38;5;254m|------------|---------|---------|-----|[0m
[38;5;247m  12[0m [38;5;254m| Rust | 1.75+ (Edition 2021) | Compiler implementation language | Already decided. Strong type system, exhaustive matching, zero-cost abstractions for compiler work |[0m
[38;5;247m  13[0m [38;5;254m| logos | 0.14 | Lexer generator | Already in workspace. Derive-based, extremely fast (GB/s), custom error types |[0m
[38;5;247m  14[0m [38;5;254m| lasso | 0.7 | String interning | Already in workspace. ThreadedRodeo for safe concurrent interning, Spur keys |[0m
[38;5;247m  15[0m [38;5;254m| codespan-reporting | 0.11 | Error display | Already in workspace. Colored terminal output, source snippets, annotations |[0m
[38;5;247m  16[0m [38;5;254m| clap | 4 (derive) | CLI argument parsing | Already in workspace. Derive API for declarative CLI definitions |[0m
[38;5;247m  17[0m [38;5;254m| thiserror | 1 | Error derive macros | Already in workspace. Ergonomic error type definitions |[0m
[38;5;247m  18[0m [38;5;254m| serde / serde_json | 1 | Serialization | Already in workspace. AST serialization for debugging and snapshot tests |[0m
[38;5;247m  19[0m [38;5;254m| insta | 1 | Snapshot testing | Already in workspace. Golden-file tests for AST and codegen output |[0m
[38;5;247m  20[0m 
[38;5;247m  21[0m [38;5;254m### Python Runtime[0m
[38;5;247m  22[0m 
[38;5;247m  23[0m [38;5;254m| Technology | Version | Purpose | Why |[0m
[38;5;247m  24[0m [38;5;254m|------------|---------|---------|-----|[0m
[38;5;247m  25[0m [38;5;254m| Python | 3.11+ | Runtime execution | Already decided. Minimum version for generated code |[0m
[38;5;247m  26[0m [38;5;254m| Pydantic | v2.0+ | Schema validation | Already decided. BaseModel for schema types, Field for bounded types |[0m
[38;5;247m  27[0m [38;5;254m| anthropic | 0.43+ | Anthropic provider | Already in pyproject.toml. Official SDK |[0m
[38;5;247m  28[0m [38;5;254m| openai | 1.0+ | OpenAI provider | Already in pyproject.toml. Official SDK |[0m
[38;5;247m  29[0m [38;5;254m| httpx | 0.25+ | Ollama provider | Already in pyproject.toml. Async HTTP for Ollama REST API |[0m
[38;5;247m  30[0m [38;5;254m| hatchling | latest | Build system | Already decided. Python package building |[0m
[38;5;247m  31[0m [38;5;254m| uv | latest | Package manager | Already decided. Fast dependency management |[0m
[38;5;247m  32[0m 
[38;5;247m  33[0m [38;5;254m### Development & Testing[0m
[38;5;247m  34[0m 
[38;5;247m  35[0m [38;5;254m| Technology | Version | Purpose | Why |[0m
[38;5;247m  36[0m [38;5;254m|------------|---------|---------|-----|[0m
[38;5;247m  37[0m [38;5;254m| pytest | latest | Python testing | Already in dev deps |[0m
[38;5;247m  38[0m [38;5;254m| pytest-asyncio | latest | Async test support | Already in dev deps. Required for testing async provider calls |[0m
[38;5;247m  39[0m [38;5;254m| ruff | latest | Python linting/formatting | Already in dev deps. Fast, replaces flake8 + black |[0m
[38;5;247m  40[0m [38;5;254m| mypy | latest | Python type checking | Already in dev deps. Strict mode enabled |[0m
[38;5;247m  41[0m 
[38;5;247m  42[0m [38;5;254m## Alternatives Considered[0m
[38;5;247m  43[0m 
[38;5;247m  44[0m [38;5;254m| Category | Recommended | Alternative | Why Not |[0m
[38;5;247m  45[0m [38;5;254m|----------|-------------|-------------|---------|[0m
[38;5;247m  46[0m [38;5;254m| Lexer | logos 0.14 | hand-written | logos is fast enough, derive-based reduces boilerplate. Hand-written only needed for exotic lexer modes |[0m
[38;5;247m  47[0m [38;5;254m| Parser | Hand-written recursive descent | pest, lalrpop, tree-sitter | LL(1) grammar with one LL(2) point is simple enough. Parser generators add complexity and limit error recovery |[0m
[38;5;247m  48[0m [38;5;254m| String interning | lasso 0.7 | string-interner, manual HashMap | lasso has ThreadedRodeo, well-maintained, good API |[0m
[38;5;247m  49[0m [38;5;254m| Code generation | Hand-written CodeWriter | genco, tera/handlebars | genco requires Rust 1.88+. Template engines add complexity for single-target codegen |[0m
[38;5;247m  50[0m [38;5;254m| Error display | codespan-reporting | ariadne, miette | codespan-reporting is stable, well-documented, version already pinned |[0m
[38;5;247m  51[0m [38;5;254m| Python HTTP | httpx | aiohttp, requests | httpx supports async natively, already in deps, used by anthropic/openai SDKs |[0m
[38;5;247m  52[0m 
[38;5;247m  53[0m [38;5;254m## Installation[0m
[38;5;247m  54[0m 
[38;5;247m  55[0m [38;5;254m```bash[0m
[38;5;247m  56[0m [38;5;254m# Rust workspace (already configured)[0m
[38;5;247m  57[0m [38;5;254mcargo build --workspace[0m
[38;5;247m  58[0m 
[38;5;247m  59[0m [38;5;254m# Python runtime[0m
[38;5;247m  60[0m [38;5;254mcd python && uv sync[0m
[38;5;247m  61[0m [38;5;254m```[0m
[38;5;247m  62[0m 
[38;5;247m  63[0m [38;5;254mNo additional dependencies are needed. The stack is fully defined in `Cargo.toml` and `python/pyproject.toml`.[0m
[38;5;247m  64[0m 
[38;5;247m  65[0m [38;5;254m## Sources[0m
[38;5;247m  66[0m 
[38;5;247m  67[0m [38;5;254m- Workspace Cargo.toml (pinned versions)[0m
[38;5;247m  68[0m [38;5;254m- python/pyproject.toml (pinned versions)[0m
[38;5;247m  69[0m [38;5;254m- [Logos crate](https://crates.io/crates/logos)[0m
[38;5;247m  70[0m [38;5;254m- [Lasso crate](https://crates.io/crates/lasso)[0m
[38;5;247m  71[0m [38;5;254m- [codespan-reporting](https://crates.io/crates/codespan-reporting)[0m
[38;5;247m  72[0m [38;5;254m- [genco crate](https://crates.io/crates/genco) -- evaluated, rejected due to Rust 1.88 requirement[0m
