---
phase: 6
slug: cli-and-integration
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-17
---

# Phase 6 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust) + pytest 7.x (Python) |
| **Config file** | `Cargo.toml` (workspace) + `python/pyproject.toml` |
| **Quick run command** | `cargo test -p eaml-cli` |
| **Full suite command** | `make test` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p eaml-cli`
- **After every plan wave:** Run `make test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 06-01-01 | 01 | 1 | CLI-01 | integration | `cargo test -p eaml-cli -- compile` | ❌ W0 | ⬜ pending |
| 06-01-02 | 01 | 1 | CLI-02 | integration | `cargo test -p eaml-cli -- check` | ❌ W0 | ⬜ pending |
| 06-01-03 | 01 | 1 | CLI-03 | integration | `cargo test -p eaml-cli -- error_display` | ❌ W0 | ⬜ pending |
| 06-01-04 | 01 | 1 | CLI-04 | integration | `cargo test -p eaml-cli -- exit_code` | ❌ W0 | ⬜ pending |
| 06-02-01 | 02 | 2 | INT-01 | integration | `cargo test -p eaml-cli -- examples` | ❌ W0 | ⬜ pending |
| 06-02-02 | 02 | 2 | INT-02, GEN-11 | snapshot | `cargo test -p eaml-cli -- mypy` | ❌ W0 | ⬜ pending |
| 06-02-03 | 02 | 2 | INT-03, GEN-12 | e2e | `cd python && uv run pytest tests/test_e2e.py` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/eaml-cli/tests/cli_tests.rs` — integration test stubs for CLI-01..CLI-04
- [ ] `crates/eaml-cli/tests/example_tests.rs` — example compilation tests for INT-01
- [ ] assert_cmd + predicates + tempfile in eaml-cli dev-dependencies

*Existing Rust test infrastructure and Python pytest cover foundations.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Colored error output | CLI-03 | Terminal color rendering | Run `eamlc check bad.eaml` in terminal, verify colored underlines |
| LLM API call e2e | GEN-12 | Requires API keys | Set ANTHROPIC_API_KEY, run generated sentiment.py, verify JSON output |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
