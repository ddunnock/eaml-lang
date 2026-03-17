//! Integration tests for example EAML programs.
//!
//! Covers:
//! - INT-01: All compilable examples compile successfully
//! - INT-03: Capability error detection (CAP010)
//! - GEN-11: Generated Python passes mypy
//! - INT-02/GEN-12: LLM end-to-end execution (ignored, requires API key)

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::{Path, PathBuf};
use std::process;
use tempfile::TempDir;

/// Returns the workspace root directory (two levels above the crate manifest).
fn workspace_root() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root should exist")
        .to_path_buf()
}

/// Helper: build a Command for the `eamlc` binary, with cwd set to workspace root.
fn eamlc() -> Command {
    let mut cmd = Command::cargo_bin("eamlc").unwrap();
    cmd.current_dir(workspace_root());
    cmd
}

/// Check if mypy is available on PATH.
fn mypy_available() -> bool {
    process::Command::new("mypy")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Run mypy on a generated Python file. Returns true if mypy exits 0.
fn run_mypy(py_file: &Path) -> bool {
    let output = process::Command::new("mypy")
        .arg("--ignore-missing-imports")
        .arg(py_file)
        .output()
        .expect("failed to run mypy");

    if !output.status.success() {
        eprintln!(
            "mypy failed for {}:\n{}",
            py_file.display(),
            String::from_utf8_lossy(&output.stdout)
        );
    }

    output.status.success()
}

/// Helper: compile an example to a temp dir and return (tmpdir, py_path).
fn compile_example(example_path: &str, stem: &str) -> (TempDir, PathBuf) {
    let tmpdir = TempDir::new().unwrap();
    eamlc()
        .args(["compile", example_path, "-o"])
        .arg(tmpdir.path())
        .assert()
        .success();

    let py_path = tmpdir.path().join(format!("{stem}.py"));
    assert!(
        py_path.exists(),
        "{stem}.py should be created after compilation"
    );

    (tmpdir, py_path)
}

// ===================================================================
// INT-01: All compilable examples compile successfully
// ===================================================================

#[test]
fn example_01_minimal_compiles() {
    compile_example("examples/01-minimal/minimal.eaml", "minimal");
}

#[test]
fn example_02_sentiment_compiles() {
    compile_example("examples/02-sentiment/sentiment.eaml", "sentiment");
}

#[test]
fn example_03_python_bridge_compiles() {
    compile_example("examples/03-python-bridge/bridge.eaml", "bridge");
}

#[test]
fn example_04_multi_tool_agent_compiles() {
    compile_example("examples/04-multi-tool-agent/agent.eaml", "agent");
}

#[test]
fn example_07_all_type_variants_compiles() {
    compile_example("examples/07-all-type-variants/types.eaml", "types");
}

// ===================================================================
// INT-03: Capability error detection
// ===================================================================

#[test]
fn example_06_bad_model_fails_with_cap010() {
    eamlc()
        .args(["check", "examples/06-capability-error/bad_model.eaml"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("CAP010"))
        .stderr(predicate::str::contains("WeakModel"))
        .stderr(predicate::str::contains("json_mode"));
}

// ===================================================================
// GEN-11: Generated Python passes mypy
// ===================================================================

#[test]
fn generated_minimal_passes_mypy() {
    if !mypy_available() {
        eprintln!("mypy not found, skipping");
        return;
    }
    let (_tmpdir, py_path) = compile_example("examples/01-minimal/minimal.eaml", "minimal");
    assert!(run_mypy(&py_path), "minimal.py should pass mypy");
}

#[test]
fn generated_sentiment_passes_mypy() {
    if !mypy_available() {
        eprintln!("mypy not found, skipping");
        return;
    }
    let (_tmpdir, py_path) = compile_example("examples/02-sentiment/sentiment.eaml", "sentiment");
    assert!(run_mypy(&py_path), "sentiment.py should pass mypy");
}

#[test]
fn generated_types_passes_mypy() {
    if !mypy_available() {
        eprintln!("mypy not found, skipping");
        return;
    }
    let (_tmpdir, py_path) = compile_example("examples/07-all-type-variants/types.eaml", "types");

    // Rename types.py to eaml_types.py to avoid shadowing Python's stdlib
    // `types` module, which mypy rejects.
    let renamed = _tmpdir.path().join("eaml_types.py");
    std::fs::rename(&py_path, &renamed).unwrap();
    assert!(run_mypy(&renamed), "types.py should pass mypy");
}

// ===================================================================
// INT-02 + GEN-12: LLM end-to-end execution (requires API key)
// ===================================================================

/// End-to-end test: compile sentiment.eaml and run the generated Python
/// against a real LLM API. Requires:
/// - ANTHROPIC_API_KEY environment variable set
/// - eaml_runtime Python package installed (`cd python && uv pip install -e .`)
///
/// Run with: `cargo test -p eaml-cli --test example_tests -- --ignored`
#[test]
#[ignore]
fn run_sentiment_with_llm() {
    if std::env::var("ANTHROPIC_API_KEY").is_err() {
        eprintln!("ANTHROPIC_API_KEY not set, skipping LLM e2e test");
        return;
    }

    let (_tmpdir, py_path) = compile_example("examples/02-sentiment/sentiment.eaml", "sentiment");

    // Try python3 first, then python.
    let python_cmd = if process::Command::new("python3")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        "python3"
    } else {
        "python"
    };

    let output = process::Command::new(python_cmd)
        .arg(&py_path)
        .output()
        .expect("failed to run python");

    if !output.status.success() {
        eprintln!(
            "Python execution failed:\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }

    assert!(
        output.status.success(),
        "Generated sentiment.py should execute successfully with LLM API"
    );
}
