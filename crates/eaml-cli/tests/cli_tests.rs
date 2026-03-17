//! Integration tests for the `eamlc` CLI commands.
//!
//! Tests cover: compile, check, run, --version, exit codes, and output behavior.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
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

#[test]
fn compile_minimal_produces_py_file() {
    let tmpdir = TempDir::new().unwrap();
    eamlc()
        .args(["compile", "examples/01-minimal/minimal.eaml", "-o"])
        .arg(tmpdir.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("Compiled"));

    let py_path = tmpdir.path().join("minimal.py");
    assert!(py_path.exists(), "minimal.py should be created");

    let content = fs::read_to_string(&py_path).unwrap();
    assert!(
        content.contains("class Greeting(BaseModel):"),
        "Generated code should contain Greeting class"
    );
    assert!(
        content.contains("from pydantic import BaseModel"),
        "Generated code should import BaseModel"
    );
}

#[test]
fn compile_without_output_flag_uses_input_dir() {
    let tmpdir = TempDir::new().unwrap();
    let eaml_path = tmpdir.path().join("minimal.eaml");
    let src = workspace_root().join("examples/01-minimal/minimal.eaml");
    fs::copy(&src, &eaml_path).unwrap();

    eamlc().arg("compile").arg(&eaml_path).assert().success();

    let py_path = tmpdir.path().join("minimal.py");
    assert!(
        py_path.exists(),
        "minimal.py should be created next to the source file"
    );
}

#[test]
fn check_valid_file_exits_zero() {
    eamlc()
        .args(["check", "examples/01-minimal/minimal.eaml"])
        .assert()
        .success()
        .stderr(predicate::str::contains("no errors found"));
}

#[test]
fn check_does_not_produce_output_file() {
    let tmpdir = TempDir::new().unwrap();
    let eaml_path = tmpdir.path().join("minimal.eaml");
    let src = workspace_root().join("examples/01-minimal/minimal.eaml");
    fs::copy(&src, &eaml_path).unwrap();

    eamlc().arg("check").arg(&eaml_path).assert().success();

    let py_path = tmpdir.path().join("minimal.py");
    assert!(
        !py_path.exists(),
        "check command should not produce a .py file"
    );
}

#[test]
fn check_bad_model_exits_one_with_cap010() {
    eamlc()
        .args(["check", "examples/06-capability-error/bad_model.eaml"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("CAP010"));
}

#[test]
fn compile_bad_model_exits_one() {
    let tmpdir = TempDir::new().unwrap();
    eamlc()
        .args([
            "compile",
            "examples/06-capability-error/bad_model.eaml",
            "-o",
        ])
        .arg(tmpdir.path())
        .assert()
        .code(1);

    let py_path = tmpdir.path().join("bad_model.py");
    assert!(
        !py_path.exists(),
        "no output file should be created on compile error"
    );
}

#[test]
fn compile_error_shows_summary() {
    eamlc()
        .args(["check", "examples/06-capability-error/bad_model.eaml"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("aborting due to"));
}

#[test]
fn nonexistent_file_exits_two() {
    eamlc()
        .args(["compile", "nonexistent_file_12345.eaml"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("could not read"));
}

#[test]
fn version_flag_prints_version() {
    eamlc()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("eamlc"));
}

#[test]
fn run_command_compiles_and_keeps_file() {
    let tmpdir = TempDir::new().unwrap();

    // The `run` command will compile successfully but Python execution may fail
    // (exit code 3) because the generated code requires eaml_runtime. We only
    // verify that the compiled .py file exists.
    let _assert = eamlc()
        .args(["run", "examples/01-minimal/minimal.eaml", "-o"])
        .arg(tmpdir.path())
        .assert();

    // The command may exit 0 or 3 (runtime error) depending on Python env.
    // We do NOT assert success -- we only check the file was produced.

    let py_path = tmpdir.path().join("minimal.py");
    assert!(
        py_path.exists(),
        "run command should produce the .py file even if Python execution fails"
    );
}
