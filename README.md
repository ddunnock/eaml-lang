# EAML — Engineering AI Markup Language

> A typed, compiled DSL for structured LLM interaction with in-process Python bridging.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Version](https://img.shields.io/badge/version-0.1.0--alpha-orange.svg)]()
[![Rust](https://img.shields.io/badge/compiler-Rust-orange.svg)](https://www.rust-lang.org/)
[![Python](https://img.shields.io/badge/runtime-Python%203.11%2B-3776AB.svg)](https://www.python.org/)
[![Pydantic v2](https://img.shields.io/badge/validation-Pydantic%20v2-E92063.svg)](https://docs.pydantic.dev/latest/)
[![Status: Pre-release](https://img.shields.io/badge/status-pre--release-yellow.svg)]()

---

EAML lets you write LLM prompts as first-class typed functions. You declare your schemas, your models, and your prompts in `.eaml` files. The `eamlc` compiler checks your types at compile time, validates capability requirements against your model declarations, and emits Python code backed by Pydantic v2 — which your existing Python environment runs directly.

```eaml
schema EntityResult {
    label: string
    score: float<0.0, 1.0>
    tags:  string[]
    source: string?
}

model Claude = Model(
    id:       "claude-sonnet-4-20250514",
    provider: "anthropic",
    caps:     [json_mode, tools, vision]
)

prompt extractEntities(text: string) requires [json_mode]
    -> EntityResult[] {
    user: "Extract all named entities from the following text: {text}"
    max_retries: 2
}
```

```bash
eamlc compile analysis.eaml
python analysis.py
```

---

## Table of Contents

- [Why EAML](#why-eaml)
- [How It Works](#how-it-works)
- [Key Differentiators](#key-differentiators)
- [Requirements](#requirements)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Language Reference](#language-reference)
- [Project Structure](#project-structure)
- [Repository Layout](#repository-layout)
- [Roadmap](#roadmap)
- [Contributing](#contributing)
- [License](#license)

---

## Why EAML

Most LLM code today is string concatenation wrapped in try/except blocks. You discover type mismatches at runtime, capability errors in production, and schema drift between your prompt intent and your Python models only after a failed API call.

EAML moves those checks left:

| Problem             | Current approach                                               | EAML                                                                                      |
|---------------------|----------------------------------------------------------------|-------------------------------------------------------------------------------------------|
| Schema drift        | Pydantic model and prompt written separately, diverge silently | Schema declared once, shared between prompt and generated Pydantic model                  |
| Capability mismatch | Runtime error when model doesn't support `json_mode`           | **Fatal compile-time error** — CAP010 before any code runs                                |
| Type errors         | Discovered at runtime or via test                              | Caught by semantic analysis at `eamlc compile`                                            |
| Python bridge       | Subprocess, HTTP, or wrapper boilerplate                       | In-process `python %{ }%` blocks that run in your existing `venv`                         |
| Error messages      | Generic exception messages                                     | Category-prefixed codes (SYN, SEM, CAP, TYP, PYB, RES) pointing to exact source locations |

---

## How It Works

```
your-project/
  analysis.eaml        ← you write this
       │
       ▼
  eamlc (Rust binary)  ← reads .eaml, checks types, validates capabilities
       │
       ▼
  analysis.py          ← generated Python (you run this)
  + eaml_runtime       ← pip-installable runtime (Pydantic v2, provider adapters)
       │
       ▼
  your venv            ← where pandas, numpy, anthropic SDK etc. already live
```

The compiler never executes Python. It reads `.eaml` text, runs the type checker and capability validator, and writes `.py` text. Your Python environment handles execution — including all your installed packages, your API keys, and your existing tooling.

---

## Key Differentiators

Compared to [BAML](https://github.com/BoundaryML/baml), the closest prior art:

**Compile-time capability checking.** EAML validates `requires` clauses against model `caps` declarations at compile time. Requesting `json_mode` from a model that doesn't declare it is a fatal error — `CAP010` — before any code runs. BAML defers this to runtime.

**In-process Python bridge.** Tool implementations live in `python %{ }%` blocks and run directly in the user's Python environment. No subprocess, no HTTP roundtrip, no package re-installation — your `venv` dependencies are available immediately.

**Nominal type system.** Two schemas with identical field structures are distinct types and are not interchangeable. `Confidence` and `Probability` are different types even if both have a single `score: float` field. This maps directly to how domain concepts work in engineering workflows.

**Provider-agnostic model declarations.** Provider is a string field in the model declaration. Switching between `"anthropic"`, `"openai"`, and `"ollama"` requires changing one line.

**Category-prefixed error codes.** All errors follow the format `PREFIX###` where prefix indicates the compiler phase: `SYN` (syntax), `SEM` (semantic), `CAP` (capability), `TYP` (type), `PYB` (Python bridge), `RES` (resolution). You know immediately which phase to look at.

---

## Requirements

**To compile `.eaml` files:**
- `eamlc` binary (see [Installation](#installation))

**To run generated Python files:**
- Python 3.11 or later
- `eaml_runtime` package (`pip install eaml-runtime`)
- Pydantic v2 (`pip install pydantic>=2.0`)
- Provider SDK for your chosen model provider (e.g. `pip install anthropic`)

**To build `eamlc` from source:**
- Rust stable toolchain 1.75+
- `cargo` (included with Rust)

---

## Installation

### Compiler binary (pre-built)

> Pre-built binaries are not yet available. EAML is in pre-release. Build from source below.

### Build from source

```bash
git clone https://github.com/your-org/eaml.git
cd eaml
cargo build --release
```

The compiled binary is at `target/release/eamlc`. Add it to your `PATH`:

```bash
# macOS / Linux
export PATH="$PATH:$(pwd)/target/release"

# Or copy to a directory already on PATH
cp target/release/eamlc /usr/local/bin/
```

Verify the installation:

```bash
eamlc --version
# eamlc 0.1.0
```

### Python runtime

```bash
pip install eaml-runtime pydantic>=2.0
```

> Pre-release note: `eaml-runtime` is not yet published to PyPI. Install from the repository:
> ```bash
> pip install ./python/eaml_runtime
> ```

---

## Quick Start

### 1. Write a schema and a prompt

Create `sentiment.eaml`:

```eaml
// Model declaration — provider-agnostic
model Claude = Model(
    id:       "claude-sonnet-4-20250514",
    provider: "anthropic",
    caps:     [json_mode]
)

// Schema — generates a Pydantic v2 BaseModel
schema SentimentResult {
    label:      "positive" | "negative" | "neutral"
    confidence: float<0.0, 1.0>
    summary:    string<max: 200>
}

// Prompt — typed LLM function with capability requirements
prompt analyzeSentiment(text: string) requires json_mode
    -> SentimentResult {
    system: "You are a precise sentiment analysis assistant."
    user:   "Analyze the sentiment of this text: {text}"
    temperature: 0.1
    max_retries: 2
}
```

### 2. Compile

```bash
eamlc compile sentiment.eaml
```

Successful compilation produces `sentiment.py`. Type errors, capability mismatches, and syntax errors are reported with exact source locations and codes:

```
sentiment.eaml:14:5  CAP010  Prompt 'analyzeSentiment' requires 'json_mode'
                             but model 'Claude' does not declare this capability.
                             Add 'json_mode' to Claude's caps list.
```

### 3. Run

```bash
python sentiment.py
```

The generated file imports from `eaml_runtime` and uses your Anthropic API key from the environment. The `SentimentResult` Pydantic model validates the LLM response automatically; if validation fails, the retry policy from `max_retries` applies.

### 4. Use a tool with a Python bridge

```eaml
tool fetchDocument(url: string) -> string {
    python %{
import httpx

def fetch_document(url: str) -> str:
    response = httpx.get(url, timeout=10.0)
    response.raise_for_status()
    return response.text
    }%
}
```

The `python %{ }%` block runs in your Python environment. `httpx` must be installed in your active `venv` — the compiler does not manage Python packages.

---

## Language Reference

### Declarations

| Keyword   | Purpose                                       | Example                                                                   |
|-----------|-----------------------------------------------|---------------------------------------------------------------------------|
| `model`   | Declare an LLM with provider and capabilities | `model GPT4 = Model(id: "gpt-4o", provider: "openai", caps: [json_mode])` |
| `schema`  | Define a nominal data type                    | `schema Result { score: float<0.0,1.0> }`                                 |
| `prompt`  | Typed LLM function                            | `prompt classify(text: string) requires json_mode -> Label { ... }`       |
| `tool`    | Python-bridged callable                       | `tool search(query: string) -> string { python %{ ... }% }`               |
| `agent`   | Orchestrator with model + tools + policy      | `agent Analyst { model: Claude, tools: [search] }`                        |
| `let`     | Typed variable binding                        | `let threshold: float = 0.85`                                             |
| `import`  | EAML file or Python library                   | `import "./shared.eaml"` / `import python "pandas" as pd`                 |

### Type System

| EAML Type               | Python / Pydantic                 | Notes                                       |
|-------------------------|-----------------------------------|---------------------------------------------|
| `string`                | `str`                             |                                             |
| `int`                   | `int`                             |                                             |
| `float`                 | `float`                           |                                             |
| `bool`                  | `bool`                            |                                             |
| `null`                  | `None`                            | Explicit null value, distinct from optional |
| `Tag[]`                 | `List[Tag]`                       | Single dimension only in v0.1               |
| `Tag?`                  | `Optional[Tag]`                   | Field may be absent                         |
| `Tag[]?`                | `Optional[List[Tag]]`             | Entire array may be absent                  |
| `Tag?[]`                | `List[Optional[Tag]]`             | Array required, elements may be absent      |
| `float<0.0, 1.0>`       | `float` + `Field(ge=0.0, le=1.0)` | Inclusive bounds                            |
| `string<max: 200>`      | `str` + `Field(max_length=200)`   |                                             |
| `int<min: 0, max: 100>` | `int` + `Field(ge=0, le=100)`     |                                             |
| `"yes" \| "no"`         | `Literal["yes", "no"]`            | Minimum 2 members                           |

**Typing discipline:** Nominal. Two schemas with identical fields are distinct types and are not interchangeable.

### Error Codes

| Prefix  | Phase                 | Example                                    |
|---------|-----------------------|--------------------------------------------|
| `SYN`   | Parser — syntax error | `SYN042`: Multi-dimensional array          |
| `SEM`   | Semantic analysis     | `SEM060`: Chained comparison               |
| `CAP`   | Capability validation | `CAP010`: Capability not declared on model |
| `TYP`   | Type checking         | `TYP001`: Unknown type name                |
| `PYB`   | Python bridge         | `PYB001`: Bridge block parse error         |
| `RES`   | Name resolution       | `RES001`: Undefined reference              |

---

## Project Structure

### Crate dependency graph

```
eaml-errors  ←  eaml-lexer  ←  eaml-parser  ←  eaml-semantic  ←  eaml-codegen  ←  eaml-cli
```

Each crate has a single, bounded responsibility. `eaml-errors` is at the bottom of the dependency graph — all other crates depend on it for shared error types. `eaml-cli` is the only crate that assembles the full pipeline.

| Crate           | Responsibility                                                  |
|-----------------|-----------------------------------------------------------------|
| `eaml-errors`   | Shared error types, error codes, diagnostic formatting          |
| `eaml-lexer`    | Tokenizer — handles template string modes, Python block capture |
| `eaml-parser`   | AST construction from token stream                              |
| `eaml-semantic` | Name resolution, type checking, capability validation           |
| `eaml-codegen`  | Python + Pydantic v2 emission from typed AST                    |
| `eaml-cli`      | `eamlc` binary — argument parsing, pipeline orchestration       |

The Python runtime (`python/eaml_runtime`) is a separate pip-installable package. It is not a Rust crate. It contains the provider adapters, retry logic, and base classes that generated `.py` files import from.

---

## Repository Layout

```
eaml/
├── Cargo.toml                    # Workspace manifest
├── README.md
├── LICENSE
│
├── crates/
│   ├── eaml-errors/              # Shared error types (no upstream deps)
│   ├── eaml-lexer/               # Tokenizer
│   ├── eaml-parser/              # AST construction
│   ├── eaml-semantic/            # Name resolution, type checking, cap validation
│   ├── eaml-codegen/             # Python / Pydantic v2 code generation
│   └── eaml-cli/                 # eamlc binary
│
├── python/
│   └── eaml_runtime/             # pip-installable runtime package
│       ├── pyproject.toml
│       ├── eaml_runtime/
│       │   ├── providers/        # anthropic, openai, ollama adapters
│       │   ├── retry.py          # Retry policy implementation
│       │   └── base.py           # Base classes for generated code
│       └── tests/
│
├── spec/
│   ├── grammar.ebnf              # Formal W3C EBNF grammar (v0.1.0)
│   ├── TYPESYSTEM.md             # Type system specification
│   └── ERRORS.md                 # Complete error code catalog
│
├── ai-context/                   # Grounding documents for AI-assisted development
│   ├── layer1-notation.md        # W3C EBNF notation reference
│   ├── layer2-patterns.md        # Grammar structural patterns
│   ├── layer3-prior-art.md       # Lox and BAML prior art analysis
│   ├── layer4-theory.md          # LL(1) theory, FIRST/FOLLOW, Pratt parsing
│   └── layer5-decisions.md       # Authoritative design decisions (CLOSED)
│
├── examples/
│   ├── 01-hello-world/
│   ├── 02-sentiment-analysis/
│   ├── 03-entity-extraction/
│   ├── 04-python-bridge-tool/
│   ├── 05-multi-model-agent/
│   ├── 06-capability-error/      # Negative test — demonstrates CAP010
│   └── 07-all-type-variants/     # Exercises every type form
│
├── tests/
│   ├── compile-pass/             # .eaml files that must compile successfully
│   ├── compile-fail/             # .eaml files that must fail with specific codes
│   └── codegen/                  # Generated Python must produce correct output
│
└── editors/
    └── vscode/                   # VS Code extension (syntax highlighting, LSP)
```

---

## Roadmap

EAML v0.1.0 establishes the core language. The following are explicitly post-v0.1 and are blocked with specific error codes in the current compiler:

| Feature                                        | Blocked by          | Planned  |
|------------------------------------------------|---------------------|----------|
| `pipeline` declaration and `>>` operator       | `SYN080` / `SYN081` | v0.2     |
| `enum` declaration                             | `SYN082`            | v0.2     |
| Schema inheritance (`extends`)                 | `SYN083`            | v0.2     |
| Type inference on `let` bindings               | `SEM` error         | v0.2     |
| Native tool bodies (non-Python)                | `SYN050`            | v0.3     |
| Field annotations (`@description`, `@alias`)   | `SYN090`            | v0.3     |
| VS Code extension with full LSP                | —                   | v0.2     |
| Additional providers (Google, Mistral, Cohere) | —                   | v0.2     |

---

## Contributing

EAML is in early development. The spec documents in `spec/` and `ai-context/` are the authoritative source of truth. Before opening a pull request that changes language behavior, read `ai-context/layer5-decisions.md` — decisions marked `[CLOSED]` are final for v0.1.

### First steps

```bash
git clone https://github.com/your-org/eaml.git
cd eaml
cargo build
cargo test
```

### Where to start

- **Grammar or parser bugs:** See `spec/grammar.ebnf` and `crates/eaml-parser/`
- **Type system issues:** See `spec/TYPESYSTEM.md` and `crates/eaml-semantic/`
- **Code generation:** See `spec/TYPESYSTEM.md §10` and `crates/eaml-codegen/`
- **Runtime / provider adapters:** See `python/eaml_runtime/providers/`
- **Error messages:** See `spec/ERRORS.md` and `crates/eaml-errors/`

### Spec-first policy

EAML follows a spec-first development policy. Changes to language behavior require a spec update before implementation. Grammar changes go to `spec/grammar.ebnf` first; type system changes go to `spec/TYPESYSTEM.md` first. Pull requests that implement behavior not reflected in the spec will be asked to update the spec as part of the review.

### Issues

When filing a compiler bug, include:

1. The `.eaml` source that triggers the bug
2. The error or output you received
3. What you expected instead
4. Your `eamlc --version` output

---

## License

MIT — see [LICENSE](LICENSE).

---

<sub>EAML is not affiliated with or endorsed by any LLM provider. Provider names are trademarks of their respective owners.</sub>