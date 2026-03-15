---
phase: 01
slug: error-foundation-and-lexer
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-15
---

# Phase 01 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (insta for snapshots) |
| **Config file** | Cargo.toml workspace |
| **Quick run command** | `cargo test -p eaml-errors -p eaml-lexer` |
| **Full suite command** | `cargo test --workspace` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p eaml-errors -p eaml-lexer`
- **After every plan wave:** Run `cargo test --workspace`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 5 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 01-01-01 | 01 | 1 | ERR-01 | unit | `cargo test -p eaml-errors` | ❌ W0 | ⬜ pending |
| 01-01-02 | 01 | 1 | ERR-02 | unit | `cargo test -p eaml-errors` | ❌ W0 | ⬜ pending |
| 01-01-03 | 01 | 1 | ERR-03 | unit | `cargo test -p eaml-errors` | ❌ W0 | ⬜ pending |
| 01-01-04 | 01 | 1 | ERR-04 | unit | `cargo test -p eaml-errors` | ❌ W0 | ⬜ pending |
| 01-02-01 | 02 | 2 | LEX-01 | unit+snapshot | `cargo test -p eaml-lexer` | ❌ W0 | ⬜ pending |
| 01-02-02 | 02 | 2 | LEX-02 | unit+snapshot | `cargo test -p eaml-lexer` | ❌ W0 | ⬜ pending |
| 01-02-03 | 02 | 2 | LEX-03 | unit+snapshot | `cargo test -p eaml-lexer` | ❌ W0 | ⬜ pending |
| 01-02-04 | 02 | 2 | LEX-04 | unit+snapshot | `cargo test -p eaml-lexer` | ❌ W0 | ⬜ pending |
| 01-02-05 | 02 | 2 | LEX-05 | unit+snapshot | `cargo test -p eaml-lexer` | ❌ W0 | ⬜ pending |
| 01-02-06 | 02 | 2 | LEX-06 | unit+snapshot | `cargo test -p eaml-lexer` | ❌ W0 | ⬜ pending |
| 01-02-07 | 02 | 2 | LEX-07 | unit+snapshot | `cargo test -p eaml-lexer` | ❌ W0 | ⬜ pending |
| 01-02-08 | 02 | 2 | LEX-08 | unit+snapshot | `cargo test -p eaml-lexer` | ❌ W0 | ⬜ pending |
| 01-02-09 | 02 | 2 | LEX-09 | unit+snapshot | `cargo test -p eaml-lexer` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/eaml-errors/tests/` — test stubs for ERR-01 through ERR-04
- [ ] `crates/eaml-lexer/tests/` — test stubs for LEX-01 through LEX-09
- [ ] insta snapshot infrastructure already in workspace Cargo.toml

*Existing infrastructure covers framework installation.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Colored error output | ERR-03 | Terminal rendering | Run `eamlc compile bad.eaml` and visually verify colored output |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 5s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
