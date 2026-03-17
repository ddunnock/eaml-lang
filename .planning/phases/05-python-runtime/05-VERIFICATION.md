---
phase: 05-python-runtime
verified: 2026-03-17T13:10:00Z
status: passed
score: 16/16 must-haves verified
re_verification: false
---

# Phase 5: Python Runtime Verification Report

**Phase Goal:** Generated Python code can actually execute, calling LLM providers and validating responses against Pydantic models
**Verified:** 2026-03-17T13:10:00Z
**Status:** PASSED
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| #  | Truth                                                                                                                     | Status     | Evidence                                                                                                   |
|----|---------------------------------------------------------------------------------------------------------------------------|------------|------------------------------------------------------------------------------------------------------------|
| 1  | Anthropic provider sends messages with system extracted to top-level param and JSON instruction appended                  | ✓ VERIFIED | `anthropic.py:48-65` — system_parts extracted, `kwargs["system"] = text + "\n\nRespond with valid JSON only."` |
| 2  | OpenAI provider sends messages with response_format json_object                                                           | ✓ VERIFIED | `openai.py:53` — `"response_format": {"type": "json_object"}` in kwargs                                   |
| 3  | Ollama provider sends messages via httpx to localhost:11434/v1/chat/completions                                           | ✓ VERIFIED | `ollama.py:61,64` — `url = f"{self._base_url()}/v1/chat/completions"`, `client.post(url, json=payload)`   |
| 4  | Provider selection maps 'anthropic'/'openai'/'ollama' strings to correct provider class                                   | ✓ VERIFIED | `providers/__init__.py:38-51` — three if/elif branches, `EamlConfigError` on unknown                      |
| 5  | Missing API keys raise EamlConfigError at call time with clear instructions                                               | ✓ VERIFIED | `anthropic.py:28-32` — "Set it with: export ANTHROPIC_API_KEY=your-key"; same pattern in openai.py        |
| 6  | Provider SDK exceptions are wrapped in EamlProviderError                                                                  | ✓ VERIFIED | All three providers have `except Exception as exc: raise EamlProviderError(provider=..., message=...)` blocks |
| 7  | Telemetry hooks fire events via global configure() and swallow exceptions                                                 | ✓ VERIFIED | `telemetry.py:16-38` — `configure()` sets hooks, `_fire()` catches hook exceptions with `warnings.warn()` |
| 8  | execute_prompt dispatches to correct provider, validates response, returns typed result                                   | ✓ VERIFIED | `validation.py:106-159` — extracts provider_name, calls get_provider(), calls validate_or_retry()         |
| 9  | validate_or_retry retries on Pydantic validation failure with error feedback appended to messages                         | ✓ VERIFIED | `validation.py:67-96` — loop appends `{"role": "user", "content": f"Your response was not valid. Error: {error_msg}..."}` |
| 10 | After max_retries exhausted, EamlValidationError raised with attempt count and all errors                                 | ✓ VERIFIED | `validation.py:98-103` — `raise EamlValidationError(model_id=..., attempts=max_retries, last_response=raw, errors=errors)` |
| 11 | Provider errors bubble up immediately as EamlProviderError without retry                                                  | ✓ VERIFIED | `validation.py:75` — only `(ValidationError, ValueError, json.JSONDecodeError)` caught; other exceptions pass through |
| 12 | Primitive return types (str, int, float, bool, Literal) handled via json.loads instead of model_validate_json            | ✓ VERIFIED | `validation.py:19-45` — `_is_primitive()` checks, `_validate_primitive()` uses `json.loads()`             |
| 13 | Telemetry events fire on call_start, call_end, and validation_failure during execute_prompt                               | ✓ VERIFIED | `validation.py:124,144-150` — CallStartEvent, CallEndEvent fired; `validation.py:79-87` — ValidationFailureEvent in loop |
| 14 | Agent base class has model, tools, system_prompt, max_turns, on_error attributes                                          | ✓ VERIFIED | `agent.py:20-32` — all five attributes with defaults                                                       |
| 15 | ToolMetadata dataclass has name, description, parameters, return_type, function fields                                    | ✓ VERIFIED | `agent.py:9-17` — `@dataclass class ToolMetadata` with all five fields                                    |
| 16 | Public API exports: execute_prompt, Agent, ToolMetadata, configure from eaml_runtime                                     | ✓ VERIFIED | `__init__.py:5-24` — all four imported and in `__all__`; 64/64 tests pass                                 |

**Score:** 16/16 truths verified

### Required Artifacts

| Artifact                                              | Expected                                        | Status     | Details                                                    |
|-------------------------------------------------------|-------------------------------------------------|------------|------------------------------------------------------------|
| `python/src/eaml_runtime/errors.py`                  | EamlError, EamlConfigError, EamlValidationError, EamlProviderError | ✓ VERIFIED | All four classes present with correct inheritance and attributes |
| `python/src/eaml_runtime/events.py`                  | CallStartEvent, CallEndEvent, ValidationFailureEvent, ToolCallEvent dataclasses | ✓ VERIFIED | All four `@dataclass` classes present with provider, model_id, timestamp fields |
| `python/src/eaml_runtime/telemetry.py`               | configure(), _fire() hook system                | ✓ VERIFIED | `configure(**kwargs)` and `_fire(event_name, event)` both present; `warnings.warn()` on failure |
| `python/src/eaml_runtime/providers/__init__.py`      | Provider ABC, get_provider() factory            | ✓ VERIFIED | `class Provider(ABC)`, `get_provider()`, `_provider_cache`, `clear_provider_cache()` |
| `python/src/eaml_runtime/providers/anthropic.py`     | AnthropicProvider                               | ✓ VERIFIED | `class AnthropicProvider(Provider)` with system extraction, JSON instruction, error wrapping |
| `python/src/eaml_runtime/providers/openai.py`        | OpenAIProvider                                  | ✓ VERIFIED | `class OpenAIProvider(Provider)` with response_format, OPENAI_API_KEY check |
| `python/src/eaml_runtime/providers/ollama.py`        | OllamaProvider                                  | ✓ VERIFIED | `class OllamaProvider(Provider)` with httpx, OLLAMA_BASE_URL, /v1/chat/completions |
| `python/src/eaml_runtime/validation.py`              | validate_or_retry and execute_prompt functions  | ✓ VERIFIED | Both async functions present with full retry logic, telemetry, and primitive dispatch |
| `python/src/eaml_runtime/agent.py`                   | Agent base class and ToolMetadata dataclass     | ✓ VERIFIED | Both present; Agent has 6 class attributes, ToolMetadata is a proper dataclass |
| `python/src/eaml_runtime/__init__.py`                | Public API surface                              | ✓ VERIFIED | Imports and re-exports execute_prompt, Agent, ToolMetadata, configure, and 4 error types |

### Key Link Verification

| From                                    | To                                     | Via                                            | Status     | Details                                                               |
|-----------------------------------------|----------------------------------------|------------------------------------------------|------------|-----------------------------------------------------------------------|
| `providers/__init__.py`                 | `providers/anthropic.py`               | get_provider factory imports AnthropicProvider | ✓ WIRED    | `from eaml_runtime.providers.anthropic import AnthropicProvider` at line 39 |
| `providers/anthropic.py`               | `errors.py`                            | raises EamlConfigError on missing key          | ✓ WIRED    | `raise EamlConfigError(...)` at lines 23-25 and 29-31               |
| `validation.py`                         | `providers/__init__.py`                | get_provider() call in execute_prompt          | ✓ WIRED    | `from eaml_runtime.providers import Provider, get_provider` + `get_provider(provider_name)` at line 122 |
| `validation.py`                         | `telemetry.py`                         | _fire() calls for telemetry events             | ✓ WIRED    | `from eaml_runtime.telemetry import _fire` + fired at lines 124, 79-87, 144-150 |
| `__init__.py`                           | `validation.py`                        | re-exports execute_prompt                      | ✓ WIRED    | `from eaml_runtime.validation import execute_prompt` at line 13      |
| `__init__.py`                           | `agent.py`                             | re-exports Agent, ToolMetadata                 | ✓ WIRED    | `from eaml_runtime.agent import Agent, ToolMetadata` at line 5       |

### Requirements Coverage

| Requirement | Source Plan | Description                                                                  | Status      | Evidence                                                                 |
|-------------|-------------|------------------------------------------------------------------------------|-------------|--------------------------------------------------------------------------|
| RUN-01      | 05-01       | Anthropic provider adapter calls Claude API with correct message format      | ✓ SATISFIED | `anthropic.py` sends via `client.messages.create()` with system param extracted; test `test_anthropic_send_prompt` passes |
| RUN-02      | 05-01       | OpenAI provider adapter calls GPT API with correct message format            | ✓ SATISFIED | `openai.py` sends via `client.chat.completions.create()` with `response_format`; test `test_openai_send_prompt` passes |
| RUN-03      | 05-01       | Ollama provider adapter calls local API via httpx                            | ✓ SATISFIED | `ollama.py` uses `httpx.AsyncClient.post()` to `/v1/chat/completions`; test `test_ollama_send_prompt` passes |
| RUN-04      | 05-02       | validate_or_retry validates LLM responses against Pydantic models and retries on failure | ✓ SATISFIED | `validation.py:48-103` — full retry loop with error feedback; 15 tests for `validate_or_retry` all pass |
| RUN-05      | 05-01, 05-02| Telemetry hooks fire on call_start, call_end, tool_call, validation_failure events | ✓ SATISFIED | All four event types in `events.py`; hooks fire in `validation.py` and `telemetry.py`; tests verify hook invocation |
| RUN-06      | 05-01       | Provider selection is based on model declaration's provider field            | ✓ SATISFIED | `execute_prompt` extracts `model["provider"]` then calls `get_provider(provider_name)` |
| RUN-07      | 05-01       | Runtime reads API keys from environment variables (ANTHROPIC_API_KEY, OPENAI_API_KEY) | ✓ SATISFIED | `os.environ.get("ANTHROPIC_API_KEY")` and `os.environ.get("OPENAI_API_KEY")`; missing key raises `EamlConfigError` with export instructions |
| RUN-08      | 05-01       | Runtime handles provider errors gracefully with clear error messages         | ✓ SATISFIED | All three providers wrap SDK exceptions in `EamlProviderError(provider=..., message=...)`; tests `test_*_error_wrapped` pass |

All 8 requirements are satisfied. No orphaned requirements detected.

### Anti-Patterns Found

None. No TODO/FIXME/PLACEHOLDER comments, empty return stubs, or stub implementations found in any runtime source files.

### Human Verification Required

#### 1. Live LLM API Call (Anthropic)

**Test:** Set `ANTHROPIC_API_KEY`, compile a minimal `.eaml` file, run the generated Python to call Claude and receive a validated Pydantic response.
**Expected:** A typed Python object matching the declared model is returned without error.
**Why human:** Requires live API credentials and actual Anthropic network call; cannot be verified offline.

#### 2. Live LLM API Call (OpenAI)

**Test:** Set `OPENAI_API_KEY`, compile a `.eaml` file targeting `"openai"`, run the generated Python against gpt-4o-mini.
**Expected:** Response deserialized correctly via `json_object` response format.
**Why human:** Requires live API credentials and OpenAI network call.

#### 3. Live Ollama Roundtrip

**Test:** Start a local Ollama instance, compile a `.eaml` file targeting `"ollama"`, run the generated Python.
**Expected:** Request sent to `localhost:11434/v1/chat/completions`, response validated against Pydantic model.
**Why human:** Requires a running Ollama daemon; cannot be verified without the external service.

#### 4. Retry Self-Correction in Practice

**Test:** Configure a model that initially returns invalid JSON, observe that the second or third attempt (with error feedback in the message list) succeeds.
**Expected:** The LLM reads the appended error message and corrects its output.
**Why human:** Actual LLM self-correction behavior depends on model quality; cannot be verified with mocks.

### Test Suite Summary

- **Total tests:** 64 (all passing, 0 failures)
- **Test distribution:** 10 error hierarchy + 8 telemetry + 18 providers + 15 validation + 12 execute_prompt/agent + 1 import smoke
- **Code quality:** ruff clean (`All checks passed!`), mypy strict clean (`no issues found in 10 source files`)
- **Commits verified:** 86cabc3 (feat 05-01), ffd4f0b (test 05-01), d3eb1e9 (feat 05-02), 322e704 (test 05-02) — all present in git log

### Gaps Summary

No gaps. All observable truths verified, all artifacts are substantive and wired, all key links confirmed in source, all 8 requirements satisfied, no anti-patterns detected, all 64 tests pass, ruff and mypy clean.

---

_Verified: 2026-03-17T13:10:00Z_
_Verifier: Claude (gsd-verifier)_
