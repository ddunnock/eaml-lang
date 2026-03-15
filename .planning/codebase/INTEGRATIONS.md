# External Integrations

**Analysis Date:** 2026-03-15

## APIs & External Services

**LLM Providers:**
- Anthropic Claude - LLM API for AI model execution
  - SDK: `anthropic` 0.43+
  - Auth: Set via environment variable in generated Python code
  - Models: `claude-3-5-sonnet-20241022` and others
  - Capabilities: json_mode, tools, vision, streaming, reasoning

- OpenAI GPT - LLM API for AI model execution
  - SDK: `openai` 1.0+
  - Auth: Set via environment variable in generated Python code
  - Models: `gpt-4o` and others
  - Capabilities: json_mode, tools, vision, streaming

- Ollama (Local/Self-hosted)
  - Client: `httpx` 0.25+ (HTTP client)
  - Auth: None (typically local deployment)
  - Transport: HTTP endpoint (default localhost:11434)
  - Capabilities: Basic LLM, limited capability support

## Data Storage

**Databases:**
- Not used - EAML compiler is stateless
- Generated code may connect to user-provided databases (not managed by runtime)

**File Storage:**
- Local filesystem only - Generated code may read/write files to disk
- No cloud storage integration in `eaml-runtime`

**Caching:**
- Not applicable - Runtime is stateless per request

## Authentication & Identity

**Auth Provider:**
- None - EAML is API-key based per provider SDK
- Generated code inherits auth from:
  - Environment variables: `ANTHROPIC_API_KEY`, `OPENAI_API_KEY` (provider-specific)
  - User-provided runtime configuration

**Implementation:**
- Anthropic SDK: Reads `ANTHROPIC_API_KEY` by default
- OpenAI SDK: Reads `OPENAI_API_KEY` by default
- Ollama: No authentication (assumes self-hosted or open endpoint)

## Monitoring & Observability

**Error Tracking:**
- None - EAML runtime does not integrate with error tracking services
- Generated code may emit errors; runtime provides CapabilityActivationError and CapabilityMismatchError types

**Logs:**
- Not integrated - Generated code uses standard Python logging/print
- Runtime provides detailed error messages for capability checking failures

**Telemetry:**
- Not applicable - EAML compiler is deterministic and emits no telemetry

## CI/CD & Deployment

**Hosting:**
- EAML compiler: Can run locally or in any CI/CD pipeline
- Generated code: Runs anywhere Python 3.11+ is available
- No server deployment required for compiler

**CI Pipeline:**
- None configured in repository - Relies on user's CI system
- Compilation is lightweight and suitable for standard CI workflows

## Environment Configuration

**Required env vars (for generated code execution):**
- `ANTHROPIC_API_KEY` - API key for Anthropic Claude (if using anthropic provider)
- `OPENAI_API_KEY` - API key for OpenAI GPT (if using openai provider)
- No env vars required for compilation itself

**Secrets location:**
- Secrets are NOT stored in repository
- No `.env` files tracked in git
- Users provide secrets via environment variables at runtime

## Webhooks & Callbacks

**Incoming:**
- None - EAML is a batch-oriented compiler and runtime

**Outgoing:**
- None - Generated code does not emit webhooks
- Capability system requires explicit provider declaration (no implicit external calls)

## Provider Capability Matrix

The compiler validates that prompts only require capabilities that their declared model supports. Three providers are recognized:

| Provider   | SDK Dependency | Capabilities Supported        | Runtime Adapter Location |
|------------|----------------|-------------------------------|--------------------------|
| anthropic  | anthropic      | json_mode, tools, vision, streaming, reasoning | `eaml_runtime/providers/` (planned) |
| openai     | openai         | json_mode, tools, vision, streaming | `eaml_runtime/providers/` (planned) |
| ollama     | httpx          | (limited — basic LLM only)     | `eaml_runtime/providers/` (planned) |

## Capability Checking (Compile-time)

The `eaml-semantic` crate performs capability validation:
- Verifies that each prompt's `requires` clause matches model's declared `caps`
- Errors if prompt requires capabilities model doesn't support (CAP010 = fatal)
- Warns if model declares duplicate capabilities (CAP002 = warning)
- Runtime adapter must activate capabilities at API call time

**Error Codes Related to Providers/Capabilities:**
- CAP010 - Capability mismatch (fatal)
- CAP002 - Duplicate capability declaration (warning)
- CAP020 - json_mode + string return warning
- RES values in error range for provider-specific issues

---

*Integration audit: 2026-03-15*
