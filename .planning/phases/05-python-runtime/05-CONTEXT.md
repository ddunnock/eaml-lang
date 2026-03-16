# Phase 5: Python Runtime - Context

**Gathered:** 2026-03-16
**Status:** Ready for planning

<domain>
## Phase Boundary

Generated Python code can actually execute, calling LLM providers and validating responses against Pydantic models. Covers provider adapters (Anthropic, OpenAI, Ollama), validate_or_retry logic, telemetry hooks, and error handling. The CLI binary and end-to-end integration testing are Phase 6.

</domain>

<decisions>
## Implementation Decisions

### Provider adapter design
- Abstract base class `Provider` with `send_prompt()` method
- Three concrete implementations: `AnthropicProvider`, `OpenAIProvider`, `OllamaProvider`
- Provider clients cached per-provider (singleton pattern) for connection pooling
- Provider selection based on `model["provider"]` field from codegen config dicts
- Anthropic and OpenAI use their official async SDKs (`anthropic.AsyncAnthropic`, `openai.AsyncOpenAI`)
- Ollama uses httpx directly against `http://localhost:11434/v1/chat/completions` (OpenAI-compatible endpoint)
- Ollama base URL overridable via `OLLAMA_BASE_URL` environment variable

### Structured output strategy
- JSON mode + Pydantic parse: ask LLM for JSON output via provider-specific flags, then `return_type.model_validate_json(raw_json)`
- Consistent across all three providers
- Provider adapters responsible for setting the correct JSON mode flag for their API

### Validation & retry logic
- Default `max_retries` = 3 (1 initial + 2 retries) when EAML source doesn't specify
- On validation failure, append the Pydantic error as a follow-up user message so the LLM can self-correct
- validate_or_retry only retries on Pydantic validation failures -- provider errors (rate limits, timeouts) bubble up immediately as exceptions
- After all retries exhausted: raise `EamlValidationError` with model name, attempt count, last raw response, and all validation error messages

### Telemetry hook system
- Global `eaml_runtime.configure(on_call_start=fn, on_call_end=fn, ...)` registration
- Hooks are sync-only plain functions (no async hooks)
- Typed dataclass per event: `CallStartEvent`, `CallEndEvent`, `ValidationFailureEvent`, `ToolCallEvent`
- Events carry provider, model_id, timestamps, duration, token_usage (when available), error details
- Hook exceptions swallowed with `warnings.warn()` -- telemetry never breaks business logic

### Error handling & API keys
- API keys checked at call time (not import time) -- only the provider being used needs a key
- Environment variables: `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, `OLLAMA_BASE_URL`
- Flat exception hierarchy under `EamlError`:
  - `EamlConfigError` -- missing API keys, invalid configuration
  - `EamlValidationError` -- LLM output failed Pydantic validation after retries
  - `EamlProviderError` -- provider API errors (wrapped, with status_code and provider name)
- Provider SDK exceptions (anthropic.APIError, openai.APIError, httpx errors) wrapped in `EamlProviderError`

### Claude's Discretion
- Internal module layout within `eaml_runtime/` (how to split files)
- Provider client caching mechanism (module-level dict vs class attribute)
- Exact JSON mode flags per provider (system prompt injection vs API parameter)
- Token usage extraction format (varies by provider response shape)
- `Agent` base class orchestration loop implementation details
- `ToolMetadata` dataclass field design

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Runtime API contract (defined by codegen)
- `crates/eaml-codegen/src/emitters.rs` -- Defines what generated code calls: `execute_prompt()`, `Agent` base class, `ToolMetadata`
- `crates/eaml-codegen/src/types.rs` -- ImportTracker shows exact imports generated code expects: `from eaml_runtime import execute_prompt, Agent, ToolMetadata`
- `crates/eaml-codegen/tests/snapshots/examples__generate_sentiment.snap` -- Example generated code showing execute_prompt() call signature
- `crates/eaml-codegen/tests/snapshots/examples__generate_minimal.snap` -- Minimal example: execute_prompt with model dict and return_type
- `crates/eaml-codegen/tests/snapshots/examples__generate_all_type_variants.snap` -- All type variants including Literal and primitive return types

### Language specifications
- `spec/PYTHON_BRIDGE.md` -- Python bridge block specification (tool body semantics)
- `spec/CAPABILITIES.md` -- Capability registry (capabilities list in model configs)

### Prior phase context
- `.planning/phases/04-code-generation/04-CONTEXT.md` -- Phase 4 decisions that define the runtime API contract

### Existing runtime scaffolding
- `python/pyproject.toml` -- Package config, dependencies (anthropic, openai, pydantic, httpx)
- `python/src/eaml_runtime/__init__.py` -- Current stub
- `python/src/eaml_runtime/providers/__init__.py` -- Current stub

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `python/pyproject.toml` already declares all needed dependencies: anthropic>=0.43, openai>=1.0, pydantic>=2.0, httpx>=0.25
- `python/src/eaml_runtime/providers/` directory exists as stub -- ready for provider modules
- pytest + pytest-asyncio configured with `asyncio_mode = "strict"`
- ruff (line-length 100) and mypy (strict mode) already configured

### Established Patterns
- Hatchling build system with `src/eaml_runtime` package layout
- Python 3.11+ target (can use modern typing syntax: `X | Y`, `list[T]`)
- Strict mypy means all public APIs need complete type annotations

### Integration Points
- Generated code does `from eaml_runtime import execute_prompt, Agent, ToolMetadata`
- `execute_prompt(model=dict, messages=list, return_type=Type, **opts)` is the core entry point
- `Agent` base class extended by generated agent classes with model/tools/system_prompt attributes
- `ToolMetadata(name, description, parameters, return_type)` registered per tool
- Phase 6 CLI will import the runtime to wire compile + execute

</code_context>

<specifics>
## Specific Ideas

- `execute_prompt()` should be the single public entry point that handles provider dispatch, JSON mode, Pydantic validation, and retry logic internally
- Provider adapters should be swappable for testing -- the ABC pattern enables mock providers in tests
- Ollama compatibility via the OpenAI-compatible endpoint means the message format is nearly identical to OpenAI, reducing implementation complexity
- Error feedback on retry (appending validation errors to messages) is a key differentiator for reliability

</specifics>

<deferred>
## Deferred Ideas

None -- discussion stayed within phase scope

</deferred>

---

*Phase: 05-python-runtime*
*Context gathered: 2026-03-16*
