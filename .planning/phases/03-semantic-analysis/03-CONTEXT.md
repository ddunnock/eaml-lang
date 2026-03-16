# Phase 3: Semantic Analysis - Context

**Gathered:** 2026-03-16
**Status:** Ready for planning

<domain>
## Phase Boundary

Validate that a parsed AST is semantically correct — all names resolve, types check, and capability requirements are satisfiable. Produces a symbol table and type-annotated side tables. The `eaml-semantic` crate consumes the parser's AST (Program + typed arenas) and emits SEM, TYP, CAP, and RES diagnostics. Code generation, runtime, and CLI are separate phases.

</domain>

<decisions>
## Implementation Decisions

### Diagnostic Quality
- "Did you mean?" suggestions for unresolved names (RES001) using Levenshtein distance against symbol table entries
- Duplicate declarations (RES010) show primary span at the duplicate + secondary "note: first defined here" span at the original — rustc style
- Capability mismatches (CAP010) show the full diff: "requires [json_mode, streaming] but model provides [json_mode]. Missing: streaming"
- Type errors always show expected vs actual: "expected 'SentimentResult' but found 'string'"
- Chained comparison rejection (SEM060) includes hint: "use explicit grouping: (a == b) && (b == c)"
- Bounded type errors show the specific violation: "minimum bound (5.0) exceeds maximum bound (1.0) in float<5.0, 1.0>"

### Forward Reference Rules
- Fully order-independent: all top-level declarations (model, schema, prompt, tool, agent) can reference any other regardless of source order
- Two-pass name resolution: pass 1 registers all declaration names, pass 2 resolves references
- Cyclic references between declarations are detected and rejected with an error (prevents infinite recursion in codegen)
- Let bindings are sequential: a let can only reference declarations and earlier let bindings (not later lets)
- Import declarations: validate structure and register names in symbol table, but don't resolve actual files/modules (deferred to Phase 6 CLI integration). Python imports registered as opaque names.

### Scope & Variable Rules
- Prompt body template strings: parameters + top-level let bindings are in scope. Schema field names are NOT in scope (they are type definitions, not values).
- Tool body (native expressions): same scoping as prompts — parameters + top-level let bindings visible. Python bridge bodies are opaque (not validated).
- Agent field references: model name must resolve to a declared model, tool names must resolve to declared tools. Catches typos at compile time.
- Schema field type references: all type names must resolve to known types (primitives or declared schemas). Unknown types are RES001 errors (SEM-07).

### Validation Strictness
- No unused-declaration warnings in v0.1 — files compiled individually, declarations may be consumed externally
- Empty schemas (no fields) are allowed — valid as marker types, generate empty Pydantic models
- Invalid bounded type parameters (min > max) are compile errors, not warnings (SEM-04)
- No additional warnings beyond what specs define in v0.1

### Claude's Discretion
- Symbol table internal data structure (HashMap vs arena-based)
- Three-pass pipeline internal communication mechanism
- Side table representation for type annotations (NodeId-keyed maps)
- Levenshtein distance threshold for "did you mean?" suggestions
- Cycle detection algorithm choice (DFS with coloring, topological sort, etc.)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Type System
- `spec/TYPESYSTEM.md` -- Complete type checking rules, bounded type validation, literal union rules, composite type ordering, Pydantic v2 generation mappings
- `spec/grammar.ebnf` -- Type expression grammar productions, all 84 productions

### Capability System
- `spec/CAPABILITIES.md` -- Capability subset checking algorithm, registry, CAP010 fatal semantics, provider capability mappings

### Error Codes
- `spec/ERRORS.md` -- All SEM, TYP, CAP, RES error codes with triggering rules, severity levels, and resolution guidance

### Design Decisions
- `.claude/references/eaml-layer5-design-decisions.md` -- Authoritative design decisions including SEM060 chained comparison rejection (EG-06), all [CLOSED] entries final

### Python Bridge
- `spec/PYTHON_BRIDGE.md` -- Bridge block semantics (opaque, not validated by semantic analysis)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `eaml-parser::ast` — Complete AST with typed arenas (Ast struct), all declaration nodes (ModelDecl, SchemaDecl, PromptDecl, ToolDecl, AgentDecl, ImportDecl, LetDecl), typed ID newtypes (ExprId, TypeExprId, etc.), Index trait impls for arena access
- `eaml-errors` — Diagnostic, DiagnosticCollector (with error limit), ErrorCode enum (all SYN/SEM/CAP/TYP/PYB/RES codes), Severity, codespan-reporting rendering
- `eaml-lexer` — Interner (lasso) for Spur resolution, needed to compare identifier names during name resolution
- Diagnostic struct supports secondary spans via hints — can be extended for "first defined here" notes

### Established Patterns
- NodeId-based side tables for annotations (decided in Phase 2, not mutable AST)
- Snapshot testing with insta (used in lexer and parser tests)
- Clippy with -D warnings, cargo fmt
- DiagnosticCollector accumulates up to 20 errors before stopping
- All AST enums have Error variants for recovery — semantic analysis can skip Error nodes gracefully

### Integration Points
- Semantic analysis consumes: `Program` + `Ast` arenas + `Interner` + parser `Diagnostic` list
- Semantic analysis produces: symbol table + type annotations (side tables) + accumulated diagnostics
- Codegen (Phase 4) will consume the semantic output to emit Python

</code_context>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches. All decisions followed recommended defaults aligned with established compiler conventions (rustc-style diagnostics, two-pass name resolution, sequential let bindings).

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 03-semantic-analysis*
*Context gathered: 2026-03-16*
