---
phase: 01
slug: error-foundation-and-lexer
status: complete
nyquist_compliant: true
wave_0_complete: true
created: 2026-03-15
validated: 2026-03-15
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
| **Total tests** | 122 (24 eaml-errors + 98 eaml-lexer) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p eaml-errors -p eaml-lexer`
- **After every plan wave:** Run `cargo test --workspace`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 5 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | Test Files | Status |
|---------|------|------|-------------|-----------|-------------------|------------|--------|
| 01-01-01 | 01 | 1 | ERR-01 | unit | `cargo test -p eaml-errors` | `codes_tests.rs` (all_42_error_codes_exist, error_code_display_*, prefix, number) | ✅ green |
| 01-01-02 | 01 | 1 | ERR-02 | unit | `cargo test -p eaml-errors` | `codes_tests.rs` (diagnostic_new_constructor, with_hint, no_hints) | ✅ green |
| 01-01-03 | 01 | 1 | ERR-03 | unit | `cargo test -p eaml-errors` | `collector_render_tests.rs` (to_codespan_*, render_to_string_*) | ✅ green |
| 01-01-04 | 01 | 1 | ERR-04 | unit | `cargo test -p eaml-errors` | `collector_render_tests.rs` (collector_*, overflow_after_max) | ✅ green |
| 01-02-01 | 02 | 2 | LEX-01 | unit+snapshot | `cargo test -p eaml-lexer` | `keywords.rs` (7 tests), `task1_raw_tokens.rs` | ✅ green |
| 01-02-02 | 02 | 2 | LEX-02 | unit+snapshot | `cargo test -p eaml-lexer` | `operators.rs` (5 tests), `task1_raw_tokens.rs` | ✅ green |
| 01-02-03 | 02 | 2 | LEX-03 | unit+snapshot | `cargo test -p eaml-lexer` | `literals.rs` (string/escape tests) | ✅ green |
| 01-02-04 | 02 | 2 | LEX-04 | unit+snapshot | `cargo test -p eaml-lexer` | `literals.rs` (int/float tests) | ✅ green |
| 01-02-05 | 02 | 2 | LEX-05 | unit+snapshot | `cargo test -p eaml-lexer` | `template.rs` (12 tests) | ✅ green |
| 01-02-06 | 02 | 2 | LEX-06 | unit+snapshot | `cargo test -p eaml-lexer` | `python_bridge.rs` (7 tests) | ✅ green |
| 01-02-07 | 02 | 2 | LEX-07 | unit | `cargo test -p eaml-lexer` | `literals.rs` (same_spur), `task1_raw_tokens.rs` (interner tests) | ✅ green |
| 01-02-08 | 02 | 2 | LEX-08 | unit | `cargo test -p eaml-lexer` | `comments.rs` (10 tests) | ✅ green |
| 01-02-09 | 02 | 2 | LEX-09 | unit+snapshot | `cargo test -p eaml-lexer` | `errors.rs` (7 tests), `comments.rs` (unexpected_char) | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [x] `crates/eaml-errors/tests/` — tests for ERR-01 through ERR-04 (24 tests)
- [x] `crates/eaml-lexer/tests/` — tests for LEX-01 through LEX-09 (98 tests)
- [x] insta snapshot infrastructure in workspace Cargo.toml

*All wave 0 requirements satisfied.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Colored error output | ERR-03 | Terminal rendering | Run `eamlc compile bad.eaml` and visually verify colored output |

---

## Validation Sign-Off

- [x] All tasks have automated verify commands
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all references
- [x] No watch-mode flags
- [x] Feedback latency < 5s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** validated 2026-03-15

---

## Validation Audit 2026-03-15

| Metric | Count |
|--------|-------|
| Gaps found | 0 |
| Resolved | 0 |
| Escalated | 0 |
| Total requirements | 13 |
| Total automated tests | 122 |
| Coverage | 100% |
