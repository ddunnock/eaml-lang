---
phase: 05-python-runtime
plan: 02
subsystem: runtime
tags: [pydantic, validation, retry, telemetry, agent, async]

requires:
  - phase: 05-01
    provides: "errors, events, telemetry, providers foundation"
provides:
  - "execute_prompt entry point for generated code"
  - "validate_or_retry with Pydantic and primitive type validation"
  - "Agent base class for generated agent subclasses"
  - "ToolMetadata dataclass for tool declarations"
  - "Public API surface via __init__.py"
affects: [06-integration]

tech-stack:
  added: []
  patterns: [retry-with-feedback, provider-dispatch, primitive-validation]

key-files:
  created:
    - python/src/eaml_runtime/validation.py
    - python/src/eaml_runtime/agent.py
    - python/tests/test_validation.py
    - python/tests/test_execute_prompt.py
  modified:
    - python/src/eaml_runtime/__init__.py

key-decisions:
  - "int-to-float coercion in _validate_primitive for json.loads returning int for whole numbers"
  - "Provider errors bubble through validate_or_retry unmodified; execute_prompt wraps non-EamlError in EamlProviderError"

patterns-established:
  - "Retry pattern: append error feedback as user message for LLM self-correction"
  - "Primitive vs BaseModel dispatch: _is_primitive check before validation path selection"

requirements-completed: [RUN-04, RUN-05]

duration: 3min
completed: 2026-03-17
---

# Phase 5 Plan 2: Orchestration Layer Summary

**execute_prompt pipeline with validate_or_retry, primitive/Literal/BaseModel dispatch, Agent base class, and ToolMetadata dataclass**

## Performance

- **Duration:** 3 min
- **Started:** 2026-03-17T12:54:05Z
- **Completed:** 2026-03-17T12:57:21Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments
- execute_prompt dispatches to correct provider, validates response, returns typed result with telemetry
- validate_or_retry handles BaseModel (model_validate_json), primitives (json.loads), and Literal types
- Retry loop appends error feedback as user message for LLM self-correction
- Agent base class and ToolMetadata dataclass ready for generated code subclassing
- Public API: `from eaml_runtime import execute_prompt, Agent, ToolMetadata, configure` works
- 64 total tests passing across the runtime (27 new in this plan)

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement validate_or_retry, execute_prompt, Agent, ToolMetadata, and public API** - `d3eb1e9` (feat)
2. **Task 2: Write tests for validation, execute_prompt pipeline, and Agent/ToolMetadata** - `322e704` (test)

## Files Created/Modified
- `python/src/eaml_runtime/validation.py` - validate_or_retry and execute_prompt with primitive/BaseModel dispatch
- `python/src/eaml_runtime/agent.py` - Agent base class and ToolMetadata dataclass
- `python/src/eaml_runtime/__init__.py` - Public API re-exports
- `python/tests/test_validation.py` - 15 tests for validate_or_retry
- `python/tests/test_execute_prompt.py` - 12 tests for execute_prompt, Agent, ToolMetadata

## Decisions Made
- int-to-float coercion in `_validate_primitive` since json.loads returns int for whole numbers like `3` even when float is expected
- Provider errors bubble through validate_or_retry unmodified; only execute_prompt wraps non-EamlError exceptions in EamlProviderError
- Removed unused BaseModel import from validation.py (only ValidationError needed for catch clause)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Removed unused BaseModel import**
- **Found during:** Task 1
- **Issue:** ruff flagged `BaseModel` as unused import (only `ValidationError` needed)
- **Fix:** Removed `BaseModel` from import, kept `ValidationError`
- **Files modified:** python/src/eaml_runtime/validation.py
- **Verification:** ruff check passes
- **Committed in:** d3eb1e9

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Trivial cleanup, no scope change.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Python runtime is complete: errors, events, telemetry, providers, validation, agent, public API
- Phase 6 integration testing can import and use the full runtime
- Generated code pattern `from eaml_runtime import execute_prompt` is functional

## Self-Check: PASSED

All 5 files verified present. Both task commits (d3eb1e9, 322e704) found in git log.

---
*Phase: 05-python-runtime*
*Completed: 2026-03-17*
