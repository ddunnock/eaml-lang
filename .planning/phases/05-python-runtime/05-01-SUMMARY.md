---
phase: 05-python-runtime
plan: 01
subsystem: runtime
tags: [python, pydantic, anthropic, openai, ollama, httpx, async, telemetry]

requires:
  - phase: 04-code-generation
    provides: "API contract (execute_prompt, Agent, ToolMetadata imports in generated code)"
provides:
  - "EamlError hierarchy (EamlConfigError, EamlValidationError, EamlProviderError)"
  - "Telemetry event dataclasses and global hook registry (configure/_fire)"
  - "Provider ABC with get_provider() factory and caching"
  - "AnthropicProvider, OpenAIProvider, OllamaProvider adapters"
affects: [05-02-orchestration, 06-cli-integration]

tech-stack:
  added: []
  patterns:
    - "Provider ABC with lazy SDK import and singleton caching"
    - "Telemetry hooks: sync callbacks with exception swallowing via warnings.warn"
    - "Model ID prefix stripping (split on '/' take last part)"
    - "Anthropic system message extraction to top-level param"

key-files:
  created:
    - python/src/eaml_runtime/errors.py
    - python/src/eaml_runtime/events.py
    - python/src/eaml_runtime/telemetry.py
    - python/src/eaml_runtime/providers/anthropic.py
    - python/src/eaml_runtime/providers/openai.py
    - python/src/eaml_runtime/providers/ollama.py
    - python/tests/conftest.py
    - python/tests/test_errors.py
    - python/tests/test_telemetry.py
    - python/tests/test_providers.py
  modified:
    - python/src/eaml_runtime/providers/__init__.py

key-decisions:
  - "Provider _client typed as Any to avoid requiring SDK type stubs at import time"
  - "telemetry._reset() helper added for test cleanup alongside clear_provider_cache()"

patterns-established:
  - "Provider pattern: lazy SDK import in _get_client(), API key check at call time, error wrapping in EamlProviderError"
  - "Test pattern: inject mock client via provider._client, use AsyncMock for async SDK methods"
  - "Autouse fixture pattern: clear_provider_cache() and reset_telemetry() after each test"

requirements-completed: [RUN-01, RUN-02, RUN-03, RUN-05, RUN-06, RUN-07, RUN-08]

duration: 4min
completed: 2026-03-17
---

# Phase 5 Plan 1: Runtime Foundation Summary

**Error hierarchy, telemetry hook system, and three LLM provider adapters (Anthropic/OpenAI/Ollama) with lazy SDK loading and cached clients**

## Performance

- **Duration:** 4 min
- **Started:** 2026-03-17T12:46:33Z
- **Completed:** 2026-03-17T12:50:33Z
- **Tasks:** 2
- **Files modified:** 11

## Accomplishments
- Error hierarchy with EamlError base, EamlConfigError, EamlValidationError (with model_id/attempts/errors), EamlProviderError (with provider/status_code)
- Telemetry event dataclasses (CallStartEvent, CallEndEvent, ValidationFailureEvent, ToolCallEvent) and global configure()/_fire() hook system that swallows exceptions
- Provider ABC with get_provider() factory, singleton caching, and clear_provider_cache() for tests
- Three provider adapters: Anthropic (system param extraction, JSON instruction), OpenAI (response_format json_object), Ollama (httpx POST to /v1/chat/completions)
- 37 tests covering error hierarchy, telemetry hooks, provider selection, API key handling, and mocked SDK calls

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement error hierarchy, telemetry system, and all provider adapters** - `86cabc3` (feat)
2. **Task 2: Write comprehensive tests for errors, telemetry, and all providers** - `ffd4f0b` (test)

## Files Created/Modified
- `python/src/eaml_runtime/errors.py` - EamlError, EamlConfigError, EamlValidationError, EamlProviderError
- `python/src/eaml_runtime/events.py` - CallStartEvent, CallEndEvent, ValidationFailureEvent, ToolCallEvent dataclasses
- `python/src/eaml_runtime/telemetry.py` - Global hook registry with configure(), _fire(), _reset()
- `python/src/eaml_runtime/providers/__init__.py` - Provider ABC, get_provider() factory, _provider_cache
- `python/src/eaml_runtime/providers/anthropic.py` - AnthropicProvider with system message extraction
- `python/src/eaml_runtime/providers/openai.py` - OpenAIProvider with response_format json_object
- `python/src/eaml_runtime/providers/ollama.py` - OllamaProvider via httpx to localhost:11434
- `python/tests/conftest.py` - MockProvider, sample fixtures, autouse cleanup
- `python/tests/test_errors.py` - 10 tests for error hierarchy
- `python/tests/test_telemetry.py` - 8 tests for hook system
- `python/tests/test_providers.py` - 18 tests for providers (mocked SDKs)

## Decisions Made
- Provider `_client` attribute typed as `Any` to avoid requiring SDK type stubs at import time while still passing strict mypy
- Added `telemetry._reset()` helper for test cleanup alongside `clear_provider_cache()`
- Anthropic provider appends JSON instruction even when no system messages present (belt-and-suspenders)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed unused import in test_providers.py**
- **Found during:** Task 2 (test writing)
- **Issue:** `unittest.mock.patch` was imported but not used, causing ruff F401 failure
- **Fix:** Removed unused `patch` from the import statement
- **Files modified:** python/tests/test_providers.py
- **Verification:** `make check` passes (ruff clean)
- **Committed in:** ffd4f0b (part of task commit)

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Trivial lint fix. No scope creep.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Error hierarchy, telemetry hooks, and all three provider adapters ready for Plan 02
- Plan 02 will build validate_or_retry, execute_prompt, Agent base class, and ToolMetadata on top of this foundation
- MockProvider in conftest.py ready for validation/retry testing

---
*Phase: 05-python-runtime*
*Completed: 2026-03-17*
