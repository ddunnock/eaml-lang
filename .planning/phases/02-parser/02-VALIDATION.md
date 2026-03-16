---
phase: 2
slug: parser
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-16
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

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p eaml-parser`
- **After every plan wave:** Run `cargo test --workspace && make check`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 02-01-01 | 01 | 1 | PAR-01 | unit + snapshot | `cargo test -p eaml-parser -- declarations` | ❌ W0 | ⬜ pending |
| 02-01-02 | 01 | 1 | PAR-02 | unit + snapshot | `cargo test -p eaml-parser -- type_exprs` | ❌ W0 | ⬜ pending |
| 02-01-03 | 01 | 1 | PAR-03 | unit + snapshot | `cargo test -p eaml-parser -- expressions` | ❌ W0 | ⬜ pending |
| 02-01-04 | 01 | 1 | PAR-04 | unit + snapshot | `cargo test -p eaml-parser -- templates` | ❌ W0 | ⬜ pending |
| 02-01-05 | 01 | 1 | PAR-05 | unit | `cargo test -p eaml-parser -- requires` | ❌ W0 | ⬜ pending |
| 02-01-06 | 01 | 1 | PAR-06 | unit + snapshot | `cargo test -p eaml-parser -- tool` | ❌ W0 | ⬜ pending |
| 02-01-07 | 01 | 1 | PAR-07 | unit + snapshot | `cargo test -p eaml-parser -- agent` | ❌ W0 | ⬜ pending |
| 02-01-08 | 01 | 1 | PAR-08 | unit | `cargo test -p eaml-parser -- recovery` | ❌ W0 | ⬜ pending |
| 02-01-09 | 01 | 1 | PAR-09 | unit | `cargo test -p eaml-parser -- spans` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/eaml-parser/tests/declarations.rs` — stubs for PAR-01
- [ ] `crates/eaml-parser/tests/type_exprs.rs` — stubs for PAR-02
- [ ] `crates/eaml-parser/tests/expressions.rs` — stubs for PAR-03
- [ ] `crates/eaml-parser/tests/templates.rs` — stubs for PAR-04, PAR-05
- [ ] `crates/eaml-parser/tests/recovery.rs` — stubs for PAR-08
- [ ] `crates/eaml-parser/tests/examples.rs` — integration tests against examples/*.eaml

*Existing infrastructure: insta and cargo test already configured in workspace.*

---

## Manual-Only Verifications

*All phase behaviors have automated verification.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
