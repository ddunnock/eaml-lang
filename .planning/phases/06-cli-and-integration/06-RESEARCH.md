# Phase 6: CLI and Integration - Research

**Researched:** 2026-03-17
**Domain:** Rust CLI (clap), integration testing (assert_cmd), Python interop (mypy, runtime)
**Confidence:** HIGH

## Summary

Phase 6 wires together all prior compiler phases (lexer, parser, semantic, codegen) into the `eamlc` CLI binary and validates the full pipeline end-to-end. The existing crate APIs are clean and well-defined: `eaml_parser::parse()` returns `ParseOutput`, `eaml_semantic::analyze()` returns `AnalysisOutput`, and `eaml_codegen::generate()` returns a Python string. The CLI orchestrates this pipeline, writes output files, renders diagnostics via the existing `eaml_errors::render` module, and optionally shells out to Python for `eamlc run`.

The current `main.rs` is a placeholder stub. All pipeline dependencies are already declared in `eaml-cli/Cargo.toml`. The `codespan-reporting` + `termcolor` stack already handles colored output with automatic NO_COLOR and TTY detection via `ColorChoice::Auto`. Integration testing uses `assert_cmd` (the Rust ecosystem standard) to invoke the compiled binary and assert on exit codes, stdout, and stderr.

**Primary recommendation:** Build the CLI in two waves -- first the core pipeline orchestration (`compile`, `check`, exit codes, error display), then integration tests covering all examples plus mypy and the `run` command.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Four commands: `compile`, `check`, `run`, `--version`
- `eamlc compile <file>` -- compile .eaml to .py, print success message
- `eamlc check <file>` -- validate .eaml without generating output
- `eamlc run <file>` -- compile to .py then execute via `python <file.py>` (shell out with `std::process::Command`)
- `--version` via clap's built-in version flag
- Single file per invocation (no glob/multi-file)
- No verbosity flags for v0.1
- Default: generated .py file placed in same directory as source .eaml file
- `-o / --output <dir>` flag to specify output directory (directory only, not file path)
- Filename derived from input: `sentiment.eaml` -> `sentiment.py`
- `eamlc run` keeps the generated .py file after execution (not temp)
- Exit 0: success; Exit 1: compilation error(s); Exit 2: file not found / IO error; Exit 3: runtime error
- Rustc-style summary: `error: aborting due to N previous errors` (with warnings count if any)
- Warnings always shown alongside errors (never suppressed)
- Color: auto-detect TTY + respect `NO_COLOR` env var
- Both library-level tests (fast) + CLI integration tests (invoke binary)
- CLI integration tests use temp directories for generated output
- All 7 examples tested via CLI integration tests
- GEN-11 (mypy): compile each example, run mypy, assert exit 0
- GEN-12 / INT-02 (real LLM): `#[ignore]` test, only when API key set

### Claude's Discretion
- Exact clap derive struct design and argument validation
- How `eamlc run` discovers the Python interpreter (PATH lookup strategy)
- Error recovery behavior when `python` is not found
- assert_cmd vs raw Command for integration tests
- Whether to add `codespan-reporting` as a direct dep or re-export from eaml-errors
- Exact format of the success message

### Deferred Ideas (OUT OF SCOPE)
- `eamlc run` was originally DX-02 (v2 scope) but pulled into Phase 6 as it naturally validates GEN-12
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| CLI-01 | `eamlc compile <file>` compiles .eaml to .py with exit code 0 on success | Pipeline orchestration pattern; clap derive struct with Subcommand; file I/O with exit code mapping |
| CLI-02 | `eamlc check <file>` validates .eaml without generating output | Same pipeline minus codegen step; short-circuit after semantic analysis |
| CLI-03 | CLI displays all accumulated errors/warnings using codespan-reporting | Existing `render_diagnostics()` function; merge diagnostics from parse + semantic phases; rustc-style summary |
| CLI-04 | CLI returns non-zero exit code on compilation errors | Exit code enum (0/1/2/3); `std::process::exit()` or `ExitCode` return |
| INT-01 | All 7 example programs compile successfully | 4 have .eaml files (01, 02, 06, 07); 3 are empty (.gitkeep only: 03, 04, 05) -- need .eaml files created or tests adjusted |
| INT-02 | Generated Python from sentiment.eaml runs and returns structured output from LLM | `eamlc run` shells out to Python; `#[ignore]` test requiring ANTHROPIC_API_KEY |
| INT-03 | bad_model.eaml triggers CAP010 capability mismatch error at compile time | Example 06 already has bad_model.eaml with intentional CAP010; assert exit code 1 + stderr contains "CAP010" |
| GEN-11 | Generated Python type-checks with mypy without errors | Compile examples, run `mypy --ignore-missing-imports <output.py>`; requires eaml_runtime installed or --ignore-missing-imports |
| GEN-12 | Generated Python runs and calls LLM APIs via eaml_runtime | End-to-end: compile sentiment.eaml, execute Python, verify JSON output; `#[ignore]` test |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| clap | 4.x (derive) | CLI argument parsing, subcommands | Already in workspace deps; derive feature for struct-based args |
| codespan-reporting | 0.11 | Diagnostic rendering with colored source snippets | Already used by eaml-errors; handles all formatting |
| termcolor | (transitive) | Color output with NO_COLOR/TTY auto-detection | Comes with codespan-reporting; `ColorChoice::Auto` handles everything |

### Testing
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| assert_cmd | 2.x | CLI integration testing | Binary invocation, exit code/output assertions |
| predicates | 3.x | Fluent assertion matchers for assert_cmd | Matching stderr/stdout content patterns |
| tempfile | 3.x | Temp directories for test output | Keeping examples/ clean during tests |
| insta | 1.x | Snapshot testing (already in workspace) | Snapshot error output if desired |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| assert_cmd | Raw `std::process::Command` | assert_cmd provides `cargo_bin()` discovery and fluent assertions; worth the dep |
| tempfile | Manual temp dir creation | tempfile handles cleanup automatically; standard practice |

**Installation (add to eaml-cli/Cargo.toml):**
```toml
[dev-dependencies]
assert_cmd = "2"
predicates = "3"
tempfile = "3"
```

## Architecture Patterns

### CLI Module Structure
```
crates/eaml-cli/
├── src/
│   └── main.rs           # Entry point: clap parse -> dispatch
├── tests/
│   ├── cli_compile.rs    # Integration: compile command tests
│   ├── cli_check.rs      # Integration: check command tests
│   ├── cli_run.rs        # Integration: run command tests
│   └── cli_examples.rs   # Integration: all examples + mypy
└── Cargo.toml
```

### Pattern 1: Clap Derive Subcommands
**What:** Define CLI structure as Rust structs with `#[derive(Parser)]` and `#[derive(Subcommand)]`
**When to use:** Always for clap-based CLIs
**Example:**
```rust
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "eamlc", version, about = "The EAML compiler")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile an EAML file to Python
    Compile {
        /// Input .eaml file
        file: PathBuf,
        /// Output directory (default: same as input)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Check an EAML file for errors without generating output
    Check {
        /// Input .eaml file
        file: PathBuf,
    },
    /// Compile and run an EAML file
    Run {
        /// Input .eaml file
        file: PathBuf,
        /// Output directory (default: same as input)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}
```

### Pattern 2: Pipeline Orchestration
**What:** Sequential phase execution with early-exit on errors
**When to use:** The core compile flow
**Example:**
```rust
fn compile(source: &str, filename: &str) -> (String, Vec<Diagnostic>, bool) {
    // Phase 1: Parse (includes lexing)
    let parse_output = eaml_parser::parse(source);

    // Collect parse diagnostics -- check for errors before semantic
    let mut all_diagnostics: Vec<Diagnostic> = parse_output.diagnostics.clone();
    let has_parse_errors = all_diagnostics.iter().any(|d|
        d.severity == Severity::Error || d.severity == Severity::Fatal
    );

    if has_parse_errors {
        return (String::new(), all_diagnostics, true);
    }

    // Phase 2: Semantic analysis
    let analysis = eaml_semantic::analyze(&parse_output, source);
    all_diagnostics.extend(analysis.diagnostics.clone());
    let has_errors = all_diagnostics.iter().any(|d|
        d.severity == Severity::Error || d.severity == Severity::Fatal
    );

    if has_errors {
        return (String::new(), all_diagnostics, true);
    }

    // Phase 3: Code generation
    let python_code = eaml_codegen::generate(&parse_output, &analysis, source, filename);
    (python_code, all_diagnostics, false)
}
```

### Pattern 3: Exit Code Mapping
**What:** Map error conditions to specific exit codes
**Example:**
```rust
enum ExitCode {
    Success = 0,
    CompileError = 1,
    IoError = 2,
    RuntimeError = 3,
}
```

### Pattern 4: Rustc-Style Summary
**What:** Print error/warning summary after diagnostics
**Example:**
```rust
fn print_summary(diagnostics: &[Diagnostic]) {
    let error_count = diagnostics.iter()
        .filter(|d| d.severity == Severity::Error || d.severity == Severity::Fatal)
        .count();
    let warning_count = diagnostics.iter()
        .filter(|d| d.severity == Severity::Warning)
        .count();

    if error_count > 0 {
        let warning_suffix = if warning_count > 0 {
            format!("; {} warning{} emitted", warning_count, if warning_count == 1 { "" } else { "s" })
        } else {
            String::new()
        };
        eprintln!(
            "error: aborting due to {} previous error{}{}",
            error_count,
            if error_count == 1 { "" } else { "s" },
            warning_suffix,
        );
    } else if warning_count > 0 {
        eprintln!(
            "warning: {} warning{} emitted",
            warning_count,
            if warning_count == 1 { "" } else { "s" },
        );
    }
}
```

### Pattern 5: assert_cmd Integration Tests
**What:** Test the compiled binary end-to-end
**Example:**
```rust
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

#[test]
fn compile_minimal_example() {
    let tmp = TempDir::new().unwrap();
    Command::cargo_bin("eaml-cli").unwrap()
        .arg("compile")
        .arg("examples/01-minimal/minimal.eaml")
        .arg("-o")
        .arg(tmp.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("Compiled"));

    assert!(tmp.path().join("minimal.py").exists());
}

#[test]
fn check_bad_model_reports_cap010() {
    Command::cargo_bin("eaml-cli").unwrap()
        .arg("check")
        .arg("examples/06-capability-error/bad_model.eaml")
        .assert()
        .code(1)
        .stderr(predicate::str::contains("CAP010"));
}
```

### Anti-Patterns to Avoid
- **Merging diagnostics incorrectly:** Parse diagnostics are separate from semantic diagnostics. Both must be rendered. Don't create a new `DiagnosticCollector` for CLI -- just concatenate the `Vec<Diagnostic>` from each phase.
- **Proceeding to codegen after errors:** If parse or semantic has errors, skip codegen entirely. The AST may be incomplete/invalid.
- **Using `unwrap()` for file I/O in production code:** Map all I/O errors to exit code 2 with user-friendly messages.
- **Hardcoding "python3" vs "python":** Platform-dependent. Try `python3` first, fall back to `python`. Or use `sys.executable` discovery.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| CLI argument parsing | Manual arg parsing | clap derive macros | Already in deps; handles --help, --version, validation |
| Diagnostic rendering | Custom error formatting | `eaml_errors::render::render_diagnostics()` | Already implemented with codespan-reporting |
| Color detection | Manual TTY/NO_COLOR checks | `ColorChoice::Auto` (termcolor) | Handles NO_COLOR, TERM=dumb, pipe detection automatically |
| Binary testing | Raw `std::process::Command` | `assert_cmd::Command::cargo_bin()` | Handles binary discovery, fluent assertions, better error messages |
| Temp directories | Manual mktemp | `tempfile::TempDir` | Auto-cleanup, cross-platform |

**Key insight:** Almost all hard problems are already solved by existing deps. The CLI is pure orchestration glue.

## Common Pitfalls

### Pitfall 1: Missing Example Files
**What goes wrong:** INT-01 requires "all 7 examples" compile, but examples 03 (python-bridge), 04 (multi-tool-agent), and 05 (multi-file) only have `.gitkeep` files -- no actual `.eaml` source.
**Why it happens:** These were placeholder directories created during project setup.
**How to avoid:** Either create the missing .eaml files (preferred, to fulfill INT-01 literally), or adjust tests to cover only the 4 existing examples and document why. Creating them is better since they exercise bridge/agent/multi-file features validated in prior phases.
**Warning signs:** Tests that silently skip missing files appear to pass but don't validate anything.

### Pitfall 2: Diagnostic Vec Ownership
**What goes wrong:** `parse_output.diagnostics` is `Vec<Diagnostic>` (owned), and `analysis.diagnostics` is also `Vec<Diagnostic>`. CLI needs to merge them for rendering.
**Why it happens:** Each phase returns owned diagnostics, not references.
**How to avoid:** Clone parse diagnostics before passing `parse_output` to `analyze()` (which borrows it), or collect them after analysis. The simplest approach: `let mut all_diags = parse_output.diagnostics.clone(); all_diags.extend(analysis.diagnostics);`

### Pitfall 3: codespan-reporting File ID
**What goes wrong:** `render_diagnostics` hardcodes file_id=0. The `SimpleFiles` must be set up with the correct source text at index 0.
**Why it happens:** The existing render API assumes a single-file setup.
**How to avoid:** Create `SimpleFiles`, add the source with `files.add(filename, source)`, then call `render_diagnostics(&files, &all_diagnostics)`.

### Pitfall 4: Python Interpreter Discovery
**What goes wrong:** `eamlc run` shells out to `python` but on some systems only `python3` exists, or neither is on PATH.
**Why it happens:** Platform fragmentation (macOS ships python3 only, some Linux has both).
**How to avoid:** Try `python3` first via `Command::new("python3").arg("--version")`, fall back to `python`. If neither found, exit code 3 with clear message "Python interpreter not found. Ensure python3 or python is on PATH."

### Pitfall 5: mypy Needs eaml_runtime Installed
**What goes wrong:** `mypy --ignore-missing-imports` still won't validate the eaml_runtime imports, but the generated code won't fail mypy either (the flag suppresses the error). Without the flag, mypy will error on `from eaml_runtime import execute_prompt`.
**Why it happens:** mypy needs the package importable to check types.
**How to avoid:** Use `--ignore-missing-imports` flag per CONTEXT.md decision. This validates the generated code's own type correctness without requiring the runtime to be installed in the test environment.

### Pitfall 6: assert_cmd Binary Name
**What goes wrong:** `Command::cargo_bin("eaml-cli")` looks for the binary name from `Cargo.toml` `[package] name`, but the binary name may differ if `[[bin]]` section exists.
**Why it happens:** The package name is `eaml-cli` but the binary might be named `eamlc`.
**How to avoid:** Check if there's a `[[bin]]` section in `eaml-cli/Cargo.toml`. If not, add one: `[[bin]] name = "eamlc" path = "src/main.rs"`. Then use `Command::cargo_bin("eamlc")`.

## Code Examples

### Full Pipeline in main.rs
```rust
use clap::{Parser, Subcommand};
use codespan_reporting::files::SimpleFiles;
use eaml_errors::{render, Diagnostic, Severity};
use std::fs;
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(name = "eamlc", version, about = "The EAML compiler")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Compile {
        file: PathBuf,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    Check {
        file: PathBuf,
    },
    Run {
        file: PathBuf,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

fn main() {
    let cli = Cli::parse();
    let code = match cli.command {
        Commands::Compile { file, output } => cmd_compile(&file, output.as_deref()),
        Commands::Check { file } => cmd_check(&file),
        Commands::Run { file, output } => cmd_run(&file, output.as_deref()),
    };
    process::exit(code);
}
```

### Rendering Diagnostics with Summary
```rust
fn render_and_summarize(filename: &str, source: &str, diagnostics: &[Diagnostic]) {
    if diagnostics.is_empty() {
        return;
    }
    let mut files = SimpleFiles::new();
    files.add(filename, source);
    render::render_diagnostics(&files, diagnostics);
    print_summary(diagnostics);
}
```

### assert_cmd Test with Temp Output
```rust
#[test]
fn compile_writes_output_to_specified_dir() {
    let tmp = TempDir::new().unwrap();
    Command::cargo_bin("eamlc").unwrap()
        .arg("compile")
        .arg("examples/01-minimal/minimal.eaml")
        .arg("-o")
        .arg(tmp.path())
        .assert()
        .success();

    let output = fs::read_to_string(tmp.path().join("minimal.py")).unwrap();
    assert!(output.contains("class Greeting(BaseModel):"));
}
```

### mypy Integration Test
```rust
#[test]
fn generated_python_passes_mypy() {
    let tmp = TempDir::new().unwrap();
    // Compile
    Command::cargo_bin("eamlc").unwrap()
        .arg("compile")
        .arg("examples/01-minimal/minimal.eaml")
        .arg("-o")
        .arg(tmp.path())
        .assert()
        .success();

    // Run mypy
    Command::new("mypy")
        .arg("--ignore-missing-imports")
        .arg(tmp.path().join("minimal.py"))
        .assert()
        .success();
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| structopt | clap 4 derive | clap 4.0 (2022) | structopt merged into clap; use `#[derive(Parser)]` |
| ColorChoice manual | ColorChoice::Auto | termcolor 1.2+ | Handles NO_COLOR automatically |
| Manual binary paths | assert_cmd cargo_bin | assert_cmd 2.x | Discovers test binary from cargo metadata |

**Deprecated/outdated:**
- `structopt` crate: absorbed into clap 4 derive macros
- `assert_cli` crate: replaced by `assert_cmd`

## Open Questions

1. **Missing example files (03, 04, 05)**
   - What we know: Directories exist with `.gitkeep` but no `.eaml` files
   - What's unclear: Whether these should be created in Phase 6 or if INT-01 should be interpreted as "all examples that exist"
   - Recommendation: Create .eaml files for 03 (python-bridge), 04 (multi-tool-agent). Skip 05 (multi-file) since multi-file compilation is explicitly out of scope. This gives us 6 compilable examples, with example 06 being a negative test (expected failure).

2. **Binary name configuration**
   - What we know: Package is `eaml-cli`, desired binary is `eamlc`
   - What's unclear: Whether `[[bin]]` section is needed
   - Recommendation: Add `[[bin]] name = "eamlc" path = "src/main.rs"` to eaml-cli/Cargo.toml

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cargo test (Rust) + assert_cmd 2.x for CLI integration |
| Config file | crates/eaml-cli/Cargo.toml (dev-dependencies) |
| Quick run command | `cargo test -p eaml-cli` |
| Full suite command | `make test` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CLI-01 | compile command produces .py, exit 0 | integration | `cargo test -p eaml-cli -- compile` | Wave 0 |
| CLI-02 | check command validates without output | integration | `cargo test -p eaml-cli -- check` | Wave 0 |
| CLI-03 | errors displayed via codespan-reporting | integration | `cargo test -p eaml-cli -- error_display` | Wave 0 |
| CLI-04 | non-zero exit on compile errors | integration | `cargo test -p eaml-cli -- exit_code` | Wave 0 |
| INT-01 | all examples compile | integration | `cargo test -p eaml-cli -- examples` | Wave 0 |
| INT-02 | sentiment.eaml runs with LLM output | integration (ignore) | `cargo test -p eaml-cli -- run_sentiment --ignored` | Wave 0 |
| INT-03 | bad_model.eaml triggers CAP010 | integration | `cargo test -p eaml-cli -- cap010` | Wave 0 |
| GEN-11 | generated Python passes mypy | integration | `cargo test -p eaml-cli -- mypy` | Wave 0 |
| GEN-12 | generated Python calls LLM APIs | integration (ignore) | `cargo test -p eaml-cli -- run_llm --ignored` | Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p eaml-cli`
- **Per wave merge:** `make test`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `crates/eaml-cli/tests/` -- entire test directory needs creation
- [ ] `assert_cmd`, `predicates`, `tempfile` -- add to dev-dependencies
- [ ] `[[bin]] name = "eamlc"` -- add to Cargo.toml so cargo_bin works
- [ ] Example .eaml files for 03-python-bridge, 04-multi-tool-agent -- create or skip

## Sources

### Primary (HIGH confidence)
- `crates/eaml-cli/src/main.rs` -- current stub, confirmed placeholder only
- `crates/eaml-cli/Cargo.toml` -- confirmed all pipeline deps + clap already declared
- `crates/eaml-errors/src/render.rs` -- confirmed render_diagnostics() API with ColorChoice::Auto
- `crates/eaml-errors/src/diagnostic.rs` -- confirmed DiagnosticCollector and Diagnostic types
- `crates/eaml-codegen/src/lib.rs` -- confirmed generate() signature
- `crates/eaml-parser/src/lib.rs` -- confirmed parse() returning ParseOutput
- `crates/eaml-semantic/src/lib.rs` -- confirmed analyze() returning AnalysisOutput
- `examples/` -- confirmed 4 of 7 have .eaml files; 3 are .gitkeep stubs

### Secondary (MEDIUM confidence)
- [termcolor ColorChoice docs](https://docs.rs/termcolor/latest/termcolor/enum.ColorChoice.html) -- ColorChoice::Auto respects NO_COLOR env var
- [assert_cmd docs](https://docs.rs/assert_cmd/latest/assert_cmd/) -- v2.x API with cargo_bin(), assert(), success/failure/code

### Tertiary (LOW confidence)
- None -- all findings verified against source code or official docs

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all deps already in workspace or well-established Rust crates
- Architecture: HIGH -- pipeline API fully inspected, patterns derived from existing code
- Pitfalls: HIGH -- identified from direct code inspection (missing examples, file_id, binary name)

**Research date:** 2026-03-17
**Valid until:** 2026-04-17 (stable domain, no fast-moving deps)
