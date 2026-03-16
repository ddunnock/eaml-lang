---
phase: 2
slug: parser
status: complete
nyquist_compliant: true
wave_0_complete: true
created: 2026-03-16
validated: 2026-03-16
---

# Phase 2 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | insta 1.x + cargo test |
| **Config file** | crates/eaml-parser/Cargo.toml (insta in dev-dependencies) |
| **Quick run command** | `cargo test -p eaml-parser` |
| **Full suite command** | `cargo test --workspace` |
| **Estimated runtime** | ~15 seconds |
| **Total tests** | 140 |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p eaml-parser`
- **After every plan wave:** Run `cargo test --workspace && make check`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | Test File | Count | Status |
|---------|------|------|-------------|-----------|-------------------|-----------|-------|--------|
| 02-01-01 | 01 | 1 | PAR-01 | unit + snapshot | `cargo test -p eaml-parser -- decl_` | declarations.rs | 33 | ✅ green |
| 02-01-02 | 01 | 1 | PAR-02 | unit + snapshot | `cargo test -p eaml-parser -- type_expr_` | type_exprs.rs | 16 | ✅ green |
| 02-01-03 | 01 | 1 | PAR-03 | unit + snapshot | `cargo test -p eaml-parser -- expr_` | expressions.rs | 19 | ✅ green |
| 02-01-04 | 01 | 1 | PAR-04 | unit + snapshot | `cargo test -p eaml-parser -- template_` | templates.rs | 5 | ✅ green |
| 02-01-05 | 01 | 1 | PAR-05 | unit | `cargo test -p eaml-parser -- requires` | declarations.rs | 4 | ✅ green |
| 02-01-06 | 01 | 1 | PAR-06 | unit + snapshot | `cargo test -p eaml-parser -- decl_tool` | declarations.rs | 3 | ✅ green |
| 02-01-07 | 01 | 1 | PAR-07 | unit + snapshot | `cargo test -p eaml-parser -- decl_agent` | declarations.rs | 2 | ✅ green |
| 02-01-08 | 01 | 1 | PAR-08 | unit | `cargo test -p eaml-parser -- recovery_` | recovery.rs | 12 | ✅ green |
| 02-01-09 | 01 | 1 | PAR-09 | unit | `cargo test -p eaml-parser -- spans_` | spans.rs | 9 | ✅ green |
| 02-04-EX | 04 | 1 | PAR-01..09 | integration | `cargo test -p eaml-parser -- example_` | examples.rs | 17 | ✅ green |
| 02-01-IN | 01 | 1 | PAR-08,09 | unit | `cargo test -p eaml-parser -- parser_infra` | parser_infra.rs | 29 | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [x] `crates/eaml-parser/tests/declarations.rs` — 33 tests for PAR-01, PAR-05, PAR-06, PAR-07
- [x] `crates/eaml-parser/tests/type_exprs.rs` — 16 tests for PAR-02
- [x] `crates/eaml-parser/tests/expressions.rs` — 19 tests for PAR-03
- [x] `crates/eaml-parser/tests/templates.rs` — 5 tests for PAR-04
- [x] `crates/eaml-parser/tests/recovery.rs` — 12 tests for PAR-08
- [x] `crates/eaml-parser/tests/examples.rs` — 17 integration tests against examples/*.eaml
- [x] `crates/eaml-parser/tests/spans.rs` — 9 tests for PAR-09
- [x] `crates/eaml-parser/tests/parser_infra.rs` — 29 infrastructure tests

*All Wave 0 test files exist and pass.*

---

## Manual-Only Verifications

*All phase behaviors have automated verification.*

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 15s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** complete

---

## Validation Audit 2026-03-16

| Metric | Count |
|--------|-------|
| Requirements | 9 (PAR-01 through PAR-09) |
| Gaps found | 0 |
| Resolved | 0 |
| Escalated | 0 |
| Total tests | 140 |
| Test files | 8 |
