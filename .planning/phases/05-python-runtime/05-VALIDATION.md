---
phase: 5
slug: python-runtime
status: approved
nyquist_compliant: true
wave_0_complete: true
created: 2026-03-16
updated: 2026-03-17
---

# Phase 5 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | pytest 7.x + pytest-asyncio |
| **Config file** | `python/pyproject.toml` |
| **Quick run command** | `cd python && uv run pytest tests/ -x -q` |
| **Full suite command** | `cd python && uv run pytest tests/ -v` |
| **Estimated runtime** | ~1 second |

---

## Sampling Rate

- **After every task commit:** Run `cd python && uv run pytest tests/ -x -q`
- **After every plan wave:** Run `cd python && uv run pytest tests/ -v`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 1 second

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 05-01-T1 | 01 | 1 | RUN-01, RUN-02, RUN-03, RUN-06, RUN-07, RUN-08 | unit | `cd python && uv run pytest tests/test_providers.py tests/test_errors.py -x` | ✅ | ✅ green |
| 05-01-T2 | 01 | 1 | RUN-05 | unit | `cd python && uv run pytest tests/test_telemetry.py -x` | ✅ | ✅ green |
| 05-02-T1 | 02 | 2 | RUN-04, RUN-05 | unit | `cd python && uv run pytest tests/test_validation.py tests/test_execute_prompt.py -x` | ✅ | ✅ green |
| 05-02-T2 | 02 | 2 | RUN-04, RUN-05 | unit | `cd python && uv run pytest tests/test_validation.py tests/test_execute_prompt.py -x` | ✅ | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Requirement Coverage

| Requirement | Description | Test Files | Test Count | Status |
|-------------|-------------|------------|------------|--------|
| RUN-01 | Anthropic provider adapter | test_providers.py | 5 | COVERED |
| RUN-02 | OpenAI provider adapter | test_providers.py | 3 | COVERED |
| RUN-03 | Ollama provider adapter | test_providers.py | 3 | COVERED |
| RUN-04 | validate_or_retry | test_validation.py | 15 | COVERED |
| RUN-05 | Telemetry hooks | test_telemetry.py, test_validation.py, test_execute_prompt.py | 12 | COVERED |
| RUN-06 | Provider selection | test_providers.py | 5 | COVERED |
| RUN-07 | API keys from env | test_providers.py | 2 | COVERED |
| RUN-08 | Provider error handling | test_providers.py, test_execute_prompt.py | 5 | COVERED |

**Total: 67 automated tests across 6 test files. All 8 requirements covered.**

---

## Wave 0 Requirements

- [x] `python/tests/conftest.py` — shared fixtures (mock providers, sample schemas)
- [x] `python/tests/helpers.py` — MockProvider, ErrorProvider, Greeting model
- [x] `python/tests/test_providers.py` — 18 tests for RUN-01, RUN-02, RUN-03, RUN-06, RUN-07, RUN-08
- [x] `python/tests/test_validation.py` — 17 tests for RUN-04, RUN-05
- [x] `python/tests/test_telemetry.py` — 8 tests for RUN-05
- [x] `python/tests/test_execute_prompt.py` — 13 tests for RUN-04, RUN-05, RUN-08
- [x] `python/tests/test_errors.py` — 10 tests for error hierarchy

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Real LLM provider call | RUN-01 | Requires API keys and live network | Set ANTHROPIC_API_KEY, run `uv run pytest tests/test_live.py -m live` |

*All other behaviors have automated verification.*

---

## Validation Audit 2026-03-17

| Metric | Count |
|--------|-------|
| Gaps found | 0 |
| Resolved | 0 |
| Escalated | 0 |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 1s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-03-17
