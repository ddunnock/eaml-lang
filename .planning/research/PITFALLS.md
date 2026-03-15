# Pitfalls Research

**Domain:** Rust compiler for LLM DSL targeting Python
**Researched:** 2026-03-15
**Confidence:** HIGH

## Critical Pitfalls

### Pitfall 1: Python Indentation Bugs in Codegen

**What goes wrong:**
Generated Python has incorrect indentation — off by one level, mixed tabs/spaces, or inconsistent indent inside nested structures (class method inside class inside module).

**Why it happens:**
String concatenation for code generation makes indentation invisible until runtime. Nested structures (schema fields inside class inside module) compound the problem.

**How to avoid:**
Use a CodeWriter with explicit `indent()`/`dedent()` calls. Never hard-code spaces in output strings. Test every codegen path with snapshot tests that capture exact whitespace.

**Warning signs:**
- `IndentationError` or `SyntaxError` when importing generated Python
- Snapshot tests with inconsistent leading whitespace

**Phase to address:** Phase 5 (Codegen) — CodeWriter must be the first thing built

---

### Pitfall 2: Template String Brace Depth Tracking

**What goes wrong:**
Lexer misidentifies `}` as end-of-interpolation when it's actually a closing brace inside a nested expression like `{items[0]}` or `{dict["key"]}`.

**Why it happens:**
Template string interpolation `{expr}` requires tracking brace depth. The lexer must count `{` and `}` to know when interpolation ends. Off-by-one errors are common.

**How to avoid:**
Maintain an explicit `brace_depth: usize` counter in the lexer. Increment on `{`, decrement on `}`. Only exit interpolation mode when depth reaches 0. Test with deeply nested expressions.

**Warning signs:**
- Truncated interpolation expressions in AST
- Lexer errors on valid template strings with nested braces

**Phase to address:** Phase 2 (Lexer) — template string mode implementation

---

### Pitfall 3: Python Bridge `}%` Delimiter Collision

**What goes wrong:**
Python code inside `%{ }%` contains the literal string `}%` (e.g., in f-strings like `f"{value}% done"`), causing premature block termination.

**Why it happens:**
The `}%` delimiter was chosen because it's rare in Python, but f-strings with `}` followed by `%` can produce it. This is documented as a known edge case (EG-02).

**How to avoid:**
Document the workaround: use `str.format()` instead of f-strings when the string contains `}%`. The lexer scans for literal `}%` — no escape mechanism exists in v0.1.

**Warning signs:**
- Truncated python bridge blocks
- Unexpected tokens after a python block

**Phase to address:** Phase 2 (Lexer) — python block mode; Phase 5 (Codegen) — document in generated comments

---

### Pitfall 4: Capability Subset Check False Positives

**What goes wrong:**
Compiler reports CAP010 (capability mismatch) for valid programs because the capability registry doesn't recognize a capability name, or the subset logic is wrong.

**Why it happens:**
Open identifier system means any string can be a capability name. The registry must distinguish between known capabilities (json_mode, tools, vision) and unknown-but-valid custom capabilities.

**How to avoid:**
CAP010 only fires when a prompt `requires` a capability the model doesn't `provide`. Unknown capabilities should trigger CAP002 (warning for duplicate/unknown), not CAP010.

**Warning signs:**
- Valid example programs fail capability checking
- Users can't use custom capability names

**Phase to address:** Phase 4 (Semantic) — capability checking pass

---

### Pitfall 5: Span Offset Drift Across Lexer Modes

**What goes wrong:**
Error messages point to wrong source locations because byte offsets get corrupted when switching between lexer modes (normal → template string → interpolation → normal).

**Why it happens:**
Each lexer mode may track its own cursor position. When switching modes, the starting offset must be carried over correctly. Off-by-one on mode entry/exit is common.

**How to avoid:**
Single source of truth for byte offset. The cursor position must be authoritative regardless of mode. Test span accuracy by verifying that `source[span.start..span.end]` matches the expected token text.

**Warning signs:**
- Error underlines point to wrong text
- Span lengths don't match token text lengths

**Phase to address:** Phase 2 (Lexer) — span tracking must be tested from day one

---

### Pitfall 6: Pydantic v2 API Surface Differences

**What goes wrong:**
Generated Python uses Pydantic v1 API (e.g., `schema()`, `.dict()`, `validator`) instead of v2 API (`model_json_schema()`, `.model_dump()`, `field_validator`).

**Why it happens:**
Training data and examples online still heavily reference Pydantic v1. Generated code must use v2 exclusively.

**How to avoid:**
Codegen templates must be verified against Pydantic v2 docs. Key mappings: `BaseModel.schema()` → `BaseModel.model_json_schema()`, `.dict()` → `.model_dump()`, `@validator` → `@field_validator`.

**Warning signs:**
- `DeprecationWarning` in generated Python
- `AttributeError` when generated code calls v1 methods

**Phase to address:** Phase 5 (Codegen) — schema generation; Phase 6 (Runtime) — validation calls

---

### Pitfall 7: Bool Subclasses Int in Python

**What goes wrong:**
Python bridge return type validation for `-> int` accepts `True`/`False` because `isinstance(True, int)` is `True` in Python.

**Why it happens:**
Python's `bool` subclasses `int`. This is a known edge case documented in PYTHON_BRIDGE.md.

**How to avoid:**
Generated validation code must check `isinstance(x, bool)` before `isinstance(x, int)` and reject booleans when int is expected. This is already specified in PYB-MAR rules.

**Warning signs:**
- Bridge functions accepting `True`/`False` as valid integers

**Phase to address:** Phase 6 (Runtime) — return type validation

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Skip error recovery in parser | Simpler parser, faster to implement | Only first error reported per file | Acceptable for v0.1; improve in v1.x |
| Single-file compilation only | No module/import resolution needed | Can't split large EAML projects | Acceptable — imports are post-MVP |
| Sync-only runtime | Simpler provider adapters | Blocks on API calls | Acceptable for v0.1; async is future |
| No incremental compilation | Don't need to cache intermediate results | Recompiles entire file on every change | Acceptable — EAML files are small |

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| Anthropic API | Using `messages` without `max_tokens` | Always set `max_tokens` — Anthropic requires it |
| OpenAI API | Using `response_format: {"type": "json_object"}` without JSON instruction in prompt | Include "respond in JSON" in system message when json_mode enabled |
| Ollama API | Assuming OpenAI-compatible endpoint | Use Ollama's native `/api/generate` or `/api/chat` endpoint |
| Pydantic v2 | Calling `model_rebuild()` on non-recursive schemas | Only call `model_rebuild()` when schema has self-references (SEM070 warning) |

## "Looks Done But Isn't" Checklist

- [ ] **Lexer:** Template strings with nested `{dict[key]}` expressions — verify brace depth
- [ ] **Lexer:** Python blocks containing string literals with `}%` — verify edge case EG-02
- [ ] **Parser:** Error recovery after first syntax error — does it continue or crash?
- [ ] **Semantic:** Recursive schema detection — does `model_rebuild()` get emitted?
- [ ] **Codegen:** Generated imports are deduplicated and sorted — Python style
- [ ] **Codegen:** Nullable types emit `Optional[T]` not `T | None` (Pydantic v2 compat)
- [ ] **Runtime:** Provider API keys from environment variables — not hardcoded
- [ ] **Runtime:** Retry loop has max attempts — not infinite
- [ ] **CLI:** Non-zero exit code on compilation errors — scripts depend on this

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Python indentation bugs | Phase 5 (Codegen) | Snapshot tests of generated Python; mypy type-check of output |
| Brace depth tracking | Phase 2 (Lexer) | Test nested `{dict[key]}` in template strings |
| `}%` delimiter collision | Phase 2 (Lexer) | Test python block containing `f"{x}% done"` |
| Capability false positives | Phase 4 (Semantic) | Test all example programs pass capability check |
| Span offset drift | Phase 2 (Lexer) | Assert `source[span]` matches token text |
| Pydantic v2 API | Phase 5 (Codegen) | Generated Python passes mypy + imports without deprecation warnings |
| Bool subclasses int | Phase 6 (Runtime) | Test bridge return type validation rejects `True` for int |

## Sources

- EAML spec documents (spec/PYTHON_BRIDGE.md — EG-02 edge case, PYB-MAR rules)
- Pydantic v2 migration guide
- Python data model (bool subclass of int)
- Compiler construction best practices (Crafting Interpreters)

---
*Pitfalls research for: Rust compiler for LLM DSL*
*Researched: 2026-03-15*
