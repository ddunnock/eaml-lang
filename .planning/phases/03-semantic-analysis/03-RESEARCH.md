# Phase 3: Semantic Analysis - Research

**Researched:** 2026-03-16
**Domain:** Compiler semantic analysis (name resolution, type checking, capability validation)
**Confidence:** HIGH

## Summary

Phase 3 implements the `eaml-semantic` crate: a multi-pass semantic analyzer that takes the parser's AST and produces a symbol table, type annotations (side tables keyed by node IDs), and accumulated diagnostics. The crate skeleton already exists with dependencies on `eaml-errors`, `eaml-lexer`, and `eaml-parser`.

The analyzer has three logical phases executed in order: (1) RESOLVE -- two-pass name resolution that registers all declarations then resolves references, (2) TYPE -- type checking including bounded type validation, schema field validation, expression validation, and (3) CAP -- capability subset checking. Each phase emits diagnostics from the existing ErrorCode enum. The error codes are already defined in `eaml-errors`; this phase wires them to triggering conditions.

**Primary recommendation:** Implement as a three-pass pipeline: Pass 1 collects all top-level declaration names into a symbol table (enabling forward references). Pass 2 walks the AST resolving all identifier references against the symbol table and validating types. Pass 3 performs capability subset checking on prompt-model bindings. Use `HashMap<Spur, SymbolInfo>` for the symbol table (leveraging the existing lasso interning), and `HashMap<ExprId, TypeInfo>` / `HashMap<TypeExprId, ResolvedType>` for side tables.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- "Did you mean?" suggestions for unresolved names (RES001) using Levenshtein distance against symbol table entries
- Duplicate declarations (RES010) show primary span at the duplicate + secondary "note: first defined here" span at the original -- rustc style
- Capability mismatches (CAP010) show the full diff: "requires [json_mode, streaming] but model provides [json_mode]. Missing: streaming"
- Type errors always show expected vs actual: "expected 'SentimentResult' but found 'string'"
- Chained comparison rejection (SEM060) includes hint: "use explicit grouping: (a == b) && (b == c)"
- Bounded type errors show the specific violation: "minimum bound (5.0) exceeds maximum bound (1.0) in float<5.0, 1.0>"
- Fully order-independent: all top-level declarations (model, schema, prompt, tool, agent) can reference any other regardless of source order
- Two-pass name resolution: pass 1 registers all declaration names, pass 2 resolves references
- Cyclic references between declarations are detected and rejected with an error (prevents infinite recursion in codegen)
- Let bindings are sequential: a let can only reference declarations and earlier let bindings (not later lets)
- Import declarations: validate structure and register names in symbol table, but don't resolve actual files/modules (deferred to Phase 6 CLI integration). Python imports registered as opaque names.
- Prompt body template strings: parameters + top-level let bindings are in scope. Schema field names are NOT in scope.
- Tool body (native expressions): same scoping as prompts. Python bridge bodies are opaque (not validated).
- Agent field references: model name must resolve to a declared model, tool names must resolve to declared tools.
- Schema field type references: all type names must resolve to known types (primitives or declared schemas).
- No unused-declaration warnings in v0.1
- Empty schemas (no fields) are allowed
- Invalid bounded type parameters (min > max) are compile errors, not warnings (SEM-04)
- No additional warnings beyond what specs define in v0.1

### Claude's Discretion
- Symbol table internal data structure (HashMap vs arena-based)
- Three-pass pipeline internal communication mechanism
- Side table representation for type annotations (NodeId-keyed maps)
- Levenshtein distance threshold for "did you mean?" suggestions
- Cycle detection algorithm choice (DFS with coloring, topological sort, etc.)

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| SEM-01 | Name resolution populates symbol table with all top-level declarations | Symbol table design, two-pass resolution pattern |
| SEM-02 | Name resolution detects duplicate declarations (RES010) | Error code gap analysis (see Open Questions), Diagnostic secondary spans |
| SEM-03 | Name resolution detects undefined references (RES001) | Levenshtein "did you mean?" pattern, scope rules |
| SEM-04 | Type checker validates bounded type parameters (float/int/string bounds) | Bounded type validation rules from TYPESYSTEM.md, TYP030/031/032 codes |
| SEM-05 | Type checker validates literal union members are consistent types | TYP040 duplicate member warning, literal union rules |
| SEM-06 | Type checker validates composite type modifiers (T?, T[], T[]?, T?[], T?[]?) | Composite ordering rules from TYPESYSTEM.md |
| SEM-07 | Type checker validates schema field types resolve to known types | TYP010 unknown type, forward reference resolution |
| SEM-08 | Capability checker performs subset check: prompt requires <= model capabilities | CAP checking algorithm from CAPABILITIES.md |
| SEM-09 | Capability checker emits CAP010 FATAL on capability mismatch | CAP010 fatal semantics, compilation halt behavior |
| SEM-10 | Template string interpolation validates referenced variables are in scope | Scope rules: params + let bindings, NOT schema fields |
| SEM-11 | Semantic analysis emits all SEM, TYP, CAP, and RES error codes from spec | Complete error code catalog mapped below |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| lasso | 0.7 | String interning (Spur keys for symbol table) | Already in workspace, parser uses Spur for all identifiers |
| eaml-errors | workspace | Diagnostic, ErrorCode, Severity, DiagnosticCollector | All error infrastructure already built |
| eaml-parser | workspace | AST types, Program, Ast arenas, node IDs | Input to semantic analysis |
| eaml-lexer | workspace | Interner for Spur resolution | Needed to compare/display identifier names |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| insta | 1 | Snapshot testing for diagnostic output | Test error messages match expected format |
| strsim | 0.11 | Levenshtein/edit distance for "did you mean?" | RES001/TYP010 suggestions -- avoids hand-rolling |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| strsim | hand-rolled Levenshtein | strsim is ~50 lines of dep but handles Unicode correctly and provides multiple distance metrics |
| HashMap<Spur, _> | Vec + linear scan | HashMap is O(1) lookup; with interned Spur keys it's highly efficient |

**Installation:**
```bash
# Add to workspace Cargo.toml
strsim = "0.11"

# Add to eaml-semantic/Cargo.toml
[dependencies]
eaml-errors = { path = "../eaml-errors" }
eaml-lexer = { path = "../eaml-lexer" }
eaml-parser = { path = "../eaml-parser" }
strsim = { workspace = true }

[dev-dependencies]
insta = { workspace = true }
```

## Architecture Patterns

### Recommended Project Structure
```
crates/eaml-semantic/
├── src/
│   ├── lib.rs           # Public API: analyze() -> AnalysisOutput
│   ├── symbol_table.rs  # SymbolTable, SymbolKind, SymbolInfo
│   ├── resolver.rs      # Two-pass name resolution (RESOLVE phase)
│   ├── type_checker.rs  # Type validation (TYPE phase)
│   ├── cap_checker.rs   # Capability subset checking (CAP phase)
│   └── scope.rs         # Scope management for template string / let binding validation
├── tests/
│   ├── test_helpers.rs  # Shared helpers (parse + analyze, assert diagnostics)
│   ├── resolution.rs    # Name resolution tests (SEM-01, SEM-02, SEM-03)
│   ├── types.rs         # Type checking tests (SEM-04 through SEM-07)
│   ├── capabilities.rs  # Capability checking tests (SEM-08, SEM-09)
│   ├── scoping.rs       # Scope/template variable tests (SEM-10)
│   └── snapshots/       # insta snapshot files
└── Cargo.toml
```

### Pattern 1: Three-Pass Pipeline
**What:** Semantic analysis runs three passes over the AST in order: RESOLVE, TYPE, CAP. Each pass has its own module and produces data consumed by later passes.
**When to use:** Always -- this is the locked architecture from CONTEXT.md.

```rust
// src/lib.rs
pub struct AnalysisOutput {
    pub symbols: SymbolTable,
    pub type_annotations: TypeAnnotations,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn analyze(
    program: &Program,
    ast: &Ast,
    interner: &Interner,
    source: &str,
) -> AnalysisOutput {
    let mut diags = DiagnosticCollector::new(20);

    // Pass 1+2: Name resolution
    let symbols = resolver::resolve(program, ast, interner, &mut diags);

    // Pass 3: Type checking (uses symbol table)
    let type_annotations = type_checker::check(program, ast, interner, &symbols, &mut diags);

    // Pass 4: Capability checking (uses symbol table + type annotations)
    cap_checker::check(program, ast, interner, &symbols, &mut diags);

    AnalysisOutput {
        symbols,
        type_annotations,
        diagnostics: diags.into_diagnostics(),
    }
}
```

### Pattern 2: Symbol Table with Spur Keys
**What:** HashMap keyed by lasso::Spur (interned string ID) mapping to symbol info.
**When to use:** For all name lookups during resolution and type checking.

```rust
// src/symbol_table.rs
use lasso::Spur;
use eaml_errors::Span;
use eaml_parser::ast::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum SymbolKind {
    Model(ModelDeclId),
    Schema(SchemaDeclId),
    Prompt(PromptDeclId),
    Tool(ToolDeclId),
    Agent(AgentDeclId),
    Import(ImportDeclId),
    Let(LetDeclId),
    Primitive,  // string, int, float, bool, null
    Param,      // function/prompt parameter (scoped)
}

#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub kind: SymbolKind,
    pub span: Span,
    pub name_spur: Spur,
}

pub struct SymbolTable {
    /// Top-level declarations (global scope)
    declarations: HashMap<Spur, SymbolInfo>,
    /// Primitive type names (pre-populated)
    primitives: HashMap<Spur, ()>,
}
```

### Pattern 3: Side Tables for Type Annotations
**What:** HashMap keyed by TypeExprId / ExprId mapping to resolved type info.
**When to use:** Type checker populates these; codegen consumes them.

```rust
// src/type_checker.rs
use std::collections::HashMap;
use eaml_parser::ast::{TypeExprId, ExprId};

#[derive(Debug, Clone)]
pub enum ResolvedType {
    Primitive(PrimitiveType),
    Schema(SchemaDeclId),
    Array(Box<ResolvedType>),
    Optional(Box<ResolvedType>),
    LiteralUnion(Vec<String>),
    Error, // propagate errors without cascading diagnostics
}

pub struct TypeAnnotations {
    pub type_exprs: HashMap<TypeExprId, ResolvedType>,
    pub exprs: HashMap<ExprId, ResolvedType>,
}
```

### Pattern 4: ErrorNode Propagation (Cascade Suppression)
**What:** When the parser produces Error variant nodes, or when resolution fails, downstream checks skip those nodes to avoid cascading errors.
**When to use:** Every validation function should check for Error variants first.

```rust
fn check_type_expr(&mut self, id: TypeExprId) -> ResolvedType {
    match &self.ast[id] {
        TypeExpr::Error(_) => ResolvedType::Error,  // skip -- already reported
        TypeExpr::Named(spur, span) => {
            // ... resolve type name
        }
        // ... other variants
    }
}

fn check_expr(&mut self, id: ExprId) -> ResolvedType {
    match &self.ast[id] {
        Expr::Error(_) => ResolvedType::Error,  // skip
        // ... check returns Error? Don't emit more diagnostics
    }
}
```

### Anti-Patterns to Avoid
- **Mutating the AST:** Side tables are the decided approach. Never add mutable fields to AST nodes.
- **Single-pass resolution:** Forward references require two passes. Trying to resolve in one pass causes order-dependent failures.
- **Cascading error storms:** When a name fails to resolve, all downstream uses of that name should NOT each produce a new error. Use the Error/ResolvedType::Error propagation pattern.
- **Validating Python bridge bodies:** Bridge blocks are opaque strings. Semantic analysis MUST NOT attempt to parse or validate them.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Edit distance for suggestions | Custom Levenshtein | `strsim::levenshtein()` or `strsim::jaro_winkler()` | Unicode edge cases, performance tested |
| Diagnostic accumulation | Custom Vec + limit logic | `DiagnosticCollector` from eaml-errors | Already handles error limits, severity filtering |
| String interning | Custom intern table | lasso `Interner` from eaml-lexer | Already used throughout lexer/parser, Spur is the identity key |
| AST traversal | Manual index arithmetic | `Ast` Index impls (`ast[expr_id]`, `ast[type_id]`) | Type-safe arena access already implemented |

**Key insight:** The existing infrastructure (Diagnostic, ErrorCode, Span, Ast arenas, Interner) covers 80% of what semantic analysis needs. The new code is primarily validation logic and data structures (symbol table, side tables).

## Common Pitfalls

### Pitfall 1: Missing RES010 Error Code
**What goes wrong:** The CONTEXT.md and REQUIREMENTS.md reference "RES010" for duplicate declarations, but the `ErrorCode` enum in `eaml-errors/src/codes.rs` only has `Res001`. There is no `Res010` variant. The spec `ERRORS.md` also does not define RES010 -- it has RES001 (undefined reference) and reserves RES002-099.
**Why it happens:** The requirements doc appears to use "RES010" as a conceptual label, but the actual error code was never added to the enum or spec.
**How to avoid:** Add a `Res010` variant to the `ErrorCode` enum for duplicate top-level declarations. This is within the reserved RES range and follows the convention. Update `prefix()`, `number()`, and `Display` implementations.
**Warning signs:** Compilation fails when trying to use `ErrorCode::Res010`.

### Pitfall 2: Scope Confusion for Template Strings
**What goes wrong:** Template interpolation variables like `{name}` in prompt bodies must resolve to parameters or let bindings, NOT schema field names. A user writing `prompt Foo(x: SomeSchema) -> string { user: "{x.field}" }` is valid (field access on a parameter), but `{field}` alone referring to a schema field is NOT valid.
**Why it happens:** Schema fields define types, not runtime values. Only parameters and let bindings are values in scope.
**How to avoid:** Build a scope for each prompt/tool body containing only: (1) declared parameters, (2) preceding let bindings. Walk template string interpolation expressions against this scope.
**Warning signs:** Tests pass that shouldn't, or false positives on valid field access expressions.

### Pitfall 3: Literal Union Member Validation with Spans
**What goes wrong:** `LiteralUnion.members` stores `Vec<Span>` (byte ranges), not parsed values. To check for duplicate members or validate consistency, you need to extract the string content from the source text using the spans.
**Why it happens:** The parser stores spans for efficiency; the semantic analyzer must slice the source to get actual values.
**How to avoid:** Use `&source[span.clone()]` to extract literal values. Strip quotes when comparing for duplicates.
**Warning signs:** Comparing raw spans instead of string content, off-by-one errors with quote stripping.

### Pitfall 4: Bounded Type Parameters -- Positional vs Named
**What goes wrong:** Bounded types support both `float<0.0, 1.0>` (positional: first=min, second=max) and `float<max: 1.0, min: 0.0>` (named: order doesn't matter). The `BoundParam` struct has an `Option<Spur>` name field -- `None` means positional.
**Why it happens:** The grammar allows both forms; the type checker must handle the mapping correctly.
**How to avoid:** Normalize to named form first: if positional and 2 params, map to (min, max). If positional and 1 param, it's min for int/float, or an error. Then validate min <= max.
**Warning signs:** Reversed bounds not caught, or single positional param treated as max instead of min.

### Pitfall 5: CAP010 is FATAL -- Halts Compilation
**What goes wrong:** CAP010 (capability mismatch) has FATAL severity, meaning compilation should halt. But the `DiagnosticCollector` treats Fatal and Error the same way (both count toward limit). The distinction matters for downstream phases.
**Why it happens:** The spec says "FATAL errors at any stage prevent proceeding to the next stage."
**How to avoid:** After capability checking, inspect diagnostics for any Fatal-severity entries. If found, the overall analysis should signal that codegen must NOT proceed. The AnalysisOutput can carry a `has_fatal: bool` flag.
**Warning signs:** Codegen runs after a CAP010 error and produces invalid Python.

### Pitfall 6: Let Binding Sequential Scope
**What goes wrong:** Let bindings within prompt/tool bodies are sequential: `let a` can reference declarations and earlier lets, but NOT later lets. This differs from top-level declarations which are order-independent.
**Why it happens:** Two different scoping rules in the same crate.
**How to avoid:** When resolving let binding expressions, add each let to the local scope AFTER resolving its value expression, not before.
**Warning signs:** Forward references between let bindings silently resolve instead of producing RES001.

## Code Examples

### Example 1: Two-Pass Resolution
```rust
// Pass 1: Register all top-level declarations
fn register_declarations(
    program: &Program,
    ast: &Ast,
    interner: &Interner,
    symbols: &mut SymbolTable,
    diags: &mut DiagnosticCollector,
) {
    for decl_id in &program.declarations {
        match decl_id {
            DeclId::Schema(id) => {
                let schema = &ast[*id];
                let name = schema.name;
                if let Some(existing) = symbols.get(name) {
                    // RES010: Duplicate declaration
                    diags.emit(Diagnostic::new(
                        ErrorCode::Res010,  // needs to be added
                        format!(
                            "duplicate definition of '{}'",
                            interner.resolve(&name)
                        ),
                        schema.span.clone(),
                        Severity::Error,
                        "redefined here".to_string(),
                    ).with_hint(format!(
                        "note: first defined at {:?}",
                        existing.span
                    )));
                } else {
                    symbols.insert(name, SymbolInfo {
                        kind: SymbolKind::Schema(*id),
                        span: schema.span.clone(),
                        name_spur: name,
                    });
                }
            }
            DeclId::Model(id) => { /* similar */ }
            DeclId::Error(_) => { /* skip error recovery nodes */ }
            // ... other declaration types
        }
    }
}
```

### Example 2: "Did You Mean?" Suggestion
```rust
use strsim::levenshtein;

fn suggest_similar(name: &str, symbols: &SymbolTable, interner: &Interner) -> Option<String> {
    let threshold = 3; // max edit distance
    let mut best: Option<(usize, String)> = None;

    for (spur, _info) in symbols.iter() {
        let candidate = interner.resolve(spur);
        let dist = levenshtein(name, candidate);
        if dist <= threshold && dist > 0 {
            if best.as_ref().map_or(true, |(d, _)| dist < *d) {
                best = Some((dist, candidate.to_string()));
            }
        }
    }

    best.map(|(_, s)| s)
}
```

### Example 3: Capability Subset Check
```rust
fn check_capability_mismatch(
    prompt: &PromptDecl,
    model: &ModelDecl,
    interner: &Interner,
    diags: &mut DiagnosticCollector,
) {
    let Some(requires) = &prompt.requires else { return };

    let model_caps: HashSet<Spur> = model.caps.iter().map(|(s, _)| *s).collect();
    let mut missing = Vec::new();

    for (cap_spur, cap_span) in &requires.caps {
        if !model_caps.contains(cap_spur) {
            missing.push((interner.resolve(cap_spur).to_string(), cap_span.clone()));
        }
    }

    if !missing.is_empty() {
        let missing_names: Vec<&str> = missing.iter().map(|(n, _)| n.as_str()).collect();
        let model_cap_names: Vec<String> = model.caps
            .iter()
            .map(|(s, _)| interner.resolve(s).to_string())
            .collect();

        diags.emit(Diagnostic::new(
            ErrorCode::Cap010,
            format!(
                "model '{}' is missing required capabilities: [{}]. Required by prompt '{}'. \
                 Model supports: [{}]",
                interner.resolve(&model.name),
                missing_names.join(", "),
                interner.resolve(&prompt.name),
                model_cap_names.join(", "),
            ),
            requires.span.clone(),
            Severity::Fatal,  // CAP010 is FATAL
            "required here".to_string(),
        ));
    }
}
```

### Example 4: Bounded Type Validation
```rust
fn validate_bounded_type(
    base: Spur,
    params: &[BoundParam],
    span: &Span,
    source: &str,
    interner: &Interner,
    diags: &mut DiagnosticCollector,
) {
    let base_name = interner.resolve(&base);

    // TYP032: Only string, int, float accept bounds
    if !matches!(base_name, "string" | "int" | "float") {
        diags.emit(Diagnostic::new(
            ErrorCode::Typ032,
            format!("type '{}' does not accept bounded parameters", base_name),
            span.clone(),
            Severity::Error,
            "bounded type not allowed here".to_string(),
        ));
        return;
    }

    // Normalize positional to named: first=min, second=max
    let (min_val, max_val) = extract_min_max(params, source, interner);

    // TYP030: min > max
    if let (Some(min), Some(max)) = (&min_val, &max_val) {
        if min > max {
            diags.emit(Diagnostic::new(
                ErrorCode::Typ030,
                format!("lower bound ({}) exceeds upper bound ({})", min, max),
                span.clone(),
                Severity::Error,
                "invalid bound range".to_string(),
            ));
        }
    }

    // TYP031: String length bounds must be non-negative integers
    if base_name == "string" {
        // validate min/max are non-negative
    }
}
```

## Complete Error Code Map for Phase 3

All error codes that `eaml-semantic` must emit, extracted from spec/ERRORS.md:

### RESOLVE Phase
| Code | Condition | Severity |
|------|-----------|----------|
| SEM010 | Python import after non-import declaration | ERROR |
| RES001 | Identifier cannot be resolved after pass 2 | ERROR |
| (RES010) | Duplicate top-level declaration name | ERROR (needs new ErrorCode variant) |

### TYPE Phase
| Code | Condition | Severity |
|------|-----------|----------|
| SEM020 | Duplicate field name in schema | ERROR |
| SEM025 | Prompt body missing user: field | ERROR |
| SEM030 | Unknown bounded parameter name (not min/max) | ERROR |
| SEM035 | Bounded type used in parameter position | ERROR |
| SEM040 | Tool body has no implementation (empty) | ERROR |
| SEM060 | Chained comparison (a == b == c) | ERROR |
| SEM061 | Positional argument after named argument | ERROR |
| SEM070 | Recursive schema reference | WARNING |
| TYP001 | Schema shadows built-in type name | WARNING |
| TYP002 | Schema named 'void' (reserved) | WARNING |
| TYP003 | Type mismatch (expected vs actual) | ERROR |
| TYP010 | Unknown type name | ERROR |
| TYP030 | Lower bound exceeds upper bound | ERROR |
| TYP031 | Invalid string length bound (negative) | ERROR |
| TYP032 | Bounds on non-boundable type | ERROR |
| TYP040 | Duplicate literal union member | WARNING |
| PYB010 | Unknown provider string | WARNING |

### CAP Phase
| Code | Condition | Severity |
|------|-----------|----------|
| CAP001 | Unknown capability name | WARNING |
| CAP002 | Duplicate capability name | WARNING |
| CAP010 | Capability mismatch (prompt requires > model caps) | FATAL |
| CAP020 | json_mode with string return type | WARNING |

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Mutable AST annotations | Side tables (HashMap by NodeId) | Decided Phase 2 | Keep AST immutable; side tables enable parallel analysis later |
| Single-pass resolution | Two-pass (register then resolve) | Standard compiler practice | Enables forward references between declarations |
| Abort on first error | Accumulate up to 20 errors | Built in Phase 1 | Better developer experience, catches more issues per compile |

## Open Questions

1. **RES010 Error Code Gap**
   - What we know: Requirements and CONTEXT.md reference "RES010" for duplicate top-level declarations. The ErrorCode enum and spec/ERRORS.md do not define this code. The reserved range RES002-099 is available.
   - What's unclear: Whether to add RES010 to the enum (and update codes.rs), or use an existing code like SEM020 (currently "duplicate field name" within a schema, not duplicate declarations).
   - Recommendation: Add `Res010` to the ErrorCode enum. SEM020 is specifically for duplicate *field names within a schema*, which is a different condition. RES010 for duplicate *top-level declaration names* is semantically correct and follows the existing code range conventions. The number 010 fits the RES prefix range pattern used by other categories.

2. **Diagnostic Secondary Spans**
   - What we know: The CONTEXT.md requires "note: first defined here" secondary spans for duplicate declarations (rustc style). The current `Diagnostic` struct has a `hints: Vec<String>` field but no secondary span field.
   - What's unclear: Whether `hints` is sufficient (text-only, no span highlighting) or whether `Diagnostic` needs a `secondary_spans: Vec<(Span, String)>` field for proper codespan-reporting display.
   - Recommendation: Add a `secondary_labels: Vec<(Span, String)>` field to `Diagnostic` and wire it through the codespan-reporting renderer. This enables "note: first defined here" with an actual source underline, matching the rustc style the user requested and leveraging codespan-reporting's built-in support for secondary labels.

3. **Prompt-to-Model Binding for CAP Check**
   - What we know: The CAP check needs to know which model a prompt is called with. In EAML, prompts are not statically bound to models -- the binding happens at the call site (agent's `model:` field references a model, and agent's tools reference prompts). However, the CONTEXT.md example (`bad_model.eaml`) shows a prompt with `requires json_mode` and a model with `caps: []` -- the check fires when any model in the file lacks the required capabilities.
   - What's unclear: The exact binding mechanism. Is it: (a) agent model + agent's implicit prompt routing, (b) check all model/prompt combinations, or (c) something else?
   - Recommendation: For v0.1, the most practical approach is to check capability satisfaction through the agent binding: when an agent references a model and tools (which may invoke prompts), verify the model's caps cover all prompts' requires clauses in the file. If no agent exists, check all possible model-prompt combinations. The spec says "prompt requires <= model capabilities" which implies the binding happens wherever a prompt could be routed to a model.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cargo test (built-in) + insta 1.x (snapshots) |
| Config file | Cargo.toml (workspace test config) |
| Quick run command | `cargo test -p eaml-semantic` |
| Full suite command | `cargo test --workspace` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SEM-01 | Symbol table populated with all decl types | unit | `cargo test -p eaml-semantic -- resolution::test_symbol_table` | -- Wave 0 |
| SEM-02 | Duplicate declarations produce RES010 | unit | `cargo test -p eaml-semantic -- resolution::test_duplicate` | -- Wave 0 |
| SEM-03 | Undefined references produce RES001 with suggestions | unit | `cargo test -p eaml-semantic -- resolution::test_undefined` | -- Wave 0 |
| SEM-04 | Bounded type params validated (min>max, etc.) | unit | `cargo test -p eaml-semantic -- types::test_bounded` | -- Wave 0 |
| SEM-05 | Literal union duplicate members detected | unit | `cargo test -p eaml-semantic -- types::test_literal_union` | -- Wave 0 |
| SEM-06 | Composite type modifiers validated | unit | `cargo test -p eaml-semantic -- types::test_composite` | -- Wave 0 |
| SEM-07 | Schema field types resolve to known types | unit | `cargo test -p eaml-semantic -- types::test_type_resolution` | -- Wave 0 |
| SEM-08 | Capability subset check works | unit | `cargo test -p eaml-semantic -- capabilities::test_cap_check` | -- Wave 0 |
| SEM-09 | CAP010 emitted as FATAL | unit | `cargo test -p eaml-semantic -- capabilities::test_cap010_fatal` | -- Wave 0 |
| SEM-10 | Template vars validated in scope | unit | `cargo test -p eaml-semantic -- scoping::test_template_scope` | -- Wave 0 |
| SEM-11 | All SEM/TYP/CAP/RES codes emittable | integration | `cargo test -p eaml-semantic -- integration::test_all_codes` | -- Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p eaml-semantic`
- **Per wave merge:** `cargo test --workspace`
- **Phase gate:** Full suite green + `make check` before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `crates/eaml-semantic/tests/test_helpers.rs` -- shared parse+analyze helper, assert_has_code, assert_no_errors
- [ ] `crates/eaml-semantic/tests/resolution.rs` -- SEM-01, SEM-02, SEM-03 tests
- [ ] `crates/eaml-semantic/tests/types.rs` -- SEM-04 through SEM-07 tests
- [ ] `crates/eaml-semantic/tests/capabilities.rs` -- SEM-08, SEM-09 tests
- [ ] `crates/eaml-semantic/tests/scoping.rs` -- SEM-10 tests
- [ ] `crates/eaml-errors/src/codes.rs` -- needs Res010 variant added
- [ ] `crates/eaml-errors/src/diagnostic.rs` -- needs secondary_labels field for rustc-style "first defined here" notes
- [ ] `strsim` dependency added to workspace Cargo.toml

## Sources

### Primary (HIGH confidence)
- `spec/ERRORS.md` -- Complete error code catalog with 38 compiler codes, severity levels, triggering conditions
- `spec/TYPESYSTEM.md` -- Complete type system spec with all TS-* rules, bounded type validation, composite ordering
- `spec/CAPABILITIES.md` -- Capability subset checking algorithm, CAP010 fatal semantics, built-in registry
- `spec/PYTHON_BRIDGE.md` -- Bridge blocks are opaque, not validated by semantic analysis
- `crates/eaml-errors/src/codes.rs` -- Existing ErrorCode enum (verified: no Res010)
- `crates/eaml-errors/src/diagnostic.rs` -- Diagnostic struct (verified: hints but no secondary spans)
- `crates/eaml-parser/src/ast.rs` -- Complete AST types with typed arenas and node IDs

### Secondary (MEDIUM confidence)
- `strsim` crate -- Levenshtein distance implementation, standard choice for Rust edit distance

### Tertiary (LOW confidence)
- None -- all findings verified against authoritative project specs

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all libraries already in workspace except strsim (well-known, stable)
- Architecture: HIGH -- three-pass pipeline, symbol table, side tables all decided in CONTEXT.md and prior phases
- Pitfalls: HIGH -- identified from direct spec analysis and code inspection, especially the RES010 gap and Diagnostic secondary span limitation
- Error codes: HIGH -- complete catalog extracted from spec/ERRORS.md cross-referenced with ErrorCode enum

**Research date:** 2026-03-16
**Valid until:** 2026-04-16 (stable -- specs are authoritative and frozen for v0.1)
