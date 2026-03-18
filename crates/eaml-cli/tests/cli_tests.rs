//! Integration tests for the `eamlc` CLI commands.
//!
//! Tests cover: compile, check, run, --version, exit codes, and output behavior.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

const MINIMAL_EXAMPLE: &str = "examples/01-minimal/minimal.eaml";
const BAD_MODEL_EXAMPLE: &str = "examples/06-capability-error/bad_model.eaml";

/// Returns the workspace root directory (two levels above the crate manifest).
fn workspace_root() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root should exist")
        .to_path_buf()
}

/// Build a Command for the `eamlc` binary, with cwd set to workspace root.
fn eamlc() -> Command {
    let mut cmd = Command::cargo_bin("eamlc").unwrap();
    cmd.current_dir(workspace_root());
    cmd
}

/// Copy an example file into a temp directory and return (tmpdir, copied_path).
fn copy_example_to_tmpdir(example: &str) -> (TempDir, PathBuf) {
    let tmpdir = TempDir::new().unwrap();
    let src = workspace_root().join(example);
    let filename = src.file_name().expect("example should have a filename");
    let dest = tmpdir.path().join(filename);
    fs::copy(&src, &dest).unwrap();
    (tmpdir, dest)
}

#[test]
fn compile_minimal_produces_py_file() {
    let tmpdir = TempDir::new().unwrap();
    eamlc()
        .args(["compile", MINIMAL_EXAMPLE, "-o"])
        .arg(tmpdir.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("Compiled"));

    let content = fs::read_to_string(tmpdir.path().join("minimal.py")).unwrap();
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
    let (tmpdir, eaml_path) = copy_example_to_tmpdir(MINIMAL_EXAMPLE);

    eamlc().arg("compile").arg(&eaml_path).assert().success();

    assert!(
        tmpdir.path().join("minimal.py").exists(),
        "minimal.py should be created next to the source file"
    );
}

#[test]
fn check_valid_file_exits_zero() {
    eamlc()
        .args(["check", MINIMAL_EXAMPLE])
        .assert()
        .success()
        .stderr(predicate::str::contains("no errors found"));
}

#[test]
fn check_does_not_produce_output_file() {
    let (tmpdir, eaml_path) = copy_example_to_tmpdir(MINIMAL_EXAMPLE);

    eamlc().arg("check").arg(&eaml_path).assert().success();

    assert!(
        !tmpdir.path().join("minimal.py").exists(),
        "check command should not produce a .py file"
    );
}

#[test]
fn check_bad_model_exits_one_with_cap010() {
    eamlc()
        .args(["check", BAD_MODEL_EXAMPLE])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("CAP010"));
}

#[test]
fn compile_bad_model_exits_one() {
    let tmpdir = TempDir::new().unwrap();
    eamlc()
        .args(["compile", BAD_MODEL_EXAMPLE, "-o"])
        .arg(tmpdir.path())
        .assert()
        .code(1);

    assert!(
        !tmpdir.path().join("bad_model.py").exists(),
        "no output file should be created on compile error"
    );
}

#[test]
fn compile_error_shows_summary() {
    eamlc()
        .args(["check", BAD_MODEL_EXAMPLE])
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

    // The run command may exit 0 or 3 (runtime error) depending on whether
    // eaml_runtime is installed. We only verify the compiled .py file exists.
    let _ = eamlc()
        .args(["run", MINIMAL_EXAMPLE, "-o"])
        .arg(tmpdir.path())
        .assert();

    assert!(
        tmpdir.path().join("minimal.py").exists(),
        "run command should produce the .py file even if Python execution fails"
    );
}
