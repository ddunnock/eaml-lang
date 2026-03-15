# CONVENTIONS.md — EAML Coding Conventions

## 1. Code Style Configuration

### Rust
- **Edition**: 2021 (workspace-level)
- **MSRV**: 1.75
- **Formatter**: `cargo fmt --all` (default rustfmt settings, no `.rustfmt.toml`)
- **Linter**: `cargo clippy --workspace -- -D warnings` (warnings are errors)
- **No explicit `clippy.toml`** — uses default clippy rules

### Python
- **Min version**: 3.11 (runtime), 3.12 (root project)
- **Formatter/Linter**: ruff
  - `target-version = "py311"`
  - `line-length = 100`
- **Type checker**: mypy (strict mode)
  - `strict = true`
  - `warn_return_any = true`
  - `warn_unused_configs = true`
- **Package manager**: uv (not pip/poetry)

## 2. Naming Patterns

### Rust
| Element | Convention | Example |
|---------|-----------|---------|
| Crate names | kebab-case | `eaml-lexer`, `eaml-codegen` |
| Types/Structs | PascalCase | `TokenStream`, `AnalyzedProgram` |
| Enum variants | PascalCase | `TokenKind::Identifier` |
| Functions/methods | snake_case | `lex()`, `parse()`, `analyze()` |
| Constants | SCREAMING_SNAKE | `MAX_RECURSION_DEPTH` |
| Modules | snake_case | (follows file names) |

### Python
| Element | Convention | Example |
|---------|-----------|---------|
| Modules | snake_case | `eaml_runtime` |
| Classes | PascalCase | (to be defined) |
| Functions | snake_case | (to be defined) |
| Private | Leading underscore | `_internal_fn` |
| Exports | Explicit `__all__` | `__all__: list[str] = []` |

### Error Codes
- Format: `PREFIX` + 3-digit number
- Prefixes: SYN (syntax), SEM (semantic), TYP (types), CAP (capability), PYB (bridge), RES (resolution)
- Ranges documented in `spec/ERRORS.md` §1.5

## 3. Error Handling

### Rust Pattern
- **Framework**: `thiserror` for custom error derives
- **Display**: `codespan-reporting` for colored terminal output with source context
- **Structure**: `Diagnostic` struct with code, message, severity, source location, hints
- **Severity levels**: FATAL (halts), ERROR (continues), WARNING (non-blocking)
- **Accumulation**: Up to 20 errors before aborting (overridable with `--max-errors N`)
- **Foundation**: `eaml-errors` crate has zero eaml dependencies — all other crates depend on it

### Python Pattern
- Type hints required (mypy strict)
- Google-style docstrings
- Explicit error propagation

## 4. Module Organization

### Rust
- Single `src/lib.rs` per crate as public API surface
- Strict crate boundaries — no cross-layer knowledge
- Workspace dependencies pinned in root `Cargo.toml`, referenced with `{ workspace = true }`
- Internal deps use path references: `{ path = "../eaml-errors" }`

### Python
- Package structure: `python/src/eaml_runtime/`
- Submodules by domain: `providers/`
- Tests separate: `python/tests/`

## 5. Documentation Style

### Rust
- Crate-level docs: `//!` comments in `lib.rs`
- Public API docs: `///` doc comments
- Implementation notes: `//` inline comments
- Example from `eaml-lexer`:
  ```rust
  //! EAML lexer — tokenizes EAML source into a token stream.
  //!
  //! Public API: `lex(source: &str) -> TokenStream`
  ```

### Python
- Module docstrings: Triple-quoted at top
- Explicit exports via `__all__`

### Specifications
- W3C EBNF for grammar (`spec/grammar.ebnf`)
- Markdown for domain specs (`spec/*.md`)
- Production numbers `[1]`–`[84]` for cross-referencing
- Error code references in grammar comments: `[sem: SEM025]`

## 6. Import Organization

### Rust (workspace Cargo.toml)
All shared dependencies centrally pinned:
```toml
[workspace.dependencies]
logos = "0.14"
lasso = "0.7"
insta = "1"
codespan-reporting = "0.11"
clap = { version = "4", features = ["derive"] }
thiserror = "1"
```

### Python (pyproject.toml)
```toml
[project]
dependencies = [
    "anthropic>=0.43",
    "openai>=1.0",
    "pydantic>=2.0",
    "httpx>=0.25",
]
```

## 7. Commit Conventions
- Conventional prefixes: `feat`, `fix`, `refactor`, `test`, `docs`, `chore`
- No "Generated with Claude Code" or "Co-Authored-By" lines
- Run `make check` before every commit
- Atomic commits — one logical change per commit