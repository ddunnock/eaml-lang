---
phase: 3
slug: semantic-analysis
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-16
---

# Phase 3 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (insta snapshots + unit tests) |
| **Config file** | `crates/eaml-semantic/Cargo.toml` |
| **Quick run command** | `cargo test -p eaml-semantic` |
| **Full suite command** | `cargo test --workspace` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p eaml-semantic`
- **After every plan wave:** Run `cargo test --workspace`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 5 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 03-01-01 | 01 | 1 | SEM-01 | unit | `cargo test -p eaml-semantic resolve` | W0 | pending |
| 03-01-02 | 01 | 1 | SEM-02 | unit | `cargo test -p eaml-semantic resolve` | W0 | pending |
| 03-01-03 | 01 | 1 | SEM-03 | unit | `cargo test -p eaml-semantic resolve` | W0 | pending |
| 03-02-01 | 02 | 2 | SEM-04 | unit | `cargo test -p eaml-semantic types` | W0 | pending |
| 03-02-02 | 02 | 2 | SEM-05 | unit | `cargo test -p eaml-semantic types` | W0 | pending |
| 03-02-03 | 02 | 2 | SEM-06 | unit | `cargo test -p eaml-semantic types` | W0 | pending |
| 03-02-04 | 02 | 2 | SEM-07 | unit | `cargo test -p eaml-semantic types` | W0 | pending |
| 03-02-05 | 02 | 2 | SEM-10 | unit | `cargo test -p eaml-semantic scoping` | W0 | pending |
| 03-03-01 | 03 | 3 | SEM-08 | unit | `cargo test -p eaml-semantic caps` | W0 | pending |
| 03-03-02 | 03 | 3 | SEM-09 | unit | `cargo test -p eaml-semantic caps` | W0 | pending |
| 03-03-03 | 03 | 3 | SEM-11 | unit | `cargo test -p eaml-semantic integration` | W0 | pending |

*Status: pending / green / red / flaky*

---

## Wave 0 Requirements

- [ ] `crates/eaml-semantic/tests/resolve.rs` — stubs for SEM-01, SEM-02, SEM-03
- [ ] `crates/eaml-semantic/tests/types.rs` — stubs for SEM-04 through SEM-07
- [ ] `crates/eaml-semantic/tests/scoping.rs` — stubs for SEM-10
- [ ] `crates/eaml-semantic/tests/caps.rs` — stubs for SEM-08, SEM-09
- [ ] `crates/eaml-semantic/tests/integration.rs` — stubs for SEM-11
- [ ] `crates/eaml-semantic/tests/test_helpers.rs` — shared test utilities

*Existing infrastructure (insta, codespan-reporting) covers framework needs.*

---

## Manual-Only Verifications

*All phase behaviors have automated verification.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 5s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
