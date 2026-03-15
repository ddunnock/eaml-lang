# STRUCTURE.md — EAML Project Directory Layout

## 1. Root Directory

```
eaml-lang/
├── Cargo.toml                    # Rust workspace manifest (6 crates)
├── Cargo.lock                    # Locked dependency versions
├── pyproject.toml                # Root Python workspace (mypy config)
├── Makefile                      # Unified build commands
├── README.md                     # Project overview
├── CHANGELOG.md                  # Version history
├── CLAUDE.md                     # AI development guidelines
├── LICENSE                       # Apache-2.0
│
├── crates/                       # Rust compiler (6 crates)
│   ├── eaml-errors/              # Shared error types (zero deps)
│   ├── eaml-lexer/               # Tokenizer (logos, lasso)
│   ├── eaml-parser/              # Recursive descent parser
│   ├── eaml-semantic/            # Name resolution, type/capability checking
│   ├── eaml-codegen/             # Python code generation
│   └── eaml-cli/                 # CLI binary (eamlc)
│
├── python/                       # Python runtime package
│   ├── pyproject.toml            # eaml-runtime (hatchling)
│   ├── src/eaml_runtime/         # Runtime: providers, validation
│   │   ├── __init__.py
│   │   └── providers/
│   │       └── __init__.py
│   └── tests/
│       └── __init__.py
│
├── spec/                         # AUTHORITATIVE specifications
│   ├── grammar.ebnf              # W3C EBNF (84 productions)
│   ├── TYPESYSTEM.md             # Type system rules
│   ├── CAPABILITIES.md           # Capability registry & rules
│   ├── PYTHON_BRIDGE.md          # python %{ }% spec (22 rules)
│   └── ERRORS.md                 # Error code catalog (38+ codes)
│
├── examples/                     # EAML example programs
│   ├── 01-minimal/               # Smoke test (no capabilities)
│   ├── 02-sentiment/             # Real-world (json_mode, bounds)
│   ├── 03-python-bridge/         # Bridge block examples
│   ├── 04-multi-tool-agent/      # Agent with tools
│   ├── 05-multi-file/            # Import/module composition
│   ├── 06-capability-error/      # Negative test (CAP010)
│   └── 07-all-type-variants/     # Complete type system exercise
│
├── tests/                        # Integration test fixtures
│   ├── compile/{positive,negative}/
│   ├── fixtures/
│   └── runtime/
│
├── docs/                         # mdBook documentation (scaffolded)
│
├── .claude/references/           # 5-layer AI grounding documents
│   ├── eaml-layer1-notation-reference.md
│   ├── eaml-layer2-grammar-patterns.md
│   ├── eaml-layer3-prior-art.md
│   ├── eaml-layer4-compiler-theory.md
│   └── eaml-layer5-design-decisions.md   # AUTHORITATIVE
│
├── .github/workflows/            # CI/CD
│   ├── ci.yml                    # Rust + Python checks & tests
│   ├── release.yml               # Multi-platform binary builds
│   └── python-publish.yml        # PyPI publishing
│
└── .codegraph/                   # CodeGraph semantic index
```

## 2. Rust Crate Internal Layout

Each crate follows standard layout:
```
crates/eaml-{name}/
├── Cargo.toml              # Per-crate manifest (workspace refs)
├── src/
│   └── lib.rs              # Public API + module declarations
└── tests/
    └── .gitkeep            # Placeholder (tests TBD)
```

Exception: `eaml-codegen/tests/snapshots/` contains insta snapshot files.

## 3. Workspace Dependencies

All shared deps pinned in root `Cargo.toml`, referenced with `{ workspace = true }`:

| Dependency | Version | Purpose |
|-----------|---------|---------|
| `logos` | 0.14 | Lexer generator |
| `lasso` | 0.7 | String interning |
| `insta` | 1 | Snapshot testing |
| `codespan-reporting` | 0.11 | Error display |
| `clap` | 4 (derive) | CLI argument parsing |
| `serde` / `serde_json` | 1 | Serialization |
| `thiserror` | 1 | Error derive macros |
| `tower-lsp` | 0.20 | LSP server (Phase 7) |

## 4. Python Runtime Structure

```
python/
├── pyproject.toml          # hatchling build, min Python 3.11
├── src/eaml_runtime/
│   ├── __init__.py         # Public API (stub)
│   └── providers/          # LLM provider adapters
│       └── __init__.py     # (anthropic, openai, ollama — TBD)
└── tests/
    └── __init__.py
```

Dependencies: `anthropic>=0.43`, `openai>=1.0`, `pydantic>=2.0`, `httpx>=0.25`
Dev: `pytest`, `pytest-asyncio`, `ruff`, `mypy`

## 5. Naming Conventions

| Domain | Convention | Example |
|--------|-----------|---------|
| Crate names | kebab-case | `eaml-lexer` |
| Rust types | PascalCase | `TokenStream`, `AnalyzedProgram` |
| Rust functions | snake_case | `lex`, `parse`, `analyze` |
| Rust constants | SCREAMING_SNAKE | `MAX_RECURSION_DEPTH` |
| Python modules | snake_case | `eaml_runtime` |
| EAML keywords | lowercase | `model`, `schema`, `prompt` |
| Error codes | PREFIX + NNN | `SYN042`, `CAP010` |
| Spec files | UPPERCASE.md | `TYPESYSTEM.md`, `ERRORS.md` |
| Example dirs | NN-descriptive | `01-minimal`, `02-sentiment` |

## 6. Build Commands

```makefile
make build       # cargo build --workspace
make test        # cargo test --workspace + pytest
make check       # clippy -D warnings + ruff + mypy
make fmt         # rustfmt + ruff format
make run FILE=x  # cargo run -p eaml-cli -- compile x
make review      # cargo insta review
```

## 7. Key Files Quick Reference

| File | Purpose |
|------|---------|
| `Cargo.toml` | Workspace manifest |
| `Makefile` | Build commands |
| `spec/grammar.ebnf` | Formal grammar (84 productions) |
| `spec/ERRORS.md` | Error code catalog |
| `crates/eaml-errors/src/lib.rs` | Shared error types |
| `crates/eaml-lexer/src/lib.rs` | Tokenizer |
| `crates/eaml-parser/src/lib.rs` | AST parser |
| `crates/eaml-semantic/src/lib.rs` | Type/capability analysis |
| `crates/eaml-codegen/src/lib.rs` | Code generator |
| `crates/eaml-cli/src/main.rs` | CLI entry point |
| `.claude/references/eaml-layer5-design-decisions.md` | AUTHORITATIVE decisions |
