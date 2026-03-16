# Requirements: EAML Compiler

**Defined:** 2026-03-15
**Core Value:** The compiler must correctly translate all v0.1 EAML constructs into runnable Python that type-checks, imports cleanly, and calls LLM APIs when executed.

## v1 Requirements

### Error Foundation

- [x] **ERR-01**: Compiler defines all error codes from spec/ERRORS.md as a Rust enum (SYN, SEM, CAP, TYP, PYB, RES)
- [x] **ERR-02**: Diagnostic struct carries error code, message, source span, severity, and optional hints
- [x] **ERR-03**: Errors display with codespan-reporting showing colored source snippets and underlines
- [x] **ERR-04**: Multiple errors accumulate per compilation (not abort-on-first)

### Lexer

- [x] **LEX-01**: Lexer tokenizes all keywords from grammar.ebnf (model, schema, prompt, tool, agent, import, let, if, else, return, true, false, null, requires)
- [x] **LEX-02**: Lexer tokenizes all operators and delimiters from grammar.ebnf
- [x] **LEX-03**: Lexer tokenizes string literals (double-quoted) with escape sequences
- [x] **LEX-04**: Lexer tokenizes numeric literals (integers and floats)
- [x] **LEX-05**: Lexer tokenizes template strings with `{expr}` interpolation tracking brace depth correctly
- [x] **LEX-06**: Lexer captures python bridge blocks `python %{ ... }%` as opaque content
- [x] **LEX-07**: Lexer interns identifiers via lasso for memory-efficient deduplication
- [x] **LEX-08**: Lexer skips comments (`//`, `/* */`) while preserving accurate byte-offset spans
- [x] **LEX-09**: Lexer emits SYN error codes for malformed tokens with accurate source positions

### Parser

- [ ] **PAR-01**: Parser produces AST nodes for all 7 top-level declaration types (model, schema, prompt, tool, agent, import, let)
- [x] **PAR-02**: Parser handles type expressions: primitives, named types, arrays, optionals, bounded types, literal unions
- [x] **PAR-03**: Parser handles expressions via Pratt parsing (identifiers, literals, field access, function calls, binary ops)
- [x] **PAR-04**: Parser handles prompt body with system/user/assistant message sections and template strings
- [ ] **PAR-05**: Parser handles `requires` clauses on prompt declarations
- [ ] **PAR-06**: Parser handles tool declarations with parameter lists, return types, and python bridge bodies
- [ ] **PAR-07**: Parser handles agent declarations with model binding, tools list, and configuration
- [x] **PAR-08**: Parser recovers from syntax errors via synchronization points and continues parsing
- [x] **PAR-09**: Every AST node carries source span information for error reporting

### Semantic Analysis

- [ ] **SEM-01**: Name resolution populates symbol table with all top-level declarations
- [ ] **SEM-02**: Name resolution detects duplicate declarations (RES010)
- [ ] **SEM-03**: Name resolution detects undefined references (RES001)
- [ ] **SEM-04**: Type checker validates bounded type parameters (float<min,max>, int<min,max>, string<minLen,maxLen>)
- [ ] **SEM-05**: Type checker validates literal union members are consistent types
- [ ] **SEM-06**: Type checker validates composite type modifiers (T?, T[], T[]?, T?[], T?[]?)
- [ ] **SEM-07**: Type checker validates schema field types resolve to known types
- [ ] **SEM-08**: Capability checker performs subset check: prompt requires ⊆ model capabilities
- [ ] **SEM-09**: Capability checker emits CAP010 FATAL on capability mismatch
- [ ] **SEM-10**: Template string interpolation validates referenced variables are in scope
- [ ] **SEM-11**: Semantic analysis emits all SEM, TYP, CAP, and RES error codes from spec

### Code Generation

- [ ] **GEN-01**: CodeWriter handles Python indentation correctly with explicit indent/dedent tracking
- [ ] **GEN-02**: Schema declarations generate Pydantic v2 BaseModel classes with correct field types
- [ ] **GEN-03**: Bounded types generate Pydantic Field constraints (ge, le, min_length, max_length)
- [ ] **GEN-04**: Literal union types generate `Literal["a", "b", "c"]` type annotations
- [ ] **GEN-05**: Prompt declarations generate async Python functions with system/user messages
- [ ] **GEN-06**: Template string interpolation generates Python f-strings in prompt bodies
- [ ] **GEN-07**: Model declarations generate provider configuration dicts
- [ ] **GEN-08**: Tool declarations generate Python functions with bridge block bodies
- [ ] **GEN-09**: Agent declarations generate orchestration classes with tool dispatch
- [ ] **GEN-10**: Generated Python imports are deduplicated, sorted, and include eaml_runtime
- [ ] **GEN-11**: Generated Python type-checks with mypy without errors
- [ ] **GEN-12**: Generated Python runs and calls LLM APIs via eaml_runtime

### Python Runtime

- [ ] **RUN-01**: Anthropic provider adapter calls Claude API with correct message format
- [ ] **RUN-02**: OpenAI provider adapter calls GPT API with correct message format
- [ ] **RUN-03**: Ollama provider adapter calls local API via httpx
- [ ] **RUN-04**: validate_or_retry validates LLM responses against Pydantic models and retries on failure
- [ ] **RUN-05**: Telemetry hooks fire on call_start, call_end, tool_call, validation_failure events
- [ ] **RUN-06**: Provider selection is based on model declaration's provider field
- [ ] **RUN-07**: Runtime reads API keys from environment variables (ANTHROPIC_API_KEY, OPENAI_API_KEY)
- [ ] **RUN-08**: Runtime handles provider errors gracefully with clear error messages

### CLI

- [ ] **CLI-01**: `eamlc compile <file>` compiles .eaml to .py with exit code 0 on success
- [ ] **CLI-02**: `eamlc check <file>` validates .eaml without generating output
- [ ] **CLI-03**: CLI displays all accumulated errors/warnings using codespan-reporting
- [ ] **CLI-04**: CLI returns non-zero exit code on compilation errors

### Integration

- [ ] **INT-01**: All 7 example programs (01-minimal through 07-all-type-variants) compile successfully
- [ ] **INT-02**: Generated Python from sentiment.eaml runs and returns structured output from LLM
- [ ] **INT-03**: bad_model.eaml triggers CAP010 capability mismatch error at compile time

## v2 Requirements

### IDE Support

- **IDE-01**: LSP server provides diagnostics on save
- **IDE-02**: VS Code extension with syntax highlighting
- **IDE-03**: Go-to-definition for schema/model references

### Language Features

- **LANG-01**: Import/module system for cross-file references
- **LANG-02**: Schema inheritance (extends keyword)
- **LANG-03**: Enum types (tagged unions)
- **LANG-04**: Pipeline operators for data flow

### Developer Experience

- **DX-01**: `eamlc fmt` for code formatting
- **DX-02**: `eamlc run` for compile + execute in one step
- **DX-03**: `--check-python` flag for python bridge type validation

## Out of Scope

| Feature | Reason |
|---------|--------|
| Async python bridge blocks | Closed as unsupported in v0.1 (OQ-01) |
| Type inference | All annotations explicit in v0.1 per spec |
| Multi-file compilation | Module system is post-MVP |
| Unsafe Rust code | Project constraint — zero unsafe blocks |
| Watch mode / hot reload | CLI rerun is fast enough for v0.1 |
| Auto-retry without limits | Dangerous — validate_or_retry has configurable max_retries |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| ERR-01 | Phase 1 | Complete |
| ERR-02 | Phase 1 | Complete |
| ERR-03 | Phase 1 | Complete |
| ERR-04 | Phase 1 | Complete |
| LEX-01 | Phase 1 | Complete |
| LEX-02 | Phase 1 | Complete |
| LEX-03 | Phase 1 | Complete |
| LEX-04 | Phase 1 | Complete |
| LEX-05 | Phase 1 | Complete |
| LEX-06 | Phase 1 | Complete |
| LEX-07 | Phase 1 | Complete |
| LEX-08 | Phase 1 | Complete |
| LEX-09 | Phase 1 | Complete |
| PAR-01 | Phase 2 | Pending |
| PAR-02 | Phase 2 | Complete |
| PAR-03 | Phase 2 | Complete |
| PAR-04 | Phase 2 | Complete |
| PAR-05 | Phase 2 | Pending |
| PAR-06 | Phase 2 | Pending |
| PAR-07 | Phase 2 | Pending |
| PAR-08 | Phase 2 | Complete |
| PAR-09 | Phase 2 | Complete |
| SEM-01 | Phase 3 | Pending |
| SEM-02 | Phase 3 | Pending |
| SEM-03 | Phase 3 | Pending |
| SEM-04 | Phase 3 | Pending |
| SEM-05 | Phase 3 | Pending |
| SEM-06 | Phase 3 | Pending |
| SEM-07 | Phase 3 | Pending |
| SEM-08 | Phase 3 | Pending |
| SEM-09 | Phase 3 | Pending |
| SEM-10 | Phase 3 | Pending |
| SEM-11 | Phase 3 | Pending |
| GEN-01 | Phase 4 | Pending |
| GEN-02 | Phase 4 | Pending |
| GEN-03 | Phase 4 | Pending |
| GEN-04 | Phase 4 | Pending |
| GEN-05 | Phase 4 | Pending |
| GEN-06 | Phase 4 | Pending |
| GEN-07 | Phase 4 | Pending |
| GEN-08 | Phase 4 | Pending |
| GEN-09 | Phase 4 | Pending |
| GEN-10 | Phase 4 | Pending |
| GEN-11 | Phase 4 | Pending |
| GEN-12 | Phase 4 | Pending |
| RUN-01 | Phase 5 | Pending |
| RUN-02 | Phase 5 | Pending |
| RUN-03 | Phase 5 | Pending |
| RUN-04 | Phase 5 | Pending |
| RUN-05 | Phase 5 | Pending |
| RUN-06 | Phase 5 | Pending |
| RUN-07 | Phase 5 | Pending |
| RUN-08 | Phase 5 | Pending |
| CLI-01 | Phase 6 | Pending |
| CLI-02 | Phase 6 | Pending |
| CLI-03 | Phase 6 | Pending |
| CLI-04 | Phase 6 | Pending |
| INT-01 | Phase 6 | Pending |
| INT-02 | Phase 6 | Pending |
| INT-03 | Phase 6 | Pending |

**Coverage:**
- v1 requirements: 55 total
- Mapped to phases: 55
- Unmapped: 0 ✓

---
*Requirements defined: 2026-03-15*
*Last updated: 2026-03-15 after initial definition*
