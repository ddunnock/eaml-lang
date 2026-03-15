# CONCERNS.md — EAML Technical Debt & Known Issues

## 1. Critical: Python Runtime is Stub-Only

**Current state**: `python/src/eaml_runtime/` contains only empty stubs (~8 lines total).

**What's missing**:
- Provider implementations (anthropic, openai, ollama)
- Validation utilities referenced in spec
- Retry/resilience mechanism (`validateOrRetry()`)
- Type marshaling functions (parameter coercion, return validation)

**Impact**: Generated Python code will import from `eaml_runtime` which doesn't exist. End-to-end testing blocked until provider implementations are added.

**Priority**: P0 — Must be implemented before codegen can be tested.

## 2. Critical: Compiler Crates are Stubs

All Rust crates in `crates/` have scaffolded `lib.rs` files but minimal implementation. The compiler pipeline (lex → parse → semantic → codegen) is defined architecturally but not yet functional.

**Status**: Phase-appropriate — spec/design complete, implementation beginning.

## 3. Known Edge Cases (Documented)

### 3.1 Python Bridge `}%` Delimiter (ERRATA)
F-strings containing `}%` can prematurely close bridge blocks:
```python
python %{
  result = f"{value}% done"  # Lexer sees }% as block end!
}%
```
**Workaround**: Use `str.format()` instead. Documented in Layer 5 §5.1, grammar.ebnf [18].

### 3.2 Python `bool` Subclasses `int` (FIXED)
Validators now exclude `bool` via explicit check. Fixed in commit `11445ff` (4 locations).

### 3.3 Recursive Schemas (SEM070 WARNING)
Self-referential schemas allowed but require manual `model_rebuild()` call. No automatic codegen support.

## 4. Specification Gaps

### 4.1 Runtime Return Type Validation
Bridge block return types are NOT validated at compile time — only at runtime via Pydantic `model_validate()`. This is by design (Python is dynamic), but users may expect compile-time safety.

### 4.2 No Exception Wrapping
Python bridge exceptions propagate naked (no wrapper). OQ-05 closed as intentional for v0.1.

### 4.3 Circular Import Detection
Not implemented in v0.1. Users responsible for ensuring acyclic import DAG. OQ-02 closed, detection may be added post-MVP.

### 4.4 Vision Capability Detection
Uses naming heuristics (field names like "image", "photo") rather than formal rules. OQ-04 closed for v0.1.

## 5. Test Coverage Gaps

| Area | Status |
|------|--------|
| Lexer unit tests | `.gitkeep` placeholder only |
| Parser unit tests | `.gitkeep` placeholder only |
| Semantic analysis tests | `.gitkeep` placeholder only |
| Codegen snapshot tests | Directory scaffolded |
| CLI integration tests | None |
| Python runtime tests | Empty |
| Bridge edge case regression | Missing (f-string, bool/int) |
| Stress tests | None |

## 6. Dependency Concerns

### 6.1 Loose Python Version Bounds
```toml
anthropic>=0.43     # No upper bound — may accept breaking changes
openai>=1.0         # Evolving rapidly
pydantic>=2.0       # Stable but no upper bound
```
**Recommendation**: Tighten to ranges for production (`anthropic>=0.43,<1.0`).

### 6.2 tower-lsp Unused
`tower-lsp = "0.20"` in workspace deps but not referenced by any crate. Reserved for Phase 7 LSP server. No runtime cost, adds minor build overhead.

## 7. Security Considerations

### 7.1 API Key Exposure
Users can hardcode API keys in `python %{ }%` blocks. No language-level protection — documentation should emphasize environment variables.

### 7.2 No Input Validation in Bridge Blocks
EAML validates types (e.g., `string`) but not semantics (e.g., path safety). Tool authors responsible for input validation within bridge blocks.

### 7.3 No Rate Limiting
No built-in timeout/circuit breaker for LLM calls. Default timeouts managed by provider SDKs.

## 8. Grammar Fragile Points

Four documented ambiguity points requiring careful maintenance:

1. **Type `<` vs comparison `<`**: Disambiguated via `parsing_type_expr` flag
2. **Tool body `{` vs `python %{`**: Resolved via lookahead peek
3. **Template string `{` vs `{{`**: Lexer-level TEMPLATE_STRING mode
4. **Import forms**: EAML vs Python distinguished by second token (LL(2) point)

Any grammar changes near these points require FIRST/FOLLOW reverification. See Layer 5 §12.

## 9. Pydantic v2 Hard Dependency

Generated code targets Pydantic v2 only. Key patterns:
- `Annotated[float, Field(ge=0, le=1)]` for bounded types
- `Literal["a", "b"]` for unions
- `model_validate()` / `model_validate_json()` for validation
- `model_rebuild()` for recursive schemas

If Pydantic v2 API changes, codegen breaks. Monitor releases and test against latest v2.x in CI.

## 10. Post-MVP Features (Blocked with Error Codes)

All reserved with specific SYN/SEM error codes:
- Multi-dimensional arrays (SYN042)
- Type inference on `let` (SEM050)
- Enum declarations (SYN082)
- Schema inheritance (SYN083)
- Pipeline operator (SYN080/081)
- Field annotations (SYN090)
- Native tool bodies (SYN050)
- Async bridge blocks
- LSP server (tower-lsp)
- Package/registry imports