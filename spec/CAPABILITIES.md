# EAML Capability System Specification

**Version:** 0.1.0
**Date:** 2026-03-15
**Status:** AUTHORITATIVE

---

## Abstract

This document is the complete capability system specification for EAML (Engineering AI
Markup Language) version 0.1.0. It defines the semantics of model capability declarations,
prompt capability requirements, the compile-time checking algorithm, and the runtime
activation contract for provider adapters.

This document serves three consumers:

1. **Compiler semantic analysis** (CAP phase in `eaml-semantic` crate) — every capability
   checking decision the compiler makes MUST be traceable to a rule in this document.
2. **Runtime adapter layer** (`eaml_runtime` package) — every provider adapter MUST know
   which capabilities it supports and how to activate them at API call time.
3. **EAML language users** — every rule is explained in terms an engineer who understands
   what `json_mode` and `streaming` mean from provider documentation can apply when
   writing `.eaml` files.

### Normative Language

The key words "MUST", "MUST NOT", "SHALL", "SHOULD", "MAY" in this document are to
be interpreted as described in [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

### Related Documents

| Document                                    | Relationship                                                                       |
|---------------------------------------------|------------------------------------------------------------------------------------|
| `spec/grammar.ebnf`                         | Syntactic contract — this document cites grammar productions by number             |
| `spec/TYPESYSTEM.md`                        | Type-level contract — capability rules interact with type rules at specific points |
| `spec/ERRORS.md`                            | Error code catalog — capability errors documented here are registered there        |
| Layer 5 (`eaml-layer5-design-decisions.md`) | Authoritative design decisions — this document implements them                     |

### Three-Layer Validation Model

EAML employs three distinct validation layers, each owned by different code and triggered
at different times:

| Layer                             | Phase        | Owner                          | What It Catches                                                    |
|-----------------------------------|--------------|--------------------------------|--------------------------------------------------------------------|
| **Layer 1** — Type checking       | Compile-time | `eaml-semantic` (type checker) | Schema field types resolve, return types match, parameters correct |
| **Layer 2** — Capability checking | Compile-time | `eaml-semantic` (CAP checker)  | Prompt requires capabilities that the model declares               |
| **Layer 3** — Pydantic validation | Runtime      | `eaml_runtime` + Pydantic v2   | LLM output conforms to declared schema structure                   |

Layer 2 (THIS document) runs after name resolution and type checking (Layer 1) but
before code generation. Layer 3 is documented in TYPESYSTEM.md §1.2. The layers are
independent: a prompt can be type-correct and capability-correct at compile time but
still receive a non-conforming LLM response at runtime.

**Relationship to TYPESYSTEM.md §1.2:** TYPESYSTEM.md describes a "two-layer validation
model" covering type checking (its Layer 1) and runtime Pydantic validation (its Layer 2).
Capability checking is a third layer that sits between them. TYPESYSTEM.md §1.2 mentions
capability checking in its compile-time list but does not specify it — this document does.

### How to Read This Document

**Rule blocks** follow a consistent format throughout:

```
RULE [CAP-CAT-NN]: [Short imperative title]

  Plain English: [One-paragraph description accessible to any engineer]
  Formal:        [Notation where applicable]
  Grammar:       Production [N] in grammar.ebnf
  Valid:          [EAML code example that is correct]
  Invalid:       [EAML code example that triggers an error] → Error [CODE]: [msg]
  Runtime:       [What the runtime adapter must do for this capability]
  Notes:         [Cross-references, edge cases, rationale]
```

The `Runtime:` field is unique to CAPABILITIES.md (not present in TYPESYSTEM.md) because
capability rules have both compile-time consequences (for the compiler) and runtime
consequences (for the adapter). TYPESYSTEM.md rules have no `Runtime:` field.

**Grammar citations** use the format `Production [N]` where N is the production number
in `spec/grammar.ebnf`. All cited production numbers have been physically verified.

**Error citations** use the format `CAP0NN` for capability-specific errors. All error
codes are cataloged in Section 9 and cross-referenced from the rule blocks that trigger them.

### Table of Contents

1. [Capability System Philosophy](#1-capability-system-philosophy)
   - 1.1 What a Capability Is
   - 1.2 Compile-Time Checking: EAML vs BAML
   - 1.3 The Three Validation Layers
   - 1.4 Capability Scope in v0.1
   - 1.5 Design Decisions Summary
2. [Built-In Capability Registry](#2-built-in-capability-registry)
   - 2.1 json_mode
   - 2.2 tools
   - 2.3 vision
   - 2.4 streaming
   - 2.5 reasoning
   - 2.6 Built-In Capability Summary
   - 2.7 Capability Name Casing
3. [Model Capability Declarations](#3-model-capability-declarations)
   - 3.1 caps: Field in Model Declarations
   - 3.2 Capability Name Resolution in Model Declarations
   - 3.3 Multiple Models and Capability Sets
   - 3.4 Provider-Agnostic Architecture
4. [requiresClause Syntax and Semantics](#4-requiresclause-syntax-and-semantics)
   - 4.1 requiresClause Forms
   - 4.2 Capability Name Resolution in requiresClause
   - 4.3 requiresClause Position
   - 4.4 Duplicate Capability Names
5. [Compile-Time Capability Checking](#5-compile-time-capability-checking)
   - 5.1 The Capability Check Algorithm
   - 5.2 Static Call Site Binding
   - 5.3 CAP010: Capability Mismatch Error
   - 5.4 Capability Checking is Fatal
6. [Capability and Type System Interaction](#6-capability-and-type-system-interaction)
   - 6.1 json_mode and Return Type
   - 6.2 vision and Input Types
   - 6.3 tools Capability and Tool Declarations
7. [Runtime Adapter Behavior](#7-runtime-adapter-behavior)
   - 7.1 Runtime Capability Activation Contract
   - 7.2 Per-Provider Activation Rules
   - 7.3 Capability Ordering and Combinations
8. [Custom Capabilities](#8-custom-capabilities)
   - 8.1 Open Identifier Registry
   - 8.2 Custom Capability Rules
9. [Capability Error Catalog](#9-capability-error-catalog)
10. [Post-MVP Capability Features](#10-post-mvp-capability-features)

---

## 1. Capability System Philosophy

### 1.1 What a Capability Is

A **capability** is a named feature of an LLM provider's API that a prompt can require
in order to function correctly. Capabilities affect how the runtime adapter constructs
API calls and how the provider processes them. There are four categories:

| Category              | Capabilities  | What Changes                                                      |
|-----------------------|---------------|-------------------------------------------------------------------|
| API call construction | `json_mode`   | The adapter adds structured output parameters to the API request  |
| Prompt routing        | `streaming`   | The adapter uses a streaming response delivery mechanism          |
| Input handling        | `vision`      | The adapter enables image attachment in prompt messages           |
| Inference behavior    | `reasoning`   | The adapter activates chain-of-thought / extended reasoning modes |
| Tool integration      | `tools`       | The adapter includes tool schemas in the API request              |

The runtime adapter uses the declared capability set to construct the correct API call
for each provider. A prompt that requires `json_mode` but is routed to a model without
`json_mode` would produce garbage output — catching this at compile time prevents
silent failures.

### 1.2 Compile-Time Checking: EAML vs BAML

EAML capability checking is **compile-time and fatal**. A prompt that requires a
capability not declared by its model cannot be compiled — the compiler emits CAP010
and halts. This is a deliberate design choice documented in Layer 5 §6.3 [CLOSED].

**Contrast with BAML:** BAML (BoundaryML's LLM DSL) defers capability checking to
runtime. Any BAML function can be called against any client — capability mismatches
are discovered only when the API call fails in production (Layer 3 §4.5). EAML makes
this a compile-time error because in engineering workflows, a prompt that requires
structured JSON output is architecturally dependent on `json_mode` support. This is a
structural dependency — not a runtime condition. It belongs at compile time.

EAML's capability checking is unique. It is not "similar to TypeScript type checking"
(TypeScript has no capability concept). It is not "like Python decorators" (decorators
are runtime metadata). The compile-time CAP check is a novel feature of EAML that
ensures prompt-to-model compatibility is verified before any code is generated.

### 1.3 The Three Validation Layers

EAML has three distinct validation layers, each catching a different class of error:

**Layer 1 — Compile-time TYPE checking** (TYPESYSTEM.md):
Ensures schema field types are valid, return types match, parameters have correct types,
bounded types have valid constraints. Errors: TYP0xx series.

**Layer 2 — Compile-time CAPABILITY checking** (THIS document):
Ensures every prompt's `requires` clause is satisfied by its model's `caps` declaration.
Errors: CAP0xx series.

**Layer 3 — Runtime PYDANTIC validation** (TYPESYSTEM.md §1.2):
Ensures LLM output conforms to the declared schema structure. The `max_retries` field
(Production [33]) controls retry behavior on validation failure.

The layers are independent: a prompt can be type-correct and capability-correct at
compile time, but still receive a non-conforming LLM response at runtime. The retry
policy (`max_retries`) addresses Layer 3 failures. There is no compile-time equivalent
for Layer 3 because LLM output is inherently unpredictable.

### 1.4 Capability Scope in v0.1

**IN SCOPE:**

| Feature                                                                              | Section  |
|--------------------------------------------------------------------------------------|----------|
| Five built-in capabilities: `json_mode`, `tools`, `vision`, `streaming`, `reasoning` | §2       |
| `requires` clause on prompt declarations only                                        | §4       |
| Model `caps` declaration                                                             | §3       |
| Compile-time CAP010 mismatch checking                                                | §5       |
| Open identifier registry with CAP001 warning for unknown names                       | §8       |
| Runtime capability activation contract                                               | §7       |

**OUT OF SCOPE (v0.1):**

| Feature                          | Status                                                               | Error Code                     |
|----------------------------------|----------------------------------------------------------------------|--------------------------------|
| `requires` on tool declarations  | Not in grammar — Production [34] `toolDecl` has no `requiresClause`  | Parse error (SYN) if attempted |
| `requires` on agent declarations | Not in grammar — Production [38] `agentDecl` has no `requiresClause` | Parse error (SYN) if attempted |
| Capability inheritance           | Not specified in Layer 5                                             | Not in grammar                 |
| Conditional capabilities         | Not specified in Layer 5                                             | Not in grammar                 |

### 1.5 Design Decisions Summary

| Decision           | Value                                              | Layer 5 Reference     | Rationale                                      |
|--------------------|----------------------------------------------------|-----------------------|------------------------------------------------|
| Capability names   | Open identifiers, semantic validation              | §6.1                  | Extensible without grammar changes             |
| Built-in count     | 5 (json_mode, tools, vision, streaming, reasoning) | §6.1                  | Covers core LLM API features                   |
| Unknown capability | CAP001 warning (--strict-caps promotes to error)   | §6.1                  | Forward-compatible with future capabilities    |
| Requires syntax    | Bare or bracketed form                             | §6.2 [GRAMMAR IMPACT] | Ergonomic for single and multiple capabilities |
| Mismatch severity  | Fatal compile-time error (CAP010)                  | §6.3 [CLOSED]         | Prevents silent runtime failures               |
| Runtime guard      | Always emitted as defense-in-depth                 | §6.3                  | Catches misuse from non-EAML code              |
| Case sensitivity   | Fully case-sensitive (EG-09)                       | §14 EG-09             | Consistent with EAML lexical rules             |
| Registry model     | Open — any IDENT is grammatically valid            | §6.1                  | Tracks rapidly evolving LLM landscape          |

---

## 2. Built-In Capability Registry

The following five capabilities are registered in the EAML capability registry for v0.1.
Each is specified as a complete rule block. The registry is extensible — see §8 for
custom capability names.

### 2.1 json_mode

**RULE CAP-REG-01: json_mode capability**

> Plain English: The `json_mode` capability declares that a model can return structured
> JSON output. When active, the runtime adapter instructs the provider API to constrain
> the model's output to valid JSON conforming to the prompt's return type schema.
>
> Formal: `json_mode ∈ BUILT_IN_CAPS`
>
> Grammar: Production [28] `capList` — `json_mode` appears as an IDENT token in the
> capability list. Production [76] `requiresClause` — appears as an IDENT in the
> requires clause.
>
> Valid:
> ```eaml
> model Claude = Model(
>   id: "claude-sonnet-4-20250514",
>   provider: "anthropic",
>   caps: [json_mode, tools]
> )
>
> prompt Classify(text: string) requires json_mode -> SentimentResult {
>   user: "Classify sentiment: {text}"
> }
> ```
>
> Invalid:
> ```eaml
> model Basic = Model(id: "basic-v1", provider: "openai", caps: [])
>
> prompt Classify(text: string) requires json_mode -> SentimentResult {
>   user: "Classify: {text}"
> }
> // When called with Basic:
> // → CAP010: Model 'Basic' is missing required capabilities: [json_mode]
> ```
>
> Runtime: The adapter MUST apply the following API modifications per provider:
>
> | Provider      | API Parameter                                                                                                                                                                       |
> |---------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
> | `"anthropic"` | Not a separate parameter — Anthropic uses tool-based structured output. The adapter passes the return type's JSON schema as a tool definition and extracts the structured response. |
> | `"openai"`    | Adds `response_format: {"type": "json_schema", "json_schema": {"schema": <return_type_schema>}}` to the API request body.                                                           |
> | `"ollama"`    | Adds `format: "json"` to the API request body. Schema enforcement is best-effort.                                                                                                   |
>
> Notes: Interacts with the type system — see §6.1 (CAP-TYP-01) for the
> `json_mode` / return type interaction rule.

### 2.2 tools

**RULE CAP-REG-02: tools capability**

> Plain English: The `tools` capability declares that a model supports tool/function
> calling. When active, the runtime adapter includes tool schemas in the API request,
> allowing the model to invoke declared tools during inference.
>
> Formal: `tools ∈ BUILT_IN_CAPS`
>
> Grammar: Production [28] `capList`, Production [76] `requiresClause`.
>
> Valid:
> ```eaml
> model Claude = Model(
>   id: "claude-sonnet-4-20250514",
>   provider: "anthropic",
>   caps: [json_mode, tools]
> )
>
> tool GetWeather(city: string) -> WeatherData {
>   python %{
>     return fetch_weather(city)
>   }%
> }
>
> agent WeatherAgent {
>   model: Claude
>   tools: [GetWeather]
>   system: "Help users with weather queries."
>   max_turns: 5
>   on_error: retry(2) then fail
> }
> ```
>
> Invalid: A model without `tools` in its `caps` used in an agent that references tools:
> ```eaml
> model NoTools = Model(id: "basic-v1", provider: "openai", caps: [json_mode])
>
> agent BrokenAgent {
>   model: NoTools
>   tools: [GetWeather]
>   // → CAP010: Model 'NoTools' is missing required capabilities: [tools]
> }
> ```
>
> Runtime: The adapter MUST include tool schemas in the API request:
>
> | Provider      | API Parameter                                                                                                           |
> |---------------|-------------------------------------------------------------------------------------------------------------------------|
> | `"anthropic"` | Adds `tools: [{"name": "<tool_name>", "description": "...", "input_schema": {...}}]` to the API request body.           |
> | `"openai"`    | Adds `tools: [{"type": "function", "function": {"name": "<tool_name>", "parameters": {...}}}]` to the API request body. |
> | `"ollama"`    | Adds `tools: [...]` in OpenAI-compatible format. Tool support depends on the specific model.                            |
>
> Notes: The `tools` capability in a model's `caps` is checked when the model is used in
> an agent that declares a `tools:` field (Production [39] `agentField`). See §6.3
> (CAP-TYP-03) for the interaction with tool declarations.

### 2.3 vision

**RULE CAP-REG-03: vision capability**

> Plain English: The `vision` capability declares that a model accepts image inputs.
> When active, the runtime adapter formats prompt messages to include image data
> alongside text content.
>
> Formal: `vision ∈ BUILT_IN_CAPS`
>
> Grammar: Production [28] `capList`, Production [76] `requiresClause`.
>
> Valid:
> ```eaml
> model Claude = Model(
>   id: "claude-sonnet-4-20250514",
>   provider: "anthropic",
>   caps: [json_mode, tools, vision]
> )
>
> prompt DescribeImage(image_url: string) requires vision -> string {
>   user: "Describe this image: {image_url}"
> }
> ```
>
> Invalid:
> ```eaml
> model TextOnly = Model(id: "text-model", provider: "openai", caps: [json_mode])
>
> prompt DescribeImage(image_url: string) requires vision -> string {
>   user: "Describe this image: {image_url}"
> }
> // When called with TextOnly:
> // → CAP010: Model 'TextOnly' is missing required capabilities: [vision]
> ```
>
> Runtime: The adapter MUST format image content in the user message:
>
> | Provider      | API Parameter                                                                                                                      |
> |---------------|------------------------------------------------------------------------------------------------------------------------------------|
> | `"anthropic"` | Formats user message content as `[{"type": "image", "source": {"type": "url", "url": "<url>"}}, {"type": "text", "text": "..."}]`. |
> | `"openai"`    | Formats user message content as `[{"type": "image_url", "image_url": {"url": "<url>"}}, {"type": "text", "text": "..."}]`.         |
> | `"ollama"`    | Adds `images: ["<base64_data>"]` to the API request body.                                                                          |
>
> Notes: See §6.2 (CAP-TYP-02) for the interaction between `vision` and parameter
> types. In v0.1, image data is passed through string parameters (URLs or base64);
> the type system does not have a dedicated image type.

### 2.4 streaming

**RULE CAP-REG-04: streaming capability**

> Plain English: The `streaming` capability declares that a model supports token-by-token
> streaming of responses. When active, the runtime adapter uses streaming API endpoints
> and delivers response tokens incrementally.
>
> Formal: `streaming ∈ BUILT_IN_CAPS`
>
> Grammar: Production [28] `capList`, Production [76] `requiresClause`.
>
> Valid:
> ```eaml
> model Claude = Model(
>   id: "claude-sonnet-4-20250514",
>   provider: "anthropic",
>   caps: [json_mode, tools, vision, streaming]
> )
>
> prompt StreamedSummary(text: string) requires streaming -> string {
>   user: "Summarize: {text}"
> }
> ```
>
> Invalid:
> ```eaml
> model NoStream = Model(id: "batch-model", provider: "openai", caps: [json_mode])
>
> prompt StreamedSummary(text: string) requires streaming -> string {
>   user: "Summarize: {text}"
> }
> // When called with NoStream:
> // → CAP010: Model 'NoStream' is missing required capabilities: [streaming]
> ```
>
> Runtime: The adapter MUST use the streaming API endpoint:
>
> | Provider      | API Parameter                                                                                                                   |
> |---------------|---------------------------------------------------------------------------------------------------------------------------------|
> | `"anthropic"` | Uses `client.messages.stream(...)` instead of `client.messages.create(...)`. Returns an async iterator of `MessageStreamEvent`. |
> | `"openai"`    | Adds `stream: true` to the API request body. Returns an async iterator of `ChatCompletionChunk`.                                |
> | `"ollama"`    | Adds `stream: true` to the API request body (default behavior).                                                                 |
>
> Notes: Streaming affects the generated Python code's call pattern — the codegen
> produces `async for chunk in ...` rather than `result = await ...`. This is a
> codegen concern, not a semantic analysis concern.

### 2.5 reasoning

**RULE CAP-REG-05: reasoning capability**

> Plain English: The `reasoning` capability declares that a model supports extended
> reasoning chains (chain-of-thought / thinking modes). When active, the runtime
> adapter enables reasoning-specific API parameters.
>
> Formal: `reasoning ∈ BUILT_IN_CAPS`
>
> Grammar: Production [28] `capList`, Production [76] `requiresClause`.
>
> Valid:
> ```eaml
> model ClaudeOpus = Model(
>   id: "claude-opus-4-20250514",
>   provider: "anthropic",
>   caps: [json_mode, tools, vision, streaming, reasoning]
> )
>
> prompt DeepAnalysis(problem: string) requires reasoning -> AnalysisResult {
>   user: "Analyze this problem step by step: {problem}"
> }
> ```
>
> Invalid:
> ```eaml
> model FastModel = Model(id: "fast-v1", provider: "openai", caps: [json_mode])
>
> prompt DeepAnalysis(problem: string) requires reasoning -> AnalysisResult {
>   user: "Analyze: {problem}"
> }
> // When called with FastModel:
> // → CAP010: Model 'FastModel' is missing required capabilities: [reasoning]
> ```
>
> Runtime: The adapter MUST enable reasoning/thinking mode:
>
> | Provider      | API Parameter                                                                                                                                  |
> |---------------|------------------------------------------------------------------------------------------------------------------------------------------------|
> | `"anthropic"` | Adds `thinking: {"type": "enabled", "budget_tokens": <budget>}` to the API request body. Budget defaults to provider maximum if not specified. |
> | `"openai"`    | Uses reasoning-capable model endpoint (e.g., `o3`, `o4-mini`). Reasoning is implicit in model selection; no additional parameter needed.       |
> | `"ollama"`    | Provider-specific — depends on model support. The adapter passes model-specific reasoning parameters if available.                             |
>
> Notes: The `reasoning` capability is the most provider-specific of the five built-in
> capabilities. Implementation details vary significantly across providers. The adapter
> layer absorbs this variation.

### 2.6 Built-In Capability Summary

| Capability   | API Impact                  | Type Constraint                                      | Runtime Adapter Action                            |
|--------------|-----------------------------|------------------------------------------------------|---------------------------------------------------|
| `json_mode`  | Structured JSON output mode | Return type SHOULD be schema or literal union (§6.1) | Add response format / structured output parameter |
| `tools`      | Tool/function calling       | Required when agent uses tools (§6.3)                | Include tool schemas in API request               |
| `vision`     | Image input support         | None in v0.1 (§6.2)                                  | Format messages with image content                |
| `streaming`  | Token streaming             | None                                                 | Use streaming API endpoint                        |
| `reasoning`  | Extended reasoning chains   | None                                                 | Enable thinking/reasoning mode                    |

### 2.7 Capability Name Casing

**RULE CAP-REG-06: Capability names are case-sensitive**

> Plain English: Capability names are case-sensitive IDENT tokens. The built-in registry
> contains only lowercase snake_case names. `JSON_MODE` is not the same as `json_mode`.
> An unrecognized capability name (including casing variants of built-in names) triggers
> CAP001.
>
> Formal: Capability name matching is an exact string comparison against the registry.
> `∀ name ∈ requiresClause ∪ capList: name is matched case-sensitively`.
>
> Grammar: Production [28] `capList` and Production [76] `requiresClause` both use IDENT
> tokens. IDENT is case-sensitive per Production [5] and Layer 5 §2.1 [CLOSED].
>
> Valid:
> ```eaml
> caps: [json_mode, tools]    // correct — lowercase snake_case
> requires json_mode          // correct
> ```
>
> Invalid:
> ```eaml
> caps: [JSON_MODE]
> // → CAP001: Unknown capability 'JSON_MODE'. Built-in capabilities are
> //   lowercase: json_mode, tools, vision, streaming, reasoning.
> ```
>
> Runtime: No runtime impact — this is a compile-time check only.
>
> Notes: Layer 5 §14 EG-09: "The EAML capability registry is case-sensitive.
> 'json_mode' and 'JSON_MODE' are different capability names. The built-in
> registry uses snake_case only."

---

## 3. Model Capability Declarations

### 3.1 caps: Field in Model Declarations

**RULE CAP-MDL-01: Model capability declaration**

> Plain English: Every model declaration includes a `caps:` field that lists the
> capabilities the model supports. The `caps` list is the model's capability
> declaration — it is evaluated at compile time. The developer is responsible for
> declaring only capabilities the provider genuinely supports. The compiler does NOT
> verify the declaration against the actual provider's API.
>
> Formal: `model M = Model(..., caps: C) ⟹ caps(M) = C`
>
> Grammar: Production [27] `modelDecl` — `"caps" ":" "[" capList? "]"`.
> Production [28] `capList` — `IDENT ( "," IDENT )*` with `[sem: cap-registry]`.
>
> Valid:
> ```eaml
> model Claude = Model(
>   id: "claude-sonnet-4-20250514",
>   provider: "anthropic",
>   caps: [json_mode, tools, vision]
> )
>
> model Minimal = Model(
>   id: "minimal-v1",
>   provider: "openai",
>   caps: []
> )
> ```
>
> Invalid: An empty `caps` list (`caps: []`) is valid grammar — it means the model
> supports no capabilities. Any prompt with a non-empty `requiresClause` that uses
> this model produces CAP010.
>
> Runtime: No direct runtime impact — the `caps` declaration informs compile-time
> checking. However, the generated Python code includes the capability set in the
> model configuration for runtime guard validation (Layer 5 §6.3).
>
> Notes: Layer 5 §10.2 [CLOSED]. The `caps` field uses `capList?` (optional) in
> Production [27], meaning `caps: []` with no identifiers inside the brackets is valid.

### 3.2 Capability Name Resolution in Model Declarations

**RULE CAP-MDL-02: Capability name validation in caps lists**

> Plain English: Capability names in `caps:` lists are resolved against the capability
> registry at compile time. Unknown capability names produce a CAP001 warning. The
> `--strict-caps` compiler flag promotes CAP001 to a fatal error.
>
> Formal: `∀ name ∈ caps(M): name ∉ REGISTRY ⟹ emit CAP001 warning`
>
> Grammar: `[sem: cap-registry]` annotation on Production [28] `capList`.
>
> Valid:
> ```eaml
> caps: [json_mode, tools]          // both are registered built-in names
> ```
>
> Invalid:
> ```eaml
> caps: [json_mode, Json_Mode]
> // → CAP001 warning: Unknown capability 'Json_Mode'. Built-in capabilities
> //   are lowercase: json_mode, tools, vision, streaming, reasoning.
> ```
>
> Runtime: No runtime impact — compile-time validation only.
>
> Notes: Case sensitivity applies per CAP-REG-06. The warning (not error) behavior
> for unknown names is specified in Layer 5 §6.1 — this allows engineers to declare
> capabilities before the registry is updated. See §8 for custom capabilities.

### 3.3 Multiple Models and Capability Sets

**RULE CAP-MDL-03: Independent capability sets per model**

> Plain English: Each model has its own independent `caps` declaration. If a program
> declares multiple models, each has its own capability set. A prompt's `requiresClause`
> is checked against the specific model it is called with at each call site.
>
> Formal: `caps(M1) and caps(M2) are independent sets. ∀ call site S using model M:
> check requires(prompt(S)) ⊆ caps(M)`.
>
> Grammar: Production [27] `modelDecl` — each `modelDecl` has its own `capList`.
>
> Valid:
> ```eaml
> model Claude = Model(
>   id: "claude-sonnet-4-20250514", provider: "anthropic",
>   caps: [json_mode, tools, vision]
> )
>
> model GPT = Model(
>   id: "gpt-4o", provider: "openai",
>   caps: [json_mode, tools]
> )
>
> prompt Classify(text: string) requires json_mode -> SentimentResult {
>   user: "Classify: {text}"
> }
>
> // Both Claude.call(Classify(...)) and GPT.call(Classify(...)) are valid
> // because both declare json_mode.
> ```
>
> Invalid:
> ```eaml
> model Local = Model(id: "llama3.2", provider: "ollama", caps: [tools])
>
> // Local.call(Classify(...)) → CAP010: Model 'Local' missing [json_mode]
> // Claude.call(Classify(...)) → valid (Claude has json_mode)
> ```
>
> Runtime: The adapter resolves the model's provider and capability set independently
> for each call site at runtime.
>
> Notes: **[CLOSED in v0.1.0 remediation]** Call site syntax
> (`Model.call(Prompt(...))`) is not formally defined as a grammar production in v0.1.
> The capability checker resolves model-prompt bindings through two mechanisms:
> (1) agent `model:` field bindings (Production [38] `agentDecl`), and
> (2) dot-call expressions in let bindings where the model is explicitly named.
> This covers the two documented usage patterns. A formal call-site grammar production
> (`IDENT "." "call" "(" expr ")"`) is planned for v0.2.

### 3.4 Provider-Agnostic Architecture

**RULE CAP-MDL-04: Capability declarations are provider-agnostic**

> Plain English: The `caps:` list declares what the model supports, regardless of
> provider. The `provider:` field tells the runtime adapter which SDK to use. These
> are orthogonal: two models from the same provider MAY have different capability
> sets. The compiler does NOT verify that a capability is plausible for a given
> provider — that is a developer responsibility and a runtime concern.
>
> Formal: `caps(M)` and `provider(M)` are independent attributes. No validity
> relationship is enforced between them at compile time.
>
> Grammar: Production [27] `modelDecl` — `"provider" ":" STRING` and
> `"caps" ":" "[" capList? "]"` are separate fields.
>
> Valid:
> ```eaml
> // Same provider, different capability sets
> model GPT4o = Model(
>   id: "gpt-4o", provider: "openai",
>   caps: [json_mode, tools, vision, streaming]
> )
>
> model GPT35 = Model(
>   id: "gpt-3.5-turbo", provider: "openai",
>   caps: [json_mode, tools]    // no vision on 3.5
> )
> ```
>
> Runtime: The adapter uses the `provider` field to select the SDK and the `caps`
> field to determine which API parameters to include. If a capability is declared
> in `caps` but the provider does not actually support it, the adapter raises
> `CapabilityActivationError` at runtime (see §7.1).
>
> Notes: Layer 5 §10.2 [CLOSED]: "EAML is PROVIDER-AGNOSTIC." The provider-agnostic
> architecture means adding new providers (Gemini, Bedrock) requires no grammar
> changes — only a new runtime adapter entry.

---

## 4. requiresClause Syntax and Semantics

### 4.1 requiresClause Forms

**RULE CAP-REQ-01: Bare form — single capability**

> Plain English: A prompt MAY declare a single required capability using the bare form
> `requires IDENT` without brackets.
>
> Formal: `requiresClause → "requires" IDENT ⟹ requires(prompt) = {IDENT}`
>
> Grammar: Production [76] `requiresClause` — first alternative: `"requires" IDENT`.
>
> Valid:
> ```eaml
> prompt Classify(text: string) requires json_mode -> SentimentResult {
>   user: "Classify: {text}"
> }
> ```
>
> Runtime: The single capability is passed to the adapter as a set of one element.
>
> Notes: Both `requires json_mode` and `requires [json_mode]` produce the same AST
> node: `RequiresClause { capabilities: vec![json_mode] }`.

**RULE CAP-REQ-02: Bracketed form — zero or more capabilities**

> Plain English: A prompt MAY declare zero or more required capabilities using the
> bracketed form `requires [cap1, cap2, ...]`. The empty form `requires []` is valid
> and semantically equivalent to omitting the `requiresClause` entirely.
>
> Formal: `requiresClause → "requires" "[" IDENT* "]" ⟹ requires(prompt) = {IDENTs}`
>
> Grammar: Production [76] `requiresClause` — second alternative:
> `"requires" "[" ( IDENT ( "," IDENT )* )? "]"`.
>
> Valid:
> ```eaml
> prompt Multi(text: string) requires [json_mode, tools] -> Result {
>   user: "Process: {text}"
> }
>
> prompt NoReqs(text: string) requires [] -> string {
>   user: "Echo: {text}"
> }
> ```
>
> Runtime: The capability set (possibly empty) is passed to the adapter.
>
> Notes: Production [76] in grammar.ebnf explicitly documents: "`requires []` (empty
> brackets) is syntactically valid and semantically equivalent to omitting the
> requiresClause entirely. This is intentional (Layer 5 §6.2) — no linter warning
> emitted."

**RULE CAP-REQ-03: Absent requiresClause — implicit empty requirement**

> Plain English: A prompt with no `requiresClause` has an implicit empty capability
> requirement set. This is NOT the same as "unchecked" — it means the prompt requires
> no capabilities and can be called with any model, including models with `caps: []`.
>
> Formal: `promptDecl without requiresClause ⟹ requires(prompt) = ∅`
>
> Grammar: Production [31] `promptDecl` — `requiresClause?` — the `?` makes it
> optional. ε/FOLLOW safety verified: `FIRST(requiresClause) = {"requires"}`,
> `FOLLOW(requiresClause in promptDecl) = {"->"}`, intersection = ∅. LL(1).
> (Layer 4 §5.5, grammar.ebnf Production [31] comment.)
>
> Valid:
> ```eaml
> // No requires clause — works with any model
> prompt Echo(text: string) -> string {
>   user: "Echo: {text}"
> }
> ```
>
> Runtime: No capability activation needed — the adapter makes a standard API call
> without capability-specific parameters.
>
> Notes: The distinction between "implicit empty" and "unchecked" is critical. An
> unchecked prompt would bypass the CAP checker entirely. An implicit empty prompt
> passes through the CAP checker with `R = ∅`, which is always a subset of any
> capability set `C`. The check succeeds vacuously: `∅ ⊆ C` for all `C`.

### 4.2 Capability Name Resolution in requiresClause

**RULE CAP-REQ-04: Capability name validation in requiresClause**

> Plain English: Capability names in `requiresClause` are resolved against the
> capability registry at compile time, using the same rules as `caps:` lists.
> Unknown capability names produce a CAP001 warning (or error with `--strict-caps`).
>
> Formal: `∀ name ∈ requires(prompt): name ∉ REGISTRY ⟹ emit CAP001 warning`
>
> Grammar: `[sem: cap-registry]` annotation on Production [76] `requiresClause`.
>
> Valid:
> ```eaml
> requires json_mode           // registered built-in
> requires [tools, vision]     // both registered
> ```
>
> Invalid:
> ```eaml
> requires json_Mode
> // → CAP001 warning: Unknown capability 'json_Mode'.
> ```
>
> Runtime: No runtime impact — compile-time validation only.
>
> Notes: Same resolution rules as CAP-MDL-02. Same error code (CAP001).

### 4.3 requiresClause Position

**RULE CAP-REQ-05: requiresClause appears only in promptDecl in v0.1**

> Plain English: The `requiresClause` appears only in prompt declarations in v0.1.
> It does NOT appear in tool declarations, agent declarations, let bindings, or
> schema declarations. If a user writes `requires` in a context other than a prompt
> declaration, the parser produces a syntax error.
>
> Grammar: `requiresClause` is referenced only in Production [31] `promptDecl`.
> It is NOT referenced in Production [34] `toolDecl`, Production [38] `agentDecl`,
> or any other declaration production.
>
> Valid:
> ```eaml
> prompt Foo(x: string) requires json_mode -> Result { ... }
> ```
>
> Invalid:
> ```eaml
> tool Bar(x: string) requires json_mode -> Result { ... }
> // → SYN error: unexpected token 'requires' in tool declaration
> ```
>
> Runtime: No runtime impact — grammar-level restriction.
>
> Notes: **v0.1 Restriction:** `requiresClause` is only valid on prompt declarations.
> Tool and agent capability requirements are Post-MVP (§10). Layer 5 §6.2 specifies
> the `requiresClause` grammar only in the context of prompt declarations.

### 4.4 Duplicate Capability Names

**RULE CAP-REQ-06: Duplicate capability names in a single clause**

> Plain English: Duplicate capability names within a single `requiresClause` or
> `capList` SHOULD produce a CAP002 warning. The semantic meaning is the same as
> listing the capability once — no incorrect behavior results. This is not a fatal
> error because the duplicate is redundant, not harmful.
>
> Formal: `∀ list ∈ {requiresClause, capList}: |set(list)| < |list| ⟹ emit CAP002 warning`
>
> Grammar: Production [28] `capList` and Production [76] `requiresClause` both allow
> repeated IDENT tokens syntactically.
>
> Valid (with warning):
> ```eaml
> requires [json_mode, json_mode]
> // → CAP002 warning: Duplicate capability 'json_mode' in requires clause.
> //   The duplicate is ignored.
> ```
>
> Runtime: Duplicates are deduplicated before passing to the adapter. The capability
> is activated once regardless of how many times it appears.
>
> Notes: **[CLOSED in v0.1.0 remediation]** CAP002 warning adopted. Layer 5 does not
> explicitly address duplicates; this decision is an EAML spec-level choice. Rationale:
> duplicates are harmless redundancy, not a semantic error. A warning helps the
> developer clean up without blocking compilation.

---

## 5. Compile-Time Capability Checking

### 5.1 The Capability Check Algorithm

**RULE CAP-CHK-01: Capability subset check**

> Plain English: For each prompt invocation in the program, the compiler verifies
> that every capability the prompt requires is declared by the model used at that
> call site. If any required capability is missing from the model's `caps` list,
> the compiler emits CAP010 — a fatal error that prevents code generation.
>
> Formal:
> ```
> For each prompt call site S in the program:
>   Let R = requires(prompt(S))    // set of capabilities in the prompt's requiresClause
>                                  // (empty set if requiresClause is absent)
>   Let C = caps(model(S))         // set of capabilities in the model's caps: list
>   If R ⊄ C (R is not a subset of C):
>     Let missing = R \ C          // capabilities required but not declared
>     Emit CAP010 for each capability in missing.
> ```
>
> The check is: **R ⊆ C** — the prompt's required set must be a subset of the model's
> capability set. In plain terms: every capability the prompt needs, the model must have.
>
> Grammar: `[sem: cap-registry]` annotations on Production [28] `capList` and
> Production [76] `requiresClause` mark the two inputs to this algorithm.
>
> **Timing in the compiler pipeline:**
>
> | Phase                       | Crate               | What Happens                            |
> |-----------------------------|---------------------|-----------------------------------------|
> | 1. Lexing                   | `eaml-lexer`        | Tokenize source into token stream       |
> | 2. Parsing                  | `eaml-parser`       | Build AST from tokens                   |
> | 3. Name resolution (pass 1) | `eaml-semantic`     | Collect all declaration names           |
> | 4. Name resolution (pass 2) | `eaml-semantic`     | Resolve all references                  |
> | 5. Type checking            | `eaml-semantic`     | Verify types (TYPESYSTEM.md)            |
> | **6. Capability checking**  | **`eaml-semantic`** | **Verify capabilities (THIS document)** |
> | 7. Code generation          | `eaml-codegen`      | Emit Python/Pydantic code               |
>
> Capability checking runs AFTER type checking (the prompt and model must be valid
> typed entities before their capabilities are compared) and BEFORE code generation
> (invalid capability combinations must not reach codegen).
>
> Runtime: No runtime action — this is a compile-time check. However, see §7.1 for
> the complementary runtime guard.
>
> Notes: Layer 5 §6.3 [CLOSED]. This check is the core of the capability system —
> every other rule supports this algorithm.

### 5.2 Static Call Site Binding

**RULE CAP-CHK-02: Model-prompt binding at call sites**

> Plain English: The capability check requires knowing which model is used at each
> prompt call site. In v0.1, the binding is determined through two mechanisms:
> (a) Agent declarations bind a model to the prompts/tools used by that agent, and
> (b) Explicit call expressions at let-binding sites name the model directly.
>
> Grammar: Production [38] `agentDecl` — the `model:` field (Production [39]) binds
> a model to the agent. Production [41] `letDecl` — the call expression names the
> model in method-call syntax.
>
> Valid:
> ```eaml
> // Binding via agent declaration
> agent MyAgent {
>   model: Claude          // binds Claude to this agent's tools and prompts
>   tools: [GetWeather]
>   system: "Help users."
>   max_turns: 5
>   on_error: fail
> }
>
> // Binding via explicit call
> let result: SentimentResult = Claude.call(Classify(text: "hello"))
> ```
>
> Runtime: The runtime receives the compiled binding and uses the correct model for
> each call.
>
> Notes: **[CLOSED in v0.1.0 remediation]** (see §3.3 CAP-MDL-03). For v0.1, the
> compiler resolves model bindings from (1) agent `model:` fields and (2) dot-call
> expressions in let bindings. Formal call-site grammar extension is planned for v0.2.

### 5.3 CAP010: Capability Mismatch Error

**RULE CAP-CHK-03: CAP010 — capability mismatch is fatal**

> Plain English: When a prompt requires a capability that the model at the call site
> does not declare, the compiler emits CAP010. This is a fatal compile-time error —
> the program cannot proceed to code generation until all CAP010 errors are resolved.
>
> Formal: `R \ C ≠ ∅ ⟹ ∀ cap ∈ (R \ C): emit CAP010(prompt, cap, model)`
>
> Grammar: This error is emitted by the semantic analysis phase, not the parser.
> It references productions [27], [28], [31], and [76].
>
> Message template (Layer 5 §6.3):
> ```
> CAP010: Model '{model_name}' is missing required capabilities: [{caps}]
>         Required by prompt '{prompt_name}' at line {line}:{col}
>         Hint: Model '{model_name}' supports: [{model_caps}]
>         Add the missing capabilities to the model declaration, or
>         use a model that supports {missing_caps}.
> ```
>
> Multiple missing capabilities emit **one CAP010 per missing capability**, so each
> is independently actionable. Within the CAP phase, errors accumulate subject to the
> 20-error limit (ERRORS.md §1.6). All CAP010 errors are reported before compilation
> halts. Code generation (the next phase) is blocked regardless of how many errors
> were accumulated.
>
> Valid (three resolution paths):
> ```eaml
> // Resolution (a): Add capability to model
> model Claude = Model(
>   id: "claude-sonnet-4-20250514", provider: "anthropic",
>   caps: [json_mode, tools]    // ← add json_mode here
> )
>
> // Resolution (b): Remove from requires
> prompt Classify(text: string) -> SentimentResult {    // ← remove requires clause
>   user: "Classify: {text}"
> }
>
> // Resolution (c): Use a different model
> let result: SentimentResult = CapableModel.call(Classify(text: "hello"))
> ```
>
> Runtime: In addition to the compile-time check, the generated Python code includes
> a runtime guard (Layer 5 §6.3):
> ```python
> if not model.has_caps(prompt.requires):
>     raise CapabilityError(...)
> ```
> This catches cases where EAML-compiled code is called from non-EAML code that
> bypasses the type system.
>
> Notes: Layer 5 §6.3 [CLOSED]. CAP010 is the most important error in the capability
> system. It is never a warning — it is always fatal.

### 5.4 Capability Checking is Fatal

**RULE CAP-CHK-04: All capability mismatch errors are fatal**

> Plain English: ALL capability mismatch errors (CAP010) are fatal. There are no
> CAP-level mechanisms that permit compilation to continue past a capability mismatch.
> CAP001 (unknown capability name) and CAP002 (duplicate) are warnings — they do not
> block compilation. But CAP010 (mismatch) always blocks.
>
> Formal: `CAP010 ∈ FATAL_ERRORS. CAP001, CAP002 ∈ WARNINGS.`
>
> Grammar: Not grammar-specific — this is a semantic analysis severity rule.
>
> Runtime: No runtime impact — this defines compile-time behavior only.
>
> Notes: Rationale (Layer 5 §6.3 [CLOSED]): a capability mismatch will always cause
> a runtime failure — the API call will either fail, return garbage, or produce an
> error. There is no case where "probably fine" applies to a missing required
> capability. Unlike type warnings (e.g., TYP001 for name shadowing), capability
> mismatches are structural errors.

---

## 6. Capability and Type System Interaction

### 6.1 json_mode and Return Type

**RULE CAP-TYP-01: json_mode and prompt return type interaction**

> Plain English: When a prompt declares `requires json_mode`, its return type SHOULD
> be a schema type or literal union — not bare `string`. The `json_mode` capability
> instructs the provider to return structured JSON. A prompt that requests `json_mode`
> but declares `-> string` is likely a mistake — the structured JSON will be returned
> as a raw string, losing Pydantic validation benefits.
>
> Formal: `json_mode ∈ requires(prompt) ∧ returnType(prompt) = string ⟹ emit CAP020 warning`
>
> Grammar: Production [31] `promptDecl` — `"->" typeExpr` is the return type.
> Production [76] `requiresClause` — contains `json_mode`.
>
> Valid:
> ```eaml
> // json_mode with schema return type — correct usage
> prompt Classify(text: string) requires json_mode -> SentimentResult {
>   user: "Classify: {text}"
> }
>
> // json_mode with literal union return type — also correct
> prompt Label(text: string) requires json_mode
>     -> "positive" | "negative" | "neutral" {
>   user: "Label: {text}"
> }
> ```
>
> Valid (with warning):
> ```eaml
> // json_mode with string return type — technically valid but likely unintended
> prompt Extract(text: string) requires json_mode -> string {
>   user: "Extract JSON: {text}"
> }
> // → CAP020 warning: Prompt 'Extract' requires json_mode but returns 'string'.
> //   Consider using a schema type to get Pydantic validation of the JSON response.
> ```
>
> Runtime: When `json_mode` is active, the adapter always requests structured JSON
> output from the provider. If the return type is `string`, the runtime returns the
> raw JSON string without Pydantic validation.
>
> Notes: Cross-reference: TYPESYSTEM.md TS-RET-01 (prompt return type — any valid
> `typeExpr` is accepted), TS-RET-03 (literal union as return type — permitted with
> `Literal[]` codegen). This is a CAP warning, not a type error — the type system
> accepts `-> string` regardless of capabilities. The capability system adds a
> recommendation on top.
>
> **[CLOSED in v0.1.0 remediation]** CAP020 WARNING adopted. Layer 5 does not
> explicitly specify the json_mode/return type interaction. This is a quality-of-life
> warning, not a correctness error — there are legitimate cases where an engineer
> wants raw JSON as a string.

### 6.2 vision and Input Types

**RULE CAP-TYP-02: vision capability and parameter types**

> Plain English: In v0.1, the `vision` capability does not change which parameter
> types are valid in prompts. All prompt parameters use the standard type system
> (`string`, schema types, etc.). Image data is passed as string parameters
> containing URLs or base64-encoded data. The `vision` capability enables the
> runtime adapter to format these strings as image content in the provider API call.
>
> Formal: `vision ∈ caps(M) does not modify the type checking rules for parameters.`
>
> Grammar: Production [72] `paramList` and Production [73] `param` — parameter types
> are always `typeExpr`, regardless of capability declarations.
>
> Valid:
> ```eaml
> prompt DescribeImage(image_url: string) requires vision -> string {
>   user: "Describe: {image_url}"
> }
> // image_url is a regular string parameter — the adapter interprets it as an image URL
> ```
>
> Runtime: When `vision` is active, the adapter inspects string parameters for image
> content (URLs starting with `http://`/`https://` or base64-encoded data) and formats
> them as image content blocks in the API message. The exact detection heuristic is
> adapter-specific.
>
> Notes: **[CLOSED in v0.1.0 remediation]** In v0.1, the adapter uses naming
> convention heuristics (parameters named `image`, `image_url`, `photo`, etc.) to
> detect image parameters. This is a runtime adapter concern, not a compile-time
> concern. A dedicated `Image` type is planned for Post-MVP, which would make
> detection compile-time.

### 6.3 tools Capability and Tool Declarations

**RULE CAP-TYP-03: tools capability required for tool-using agents**

> Plain English: The `tools` capability in a model's `caps:` list is required when
> the model is used in an agent that declares a `tools:` field. Tool declarations
> are typed and compiled independently of model capabilities — a model without
> `tools` in its `caps` can still exist in a program that has tool declarations.
> The capability check fires only at the agent declaration that binds the model to
> the tools.
>
> Formal: `agentDecl A with model M and tool list T: T ≠ ∅ ⟹ "tools" ∈ caps(M)`
>
> Grammar: Production [38] `agentDecl`, Production [39] `agentField` — `"tools" ":"
> "[" IDENT ("," IDENT)* "]"` binds tools to the agent. The `model:` field in the
> same agent binds the model. The compiler checks that the model's `caps` includes
> `tools` when the agent has a non-empty tool list.
>
> Valid:
> ```eaml
> model Claude = Model(
>   id: "claude-sonnet-4-20250514", provider: "anthropic",
>   caps: [json_mode, tools]
> )
>
> tool Lookup(query: string) -> SearchResult {
>   python %{ return search(query) }%
> }
>
> agent Research {
>   model: Claude              // Claude has 'tools' capability ✓
>   tools: [Lookup]
>   system: "Research assistant."
>   max_turns: 10
>   on_error: retry(2) then fail
> }
> ```
>
> Invalid:
> ```eaml
> model NoTools = Model(id: "basic", provider: "openai", caps: [json_mode])
>
> agent Broken {
>   model: NoTools             // NoTools does NOT have 'tools' capability
>   tools: [Lookup]
>   // → CAP010: Model 'NoTools' is missing required capabilities: [tools]
>   //           Required by agent 'Broken' at line N:M
> }
> ```
>
> Runtime: When tools are active, the adapter includes tool schemas in every API
> call made by the agent. The adapter handles tool call responses by invoking the
> Python implementation and feeding results back to the model.
>
> Notes: Tool declarations (Production [34] `toolDecl`) compile independently —
> they have typed signatures and Python implementations that are valid regardless
> of which model uses them. The capability check is on the model-to-agent binding,
> not on the tool declaration itself.

---

## 7. Runtime Adapter Behavior

### 7.1 Runtime Capability Activation Contract

**RULE CAP-RUN-01: Runtime adapter contract**

> Plain English: For each prompt call at runtime, the `eaml_runtime` adapter:
> 1. Receives the prompt's compiled capability set (embedded in generated code).
> 2. For each capability in the set, applies the corresponding API modification
>    for the current provider.
> 3. If the provider does not support a capability that was declared in `caps:`
>    (the developer declared a wrong capability), the adapter MUST raise
>    `CapabilityActivationError` — not silently ignore it.
>
> The compile-time CAP010 check and the runtime `CapabilityActivationError` are
> complementary, not redundant:
>
> | Check                                 | What It Catches                                      | When            |
> |---------------------------------------|------------------------------------------------------|-----------------|
> | CAP010 (compile-time)                 | Mismatch between `requires` and `caps:`              | Before codegen  |
> | `CapabilityActivationError` (runtime) | Mismatch between `caps:` and actual provider support | During API call |
>
> CAP010 catches errors in EAML source code. `CapabilityActivationError` catches
> errors in the developer's capability declaration — they claimed the model supports
> something it does not.
>
> Grammar: Not grammar-specific — this is a runtime contract.
>
> Runtime: The adapter MUST implement the following contract:
> ```python
> class CapabilityActivationError(EamlRuntimeError):
>     """Raised when a declared capability cannot be activated for the provider."""
>     def __init__(self, capability: str, provider: str, model_id: str):
>         super().__init__(
>             f"Cannot activate capability '{capability}' for provider "
>             f"'{provider}' (model '{model_id}'). The model declaration "
>             f"claims this capability but the provider does not support it."
>         )
> ```
>
> Notes: The runtime guard (Layer 5 §6.3) is always emitted by codegen. It is not
> optional. It catches misuse from non-EAML code that calls generated functions
> directly, bypassing the compiler's CAP010 check.

### 7.2 Per-Provider Activation Rules

For each of the five built-in capabilities, the following table shows how each
built-in provider activates it:

| Capability  | Provider      | API Parameter / Behavior Change                                                                                   |
|-------------|---------------|-------------------------------------------------------------------------------------------------------------------|
| `json_mode` | `"anthropic"` | Use tool-based structured output: pass return type JSON schema as a tool definition, extract structured response. |
| `json_mode` | `"openai"`    | Add `response_format: {"type": "json_schema", "json_schema": {"schema": ...}}` to request body.                   |
| `json_mode` | `"ollama"`    | Add `format: "json"` to request body. Schema enforcement is best-effort.                                          |
| `tools`     | `"anthropic"` | Add `tools: [{"name": "...", "description": "...", "input_schema": {...}}]` to request body.                      |
| `tools`     | `"openai"`    | Add `tools: [{"type": "function", "function": {"name": "...", "parameters": {...}}}]` to request body.            |
| `tools`     | `"ollama"`    | Add `tools: [...]` in OpenAI-compatible format.                                                                   |
| `vision`    | `"anthropic"` | Format user content as `[{"type": "image", "source": {...}}, {"type": "text", "text": "..."}]`.                   |
| `vision`    | `"openai"`    | Format user content as `[{"type": "image_url", "image_url": {"url": "..."}}, {"type": "text", "text": "..."}]`.   |
| `vision`    | `"ollama"`    | Add `images: ["<base64>"]` to request body.                                                                       |
| `streaming` | `"anthropic"` | Use `client.messages.stream(...)` instead of `client.messages.create(...)`.                                       |
| `streaming` | `"openai"`    | Add `stream: true` to request body.                                                                               |
| `streaming` | `"ollama"`    | Add `stream: true` to request body (default).                                                                     |
| `reasoning` | `"anthropic"` | Add `thinking: {"type": "enabled", "budget_tokens": N}` to request body.                                          |
| `reasoning` | `"openai"`    | Use reasoning-capable model endpoint. Reasoning is implicit in model selection.                                   |
| `reasoning` | `"ollama"`    | Provider-specific — depends on model support.                                                                     |

This table is the specification for adapter implementers. Where behavior is uncertain
or highly provider-specific, the adapter SHOULD consult the provider's current API
documentation and implement the most current activation mechanism. The adapter source
files are: `eaml_runtime/providers/anthropic_adapter.py`,
`eaml_runtime/providers/openai_adapter.py`, `eaml_runtime/providers/ollama_adapter.py`.

### 7.3 Capability Ordering and Combinations

**RULE CAP-RUN-02: Capability activation is order-independent**

> Plain English: When multiple capabilities are active simultaneously, all capabilities
> are activated as a set. The order in which they appear in `requires` or `caps:` does
> not affect behavior. Each capability's API modification is applied independently.
>
> Formal: Capability activation is commutative: `activate({A, B}) = activate({B, A})`.
>
> Grammar: Production [28] `capList` and Production [76] `requiresClause` list
> capabilities in source order, but the semantic representation is an unordered set.
>
> Valid:
> ```eaml
> // These are semantically identical:
> requires [json_mode, tools]
> requires [tools, json_mode]
> ```
>
> Runtime: The adapter applies each capability's API modification independently.
> When capabilities interact (e.g., `json_mode` + `tools` both modify the API
> request body), the adapter merges the modifications according to the provider's
> API specification.
>
> Notes: No known capability combinations conflict in v0.1. All five built-in
> capabilities can be active simultaneously. If a future capability introduces a
> conflict with an existing capability, the conflict MUST be documented as a new
> rule (CAP-RUN-03 or later) with a specific error or resolution strategy.

---

## 8. Custom Capabilities

### 8.1 Open Identifier Registry

Layer 5 §6.1 [CLOSED] specifies that capability names are **open identifiers validated
by semantic analysis.** Any valid IDENT token is grammatically acceptable as a capability
name. The semantic analysis pass validates against the capability registry.

This means the EAML capability registry is **open by design**: unknown capability names
are accepted with a warning (CAP001), not rejected with an error. This allows engineers
to declare capabilities before the built-in registry is updated, tracking the rapidly
evolving LLM capability landscape.

### 8.2 Custom Capability Rules

**RULE CAP-CUST-01: Custom capability names produce a warning**

> Plain English: Any IDENT that is not one of the five built-in capability names
> (`json_mode`, `tools`, `vision`, `streaming`, `reasoning`) is accepted with a
> CAP001 warning. The `--strict-caps` compiler flag promotes CAP001 to a fatal error,
> closing the registry for projects that want strict validation.
>
> Formal: `name ∉ BUILT_IN_CAPS ⟹ emit CAP001 warning (or error with --strict-caps)`
>
> Grammar: `[sem: cap-registry]` on Productions [28] and [76].
>
> Valid (with warning):
> ```eaml
> model Future = Model(
>   id: "future-model", provider: "openai",
>   caps: [json_mode, code_execution]
> )
> // → CAP001 warning: Unknown capability 'code_execution'. Built-in capabilities
> //   are: json_mode, tools, vision, streaming, reasoning.
>
> prompt RunCode(code: string) requires code_execution -> string {
>   user: "Execute: {code}"
> }
> // → CAP001 warning: Unknown capability 'code_execution' in requires clause.
> ```
>
> Runtime: The adapter receives the custom capability name but has no built-in
> activation logic for it. The adapter SHOULD log a warning: "Unknown capability
> '{name}' — no activation logic available. The capability is passed through but
> may have no effect." The adapter MUST NOT raise `CapabilityActivationError` for
> custom capabilities — the error is reserved for built-in capabilities that the
> provider does not support.
>
> Notes: Layer 5 §6.1: "Unknown capability: SEM CAP001 warning (not error) in v0.1."
> [Layer 5 §6.1 typo — correct code prefix is CAP, not SEM; the error code is CAP001.]
> "This allows engineers to declare capabilities before the registry is updated,
> using the --strict-caps flag to promote to error." The open registry design ensures
> EAML can track new provider features without requiring grammar or compiler changes.

**RULE CAP-CUST-02: Custom capabilities participate in subset checking**

> Plain English: Custom (non-built-in) capability names participate in the CAP010
> subset check identically to built-in names. If a prompt `requires [code_execution]`
> and the model's `caps` does not include `code_execution`, CAP010 is emitted —
> regardless of whether the name is in the built-in registry.
>
> Formal: The subset check `R ⊆ C` (CAP-CHK-01) uses exact string matching and
> does not distinguish between built-in and custom capability names.
>
> Grammar: Same as CAP-CHK-01.
>
> Valid:
> ```eaml
> model M = Model(id: "m", provider: "openai", caps: [code_execution])
> prompt P() requires code_execution -> string { user: "run" }
> // caps check passes: {code_execution} ⊆ {code_execution}
> // (CAP001 warning emitted for unknown name, but CAP010 check passes)
> ```
>
> Invalid:
> ```eaml
> model M = Model(id: "m", provider: "openai", caps: [json_mode])
> prompt P() requires code_execution -> string { user: "run" }
> // → CAP010: Model 'M' is missing required capabilities: [code_execution]
> ```
>
> Runtime: Custom capabilities that pass the subset check are passed through to
> the adapter, which logs them and takes no action (per CAP-CUST-01).
>
> Notes: This rule ensures that the CAP010 check is uniform — it does not special-case
> built-in vs. custom names. The only difference between built-in and custom
> capabilities is: (a) built-in names do not trigger CAP001, and (b) built-in
> capabilities have adapter activation logic.

---

## 9. Capability Error Catalog

### CAP001: Unknown capability name

**Condition:** A capability name in a `caps:` list or `requiresClause` is not in the
built-in registry.

**Severity:** WARNING (promoted to ERROR with `--strict-caps`)

**Message:** `CAP001: Unknown capability '{name}'. Built-in capabilities are: json_mode, tools, vision, streaming, reasoning.`

**Example:**
```eaml
model M = Model(id: "m", provider: "openai", caps: [JSON_MODE])
// → CAP001: Unknown capability 'JSON_MODE'. Built-in capabilities are:
//   json_mode, tools, vision, streaming, reasoning.
```

**Resolution:** Check the spelling and casing. Built-in names are lowercase snake_case.
If this is an intentional custom capability, the warning can be suppressed with
`--strict-caps=false` (default) or acknowledged as expected.

**Notes:** Triggered by CAP-MDL-02, CAP-REQ-04, CAP-REG-06, CAP-CUST-01. Layer 5 §6.1.

---

### CAP002: Duplicate capability name

**Condition:** A capability name appears more than once in a single `caps:` list
or `requiresClause`.

**Severity:** WARNING

**Message:** `CAP002: Duplicate capability '{name}' in {location}. The duplicate is ignored.`

**Example:**
```eaml
caps: [json_mode, tools, json_mode]
// → CAP002: Duplicate capability 'json_mode' in caps list. The duplicate is ignored.
```

**Resolution:** Remove the duplicate entry.

**Notes:** Triggered by CAP-REQ-06. This is OQ-02 — Layer 5 does not specify duplicate
behavior, but warning is the recommended and adopted resolution.

---

### CAP010: Capability mismatch

**Condition:** A prompt's `requiresClause` names a capability that the model used at
the call site does not declare in its `caps:` list.

**Severity:** FATAL

**Message:**
```
CAP010: Model '{model_name}' is missing required capabilities: [{caps}]
        Required by prompt '{prompt_name}' at line {line}:{col}
        Hint: Model '{model_name}' supports: [{model_caps}]
        Add the missing capabilities to the model declaration, or
        use a model that supports {missing_caps}.
```

**Example:**
```eaml
model Basic = Model(id: "basic", provider: "openai", caps: [])

prompt Classify(text: string) requires json_mode -> SentimentResult {
  user: "Classify: {text}"
}

let result: SentimentResult = Basic.call(Classify(text: "hello"))
// → CAP010: Model 'Basic' is missing required capabilities: [json_mode]
//           Required by prompt 'Classify' at line 3:1
//           Hint: Model 'Basic' supports: []
//           Add the missing capabilities to the model declaration, or
//           use a model that supports json_mode.
```

**Resolution:** (a) Add the missing capability to the model's `caps` list, (b) remove
the capability from the prompt's `requires` clause, or (c) use a different model that
declares the capability.

**Notes:** Triggered by CAP-CHK-01, CAP-CHK-03. Layer 5 §6.3 [CLOSED]. One CAP010 is
emitted per missing capability. Fatal — no path to continue compilation.

---

### CAP020: json_mode with bare string return type

**Condition:** A prompt declares `requires json_mode` (or includes `json_mode` in a
bracketed list) but its return type is bare `string`.

**Severity:** WARNING

**Message:** `CAP020: Prompt '{name}' requires json_mode but returns 'string'. Consider using a schema type to get Pydantic validation of the JSON response.`

**Example:**
```eaml
prompt Extract(text: string) requires json_mode -> string {
  user: "Extract as JSON: {text}"
}
// → CAP020: Prompt 'Extract' requires json_mode but returns 'string'.
//   Consider using a schema type to get Pydantic validation.
```

**Resolution:** Change the return type to a schema type or literal union. If raw JSON
string is intentional, the warning can be acknowledged.

**Notes:** Triggered by CAP-TYP-01. This is OQ-03 — adopted as a warning. Cross-reference:
TYPESYSTEM.md TS-RET-01.

---

### CapabilityActivationError (Runtime)

**Condition:** At runtime, the adapter attempts to activate a built-in capability that
the provider does not actually support. This occurs when the developer declared a
capability in `caps:` that the provider does not offer.

**Severity:** RUNTIME (exception raised during API call)

**Message:** `CapabilityActivationError: Cannot activate capability '{cap}' for provider '{provider}' (model '{model_id}'). The model declaration claims this capability but the provider does not support it.`

**Example:** A model declares `caps: [reasoning]` with `provider: "ollama"` for a model
that does not support reasoning. The compile-time checks pass (CAP010 only checks
`requires ⊆ caps`), but at runtime the adapter cannot activate reasoning for this
Ollama model.

**Resolution:** Remove the unsupported capability from the model's `caps:` list, or
use a different model/provider that supports it.

**Notes:** This is NOT a compile-time error — it is a runtime exception. It is listed
here for completeness because it is part of the capability system's error surface. The
compile-time CAP010 check and runtime `CapabilityActivationError` are complementary
(see §7.1 CAP-RUN-01).

---

## 10. Post-MVP Capability Features

The following capability features are explicitly out of scope for v0.1 and reserved
for future versions.

### 10.1 Tool Capability Requirements

**Feature:** Allow `requiresClause` on tool declarations.

**Why deferred:** Tool bodies in v0.1 are Python implementations (`python %{ }%`), not
LLM calls. Capabilities are relevant to LLM interactions, and tools interact with LLMs
only through agents. The agent's model binding already handles capability checking.

**Current behavior:** `requires` in a tool declaration is a parse error — `requiresClause`
is not in Production [34] `toolDecl` grammar.

**Error:** `SYN error: unexpected token 'requires'` (parser error, not a specific Post-MVP code).

**Planned v-next behavior:** `tool T(...) requires [tools] -> R { ... }` — tools that
call other tools or make LLM calls may declare their own capability requirements.

### 10.2 Agent-Level Capability Requirements

**Feature:** Allow `requiresClause` on agent declarations, enabling agents to declare
capabilities they need from their model beyond what their individual prompts require.

**Why deferred:** In v0.1, agent capability requirements are derived from the union of
their prompts' and tools' requirements. Explicit agent-level requirements add complexity
without clear v0.1 use cases.

**Current behavior:** `requires` in an agent declaration is a parse error —
`requiresClause` is not in Production [38] `agentDecl` grammar.

**Planned v-next behavior:** `agent A requires [streaming] { ... }` — agents that need
streaming for their orchestration loop can declare this independently.

### 10.3 Capability Inheritance

**Feature:** If schema `B extends A`, capabilities required by prompts using `A` are
automatically required by prompts using `B`.

**Why deferred:** Schema inheritance itself is Post-MVP (SYN083). Capability inheritance
depends on it.

**Current behavior:** Not in grammar. Schema `extends` keyword triggers SYN083.

### 10.4 Conditional Capabilities

**Feature:** Capabilities that are required only under certain conditions, e.g.,
`requires json_mode when returnType is schema`.

**Why deferred:** Adds significant complexity to the capability checker without clear
v0.1 use cases. The explicit `requires` clause is simpler and sufficient.

**Current behavior:** Not in grammar or Layer 5.

### 10.5 Strict Custom Capability Registration

**Feature:** A formal mechanism for registering custom capabilities with descriptions
and adapter activation hooks, beyond the current open-identifier approach.

**Why deferred:** The open-identifier approach with CAP001 warnings (Layer 5 §6.1)
is sufficient for v0.1. Formal registration adds ceremony without clear immediate value.

**Planned v-next behavior:** A `capability` declaration or configuration file that
registers custom capability names with their adapter activation logic.

---

## Verification Report — EAML CAPABILITIES.md v0.1.0

| Group             | Checks  | Passed  | Failed  | N/A   |
|-------------------|---------|---------|---------|-------|
| A — Completeness  | 7       | 7       | 0       | 0     |
| B — Grammar Align | 5       | 5       | 0       | 0     |
| C — Layer 5       | 5       | 5       | 0       | 0     |
| D — TYPESYSTEM.md | 4       | 4       | 0       | 0     |
| E — Quality       | 5       | 5       | 0       | 0     |
| **Total**         | **26**  | **26**  | **0**   | **0** |

Failed checks: 0
Open Questions: 0 (all closed in v0.1.0 remediation)

### A — Completeness Checks

**A1[PASS]** All five built-in capabilities have complete rule blocks in Section 2:
- `json_mode` → CAP-REG-01
- `tools` → CAP-REG-02
- `vision` → CAP-REG-03
- `streaming` → CAP-REG-04
- `reasoning` → CAP-REG-05

**A2[PASS]** Both requiresClause forms specified:
- Bare form → CAP-REQ-01
- Bracketed form → CAP-REQ-02
- Absent clause → CAP-REQ-03 (correctly described as "implicit empty, not unchecked")

**A3[PASS]** Capability check algorithm in §5.1 (CAP-CHK-01) formally specified as
R ⊄ C → CAP010. Timing documented: after type checking (step 5), before codegen (step 7).

**A4[PASS]** Four OPEN QUESTIONs from initial draft, all CLOSED in v0.1.0 remediation:
- OQ-01: Call site syntax (§3.3, §5.2) → CLOSED: two-mechanism binding for v0.1
- OQ-02: Duplicate capability names (§4.4) → CLOSED: CAP002 warning
- OQ-03: json_mode / return type interaction (§6.1) → CLOSED: CAP020 warning
- OQ-04: vision / image parameter detection (§6.2) → CLOSED: runtime heuristic

**A5[PASS]** Post-MVP capability features documented in §10:
- §10.1: Tool capability requirements
- §10.2: Agent-level capability requirements
- §10.3: Capability inheritance
- §10.4: Conditional capabilities
- §10.5: Strict custom capability registration

**A6[PASS with note]** Layer 5 [GRAMMAR IMPACT] annotations with corresponding rules:
- §6.2 [GRAMMAR IMPACT] (requiresClause syntax) → CAP-REQ-01, CAP-REQ-02 ✓
- §10.2 [GRAMMAR IMPACT] (modelDecl caps field) → CAP-MDL-01 ✓
  NOTE: Layer 5 §10.2 shows `capList` (without `?`), but grammar Production [27]
  has `capList?`. Empty caps lists (`caps: []`) are valid. CAPABILITIES.md CAP-MDL-01
  correctly uses `capList?` per the grammar. Layer 5 §10.2 has a typo.

**A7[PASS]** Error catalog (§9) is cross-referenced:
- CAP001 ← CAP-MDL-02, CAP-REQ-04, CAP-REG-06, CAP-CUST-01
- CAP002 ← CAP-REQ-06
- CAP010 ← CAP-CHK-01, CAP-CHK-03
- CAP020 ← CAP-TYP-01
- CapabilityActivationError ← CAP-RUN-01
All error codes are referenced by at least one rule block.

### B — Grammar Alignment Checks

**B1[PASS]** Grammar production citations verified against grammar.ebnf:
- [5] IDENT — line 197: confirmed exists
- [27] modelDecl — line 397: confirmed exists, contains `caps: "[" capList? "]"`
- [28] capList — line 425: confirmed exists, contains `[sem: cap-registry]`
- [31] promptDecl — line 444: confirmed exists, contains `requiresClause?`
- [33] promptField — line 461: confirmed exists
- [34] toolDecl — line 471: confirmed exists (no requiresClause)
- [38] agentDecl — line 515: confirmed exists
- [39] agentField — line 519: confirmed exists, contains tools field
- [50] literalUnion — line 624: confirmed exists (referenced via typeExpr)
- [72] paramList — line 795: confirmed exists
- [73] param — line 799: confirmed exists
- [76] requiresClause — line 821: confirmed exists with both forms
No phantom productions cited. No [42a]-style errors.

**B2[PASS]** Production [27] `modelDecl` — caps field syntax correctly cited:
Grammar text: `"caps" ":" "[" capList? "]"`. CAPABILITIES.md states this accurately.

**B3[PASS]** Production [28] `capList` — `[sem: cap-registry]` annotation cited:
Grammar text at line 426-427: `[sem: cap-registry] — names validated against the
capability registry by semantic analysis, not grammar.` CAPABILITIES.md cites this
annotation in CAP-MDL-01, CAP-MDL-02, CAP-REQ-04, and CAP-CHK-01.

**B4[PASS]** Production [31] `promptDecl` — `requiresClause?` optional position cited:
ε/FOLLOW safety verified: FIRST(requiresClause) = {"requires"}, FOLLOW = {"->"},
intersection = ∅. Matches grammar.ebnf comment at lines 446-449.

**B5[PASS]** Production [76] `requiresClause` — BOTH forms verified:
Grammar text at line 832-833:
`requiresClause ::= "requires" ( IDENT | "[" ( IDENT ( "," IDENT )* )? "]" )`
This matches CAP-REQ-01 (bare: `"requires" IDENT`) and CAP-REQ-02
(bracketed: `"requires" "[" ... "]"`). Empty brackets `requires []` is covered by
the inner `( IDENT ( "," IDENT )* )?` being optional.

### C — Layer 5 Compliance Checks

**C1[PASS]** Built-in capability names: exactly five, exactly as spelled in Layer 5 §6.1:
`json_mode`, `tools`, `vision`, `streaming`, `reasoning`. Not four, not six.

**C2[PASS]** CAP010 is fatal (not warning). The spec states this explicitly in §5.3
(CAP-CHK-03) and §5.4 (CAP-CHK-04) with Layer 5 §6.3 [CLOSED] citation. No path
exists to compile a program with an unresolved CAP010.

**C3[PASS]** requiresClause appears ONLY on promptDecl in v0.1. §4.3 (CAP-REQ-05)
specifies this restriction. §10.1 and §10.2 document Post-MVP extensions.

**C4[PASS]** Capability checking is compile-time, not runtime. §1.2 explicitly
distinguishes from BAML's runtime approach. §5.1 documents timing in compiler pipeline.

**C5[PASS]** The caps: field in modelDecl is the developer's declaration of model
capabilities and is trusted by the compiler. §3.1 (CAP-MDL-01): "The compiler does
NOT verify the declaration against the actual provider's API." §3.4 (CAP-MDL-04):
"The compiler does NOT verify that a capability is plausible for a given provider."

### D — TYPESYSTEM.md Consistency Checks

**D1[PASS]** The three-layer validation model in §1.3 is consistent with TYPESYSTEM.md
§1.2. TYPESYSTEM.md describes two layers: (1) compile-time type checking and
(2) runtime Pydantic validation. CAPABILITIES.md correctly inserts capability checking
as a third layer between them. The header section explicitly notes: "TYPESYSTEM.md
describes a 'two-layer validation model'... Capability checking is a third layer that
sits between them."

**D2[PASS]** The json_mode/return type interaction rule in §6.1 (CAP-TYP-01) cites:
- TS-RET-01 (prompt return type) — verified at TYPESYSTEM.md line 1312
- TS-RET-03 (literal union as return type) — verified at TYPESYSTEM.md line 1335
Both rule IDs exist and are accurately described.

**D3[PASS]** CAPABILITIES.md does not contradict any rule in TYPESYSTEM.md. §6.1 states
the combination of `json_mode` + `string` return type is a CAP warning, not a type
error — consistent with TS-RET-01 which accepts any valid `typeExpr` as a return type.

**D4[PASS]** The tools capability description in §6.3 (CAP-TYP-03) references:
- Production [38] agentDecl — verified at grammar.ebnf line 515
- Production [39] agentField — verified at grammar.ebnf line 519, contains
  `"tools" ":" "[" IDENT ("," IDENT)* "]"` matching the cited structure.

### E — Document Quality Checks

**E1[PASS]** Rule block format with Runtime: field used consistently. Spot-checked:
- CAP-REG-01 (§2.1) — has Runtime field with provider table ✓
- CAP-MDL-01 (§3.1) — has Runtime field ✓
- CAP-REQ-03 (§4.1) — has Runtime field ✓
- CAP-CHK-01 (§5.1) — has Runtime field ✓
- CAP-TYP-01 (§6.1) — has Runtime field ✓

**E2[PASS]** Normative language consistent: MUST for requirements (§7.1 CAP-RUN-01:
"adapter MUST apply"), SHOULD for recommendations (§6.1 CAP-TYP-01: "SHOULD be a
schema type"), MAY for optional (§4.1 CAP-REQ-01: "prompt MAY declare"). No lowercase
"should" where MUST is intended.

**E3[PASS]** Compile-time CAP010 and runtime CapabilityActivationError are distinct:
- CAP010: §5.3, §9 (Severity: FATAL, compile-time)
- CapabilityActivationError: §7.1, §9 (Severity: RUNTIME, exception)
§7.1 explicitly documents they are "complementary, not redundant" with a comparison table.

**E4[PASS]** Table of Contents matches actual document structure. All section numbers
and titles verified against the document body.

**E5[PASS]** Document is self-contained. A developer with CAPABILITIES.md, grammar.ebnf,
and TYPESYSTEM.md can implement both the semantic analysis CAP checker (§5 algorithm,
§9 error catalog) and the eaml_runtime adapter layer (§7 contract, §7.2 provider table)
without consulting Layer 1–4 documents.

### Known Limitations (v0.1.0)

1. **Call site syntax** — `Model.call(Prompt(...))` is not a formal grammar production
   in v0.1. Resolved via agent binding and let-binding dot-call. Grammar production
   planned for v0.2.

2. **Vision image parameter detection** — runtime adapter heuristic in v0.1. Formal
   `Image` type planned for Post-MVP.

3. **Post-MVP features deferred:** Tool requirements, agent requirements, capability
   inheritance, conditional capabilities, strict custom registration.

### Grammar.ebnf Production Citations

| Production          | Line  | Exists  | Description                              |
|---------------------|-------|---------|------------------------------------------|
| [5] IDENT           | 197   | ✓       | Identifier token                         |
| [27] modelDecl      | 397   | ✓       | Model declaration with caps field        |
| [28] capList        | 425   | ✓       | Capability list with [sem: cap-registry] |
| [31] promptDecl     | 444   | ✓       | Prompt declaration with requiresClause?  |
| [33] promptField    | 461   | ✓       | Prompt body fields                       |
| [34] toolDecl       | 471   | ✓       | Tool declaration (no requiresClause)     |
| [38] agentDecl      | 515   | ✓       | Agent declaration                        |
| [39] agentField     | 519   | ✓       | Agent body fields including tools        |
| [50] literalUnion   | 624   | ✓       | Literal union type                       |
| [72] paramList      | 795   | ✓       | Parameter list                           |
| [73] param          | 799   | ✓       | Parameter definition                     |
| [76] requiresClause | 821   | ✓       | Requires clause (both forms)             |

All 12 cited productions physically verified in grammar.ebnf. No phantom productions.

### Cross-References to TYPESYSTEM.md

| Rule ID   | Location  | Exists  | Description                         |
|-----------|-----------|---------|-------------------------------------|
| TS-RET-01 | Line 1312 | ✓       | Prompt return type                  |
| TS-RET-02 | Line 1359 | ✓       | Tool return type and -> null        |
| TS-RET-03 | Line 1335 | ✓       | Literal union as prompt return type |

All 3 cited TYPESYSTEM.md rules physically verified. No phantom rule IDs.