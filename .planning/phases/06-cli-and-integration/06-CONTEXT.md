# Phase 6: CLI and Integration - Context

**Gathered:** 2026-03-17
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can compile and validate EAML files from the command line, and all example programs work end-to-end. Covers the `eamlc` binary with compile/check/run/version commands, error display, exit codes, and integration tests proving the full pipeline (lex -> parse -> semantic -> codegen -> runtime execution). Multi-file compilation and watch mode are out of scope.

</domain>

<decisions>
## Implementation Decisions

### CLI command structure
- Four commands: `compile`, `check`, `run`, `--version`
- `eamlc compile <file>` — compile .eaml to .py, print success message
- `eamlc check <file>` — validate .eaml without generating output
- `eamlc run <file>` — compile to .py then execute via `python <file.py>` (shell out with `std::process::Command`)
- `--version` via clap's built-in version flag
- Single file per invocation (no glob/multi-file)
- No verbosity flags for v0.1 — default shows errors/warnings + success message

### Output file placement
- Default: generated .py file placed in same directory as source .eaml file
- `-o / --output <dir>` flag to specify output directory (directory only, not file path)
- Filename derived from input: `sentiment.eaml` -> `sentiment.py`
- `eamlc run` keeps the generated .py file after execution (not temp)

### Exit codes
- Exit 0: success
- Exit 1: compilation error(s) (syntax, semantic, type, capability)
- Exit 2: file not found / IO error
- Exit 3: runtime error (`eamlc run` — Python execution failure)

### Error display
- Rustc-style summary: `error: aborting due to N previous errors` (with warnings count if any)
- Warnings always shown alongside errors (never suppressed)
- Color: auto-detect TTY + respect `NO_COLOR` env var per no-color.org convention
- codespan-reporting already handles the diagnostic rendering — CLI just orchestrates

### Integration testing strategy
- **Both levels**: library-level tests (fast, call pipeline functions directly) + CLI integration tests (invoke eamlc binary)
- CLI integration tests use temp directories for generated output (examples/ stays clean)
- All 7 examples (01-minimal through 07-all-type-variants) tested via CLI integration tests
- GEN-11 (mypy): CLI test compiles each example, runs `mypy <output.py> --ignore-missing-imports`, asserts exit 0
- GEN-12 / INT-02 (real LLM): `#[ignore]` test that compiles sentiment.eaml, runs it, checks JSON output has expected keys. Only runs when API key is set.

### Claude's Discretion
- Exact clap derive struct design and argument validation
- How `eamlc run` discovers the Python interpreter (PATH lookup strategy)
- Error recovery behavior when `python` is not found
- assert_cmd vs raw Command for integration tests
- Whether to add `codespan-reporting` as a direct dep or re-export from eaml-errors
- Exact format of the success message ("Compiled X -> Y" vs "Wrote Y")

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements and specifications
- `spec/ERRORS.md` -- All 38 compiler diagnostic codes; CLI must render these via codespan-reporting
- `.planning/REQUIREMENTS.md` -- CLI-01 through CLI-04, INT-01 through INT-03, GEN-11, GEN-12

### CLI crate (implementation target)
- `crates/eaml-cli/src/main.rs` -- Current stub (placeholder only)
- `crates/eaml-cli/Cargo.toml` -- Dependencies: all pipeline crates + clap

### Pipeline API (what CLI orchestrates)
- `crates/eaml-codegen/src/lib.rs` -- `generate()` entry point: takes ParseOutput + AnalysisOutput + source + filename, returns String
- `crates/eaml-parser/src/lib.rs` -- Parser entry point returning ParseOutput
- `crates/eaml-semantic/src/lib.rs` -- Semantic analysis entry point returning AnalysisOutput
- `crates/eaml-errors/src/render.rs` -- codespan-reporting rendering for diagnostics
- `crates/eaml-errors/src/diagnostic.rs` -- Diagnostic and DiagnosticCollector types

### Prior phase context
- `.planning/phases/04-code-generation/04-CONTEXT.md` -- Output file structure, naming conventions, codegen API
- `.planning/phases/05-python-runtime/05-CONTEXT.md` -- Runtime API contract, provider setup, error hierarchy

### Example programs (test fixtures)
- `examples/01-minimal/minimal.eaml` through `examples/07-all-type-variants/types.eaml` -- All 7 examples for INT-01
- `examples/02-sentiment/sentiment.eaml` -- Primary end-to-end test target for INT-02
- `examples/06-capability-error/` -- Expected to trigger CAP010 for INT-03

### Design decisions
- `.claude/references/eaml-layer5-design-decisions.md` -- Authoritative design decisions (all [CLOSED] entries are final)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `eaml-errors` crate: `DiagnosticCollector`, `Diagnostic`, severity types, and codespan-reporting `render()` function
- `clap` already in Cargo.toml dependencies with derive feature
- `eaml-codegen::generate()` — single function that takes the full pipeline output and returns Python string
- `insta` for snapshot testing (workspace dependency)
- All pipeline crates have clean public APIs with documented entry points

### Established Patterns
- Each crate has a clear `ParseOutput` / `AnalysisOutput` struct as its public output
- `DiagnosticCollector` accumulates errors across phases — CLI needs to merge collectors from lex+parse+semantic
- Error rendering via `codespan_reporting::term::emit()` with `SimpleFile` source wrapper
- Tests in each crate use `tests/` directory with integration-style test files

### Integration Points
- CLI orchestrates: read file -> lex -> parse -> semantic -> codegen -> write file
- `eamlc run` extends: -> shell out to `python <output.py>`
- `eamlc check` stops after semantic analysis (no codegen)
- DiagnosticCollector from each phase needs to be checked for errors before proceeding to next phase

</code_context>

<specifics>
## Specific Ideas

- `eamlc run` shells out to `python` on the generated .py file — simple, debuggable, user can inspect the file
- Rustc-style error summary at the end of diagnostic output feels natural for this Rust-authored compiler
- Integration tests use temp directories so the examples/ directory stays clean and git-friendly

</specifics>

<deferred>
## Deferred Ideas

- `eamlc run` was originally DX-02 (v2 scope) but pulled into Phase 6 as it naturally validates GEN-12

</deferred>

---

*Phase: 06-cli-and-integration*
*Context gathered: 2026-03-17*
