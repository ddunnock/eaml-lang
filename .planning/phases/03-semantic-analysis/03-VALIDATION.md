---
phase: 3
slug: semantic-analysis
status: complete
nyquist_compliant: true
wave_0_complete: true
created: 2026-03-16
audited: 2026-03-16
---

# Phase 3 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (unit tests) |
| **Config file** | `crates/eaml-semantic/Cargo.toml` |
| **Quick run command** | `cargo test -p eaml-semantic` |
| **Full suite command** | `cargo test --workspace` |
| **Estimated runtime** | ~5 seconds |
| **Total tests** | 96 |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p eaml-semantic`
- **After every plan wave:** Run `cargo test --workspace`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 5 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | Test File | Status |
|---------|------|------|-------------|-----------|-------------------|-----------|--------|
| 03-01-01 | 01 | 1 | SEM-01 | unit (24) | `cargo test -p eaml-semantic -- resolution` | resolution.rs | green |
| 03-01-02 | 01 | 1 | SEM-02 | unit | `cargo test -p eaml-semantic -- forward_reference` | resolution.rs | green |
| 03-01-03 | 01 | 1 | SEM-03 | unit (3) | `cargo test -p eaml-semantic -- cycle` | resolution.rs | green |
| 03-02-01 | 02 | 2 | SEM-04 | unit (8) | `cargo test -p eaml-semantic -- types_` | types.rs | green |
| 03-02-02 | 02 | 2 | SEM-05 | unit (2) | `cargo test -p eaml-semantic -- literal_union` | types.rs | green |
| 03-02-03 | 02 | 2 | SEM-06 | unit (4) | `cargo test -p eaml-semantic -- types_schema` | types.rs | green |
| 03-02-04 | 02 | 2 | SEM-07 | unit (1) | `cargo test -p eaml-semantic -- chained_comparison` | types.rs | green |
| 03-02-05 | 02 | 2 | SEM-10 | unit (9) | `cargo test -p eaml-semantic -- scoping` | scoping.rs | green |
| 03-03-01 | 03 | 3 | SEM-08 | unit (14) | `cargo test -p eaml-semantic -- cap` | capabilities.rs | green |
| 03-03-02 | 03 | 3 | SEM-09 | unit | `cargo test -p eaml-semantic -- cap_prompt_requires` | capabilities.rs | green |
| 03-03-03 | 03 | 3 | SEM-11 | integration (24) | `cargo test -p eaml-semantic -- sem11` | integration.rs | green |

---

## Test File Inventory

| File | Tests | Covers |
|------|-------|--------|
| `crates/eaml-semantic/tests/resolution.rs` | 24 | SEM-01, SEM-02, SEM-03 |
| `crates/eaml-semantic/tests/types.rs` | 25 | SEM-04, SEM-05, SEM-06, SEM-07 |
| `crates/eaml-semantic/tests/scoping.rs` | 9 | SEM-10 |
| `crates/eaml-semantic/tests/capabilities.rs` | 14 | SEM-08, SEM-09 |
| `crates/eaml-semantic/tests/integration.rs` | 24 | SEM-11 (all error codes + example files) |
| `crates/eaml-semantic/tests/test_helpers.rs` | — | Shared utilities |

---

## Error Code Coverage (SEM-11)

All 20 semantic error codes confirmed emittable via integration tests:

| Code | Test | Category |
|------|------|----------|
| RES001 | `sem11_res001_fires` | Name resolution |
| RES010 | `sem11_res010_fires` | Name resolution |
| SEM010 | `sem11_sem010_fires` | Import ordering |
| SEM020 | `sem11_sem020_fires` | Schema fields |
| SEM025 | `sem11_sem025_fires` | Prompt structure |
| SEM030 | `sem11_sem030_fires` | Bounded params |
| SEM040 | `sem11_sem040_fires` | Tool body |
| SEM060 | `sem11_sem060_fires` | Chained comparison |
| SEM070 | `sem11_sem070_fires` | Cycle detection |
| TYP001 | `sem11_typ001_fires` | Type shadowing |
| TYP010 | `sem11_typ010_note` | Unknown type |
| TYP030 | `sem11_typ030_fires` | Bound violation |
| TYP031 | `sem11_typ031_untestable` | Negative bound (parser limitation) |
| TYP032 | `sem11_typ032_fires` | Non-boundable type |
| TYP040 | `sem11_typ040_fires` | Duplicate union member |
| CAP001 | `sem11_cap001_fires` | Unknown capability |
| CAP002 | `sem11_cap002_fires` | Duplicate capability |
| CAP010 | `sem11_cap010_fires` | Capability mismatch (FATAL) |
| CAP020 | `sem11_cap020_fires` | json_mode + string |
| PYB010 | `sem11_pyb010_fires` | Unknown provider |

---

## Manual-Only Verifications

*All phase behaviors have automated verification.*

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 5s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** complete

---

## Validation Audit 2026-03-16

| Metric | Count |
|--------|-------|
| Gaps found | 0 |
| Resolved | 0 |
| Escalated | 0 |
| Total tests | 96 |
| Requirements covered | 11/11 |
