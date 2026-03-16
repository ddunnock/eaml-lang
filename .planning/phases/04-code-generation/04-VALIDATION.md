---
phase: 4
slug: code-generation
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-16
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
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p eaml-codegen`
- **After every plan wave:** Run `cargo test --workspace`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 04-01-01 | 01 | 1 | GEN-01 | unit | `cargo test -p eaml-codegen -- writer` | ❌ W0 | ⬜ pending |
| 04-01-02 | 01 | 1 | GEN-10 | unit | `cargo test -p eaml-codegen -- names` | ❌ W0 | ⬜ pending |
| 04-01-03 | 01 | 1 | GEN-10 | unit | `cargo test -p eaml-codegen -- types` | ❌ W0 | ⬜ pending |
| 04-02-01 | 02 | 2 | GEN-02, GEN-03, GEN-04 | snapshot | `cargo test -p eaml-codegen -- schemas` | ❌ W0 | ⬜ pending |
| 04-02-02 | 02 | 2 | GEN-07 | snapshot | `cargo test -p eaml-codegen -- models` | ❌ W0 | ⬜ pending |
| 04-02-03 | 02 | 2 | GEN-05, GEN-06 | snapshot | `cargo test -p eaml-codegen -- prompts` | ❌ W0 | ⬜ pending |
| 04-02-04 | 02 | 2 | GEN-08 | snapshot | `cargo test -p eaml-codegen -- tools` | ❌ W0 | ⬜ pending |
| 04-02-05 | 02 | 2 | GEN-09 | snapshot | `cargo test -p eaml-codegen -- agents` | ❌ W0 | ⬜ pending |
| 04-03-01 | 03 | 3 | GEN-10, GEN-11 | snapshot | `cargo test -p eaml-codegen -- integration` | ❌ W0 | ⬜ pending |
| 04-03-02 | 03 | 3 | GEN-12 | integration | Manual: requires eaml_runtime | ❌ deferred | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/eaml-codegen/src/lib.rs` — public `generate()` function signature
- [ ] `crates/eaml-codegen/src/writer.rs` — CodeWriter struct with indent/dedent
- [ ] `crates/eaml-codegen/src/types.rs` — type annotation emission
- [ ] `crates/eaml-codegen/src/names.rs` — name conversion utilities
- [ ] `crates/eaml-codegen/tests/test_helpers.rs` — parse + analyze + generate helper
- [ ] `crates/eaml-codegen/tests/snapshots/` — snapshot directory (auto-created by insta)
- [ ] `lasso` dependency in `crates/eaml-codegen/Cargo.toml` for `Spur` type access

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Generated Python passes mypy | GEN-11 | Requires mypy + Python environment | Run `mypy <output>.py` on generated code |
| Generated Python is importable | GEN-12 | Requires eaml_runtime (Phase 5) | Deferred to Phase 5 integration |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
