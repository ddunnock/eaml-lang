---
phase: 5
slug: python-runtime
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-16
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
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cd python && uv run pytest tests/ -x -q`
- **After every plan wave:** Run `cd python && uv run pytest tests/ -v`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 05-01-01 | 01 | 1 | RUN-01 | unit | `cd python && uv run pytest tests/test_providers.py -x` | ❌ W0 | ⬜ pending |
| 05-01-02 | 01 | 1 | RUN-02 | unit | `cd python && uv run pytest tests/test_providers.py -x` | ❌ W0 | ⬜ pending |
| 05-01-03 | 01 | 1 | RUN-03 | unit | `cd python && uv run pytest tests/test_providers.py -x` | ❌ W0 | ⬜ pending |
| 05-02-01 | 02 | 1 | RUN-04 | unit | `cd python && uv run pytest tests/test_execute.py -x` | ❌ W0 | ⬜ pending |
| 05-02-02 | 02 | 1 | RUN-05 | unit | `cd python && uv run pytest tests/test_validation.py -x` | ❌ W0 | ⬜ pending |
| 05-02-03 | 02 | 1 | RUN-06 | unit | `cd python && uv run pytest tests/test_telemetry.py -x` | ❌ W0 | ⬜ pending |
| 05-03-01 | 03 | 2 | RUN-07 | integration | `cd python && uv run pytest tests/test_agent.py -x` | ❌ W0 | ⬜ pending |
| 05-03-02 | 03 | 2 | RUN-08 | integration | `cd python && uv run pytest tests/test_integration.py -x` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `python/tests/conftest.py` — shared fixtures (mock providers, sample schemas)
- [ ] `python/tests/test_providers.py` — stubs for RUN-01, RUN-02, RUN-03
- [ ] `python/tests/test_execute.py` — stubs for RUN-04
- [ ] `python/tests/test_validation.py` — stubs for RUN-05
- [ ] `python/tests/test_telemetry.py` — stubs for RUN-06
- [ ] `python/tests/test_agent.py` — stubs for RUN-07
- [ ] `python/tests/test_integration.py` — stubs for RUN-08

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Real LLM provider call | RUN-01 | Requires API keys and live network | Set ANTHROPIC_API_KEY, run `uv run pytest tests/test_live.py -m live` |
| Retry with real invalid LLM output | RUN-05 | Hard to trigger deterministically | Mock provider returns bad JSON, verify retry count |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
