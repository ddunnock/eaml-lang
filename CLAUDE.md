# CLAUDE.md — EAML Language Project

## Overview

EAML (Engineering AI Markup Language) is a compiled DSL for LLM integrations. The compiler is written in Rust and emits Python 3.11+ / Pydantic v2 code. The project has two main components:

1. **Rust compiler** (`crates/`) — lexer, parser, semantic analysis, codegen, CLI
2. **Python runtime** (`python/`) — `eaml-runtime` package consumed by generated code

## Project Structure

```
eaml-lang/
├── Cargo.toml                    # Rust workspace manifest
├── pyproject.toml                # Root Python project (mypy)
├── Makefile                      # Unified dev commands
├── spec/                         # AUTHORITATIVE language specifications
│   ├── grammar.ebnf              # W3C EBNF formal grammar (source of truth)
│   ├── TYPESYSTEM.md             # Type system rules (AUTHORITATIVE)
│   ├── CAPABILITIES.md           # Capability registry & subset-check rules
│   ├── PYTHON_BRIDGE.md          # python %{ }% specification
│   └── ERRORS.md                 # Error code catalog (SYN, SEM, CAP, TYP, PYB, RES)
├── .claude/references/           # AI grounding layers (see Grammar Reference Stack)
├── crates/                       # Rust compiler pipeline
│   ├── eaml-errors/              # Shared error types (zero deps)
│   ├── eaml-lexer/               # Tokenizer (logos, lasso)
│   ├── eaml-parser/              # Hand-written recursive descent parser
│   ├── eaml-semantic/            # Name resolution, type checking, capabilities
│   ├── eaml-codegen/             # Python/Pydantic code generation
│   └── eaml-cli/                 # CLI binary (eamlc)
├── python/                       # Python runtime package
│   ├── pyproject.toml            # eaml-runtime package (hatchling)
│   └── src/eaml_runtime/         # Runtime: providers, validation, telemetry
├── examples/                     # EAML example programs (01-minimal through 07)
├── docs/                         # mdBook documentation
└── tmp/                          # Scratch/planning files (not committed)
```

## Crate Dependency Graph

```
eaml-errors     ← no eaml deps
eaml-lexer      ← eaml-errors
eaml-parser     ← eaml-errors, eaml-lexer
eaml-semantic   ← eaml-errors, eaml-lexer, eaml-parser
eaml-codegen    ← eaml-errors, eaml-parser, eaml-semantic
eaml-cli        ← all crates
```

Each crate is independently testable. Boundaries are strict — the lexer knows nothing about the parser, the parser knows nothing about semantic analysis.

## Build & Development Commands

```bash
make build       # cargo build --workspace
make test        # cargo test --workspace && cd python && uv run pytest
make check       # cargo check + clippy -D warnings + ruff check + mypy
make fmt         # cargo fmt --all + ruff format
make review      # cargo insta review (snapshot tests)
make run FILE=x  # cargo run -p eaml-cli -- compile x
```

**Always run `make check` before committing.** Clippy warnings are errors (`-D warnings`).

## Workflow: Get-Shit-Done (GSD)

This project uses **GSD (Get-Shit-Done-CC)** for all Rust and Python work. Use GSD slash commands for:

- `/gsd:new-project` — Initialize new milestones
- `/gsd:plan-phase` — Create phase plans (creates PLAN.md with verification)
- `/gsd:execute-phase` — Execute plans with atomic commits
- `/gsd:verify-work` — Validate features through UAT
- `/gsd:progress` — Check project status and route to next action
- `/gsd:debug` — Systematic debugging with persistent state

See `/gsd:help` for the full command list.

## Test-Driven Development (TDD)

**All code in this project follows TDD.** Use the `/superpowers:test-driven-development` skill before writing implementation code.

### Rust Testing
- **Unit tests**: Co-located in each crate's `tests/` directory
- **Snapshot tests**: Use `insta` crate for AST and codegen golden tests
- Run `cargo insta review` after snapshot changes
- All tests must pass before committing: `cargo test --workspace`

### Python Testing
- **Framework**: pytest with pytest-asyncio
- **Location**: `python/tests/`
- Run: `cd python && uv run pytest`
- Type checking: `cd python && uv run mypy src/`
- Linting: `cd python && uv run ruff check .`

### TDD Cycle
1. Write a failing test that captures the expected behavior
2. Implement the minimum code to make the test pass
3. Refactor while keeping tests green
4. Run `make check` to verify everything

## Specification Authority Hierarchy

When specifications conflict, this is the resolution order (highest authority first):

1. **Layer 5** (`eaml-layer5-design-decisions.md`) — Authoritative design decisions. All entries marked `[CLOSED]` are final for v0.1.
2. **`spec/grammar.ebnf`** — Formal grammar. Must align with Layer 5.
3. **`spec/TYPESYSTEM.md`** — Type system rules. Implements Layer 5 decisions.
4. **`spec/ERRORS.md`**, **`spec/CAPABILITIES.md`**, **`spec/PYTHON_BRIDGE.md`** — Domain specs.
5. **Layers 1-4** (`.claude/references/`) — Reference material, patterns, theory.

**Layer 5 always wins** when it conflicts with any other layer.

## Grammar Reference Stack

For grammar work, load these documents **in order**:

```
1. .claude/references/eaml-layer1-notation-reference.md   (W3C EBNF operators)
2. .claude/references/eaml-layer2-grammar-patterns.md     (XPath, SPARQL patterns)
3. .claude/references/eaml-layer3-prior-art.md            (Lox, BAML patterns)
4. .claude/references/eaml-layer4-compiler-theory.md      (FIRST/FOLLOW, Pratt, LL(1))
5. .claude/references/eaml-layer5-design-decisions.md     (AUTHORITATIVE — load last)
```

Layer 5 decisions marked `[CLOSED]` are not subject to re-evaluation.

## Key Language Design Facts

- **File extension**: `.eaml`
- **Compiler binary**: `eamlc`
- **Semicolons**: Optional everywhere
- **Comments**: `//`, `/* */`, `///` (reserved for post-MVP)
- **Type system**: Nominal typing, lowercase primitives (`string`, `float`, `int`, `bool`, `null`)
- **Template strings**: `{expr}` interpolation, `{{`/`}}` for literal braces
- **Python bridge**: `python %{ ... }%` (lex/yacc-style delimiters)
- **Capabilities**: Open identifiers validated by semantic analysis (CAP010 = fatal)
- **Target output**: Python 3.11+, Pydantic v2 only
- **Providers**: `"anthropic"`, `"openai"`, `"ollama"` (provider-agnostic)

## Error Code Categories

| Prefix  | Domain                                     |
|---------|--------------------------------------------|
| SYN     | Syntax errors (lexer + parser)             |
| SEM     | Semantic analysis (name resolution, types) |
| CAP     | Capability system errors                   |
| TYP     | Type system errors                         |
| PYB     | Python bridge errors                       |
| RES     | Name resolution errors                     |

## Rust Skills — When to Use
Use these installed skills during Rust development:

| Skill                 | When to Use                                           |
|-----------------------|-------------------------------------------------------|
| `rust-skills`         | General Rust coding guidelines (179 rules)            |
| `rust-best-practices` | Writing idiomatic Rust, code review                   |
| `m01-ownership`       | Ownership/borrow/lifetime errors (E0382, E0597, etc.) |
| `m02-resource`        | Smart pointers: Box, Rc, Arc, RefCell, RAII, Drop     |
| `m03-mutability`      | Mutability issues (E0596, E0499, interior mutability) |
| `m04-zero-cost`       | Generics, traits, static/dynamic dispatch             |
| `m05-type-driven`     | Type-state patterns, PhantomData, newtype, builder    |
| `m06-error-handling`  | Result, Option, thiserror, anyhow, error propagation  |
| `m07-concurrency`     | Async/await, Tokio, Send/Sync, channels, Mutex        |
| `m09-domain`          | Domain modeling, DDD, aggregates, validation          |
| `m10-performance`     | Benchmarking, profiling, allocation, optimization     |
| `m11-ecosystem`       | Crate selection, Cargo, feature flags, workspace      |
| `m12-lifecycle`       | RAII, Drop, resource cleanup, connection pools        |
| `m13-domain-error`    | Domain error hierarchies, recovery strategies         |
| `m14-mental-model`    | Understanding Rust concepts, mental models            |
| `m15-anti-pattern`    | Code smells, common Rust mistakes                     |
| `unsafe-checker`      | Unsafe code review, FFI, raw pointers                 |
| `coding-guidelines`   | Naming, formatting, clippy, rustfmt                   |
| `rust-learner`        | Rust version info, crate lookup                       |
| `domain-cli`          | CLI tool patterns (clap, terminal, TUI)               |

## Python Skills — When to Use

| Skill                      | When to Use                                        |
|----------------------------|----------------------------------------------------|
| `python-code-style`        | Style, linting, formatting, naming, docstrings     |
| `python-design-patterns`   | Architecture decisions, KISS, SRP, composition     |
| `python-anti-patterns`     | Code review checklist, known bad practices         |
| `python-error-handling`    | Validation, exception hierarchies, partial failure |
| `python-project-structure` | Module organization, `__all__`, public API design  |
| `async-python-patterns`    | asyncio, concurrent programming, I/O-bound apps    |

## General Skills — When to Use

| Skill                                        | When to Use                                |
|----------------------------------------------|--------------------------------------------|
| `tdd-guide`                                  | TDD methodology, test generation, coverage |
| `superpowers:test-driven-development`        | Before writing any implementation code     |
| `superpowers:systematic-debugging`           | When encountering bugs or test failures    |
| `superpowers:brainstorming`                  | Before any creative/design work            |
| `superpowers:writing-plans`                  | Before multi-step implementation tasks     |
| `superpowers:verification-before-completion` | Before claiming work is done               |

## Commit Guidelines

- Do NOT add "Generated with Claude Code" or "Co-Authored-By: Anthropic" to commit messages
- Run `make check` before every commit
- Keep commits atomic — one logical change per commit
- Use conventional commit prefixes: `feat`, `fix`, `refactor`, `test`, `docs`, `chore`

## Python Runtime Details

- **Package**: `eaml-runtime` (in `python/`)
- **Build**: hatchling
- **Package manager**: uv
- **Min Python**: 3.11
- **Type checking**: mypy (strict mode)
- **Linting**: ruff (line-length 100)
- **Testing**: pytest + pytest-asyncio
- **Dependencies**: anthropic, openai, pydantic v2, httpx

## Workspace Dependencies (Rust)

All shared deps pinned in root `Cargo.toml`:
- `logos` 0.14 — Lexer generator
- `lasso` 0.7 — String interning
- `insta` 1 — Snapshot testing
- `codespan-reporting` 0.11 — Error display
- `clap` 4 (derive) — CLI argument parsing
- `serde` / `serde_json` 1 — Serialization
- `thiserror` 1 — Error derive macros
- `tower-lsp` 0.20 — LSP server (Phase 7)

Reference with `{ workspace = true }` in per-crate Cargo.toml files.
