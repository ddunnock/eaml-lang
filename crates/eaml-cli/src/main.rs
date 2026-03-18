//! `eamlc` -- the EAML compiler CLI.
//!
//! Provides compile, check, and run commands for EAML source files.

use clap::{Parser, Subcommand};
use codespan_reporting::files::SimpleFiles;
use eaml_errors::{render, Diagnostic, Severity};
use std::path::{Path, PathBuf};
use std::{fs, process};

/// Exit code: successful compilation or check.
const EXIT_SUCCESS: i32 = 0;
/// Exit code: compilation error (parse or semantic).
const EXIT_COMPILE_ERROR: i32 = 1;
/// Exit code: I/O error (file not found, write failure).
const EXIT_IO_ERROR: i32 = 2;
/// Exit code: runtime error (Python execution failed).
const EXIT_RUNTIME_ERROR: i32 = 3;

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
        /// Output directory (default: same as input file)
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
        /// Output directory (default: same as input file)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

fn main() {
    let cli = Cli::parse();
    let code = match &cli.command {
        Commands::Compile { file, output } => cmd_compile(file, output.as_deref()),
        Commands::Check { file } => cmd_check(file),
        Commands::Run { file, output } => cmd_run(file, output.as_deref()),
    };
    process::exit(code);
}

/// Returns true if the diagnostic represents a compilation error.
fn is_error(d: &Diagnostic) -> bool {
    matches!(d.severity, Severity::Error | Severity::Fatal)
}

/// Extracts the filename component from a path, falling back to "unknown.eaml".
fn filename_of(path: &Path) -> &str {
    path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown.eaml")
}

/// Computes the Python output path for a given EAML source file.
fn output_path_for(file: &Path, output_dir: Option<&Path>) -> PathBuf {
    match output_dir {
        Some(dir) => {
            let stem = file.file_stem().and_then(|s| s.to_str()).unwrap_or("out");
            dir.join(format!("{stem}.py"))
        }
        None => file.with_extension("py"),
    }
}

/// Reads source text from a file path.
///
/// Returns `Err(EXIT_IO_ERROR)` if the file cannot be read.
/// Prints a warning if the file does not have a `.eaml` extension.
fn read_source(path: &Path) -> Result<String, i32> {
    let source = fs::read_to_string(path).map_err(|err| {
        eprintln!("error: could not read file '{}': {err}", path.display());
        EXIT_IO_ERROR
    })?;

    if path.extension().and_then(|e| e.to_str()) != Some("eaml") {
        eprintln!(
            "warning: '{}' does not have a .eaml extension",
            path.display()
        );
    }

    Ok(source)
}

/// Runs the full compiler pipeline (lex -> parse -> semantic -> codegen).
///
/// Returns `(generated_code, all_diagnostics, has_errors)`.
/// If there are parse or semantic errors, `generated_code` is `None`.
fn run_pipeline(source: &str, filename: &str) -> (Option<String>, Vec<Diagnostic>, bool) {
    let parse_output = eaml_parser::parse(source);

    let mut all_diagnostics: Vec<Diagnostic> = parse_output.diagnostics.clone();

    if all_diagnostics.iter().any(is_error) {
        return (None, all_diagnostics, true);
    }

    let analysis = eaml_semantic::analyze(&parse_output, source);
    all_diagnostics.extend(analysis.diagnostics.clone());

    if all_diagnostics.iter().any(is_error) {
        return (None, all_diagnostics, true);
    }

    let python_code = eaml_codegen::generate(&parse_output, &analysis, source, filename);
    (Some(python_code), all_diagnostics, false)
}

/// Renders diagnostics to stderr and prints a summary line.
fn render_and_summarize(filename: &str, source: &str, diagnostics: &[Diagnostic]) {
    if diagnostics.is_empty() {
        return;
    }

    let mut files = SimpleFiles::new();
    files.add(filename, source);
    render::render_diagnostics(&files, diagnostics);
    print_summary(diagnostics);
}

/// Prints an error/warning summary line to stderr.
fn print_summary(diagnostics: &[Diagnostic]) {
    let error_count = diagnostics.iter().filter(|d| is_error(d)).count();
    let warning_count = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Warning)
        .count();

    if error_count > 0 {
        let warning_suffix = if warning_count > 0 {
            format!("; {warning_count} warning(s) emitted")
        } else {
            String::new()
        };
        eprintln!("error: aborting due to {error_count} previous error(s){warning_suffix}");
    } else if warning_count > 0 {
        eprintln!("warning: {warning_count} warning(s) emitted");
    }
}

/// Compiles an EAML file and writes the generated Python to disk.
///
/// Returns `(output_path, exit_code)`. On success the exit code is `EXIT_SUCCESS`
/// and the output path points to the written `.py` file.
fn compile_and_write(file: &Path, output_dir: Option<&Path>) -> (PathBuf, i32) {
    let source = match read_source(file) {
        Ok(s) => s,
        Err(code) => return (PathBuf::new(), code),
    };

    let filename = filename_of(file);
    let (python_code, diagnostics, has_errors) = run_pipeline(&source, filename);
    render_and_summarize(filename, &source, &diagnostics);

    if has_errors {
        return (PathBuf::new(), EXIT_COMPILE_ERROR);
    }

    let python_code = python_code.expect("no errors means code was generated");
    let output_path = output_path_for(file, output_dir);

    if let Err(err) = fs::write(&output_path, &python_code) {
        eprintln!("error: could not write '{}': {err}", output_path.display());
        return (PathBuf::new(), EXIT_IO_ERROR);
    }

    eprintln!("Compiled {filename} -> {}", output_path.display());
    (output_path, EXIT_SUCCESS)
}

/// Compiles an EAML file to Python.
fn cmd_compile(file: &Path, output_dir: Option<&Path>) -> i32 {
    let (_path, code) = compile_and_write(file, output_dir);
    code
}

/// Checks an EAML file for errors without generating output.
fn cmd_check(file: &Path) -> i32 {
    let source = match read_source(file) {
        Ok(s) => s,
        Err(code) => return code,
    };

    let filename = filename_of(file);
    let (_, diagnostics, has_errors) = run_pipeline(&source, filename);
    render_and_summarize(filename, &source, &diagnostics);

    if has_errors {
        return EXIT_COMPILE_ERROR;
    }

    eprintln!("{filename}: no errors found");
    EXIT_SUCCESS
}

/// Finds a working Python interpreter on PATH.
fn find_python() -> Result<&'static str, i32> {
    for cmd in ["python3", "python"] {
        if process::Command::new(cmd).arg("--version").output().is_ok() {
            return Ok(cmd);
        }
    }
    eprintln!("error: Python interpreter not found. Ensure python3 or python is on PATH.");
    Err(EXIT_RUNTIME_ERROR)
}

/// Compiles an EAML file to Python, then runs it with the Python interpreter.
fn cmd_run(file: &Path, output_dir: Option<&Path>) -> i32 {
    let (output_path, code) = compile_and_write(file, output_dir);
    if code != EXIT_SUCCESS {
        return code;
    }

    let python_cmd = match find_python() {
        Ok(cmd) => cmd,
        Err(code) => return code,
    };

    match process::Command::new(python_cmd).arg(&output_path).status() {
        Ok(status) if status.success() => EXIT_SUCCESS,
        Ok(_) => EXIT_RUNTIME_ERROR,
        Err(err) => {
            eprintln!("error: failed to run Python: {err}");
            EXIT_RUNTIME_ERROR
        }
    }
}
