# Feature Research

**Domain:** DSL compiler for LLM integrations
**Researched:** 2026-03-15
**Confidence:** HIGH

## Feature Landscape

### Table Stakes (Users Expect These)

Features users assume exist. Missing these = product feels incomplete.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Correct compilation | Programs that parse must produce valid Python | HIGH | End-to-end correctness across 84 productions |
| Meaningful error messages | Span-accurate errors with hints and suggestions | MEDIUM | codespan-reporting handles display; quality is in message content |
| Type checking | Catch type mismatches before runtime | MEDIUM | Bounded types, literal unions, nullable types per TYPESYSTEM.md |
| Capability validation | Detect model/prompt capability mismatches at compile time | MEDIUM | CAP010 is fatal; prevents runtime API errors |
| CLI with compile/check commands | Basic compiler invocation | LOW | clap derive, straightforward wiring |
| Valid Python output | Generated code must import cleanly, type-check with mypy, and run | HIGH | Python indentation, import ordering, Pydantic v2 patterns |
| Multi-provider support | Anthropic, OpenAI, Ollama at minimum | MEDIUM | Provider adapter pattern in runtime; each has SDK quirks |
| Template string interpolation | `{expr}` in prompts must resolve correctly | MEDIUM | Brace-depth counting in lexer, validation in semantic analysis |

### Differentiators (Competitive Advantage)

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Compile-time capability checking | Catch "model doesn't support json_mode" before any API call | LOW | Unique to EAML — BAML/Instructor don't do this |
| Python bridge blocks | Inline Python for data preprocessing in tools | MEDIUM | `python %{ }%` — no other LLM DSL has this |
| Declarative agent composition | `agent` blocks wire prompts + tools declaratively | HIGH | More structured than raw SDK code |
| Structured output guarantee | Pydantic validation + retry loop built into generated code | MEDIUM | validate_or_retry pattern in runtime |
| Provider-agnostic declarations | Switch providers by changing one `model` block | LOW | Runtime adapters abstract provider differences |

### Anti-Features (Commonly Requested, Often Problematic)

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Runtime type inference | "Don't make me write types" | Ambiguous types make errors cryptic; compile-time guarantees lost | Explicit types with good editor support |
| Auto-retry without limits | "Just keep trying until it works" | Infinite loops, runaway API costs | Configurable retry with exponential backoff and max attempts |
| Implicit provider selection | "Just pick the best model" | Provider choice has cost/latency/quality tradeoffs user must own | Explicit model declarations |
| Hot-reload compilation | "Watch mode like webpack" | Compiler is fast enough; file watchers add complexity | CLI rerun is sub-second for typical files |

## Feature Dependencies

```
[Error types (eaml-errors)]
    └──requires──> nothing (leaf crate)

[Lexer (eaml-lexer)]
    └──requires──> [Error types]

[Parser (eaml-parser)]
    └──requires──> [Lexer] + [Error types]

[Semantic analysis (eaml-semantic)]
    └──requires──> [Parser] + [Lexer] + [Error types]

[Codegen (eaml-codegen)]
    └──requires──> [Semantic] + [Parser] + [Error types]

[Runtime (eaml-runtime)]
    └──independent of──> Rust crates (consumed by generated Python)

[CLI (eaml-cli)]
    └──requires──> all Rust crates
    └──enhances──> [Runtime] (compile + run workflow)
```

### Dependency Notes

- **Codegen requires Semantic:** Can't emit correct Python without type/capability info
- **Runtime is independent:** Can be developed in parallel with Rust crates
- **CLI requires all crates:** Last Rust piece to implement; wires pipeline together

## MVP Definition

### Launch With (v1)

- [ ] All 7 example programs compile to valid Python — proves end-to-end pipeline
- [ ] Generated Python runs and calls LLM APIs — proves runtime works
- [ ] Error messages include source spans and hints — proves developer experience
- [ ] All 38 error codes from spec/ERRORS.md are implemented — proves spec compliance
- [ ] Three providers work (Anthropic, OpenAI, Ollama) — proves provider abstraction

### Add After Validation (v1.x)

- [ ] `eamlc fmt` command — code formatting
- [ ] `eamlc run` command — compile + execute in one step
- [ ] Better error recovery (continue parsing after first error)
- [ ] Python bridge type validation (`--check-python` flag)

### Future Consideration (v2+)

- [ ] LSP server for IDE support — needs stable compiler first
- [ ] Import/module system — cross-file references
- [ ] Schema inheritance — extends keyword
- [ ] Enum types — tagged unions
- [ ] Pipeline operators — data flow syntax

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Correct compilation (all productions) | HIGH | HIGH | P1 |
| Error messages with spans | HIGH | MEDIUM | P1 |
| Capability checking (CAP010) | HIGH | LOW | P1 |
| Multi-provider runtime | HIGH | MEDIUM | P1 |
| Template string interpolation | HIGH | MEDIUM | P1 |
| Python bridge blocks | MEDIUM | MEDIUM | P1 |
| Agent declarations | MEDIUM | HIGH | P1 |
| CLI compile/check | HIGH | LOW | P1 |
| validate_or_retry | HIGH | MEDIUM | P1 |
| `eamlc run` command | MEDIUM | LOW | P2 |
| Error recovery | MEDIUM | HIGH | P2 |
| `eamlc fmt` | LOW | MEDIUM | P3 |

## Competitor Feature Analysis

| Feature | BAML | Instructor | Marvin | EAML Approach |
|---------|------|------------|--------|---------------|
| Structured output | Pydantic-like types | Pydantic models | Pydantic models | Pydantic v2 via codegen |
| Type safety | Custom type system | Python types | Python types | Nominal types with compile-time checking |
| Multi-provider | Yes (10+ providers) | OpenAI-focused | OpenAI-focused | 3 providers (extensible) |
| Compile-time checks | Limited | None (runtime) | None (runtime) | Full: types, capabilities, names |
| Inline code | No | Python functions | Python functions | Python bridge `%{ }%` blocks |
| Agent composition | No native support | No | No | Declarative `agent` blocks |

## Sources

- BAML documentation (docs.boundaryml.com)
- Instructor library (github.com/jxnl/instructor)
- Marvin AI (github.com/prefecthq/marvin)
- EAML spec documents (spec/)

---
*Feature research for: DSL compiler for LLM integrations*
*Researched: 2026-03-15*