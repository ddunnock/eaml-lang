# Phase 5: Python Runtime - Research

**Researched:** 2026-03-16
**Domain:** Python async runtime for LLM provider integration with Pydantic validation
**Confidence:** HIGH

## Summary

Phase 5 implements the Python runtime library (`eaml_runtime`) that generated EAML code calls into. The runtime must provide three public symbols (`execute_prompt`, `Agent`, `ToolMetadata`), three provider adapters (Anthropic, OpenAI, Ollama), validation-with-retry logic, telemetry hooks, and a clean error hierarchy.

The API contract is fully defined by Phase 4 codegen snapshots. Generated code calls `await execute_prompt(model=dict, messages=list, return_type=Type, temperature=float, max_tokens=int, max_retries=int)` and extends `Agent` base class with `model`, `tools`, `system_prompt`, `max_turns`, `on_error` attributes. All three providers use JSON mode (not structured outputs) with `model_validate_json()` for Pydantic parsing.

The project already has `python/pyproject.toml` with all dependencies declared (`anthropic>=0.43`, `openai>=1.0`, `pydantic>=2.0`, `httpx>=0.25`), pytest-asyncio configured, strict mypy, and ruff. The runtime package layout at `python/src/eaml_runtime/` exists as a stub.

**Primary recommendation:** Implement bottom-up: errors -> providers -> validate_or_retry -> execute_prompt -> telemetry -> Agent. Each layer is independently testable with mocks.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Abstract base class `Provider` with `send_prompt()` method
- Three concrete implementations: `AnthropicProvider`, `OpenAIProvider`, `OllamaProvider`
- Provider clients cached per-provider (singleton pattern) for connection pooling
- Provider selection based on `model["provider"]` field from codegen config dicts
- Anthropic and OpenAI use their official async SDKs (`anthropic.AsyncAnthropic`, `openai.AsyncOpenAI`)
- Ollama uses httpx directly against `http://localhost:11434/v1/chat/completions` (OpenAI-compatible endpoint)
- Ollama base URL overridable via `OLLAMA_BASE_URL` environment variable
- JSON mode + Pydantic parse: ask LLM for JSON output via provider-specific flags, then `return_type.model_validate_json(raw_json)`
- Consistent JSON mode across all three providers
- Default `max_retries` = 3 (1 initial + 2 retries) when EAML source doesn't specify
- On validation failure, append the Pydantic error as a follow-up user message so the LLM can self-correct
- validate_or_retry only retries on Pydantic validation failures -- provider errors bubble up immediately
- After all retries exhausted: raise `EamlValidationError` with model name, attempt count, last raw response, and all validation error messages
- Global `eaml_runtime.configure(on_call_start=fn, on_call_end=fn, ...)` registration
- Hooks are sync-only plain functions (no async hooks)
- Typed dataclass per event: `CallStartEvent`, `CallEndEvent`, `ValidationFailureEvent`, `ToolCallEvent`
- Events carry provider, model_id, timestamps, duration, token_usage (when available), error details
- Hook exceptions swallowed with `warnings.warn()` -- telemetry never breaks business logic
- API keys checked at call time (not import time)
- Environment variables: `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, `OLLAMA_BASE_URL`
- Flat exception hierarchy under `EamlError`: `EamlConfigError`, `EamlValidationError`, `EamlProviderError`
- Provider SDK exceptions wrapped in `EamlProviderError`

### Claude's Discretion
- Internal module layout within `eaml_runtime/` (how to split files)
- Provider client caching mechanism (module-level dict vs class attribute)
- Exact JSON mode flags per provider (system prompt injection vs API parameter)
- Token usage extraction format (varies by provider response shape)
- `Agent` base class orchestration loop implementation details
- `ToolMetadata` dataclass field design

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| RUN-01 | Anthropic provider adapter calls Claude API with correct message format | Anthropic SDK `AsyncAnthropic.messages.create()` with system as top-level param, JSON mode via system prompt instruction |
| RUN-02 | OpenAI provider adapter calls GPT API with correct message format | OpenAI SDK `AsyncOpenAI.chat.completions.create()` with `response_format={"type": "json_object"}` |
| RUN-03 | Ollama provider adapter calls local API via httpx | httpx POST to `localhost:11434/v1/chat/completions` with OpenAI-compatible format and `response_format` |
| RUN-04 | validate_or_retry validates LLM responses against Pydantic models and retries on failure | Pydantic v2 `model_validate_json()` with `ValidationError` catch, retry with error feedback in messages |
| RUN-05 | Telemetry hooks fire on call_start, call_end, tool_call, validation_failure events | Global configure() with typed dataclass events, sync callbacks, exception swallowing |
| RUN-06 | Provider selection based on model declaration's provider field | Registry dict mapping `"anthropic"/"openai"/"ollama"` to provider class, instantiate from `model["provider"]` |
| RUN-07 | Runtime reads API keys from environment variables | `os.environ.get("ANTHROPIC_API_KEY")` at call time, `EamlConfigError` if missing |
| RUN-08 | Runtime handles provider errors gracefully with clear error messages | Wrap `anthropic.APIError`, `openai.APIError`, `httpx.HTTPError` in `EamlProviderError` |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| anthropic | >=0.43 | Anthropic Claude API client | Official SDK, already in pyproject.toml |
| openai | >=1.0 | OpenAI GPT API client | Official SDK, already in pyproject.toml |
| pydantic | >=2.0 | Response validation and schema models | Already used by generated code, `model_validate_json()` |
| httpx | >=0.25 | HTTP client for Ollama | Already in pyproject.toml, async support |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| pytest | latest | Test framework | All unit tests |
| pytest-asyncio | latest | Async test support | Testing async provider calls |
| ruff | latest | Linting + formatting | `make check` |
| mypy | latest | Static type checking | `make check` (strict mode) |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| httpx for Ollama | openai SDK pointed at Ollama | Would add OpenAI SDK dependency to Ollama path; httpx is simpler and already a dependency |
| model_validate_json() | model_validate(json.loads()) | model_validate_json is faster (skips intermediate dict) and is the Pydantic v2 recommended approach |

**Installation:** Already declared in `python/pyproject.toml`. Run `cd python && uv pip install -e ".[dev]"`.

## Architecture Patterns

### Recommended Module Layout
```
python/src/eaml_runtime/
├── __init__.py          # Public API: execute_prompt, Agent, ToolMetadata, configure
├── errors.py            # EamlError hierarchy
├── events.py            # Telemetry event dataclasses
├── telemetry.py         # Global hook registry + fire_event helpers
├── validation.py        # validate_or_retry logic
├── providers/
│   ├── __init__.py      # Provider ABC + get_provider() factory
│   ├── anthropic.py     # AnthropicProvider
│   ├── openai.py        # OpenAIProvider
│   └── ollama.py        # OllamaProvider
└── agent.py             # Agent base class + ToolMetadata
```

### Pattern 1: Provider Abstract Base Class
**What:** ABC with async `send_prompt()` method, concrete implementations per provider
**When to use:** All LLM calls go through this interface
**Example:**
```python
from abc import ABC, abstractmethod
from typing import Any

class Provider(ABC):
    @abstractmethod
    async def send_prompt(
        self,
        messages: list[dict[str, str]],
        model_id: str,
        *,
        temperature: float | None = None,
        max_tokens: int | None = None,
    ) -> str:
        """Send messages to LLM and return raw JSON string response."""
        ...
```

### Pattern 2: Provider Factory with Caching
**What:** Module-level dict caching provider instances by provider name
**When to use:** `execute_prompt()` calls `get_provider(model["provider"])` to get or create a provider
**Example:**
```python
_provider_cache: dict[str, Provider] = {}

def get_provider(provider_name: str) -> Provider:
    if provider_name not in _provider_cache:
        if provider_name == "anthropic":
            _provider_cache[provider_name] = AnthropicProvider()
        elif provider_name == "openai":
            _provider_cache[provider_name] = OpenAIProvider()
        elif provider_name == "ollama":
            _provider_cache[provider_name] = OllamaProvider()
        else:
            raise EamlConfigError(f"Unknown provider: {provider_name}")
    return _provider_cache[provider_name]
```

### Pattern 3: Validation with Retry and Error Feedback
**What:** Try Pydantic parse, on failure append error to messages and retry
**When to use:** Core of `execute_prompt()` after each LLM call
**Example:**
```python
async def validate_or_retry(
    provider: Provider,
    messages: list[dict[str, str]],
    model_id: str,
    return_type: type[BaseModel],
    max_retries: int = 3,
    **kwargs: Any,
) -> BaseModel:
    errors: list[str] = []
    raw = ""
    for attempt in range(max_retries):
        raw = await provider.send_prompt(messages, model_id, **kwargs)
        try:
            return return_type.model_validate_json(raw)
        except ValidationError as e:
            error_msg = str(e)
            errors.append(error_msg)
            # Append error feedback for LLM self-correction
            messages = [*messages, {"role": "user", "content": f"Your response was not valid JSON matching the schema. Error: {error_msg}\nPlease try again with valid JSON."}]
    raise EamlValidationError(
        model_id=model_id,
        attempts=max_retries,
        last_response=raw,
        errors=errors,
    )
```

### Pattern 4: Telemetry Hook Registry
**What:** Global mutable state with typed callbacks, fire-and-forget with exception swallowing
**When to use:** Wrap every `execute_prompt()` call with telemetry events
**Example:**
```python
import warnings
from typing import Callable

_hooks: dict[str, Callable[..., None] | None] = {
    "on_call_start": None,
    "on_call_end": None,
    "on_validation_failure": None,
    "on_tool_call": None,
}

def configure(**kwargs: Callable[..., None] | None) -> None:
    for key, value in kwargs.items():
        if key in _hooks:
            _hooks[key] = value

def _fire(event_name: str, event: object) -> None:
    hook = _hooks.get(event_name)
    if hook is not None:
        try:
            hook(event)
        except Exception as exc:
            warnings.warn(f"Telemetry hook {event_name} raised: {exc}", stacklevel=2)
```

### Anti-Patterns to Avoid
- **Importing provider SDKs at module level unconditionally:** Only import `anthropic`/`openai` inside the provider class methods. Users who only use Ollama should not need Anthropic/OpenAI packages installed.
- **Checking API keys at import time:** Keys must be checked at call time per locked decision. A user importing the runtime should not get errors until they actually call a provider.
- **Making telemetry hooks async:** Locked decision says sync-only. Async hooks would require the caller to await them, adding complexity.
- **Catching all exceptions in validate_or_retry:** Only catch `pydantic.ValidationError`. Provider errors (rate limits, network) must bubble up immediately as `EamlProviderError`.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| JSON parsing/validation | Custom JSON validator | `pydantic.BaseModel.model_validate_json()` | Handles nested types, unions, optionals, field constraints automatically |
| HTTP client for Ollama | `urllib` or `aiohttp` | `httpx.AsyncClient` | Already a dependency, connection pooling, proper async |
| API key management | Custom config loader | `os.environ.get()` + `EamlConfigError` | Simple, standard, no extra dependencies |
| Provider SDK message format | Custom HTTP to Claude/GPT APIs | `anthropic.AsyncAnthropic` / `openai.AsyncOpenAI` | Handle auth, retries, rate limiting, streaming internally |

**Key insight:** The runtime is a thin orchestration layer. Each provider SDK handles the heavy lifting (auth, retries, connection management). The runtime just dispatches, validates, and retries.

## Common Pitfalls

### Pitfall 1: Anthropic System Message Handling
**What goes wrong:** Anthropic API takes `system` as a top-level parameter, not as a message with role "system". Passing system content as a message causes errors or unexpected behavior.
**Why it happens:** OpenAI and Ollama use `{"role": "system", "content": "..."}` in the messages array, but Anthropic splits it out.
**How to avoid:** AnthropicProvider must extract system messages from the messages list and pass them via the `system` parameter to `messages.create()`.
**Warning signs:** Anthropic API errors about invalid message roles.

### Pitfall 2: JSON Extraction from LLM Responses
**What goes wrong:** LLMs sometimes wrap JSON in markdown code blocks (```json ... ```) or add preamble text before the JSON.
**Why it happens:** Even with JSON mode flags, some models/providers are inconsistent.
**How to avoid:** Add a simple JSON extraction helper that strips markdown code fences and finds the JSON object/array in the response. Try `model_validate_json(raw)` first, then fall back to extraction.
**Warning signs:** `ValidationError` on responses that visually contain valid JSON.

### Pitfall 3: Pydantic v2 model_validate_json vs model_validate
**What goes wrong:** Using `model_validate(json.loads(raw))` instead of `model_validate_json(raw)` loses Pydantic's optimized JSON parsing path.
**Why it happens:** Habit from Pydantic v1 where `parse_raw` was less common.
**How to avoid:** Always use `model_validate_json()` for JSON string input. Use `model_validate()` only for dict input.
**Warning signs:** Slightly slower validation, but functionally equivalent.

### Pitfall 4: Primitive Return Types (str, int, float, bool)
**What goes wrong:** `model_validate_json()` only works on BaseModel subclasses. Generated code can have `return_type=str` or `return_type=Literal["a", "b"]`.
**Why it happens:** Codegen snapshot `examples__generate_all_type_variants.snap` shows `return_type=str` and `return_type=Literal[...]`.
**How to avoid:** `execute_prompt()` must handle two cases: (1) BaseModel subclass -> `model_validate_json()`, (2) primitive/Literal -> `json.loads()` + type coercion/validation.
**Warning signs:** `TypeError` when calling `str.model_validate_json()`.

### Pitfall 5: pytest-asyncio Strict Mode
**What goes wrong:** Tests fail with "no current event loop" or "test not marked as async".
**Why it happens:** pyproject.toml has `asyncio_mode = "strict"`, requiring explicit `@pytest.mark.asyncio` on every async test.
**How to avoid:** Always decorate async test functions with `@pytest.mark.asyncio`.
**Warning signs:** Mysterious test collection errors.

### Pitfall 6: Ollama Response Format Differences
**What goes wrong:** Ollama's OpenAI-compatible endpoint may not fully support `response_format: {"type": "json_object"}` on all models.
**Why it happens:** Ollama's compatibility layer has gaps vs the real OpenAI API.
**How to avoid:** Also include a system prompt instruction like "Respond with valid JSON only." as a fallback. The `response_format` parameter is supported but belt-and-suspenders is safer.
**Warning signs:** Ollama returning non-JSON responses despite format parameter.

## Code Examples

### Anthropic Provider Implementation
```python
# Source: Anthropic SDK docs + CONTEXT.md locked decisions
import os
from typing import Any

class AnthropicProvider(Provider):
    _client: "anthropic.AsyncAnthropic | None" = None

    def _get_client(self) -> "anthropic.AsyncAnthropic":
        if self._client is None:
            import anthropic
            api_key = os.environ.get("ANTHROPIC_API_KEY")
            if not api_key:
                raise EamlConfigError(
                    "ANTHROPIC_API_KEY environment variable is not set. "
                    "Set it with: export ANTHROPIC_API_KEY=your-key"
                )
            self._client = anthropic.AsyncAnthropic(api_key=api_key)
        return self._client

    async def send_prompt(
        self,
        messages: list[dict[str, str]],
        model_id: str,
        *,
        temperature: float | None = None,
        max_tokens: int | None = None,
    ) -> str:
        client = self._get_client()
        # Extract system messages (Anthropic uses top-level system param)
        system_parts = [m["content"] for m in messages if m["role"] == "system"]
        non_system = [m for m in messages if m["role"] != "system"]

        # Strip provider prefix from model_id (e.g., "anthropic/claude-3..." -> "claude-3...")
        model_name = model_id.split("/", 1)[-1] if "/" in model_id else model_id

        kwargs: dict[str, Any] = {
            "model": model_name,
            "messages": non_system,
            "max_tokens": max_tokens or 4096,
        }
        if system_parts:
            # Add JSON instruction to system prompt
            system_text = "\n".join(system_parts)
            kwargs["system"] = system_text + "\n\nRespond with valid JSON only."
        else:
            kwargs["system"] = "Respond with valid JSON only."

        if temperature is not None:
            kwargs["temperature"] = temperature

        response = await client.messages.create(**kwargs)
        return response.content[0].text
```

### OpenAI Provider Implementation
```python
# Source: OpenAI SDK docs + CONTEXT.md locked decisions
class OpenAIProvider(Provider):
    _client: "openai.AsyncOpenAI | None" = None

    def _get_client(self) -> "openai.AsyncOpenAI":
        if self._client is None:
            import openai
            api_key = os.environ.get("OPENAI_API_KEY")
            if not api_key:
                raise EamlConfigError(
                    "OPENAI_API_KEY environment variable is not set. "
                    "Set it with: export OPENAI_API_KEY=your-key"
                )
            self._client = openai.AsyncOpenAI(api_key=api_key)
        return self._client

    async def send_prompt(
        self,
        messages: list[dict[str, str]],
        model_id: str,
        *,
        temperature: float | None = None,
        max_tokens: int | None = None,
    ) -> str:
        client = self._get_client()
        model_name = model_id.split("/", 1)[-1] if "/" in model_id else model_id

        kwargs: dict[str, Any] = {
            "model": model_name,
            "messages": messages,  # OpenAI supports system role in messages
            "response_format": {"type": "json_object"},
        }
        if temperature is not None:
            kwargs["temperature"] = temperature
        if max_tokens is not None:
            kwargs["max_tokens"] = max_tokens

        response = await client.chat.completions.create(**kwargs)
        return response.choices[0].message.content or ""
```

### Ollama Provider Implementation
```python
# Source: Ollama docs + CONTEXT.md locked decisions
class OllamaProvider(Provider):
    _client: "httpx.AsyncClient | None" = None

    def _get_client(self) -> "httpx.AsyncClient":
        if self._client is None:
            import httpx
            self._client = httpx.AsyncClient(timeout=120.0)
        return self._client

    def _base_url(self) -> str:
        return os.environ.get("OLLAMA_BASE_URL", "http://localhost:11434")

    async def send_prompt(
        self,
        messages: list[dict[str, str]],
        model_id: str,
        *,
        temperature: float | None = None,
        max_tokens: int | None = None,
    ) -> str:
        client = self._get_client()
        model_name = model_id.split("/", 1)[-1] if "/" in model_id else model_id

        payload: dict[str, Any] = {
            "model": model_name,
            "messages": messages,
            "response_format": {"type": "json_object"},
            "stream": False,
        }
        if temperature is not None:
            payload["temperature"] = temperature
        if max_tokens is not None:
            payload["max_tokens"] = max_tokens

        url = f"{self._base_url()}/v1/chat/completions"
        resp = await client.post(url, json=payload)
        resp.raise_for_status()
        data = resp.json()
        return data["choices"][0]["message"]["content"]
```

### execute_prompt Entry Point
```python
# Source: Codegen snapshots define the call signature
async def execute_prompt(
    *,
    model: dict[str, Any],
    messages: list[dict[str, str]],
    return_type: type,
    temperature: float | None = None,
    max_tokens: int | None = None,
    max_retries: int = 3,
) -> Any:
    provider_name = model["provider"]
    model_id = model["model_id"]
    provider = get_provider(provider_name)

    _fire("on_call_start", CallStartEvent(provider=provider_name, model_id=model_id))
    start = time.time()
    try:
        result = await validate_or_retry(
            provider, messages, model_id, return_type,
            max_retries=max_retries,
            temperature=temperature, max_tokens=max_tokens,
        )
        duration = time.time() - start
        _fire("on_call_end", CallEndEvent(provider=provider_name, model_id=model_id, duration=duration))
        return result
    except EamlError:
        raise
    except Exception as exc:
        raise EamlProviderError(provider=provider_name, message=str(exc)) from exc
```

### Error Hierarchy
```python
class EamlError(Exception):
    """Base exception for all EAML runtime errors."""

class EamlConfigError(EamlError):
    """Missing API keys or invalid configuration."""

class EamlValidationError(EamlError):
    """LLM output failed Pydantic validation after all retries."""
    def __init__(self, model_id: str, attempts: int, last_response: str, errors: list[str]):
        self.model_id = model_id
        self.attempts = attempts
        self.last_response = last_response
        self.errors = errors
        super().__init__(
            f"Validation failed after {attempts} attempts for model '{model_id}'. "
            f"Errors: {'; '.join(errors)}"
        )

class EamlProviderError(EamlError):
    """Provider API error (wrapped SDK exception)."""
    def __init__(self, provider: str, message: str, status_code: int | None = None):
        self.provider = provider
        self.status_code = status_code
        super().__init__(f"Provider '{provider}' error: {message}")
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Pydantic v1 `parse_raw()` | Pydantic v2 `model_validate_json()` | 2023 (Pydantic v2) | Faster, different API |
| Anthropic tool_use for JSON | Anthropic `output_config` structured outputs | 2025 (SDK v0.76+) | Native JSON schema enforcement -- but pyproject.toml pins >=0.43, so use system prompt + JSON instruction instead |
| OpenAI `json_object` format | OpenAI structured outputs with `json_schema` | 2024 | Schema enforcement -- but `json_object` is simpler and sufficient with Pydantic validation |
| sync provider clients | async provider clients | Standard | All three SDKs support async natively |

**Deprecated/outdated:**
- Pydantic v1 API (`parse_raw`, `parse_obj`) -- project requires v2
- `anthropic.Client` (sync) -- use `anthropic.AsyncAnthropic` per locked decision

## Open Questions

1. **Model ID prefix stripping**
   - What we know: Codegen emits model_id like `"anthropic/claude-3-5-sonnet-20241022"` with provider prefix
   - What's unclear: Should the provider strip the prefix, or should codegen not include it?
   - Recommendation: Provider strips prefix (split on `/`, take last part). This is a runtime concern and keeps codegen simple.

2. **Lazy SDK imports for optional providers**
   - What we know: Users may not have all three SDKs installed (e.g., only using Ollama)
   - What's unclear: Should providers fail at import time or call time?
   - Recommendation: Import SDKs inside provider methods (lazy). Raise `EamlConfigError` with install instructions if import fails.

3. **Agent orchestration loop**
   - What we know: Generated agent classes extend `Agent` with model, tools, system_prompt, max_turns, on_error
   - What's unclear: Exact run loop (how tools are dispatched, when to stop)
   - Recommendation: `Agent.run(input)` sends system prompt + input, checks for tool calls in response, dispatches to registered tool functions, loops until max_turns or no tool calls. This is Claude's discretion per CONTEXT.md.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | pytest + pytest-asyncio |
| Config file | `python/pyproject.toml` (asyncio_mode = "strict") |
| Quick run command | `cd python && uv run pytest tests/ -x` |
| Full suite command | `cd python && uv run pytest && uv run mypy src/ && uv run ruff check .` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| RUN-01 | Anthropic provider calls Claude API | unit (mocked) | `cd python && uv run pytest tests/test_providers.py::test_anthropic_provider -x` | Wave 0 |
| RUN-02 | OpenAI provider calls GPT API | unit (mocked) | `cd python && uv run pytest tests/test_providers.py::test_openai_provider -x` | Wave 0 |
| RUN-03 | Ollama provider calls local API | unit (mocked) | `cd python && uv run pytest tests/test_providers.py::test_ollama_provider -x` | Wave 0 |
| RUN-04 | validate_or_retry with retry logic | unit | `cd python && uv run pytest tests/test_validation.py -x` | Wave 0 |
| RUN-05 | Telemetry hooks fire events | unit | `cd python && uv run pytest tests/test_telemetry.py -x` | Wave 0 |
| RUN-06 | Provider selection from model dict | unit | `cd python && uv run pytest tests/test_providers.py::test_provider_selection -x` | Wave 0 |
| RUN-07 | API key reading from environment | unit | `cd python && uv run pytest tests/test_providers.py::test_api_key_handling -x` | Wave 0 |
| RUN-08 | Provider error wrapping | unit | `cd python && uv run pytest tests/test_errors.py -x` | Wave 0 |

### Sampling Rate
- **Per task commit:** `cd python && uv run pytest tests/ -x`
- **Per wave merge:** `cd python && uv run pytest && uv run mypy src/ && uv run ruff check .`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `python/tests/test_errors.py` -- covers error hierarchy (RUN-08)
- [ ] `python/tests/test_providers.py` -- covers provider adapters (RUN-01, RUN-02, RUN-03, RUN-06, RUN-07)
- [ ] `python/tests/test_validation.py` -- covers validate_or_retry (RUN-04)
- [ ] `python/tests/test_telemetry.py` -- covers hook system (RUN-05)
- [ ] `python/tests/test_execute_prompt.py` -- covers end-to-end execute_prompt (integration)
- [ ] `python/tests/conftest.py` -- shared fixtures (mock providers, sample models)

### Testing Strategy: Mocking Provider SDKs
All provider tests must mock the underlying SDK clients to avoid real API calls:
- Anthropic: mock `anthropic.AsyncAnthropic.messages.create`
- OpenAI: mock `openai.AsyncOpenAI.chat.completions.create`
- Ollama: mock `httpx.AsyncClient.post`

Use `unittest.mock.AsyncMock` for async method mocking. Create a `MockProvider(Provider)` for validation/retry tests that returns controlled responses.

## Sources

### Primary (HIGH confidence)
- Codegen snapshots (`examples__generate_*.snap`) -- exact API contract generated code expects
- `crates/eaml-codegen/src/emitters.rs` -- defines execute_prompt, Agent, ToolMetadata usage
- `crates/eaml-codegen/src/types.rs` -- ImportTracker defines exact import names
- `python/pyproject.toml` -- dependency versions and tool configuration
- CONTEXT.md -- locked design decisions

### Secondary (MEDIUM confidence)
- [Anthropic Structured Outputs docs](https://platform.claude.com/docs/en/build-with-claude/structured-outputs) -- JSON mode approach, system prompt pattern
- [OpenAI Structured Outputs docs](https://platform.openai.com/docs/guides/structured-outputs) -- response_format json_object parameter
- [Ollama OpenAI compatibility docs](https://docs.ollama.com/api/openai-compatibility) -- /v1/chat/completions endpoint, response_format support
- [Anthropic SDK releases](https://github.com/anthropics/anthropic-sdk-python/releases) -- output_config added in v0.76

### Tertiary (LOW confidence)
- Ollama JSON mode reliability across different models -- may vary, needs belt-and-suspenders approach

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - all dependencies already declared, versions pinned in pyproject.toml
- Architecture: HIGH - API contract fully defined by codegen snapshots, all design decisions locked
- Pitfalls: HIGH - well-known issues with provider API differences and Pydantic v2 API
- Provider specifics: MEDIUM - JSON mode flags verified via official docs, but exact behavior varies by model

**Research date:** 2026-03-16
**Valid until:** 2026-04-16 (stable domain, provider SDKs evolve slowly)
