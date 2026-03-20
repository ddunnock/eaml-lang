# Milestones

## v1.0 EAML Compiler (Shipped: 2026-03-20)

**Phases completed:** 6 phases, 18 plans
**Timeline:** 5 days (2026-03-14 → 2026-03-18)
**Codebase:** 15,048 lines Rust + 639 lines Python across 267 files

**Key accomplishments:**
1. Complete lexer with template string interpolation, python bridge capture, and error recovery
2. Hand-written recursive descent parser covering all 84 grammar productions with typed arena AST
3. Three-pass semantic analysis: name resolution with forward references, type checking with bounded types, capability subset checking with CAP010 fatal gate
4. Python/Pydantic code generation with topological schema sort, f-string interpolation, and import deduplication
5. Python runtime with Anthropic/OpenAI/Ollama provider adapters, validate_or_retry, and telemetry hooks
6. CLI binary (eamlc) with compile/check/run commands, all 7 example programs compiling, mypy-clean generated output

**Stats:** 133 commits, ~460 tests (Rust + Python), clippy/ruff/mypy clean

**Archive:** `.planning/milestones/v1.0-ROADMAP.md`, `.planning/milestones/v1.0-REQUIREMENTS.md`, `.planning/milestones/v1.0-MILESTONE-AUDIT.md`

---

