# Roadmap: EAML Compiler

## Overview

The EAML compiler is built as a strict pipeline following crate boundaries: shared error types feed the lexer, which feeds the parser, which feeds semantic analysis, which feeds code generation. The CLI and integration tests tie the pipeline together. The Python runtime develops in parallel once codegen defines the output contract.

## Milestones

- ✅ **v1.0 EAML Compiler** — Phases 1-6 (shipped 2026-03-20)

## Phases

<details>
<summary>✅ v1.0 EAML Compiler (Phases 1-6) — SHIPPED 2026-03-20</summary>

- [x] Phase 1: Error Foundation and Lexer (3/3 plans) — completed 2026-03-15
- [x] Phase 2: Parser (4/4 plans) — completed 2026-03-16
- [x] Phase 3: Semantic Analysis (3/3 plans) — completed 2026-03-16
- [x] Phase 4: Code Generation (4/4 plans) — completed 2026-03-16
- [x] Phase 5: Python Runtime (2/2 plans) — completed 2026-03-17
- [x] Phase 6: CLI and Integration (2/2 plans) — completed 2026-03-17

Full details: `.planning/milestones/v1.0-ROADMAP.md`

</details>

## Progress

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Error Foundation and Lexer | v1.0 | 3/3 | Complete | 2026-03-15 |
| 2. Parser | v1.0 | 4/4 | Complete | 2026-03-16 |
| 3. Semantic Analysis | v1.0 | 3/3 | Complete | 2026-03-16 |
| 4. Code Generation | v1.0 | 4/4 | Complete | 2026-03-16 |
| 5. Python Runtime | v1.0 | 2/2 | Complete | 2026-03-17 |
| 6. CLI and Integration | v1.0 | 2/2 | Complete | 2026-03-17 |
