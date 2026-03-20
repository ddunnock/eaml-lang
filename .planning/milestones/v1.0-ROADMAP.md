# Roadmap: EAML Compiler

## Overview

The EAML compiler is built as a strict pipeline following crate boundaries: shared error types feed the lexer, which feeds the parser, which feeds semantic analysis, which feeds code generation. The CLI and integration tests tie the pipeline together. The Python runtime develops in parallel once codegen defines the output contract. Each phase delivers a complete, testable compiler stage.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Error Foundation and Lexer** - Shared error infrastructure and complete tokenization of all EAML constructs (completed 2026-03-15)
- [x] **Phase 2: Parser** - Hand-written recursive descent parser producing AST for all 84 grammar productions (completed 2026-03-16)
- [x] **Phase 3: Semantic Analysis** - Name resolution, type checking, and capability checking across the AST (completed 2026-03-16)
- [x] **Phase 4: Code Generation** - Python/Pydantic code emission from validated AST (completed 2026-03-16)
- [x] **Phase 5: Python Runtime** - Provider adapters, validation/retry, and telemetry for generated code (completed 2026-03-17)
- [ ] **Phase 6: CLI and Integration** - CLI binary orchestrating the pipeline, validated against all example programs

## Phase Details

### Phase 1: Error Foundation and Lexer
**Goal**: The compiler can tokenize any EAML source file into a stream of typed tokens with accurate source positions, emitting structured diagnostics for malformed input
**Depends on**: Nothing (first phase)
**Requirements**: ERR-01, ERR-02, ERR-03, ERR-04, LEX-01, LEX-02, LEX-03, LEX-04, LEX-05, LEX-06, LEX-07, LEX-08, LEX-09
**Success Criteria** (what must be TRUE):
  1. Given any valid EAML source file, the lexer produces a token stream where every token carries correct byte-offset span information
  2. Template strings with nested `{expr}` interpolation (including nested braces) tokenize correctly without losing track of brace depth
  3. Python bridge blocks `python %{ ... }%` are captured as single opaque tokens preserving their full content
  4. Malformed input produces SYN-prefixed error diagnostics with colored source snippets via codespan-reporting, and lexing continues past errors to report multiple issues
  5. Identifiers are interned via lasso so that repeated identifiers share a single allocation
**Plans**: 3 plans

Plans:
- [ ] 01-01-PLAN.md -- Error foundation: ErrorCode enum, Diagnostic struct, DiagnosticCollector, codespan-reporting rendering
- [ ] 01-02-PLAN.md -- Core lexer: Token types, logos DFA, interner, Normal mode tokenization with error recovery
- [ ] 01-03-PLAN.md -- Advanced lexer: Template string interpolation with brace-depth tracking, python bridge capture

### Phase 2: Parser
**Goal**: The compiler can parse token streams into a complete AST representing all EAML language constructs
**Depends on**: Phase 1
**Requirements**: PAR-01, PAR-02, PAR-03, PAR-04, PAR-05, PAR-06, PAR-07, PAR-08, PAR-09
**Success Criteria** (what must be TRUE):
  1. All 7 top-level declaration types (model, schema, prompt, tool, agent, import, let) parse into distinct AST nodes with correct structure
  2. Type expressions including bounded types (`float<0.0, 1.0>`), literal unions (`"pos" | "neg"`), and composite modifiers (`T?[]?`) parse correctly
  3. Prompt bodies with system/user/assistant sections containing template strings produce AST nodes that preserve message structure and interpolation points
  4. A source file with multiple syntax errors produces diagnostics for each error (parser recovers at synchronization points and continues)
  5. Every AST node carries source span information traceable back to the original source
**Plans**: 4 plans

Plans:
- [ ] 02-01-PLAN.md -- AST type system (typed arenas, ID newtypes, all node types) and parser infrastructure (cursor, helpers, synchronization)
- [ ] 02-02-PLAN.md -- Leaf parsers: type expressions (bounded, modifiers, literal unions), Pratt expression parser, template string parser
- [ ] 02-03-PLAN.md -- Declaration parsers: all 7 types (import, model, schema, prompt, tool, agent, let), requires clauses, post-MVP error codes, error recovery
- [ ] 02-04-PLAN.md -- Error recovery tests, integration tests against all example files, span correctness verification

### Phase 3: Semantic Analysis
**Goal**: The compiler validates that a parsed AST is semantically correct -- all names resolve, types check, and capability requirements are satisfiable
**Depends on**: Phase 2
**Requirements**: SEM-01, SEM-02, SEM-03, SEM-04, SEM-05, SEM-06, SEM-07, SEM-08, SEM-09, SEM-10, SEM-11
**Success Criteria** (what must be TRUE):
  1. Forward references work: a prompt can reference a model declared later in the file, and the symbol table resolves it
  2. Duplicate declarations (e.g., two schemas with the same name) produce RES010 errors with both locations shown
  3. Type validation catches invalid bounded type parameters (e.g., min > max), inconsistent literal union members, and unresolved type references
  4. Capability subset checking validates that prompt `requires` clauses are subsets of the bound model's capabilities, emitting CAP010 FATAL on mismatch
  5. Template string interpolation variables are validated as in-scope, and undefined variables produce clear error messages
**Plans**: 3 plans

Plans:
- [ ] 03-01-PLAN.md -- Foundation + name resolution: error infrastructure upgrades (Res010, secondary labels), symbol table, two-pass resolver with forward references
- [ ] 03-02-PLAN.md -- Type checking: bounded type validation, literal unions, composite types, schema field resolution, template string scope validation
- [ ] 03-03-PLAN.md -- Capability checking: subset check with CAP010 FATAL, integration tests confirming all error codes emittable

### Phase 4: Code Generation
**Goal**: The compiler emits valid, runnable Python 3.11+ / Pydantic v2 code from a semantically-validated AST
**Depends on**: Phase 3
**Requirements**: GEN-01, GEN-02, GEN-03, GEN-04, GEN-05, GEN-06, GEN-07, GEN-08, GEN-09, GEN-10
**Success Criteria** (what must be TRUE):
  1. Schema declarations produce Pydantic BaseModel classes where bounded fields use `Field(ge=..., le=...)` constraints and literal unions use `Literal[...]` annotations
  2. Prompt declarations produce async Python functions that construct system/user messages with f-string interpolation matching the EAML template strings
  3. Model, tool, and agent declarations each produce their corresponding Python constructs (config dicts, functions with bridge bodies, orchestration classes)
  4. Generated Python files have correct, deduplicated imports (pydantic, typing, eaml_runtime)
  5. Generated Python is structurally correct: indentation is consistent, no syntax errors, and the output is importable as a Python module
**Plans**: 4 plans

Plans:
- [ ] 04-01-PLAN.md -- Foundation: CodeWriter (indentation), type annotation mapper (ResolvedType -> Python), name converters (snake_case, UPPER_SNAKE), test helpers
- [ ] 04-02-PLAN.md -- Schema and model emitters: Pydantic BaseModel classes with all type variants, model config dicts, let bindings
- [ ] 04-03-PLAN.md -- Prompt, tool, and agent emitters: async prompt functions, bridge tool functions with wrappers/metadata, agent classes
- [ ] 04-04-PLAN.md -- Integration: generate() wiring, import deduplication, schema topological sort, declaration ordering, full example file snapshot tests

### Phase 5: Python Runtime
**Goal**: Generated Python code can actually execute, calling LLM providers and validating responses against Pydantic models
**Depends on**: Phase 4 (defines the API contract generated code calls into)
**Requirements**: RUN-01, RUN-02, RUN-03, RUN-04, RUN-05, RUN-06, RUN-07, RUN-08
**Success Criteria** (what must be TRUE):
  1. Generated code calling `eaml_runtime` can send a prompt to Anthropic, OpenAI, or Ollama and receive a response, selected by the model declaration's provider field
  2. When an LLM returns output that does not match the expected Pydantic schema, validate_or_retry automatically retries up to the configured limit and either succeeds or raises a clear validation error
  3. Telemetry hooks fire for call_start, call_end, tool_call, and validation_failure events, enabling users to observe runtime behavior
  4. Missing or invalid API keys produce clear error messages indicating which environment variable to set
**Plans**: 2 plans

Plans:
- [ ] 05-01-PLAN.md -- Foundation: error hierarchy, telemetry events/hooks, Provider ABC, Anthropic/OpenAI/Ollama adapters, comprehensive tests
- [ ] 05-02-PLAN.md -- Orchestration: validate_or_retry, execute_prompt, Agent base class, ToolMetadata, public API, pipeline tests

### Phase 6: CLI and Integration
**Goal**: Users can compile and validate EAML files from the command line, and all example programs work end-to-end
**Depends on**: Phase 4, Phase 5
**Requirements**: CLI-01, CLI-02, CLI-03, CLI-04, INT-01, INT-02, INT-03, GEN-11, GEN-12
**Success Criteria** (what must be TRUE):
  1. `eamlc compile sentiment.eaml` produces a `.py` file and exits with code 0; `eamlc check bad_model.eaml` reports the CAP010 error and exits with non-zero code
  2. All 7 example programs (01-minimal through 07-all-type-variants) compile without errors
  3. The generated Python from sentiment.eaml can be executed and returns structured LLM output matching the declared schema
  4. Compilation errors display with colored source snippets, underlines, and error codes matching spec/ERRORS.md
  5. Generated Python passes mypy type checking without errors (GEN-11, deferred from Phase 4 -- requires CLI to produce output files)
  6. Generated Python runs and calls LLM APIs via eaml_runtime (GEN-12, deferred from Phase 4 -- requires runtime from Phase 5)
**Plans**: 2 plans

Plans:
- [ ] 06-01-PLAN.md -- CLI binary: clap subcommands (compile/check/run/--version), pipeline orchestration, error display, exit codes, missing example files
- [ ] 06-02-PLAN.md -- Integration tests: all example compilation, mypy validation, CLI command tests, LLM e2e (ignored)

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 3 -> 4 -> 5 -> 6
(Phase 5 can execute in parallel with Phase 4 once the runtime API contract is defined)

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Error Foundation and Lexer | 3/3 | Complete   | 2026-03-15 |
| 2. Parser | 4/4 | Complete   | 2026-03-16 |
| 3. Semantic Analysis | 3/3 | Complete   | 2026-03-16 |
| 4. Code Generation | 4/4 | Complete   | 2026-03-16 |
| 5. Python Runtime | 2/2 | Complete   | 2026-03-17 |
| 6. CLI and Integration | 0/2 | Not started | - |
