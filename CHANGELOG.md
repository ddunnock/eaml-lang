# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Rust workspace with 6 crates: eaml-errors, eaml-lexer, eaml-parser, eaml-semantic, eaml-codegen, eaml-cli
- Python runtime package (`eaml-runtime`) with provider stubs (anthropic, openai, ollama)
- Formal W3C EBNF grammar specification (`spec/grammar.ebnf`)
- Type system specification (`spec/TYPESYSTEM.md`)
- Capability registry specification (`spec/CAPABILITIES.md`)
- Python bridge specification (`spec/PYTHON_BRIDGE.md`)
- Error code catalog (`spec/ERRORS.md`)
- 5-layer AI grounding reference stack (`.claude/references/`)
- Example programs (01-minimal through 07-all-type-variants)
- Makefile with unified build, test, check, and fmt targets
- Pre-commit hook (`.githooks/pre-commit`) for Rust and Python linting, typechecks, and tests
