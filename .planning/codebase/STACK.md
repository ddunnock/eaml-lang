# Technology Stack

**Analysis Date:** 2026-03-15

## Languages

**Primary:**
- Rust 1.75+ - Compiler implementation (lexer, parser, semantic analysis, codegen)
- Python 3.11+ - Runtime support library and generated code target

**Secondary:**
- EBNF (W3C formal grammar) - Language specification in `spec/grammar.ebnf`

## Runtime

**Environment:**
- Rust toolchain 1.75+ (Edition 2021)
- Python 3.11+ for runtime support (`eaml-runtime`)
- Python 3.12+ for root development (mypy strict mode)

**Package Manager:**
- Cargo - Rust workspace manager
- uv - Python package manager (root and `eaml-runtime`)

## Frameworks

**Core Compiler:**
- logos 0.14 - Lexer generator (`eaml-lexer`)
- lasso 0.7 - String interning for token values

**Code Generation:**
- Pydantic v2 - Generated code uses Pydantic models for schema validation (`eaml_runtime`)
- Jinja2 (implicit in codegen) - Template rendering for Python output

**Testing:**
- insta 1 - Snapshot testing for AST and codegen (Rust)
- pytest - Python runtime testing
- pytest-asyncio - Async test support for Python

**Build/Dev:**
- clap 4 (derive) - CLI argument parsing (`eaml-cli`)
- codespan-reporting 0.11 - Error diagnostic display

## Key Dependencies

**Critical:**
- anthropic 0.43+ - Anthropic Claude provider SDK
- openai 1.0+ - OpenAI provider SDK
- pydantic 2.0+ - Runtime schema validation in generated code
- httpx 0.25+ - HTTP client for Ollama provider support
- thiserror 1 - Error type derive macros
- serde/serde_json 1 - Serialization for AST and code output
- tower-lsp 0.20 - LSP server framework (Phase 7, future)

**Infrastructure:**
- No external databases, caches, or cloud services required for compilation
- Compiler is standalone; runtime integrations happen in generated Python code

## Configuration

**Environment:**
- Rust: `CARGO_*` environment variables (standard Cargo behavior)
- Python: `uv` respects standard pip environment variables
- No `.env` file required for compilation — only for generated code execution

**Build:**
- `Cargo.toml` (root) - Workspace manifest with shared version 0.1.0
- `pyproject.toml` (root) - Mypy configuration
- `python/pyproject.toml` - Runtime package (hatchling, line-length 100)
- `Makefile` - Unified dev commands

**Key Build Targets:**
- `eaml-cli` binary (`eaml-cli` crate) - Main compiler entrypoint
- `eaml-runtime` package - Published Python package for generated code

## Platform Requirements

**Development:**
- macOS, Linux, or Windows with Rust 1.75+ and Python 3.11+
- Git for version control
- Cargo for Rust builds
- uv for Python dependency management

**Production:**
- Generated Python code runs on Python 3.11+
- Provider SDKs (anthropic, openai) installed in target environment
- Pydantic v2 required for runtime schema validation
- No additional runtime infrastructure required

## Workspace Structure

The Rust workspace contains 6 semi-independent crates:

```
eaml-errors     ← Zero deps (error types + codespan-reporting)
eaml-lexer      ← logos tokenizer
eaml-parser     ← Hand-written recursive descent parser
eaml-semantic   ← Name resolution, type checking, capability validation
eaml-codegen    ← Python/Pydantic code generation
eaml-cli        ← CLI binary (entry point)
```

Each crate has its own `Cargo.toml` using `{ workspace = true }` to reference shared dependencies.

---

*Stack analysis: 2026-03-15*
