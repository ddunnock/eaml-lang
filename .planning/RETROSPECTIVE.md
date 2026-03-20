# Project Retrospective

*A living document updated after each milestone. Lessons feed forward into future planning.*

## Milestone: v1.0 — EAML Compiler

**Shipped:** 2026-03-20
**Phases:** 6 | **Plans:** 18 | **Timeline:** 5 days

### What Was Built
- Complete Rust compiler pipeline: lexer (logos + mode switching) -> parser (recursive descent, 84 productions) -> semantic analysis (3-pass) -> Python/Pydantic codegen
- Python runtime with 3 LLM provider adapters (Anthropic, OpenAI, Ollama), validate_or_retry, and telemetry hooks
- CLI binary (`eamlc`) with compile/check/run commands
- ~460 tests across Rust and Python, all green

### What Worked
- Strict crate boundaries prevented scope creep — each crate is independently testable
- TDD approach caught regressions early, especially in codegen (snapshot tests invaluable)
- Phase-level verification (VERIFICATION.md) provided clear completion criteria
- Yolo mode with quality model profile balanced speed with correctness
- Research phase before each planning phase provided clear technical direction
- Spec-first approach (grammar.ebnf, TYPESYSTEM.md, ERRORS.md) eliminated ambiguity during implementation

### What Was Inefficient
- Phase 6 ROADMAP.md still shows "0/2 Not started" despite being complete — status tracking lagged
- Some SUMMARY files lack `requirements-completed` frontmatter, making 3-source cross-reference harder
- Code review remediation (lexer + parser) happened post-completion as separate commits rather than being caught during phase execution

### Patterns Established
- Typed arena AST with newtype IDs — prevents cross-arena indexing bugs
- DiagnosticCollector pattern — accumulates errors, continues past failures
- CodeWriter with explicit indent/dedent — deterministic Python output
- `#[ignore]` tests for live API calls — scaffolded but require credentials
- Assertion-based recovery tests rather than snapshots for behavioral stability

### Key Lessons
1. Spec documents (grammar.ebnf, TYPESYSTEM.md) are worth the upfront investment — they eliminated nearly all design ambiguity during implementation
2. Template string tokenization as "all strings are template strings" simplified the parser significantly
3. Three-pass semantic analysis (resolve → typecheck → capcheck) is the right decomposition for this language
4. Snapshot tests for codegen catch subtle regressions that unit tests miss

### Cost Observations
- Model mix: ~60% sonnet (agents), ~40% opus (orchestration)
- Sessions: ~15 across 5 days
- Notable: Phase 4 (codegen) completed fastest despite being most complex — good spec grounding

---

## Cross-Milestone Trends

### Process Evolution

| Milestone | Timeline | Phases | Key Change |
|-----------|----------|--------|------------|
| v1.0 | 5 days | 6 | Initial milestone — established TDD, spec-first, crate-boundary patterns |

### Cumulative Quality

| Milestone | Tests | Clippy | Mypy |
|-----------|-------|--------|------|
| v1.0 | ~460 | Clean (-D warnings) | Clean (strict) |

### Top Lessons (Verified Across Milestones)

1. Spec-first development pays for itself in reduced implementation ambiguity
2. Strict module boundaries (Rust crates) enable independent testing and prevent coupling
