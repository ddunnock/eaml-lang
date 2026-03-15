# TESTING.md — EAML Testing Patterns

## 1. Test Frameworks

### Rust
- **Unit tests**: `#[cfg(test)]` modules or separate files in `tests/`
- **Snapshot tests**: `insta` v1 — golden tests for AST and codegen output
- **Review workflow**: `cargo insta review` for interactive snapshot approval

### Python
- **Framework**: pytest + pytest-asyncio
- **Type checking**: mypy (strict mode)
- **Linting**: ruff (check + format)

## 2. Test File Locations

### Rust
```
crates/
├── eaml-errors/tests/.gitkeep        # Placeholder (tests TBD)
├── eaml-lexer/tests/.gitkeep         # Placeholder (tests TBD)
├── eaml-parser/tests/.gitkeep        # Placeholder (tests TBD)
├── eaml-semantic/tests/.gitkeep      # Placeholder (tests TBD)
├── eaml-codegen/tests/               # Snapshot tests
│   └── snapshots/                    # insta-managed .snap files
```

### Python
```
python/tests/
└── __init__.py                       # Empty (tests TBD)
```

### Integration Fixtures
```
tests/
├── compile/
│   ├── positive/                     # Valid EAML (expect success)
│   └── negative/                     # Invalid EAML (expect errors)
├── fixtures/                         # Test input files
└── runtime/                          # Runtime execution tests
```
Currently `.gitkeep` only — to be populated as compiler implementation proceeds.

## 3. Dev Dependencies by Crate

| Crate | Dev Dependencies |
|-------|-----------------|
| `eaml-lexer` | `insta` |
| `eaml-parser` | `insta` |
| `eaml-codegen` | `insta` |
| `eaml-semantic` | (none yet) |
| `eaml-cli` | (none yet) |

## 4. Snapshot Testing (insta)

- **Snapshot location**: `crates/eaml-codegen/tests/snapshots/`
- **File format**: `.snap` files (YAML with human-readable diffs)
- **Workflow**:
  1. Write test using `insta::assert_snapshot!()` or `assert_debug_snapshot!()`
  2. Run `cargo test` — new/changed snapshots create `.snap.new` files
  3. Run `cargo insta review` — interactive approval/rejection
  4. Commit approved snapshots

## 5. CI Pipeline

### GitHub Actions (`.github/workflows/ci.yml`)

**Job 1: Rust Check & Clippy** (runs first)
- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo fmt --all -- --check`

**Job 2: Rust Tests** (depends on Job 1)
- Matrix: `[stable, "1.75"]` (stable + MSRV)
- Command: `cargo test --workspace`
- Cache: `Swatinem/rust-cache@v2`

**Job 3: Python Lint & Type Check** (parallel with Rust)
- Working dir: `python/`
- `uv run ruff check .`
- `uv run ruff format --check .`
- `uv run mypy src/`

**Job 4: Python Tests** (depends on Job 3)
- Matrix: `[3.11, 3.12, 3.13]`
- Command: `uv run pytest`
- Cache: uv built-in via `setup-uv@v4`

### Release Workflows
- **release.yml**: Multi-platform binaries (Linux x86_64/aarch64, macOS, Windows)
- **python-publish.yml**: TestPyPI/PyPI via trusted publishing

## 6. Makefile Commands

```makefile
make test        # cargo test --workspace + cd python && uv run pytest
make check       # clippy + ruff + mypy
make fmt         # rustfmt + ruff format
make review      # cargo insta review
```

**Note**: Python test exit code 5 (no tests found) is tolerated: `|| test $$? -eq 5`

## 7. Current Test Coverage Status

| Area | Status | Notes |
|------|--------|-------|
| Lexer unit tests | Placeholder | `.gitkeep` only |
| Parser unit tests | Placeholder | `.gitkeep` only |
| Semantic unit tests | Placeholder | `.gitkeep` only |
| Codegen snapshots | Scaffolded | `snapshots/` dir exists |
| CLI integration tests | None | TBD |
| Python runtime tests | Empty | `__init__.py` only |
| Integration fixtures | Scaffolded | `positive/`/`negative/` dirs exist |

## 8. TDD Workflow (Project Standard)

Per CLAUDE.md, all code follows TDD:
1. Write a failing test capturing expected behavior
2. Implement minimum code to pass
3. Refactor while keeping tests green
4. Run `make check` to verify everything