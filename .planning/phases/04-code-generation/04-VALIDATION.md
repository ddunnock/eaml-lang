---
phase: 4
slug: code-generation
status: validated
nyquist_compliant: true
wave_0_complete: true
created: 2026-03-16
validated: 2026-03-16
---

# Phase 4 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | insta 1.x (Rust snapshot testing) + cargo test |
| **Config file** | `crates/eaml-codegen/tests/snapshots/` (auto-created by insta) |
| **Quick run command** | `cargo test -p eaml-codegen` |
| **Full suite command** | `cargo test --workspace` |
| **Estimated runtime** | ~3 seconds |
| **Total tests** | 76 (across 10 test files) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p eaml-codegen`
- **After every plan wave:** Run `cargo test --workspace`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 3 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 04-01-01 | 01 | 1 | GEN-01 | unit | `cargo test -p eaml-codegen -- writer` | ✅ `tests/writer.rs` (8 tests) | ✅ green |
| 04-01-02 | 01 | 1 | GEN-10 | unit | `cargo test -p eaml-codegen -- names` | ✅ `tests/names.rs` (8 tests) | ✅ green |
| 04-01-03 | 01 | 1 | GEN-10 | unit | `cargo test -p eaml-codegen -- types` | ✅ `tests/types.rs` (27 tests) | ✅ green |
| 04-02-01 | 02 | 2 | GEN-02, GEN-03, GEN-04 | snapshot | `cargo test -p eaml-codegen -- schemas` | ✅ `tests/schemas.rs` (9 tests) | ✅ green |
| 04-02-02 | 02 | 2 | GEN-07 | snapshot | `cargo test -p eaml-codegen -- models` | ✅ `tests/models.rs` (3 tests) | ✅ green |
| 04-02-03 | 02 | 2 | GEN-05, GEN-06 | snapshot | `cargo test -p eaml-codegen -- prompts` | ✅ `tests/prompts.rs` (4 tests) | ✅ green |
| 04-02-04 | 02 | 2 | GEN-08 | snapshot | `cargo test -p eaml-codegen -- tools` | ✅ `tests/tools.rs` (3 tests) | ✅ green |
| 04-02-05 | 02 | 2 | GEN-09 | snapshot | `cargo test -p eaml-codegen -- agents` | ✅ `tests/agents.rs` (3 tests) | ✅ green |
| 04-03-01 | 03 | 3 | GEN-10, GEN-11 | snapshot+integration | `cargo test -p eaml-codegen -- integration examples` | ✅ `tests/integration.rs` (8) + `tests/examples.rs` (3) | ✅ green |
| 04-03-02 | 03 | 3 | GEN-12 | integration | Manual: requires eaml_runtime | ❌ deferred | ⬜ deferred |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [x] `crates/eaml-codegen/src/lib.rs` — public `generate()` function signature
- [x] `crates/eaml-codegen/src/writer.rs` — CodeWriter struct with indent/dedent
- [x] `crates/eaml-codegen/src/types.rs` — type annotation emission
- [x] `crates/eaml-codegen/src/names.rs` — name conversion utilities
- [x] `crates/eaml-codegen/tests/test_helpers.rs` — parse + analyze + generate helper
- [x] `crates/eaml-codegen/tests/snapshots/` — snapshot directory (auto-created by insta)
- [x] `lasso` dependency in `crates/eaml-codegen/Cargo.toml` for `Spur` type access

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Generated Python passes mypy | GEN-11 | Requires mypy + Python environment | Run `mypy <output>.py` on generated code |
| Generated Python is importable | GEN-12 | Requires eaml_runtime (Phase 5) | Deferred to Phase 5 integration |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 15s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** validated 2026-03-16

---

## Validation Audit 2026-03-16

| Metric | Count |
|--------|-------|
| Gaps found | 0 |
| Resolved | 0 |
| Escalated | 0 |
| Total automated tests | 76 |
| Requirements covered | 10/10 (GEN-01 through GEN-10) |
| Manual-only | 2 (GEN-11, GEN-12) |
