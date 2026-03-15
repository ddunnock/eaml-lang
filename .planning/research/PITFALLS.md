# Domain Pitfalls

**Domain:** Rust compiler for LLM integration DSL targeting Python/Pydantic
**Researched:** 2026-03-15

## Critical Pitfalls

Mistakes that cause rewrites or major issues.

### Pitfall 1: Lexer Mode Switching for Template Strings

**What goes wrong:** Template strings with {expr} interpolation require the lexer to switch between "string content" mode and "expression" mode. If the brace-depth counter is wrong, the lexer misparses nested braces, object literals inside interpolations, or literal {{ / }} escapes.

**Why it happens:** logos is a stateless lexer generator -- it does not natively support mode switching. The lexer must manually track brace depth outside of logos's derive system.

**Consequences:** Template strings silently consume too much or too little input. Parser receives wrong tokens. Generated Python f-strings have incorrect interpolation.

**Prevention:** Implement brace-depth counting as a separate layer wrapping logos. Test with nested braces, escaped braces, and empty interpolation. Use snapshot tests for every template string variant from examples/07-all-type-variants.

**Detection:** Template string tests fail. Generated Python f-strings produce syntax errors.

### Pitfall 2: Python Bridge }% Delimiter False Positive

**What goes wrong:** The }% closing delimiter for python bridge blocks can appear in Python f-strings: f"{value}% done" produces literal }% that prematurely closes the block.

**Why it happens:** The lexer scans for }% as a simple byte sequence without understanding Python string context.

**Consequences:** Python code is silently truncated. SYN046 (unclosed block) fires on valid code. Generated output contains partial Python.

**Prevention:** This is a documented known limitation (Layer 5 errata EG-02). The spec recommends str.format() instead of f-strings when the result contains }%. Document this in compiler warnings. Consider adding a SYN-level hint when }% appears inside what looks like a Python f-string.

**Detection:** Bridge block tests with f-strings containing }%.

### Pitfall 3: Mutable AST Coupling Parser to Semantic Phase

**What goes wrong:** Adding Option<ResolvedType> fields to AST nodes so the semantic phase can "fill them in" during analysis.

**Why it happens:** Seems simpler than side tables at first. Avoids NodeId indirection.

**Consequences:** Parser crate gains dependency on semantic types (circular or coupling). Interior mutability (RefCell) needed for shared references during tree walks. AST types become bloated with Option fields. Breaking the strict crate boundary.

**Prevention:** Use NodeId + side tables (TypeEnvironment, SymbolTable) in eaml-semantic. AST is immutable after parsing. See Architecture Pattern 3.

**Detection:** Cargo dependency graph shows parser depending on semantic types.

### Pitfall 4: Codegen Indentation Bugs in Python Output

**What goes wrong:** Generated Python has incorrect indentation, causing IndentationError at runtime or semantically wrong nesting.

**Why it happens:** Using raw format!() or string concatenation for Python emission. Indentation context is implicit and easy to lose across conditionals.

**Consequences:** Generated code does not run. Bugs are hard to trace because they appear in output, not in the compiler source.

**Prevention:** Use a CodeWriter struct that tracks indentation level explicitly. Every line is emitted through write_line() which prepends the current indent. Use write_block() for automatic indent/dedent scoping. Snapshot test every generated output against golden files.

**Detection:** Python syntax check (python -c "compile(...)") on generated output.

### Pitfall 5: Forward Reference Resolution

**What goes wrong:** A prompt references a schema that is declared later in the file. If the name resolution pass walks top-to-bottom and resolves immediately, the schema is not yet in the symbol table.

**Why it happens:** Single-pass name resolution that resolves references during the same pass that collects declarations.

**Consequences:** RES010 (undefined name) false positives on valid code. Users forced into declaration-order dependency.

**Prevention:** Two-sub-pass name resolution: (a) collect all declaration names into symbol table, (b) resolve all references against the now-complete table. This is standard in languages without forward-declaration requirements.

**Detection:** Test with prompt-before-schema ordering.

## Moderate Pitfalls

### Pitfall 6: Bool Subclasses Int in Python

**What goes wrong:** Python bridge blocks with -> int return type accept True/False because Python bool is a subclass of int.

**Prevention:** Generated validation code must use isinstance(v, int) and not isinstance(v, bool) check. Already documented in PYTHON_BRIDGE.md.

### Pitfall 7: Lasso Interner Lifetime Management

**What goes wrong:** The ThreadedRodeo (string interner) is created in the lexer but its Spur keys are used throughout the parser and semantic phases. If the interner is dropped or not passed through, Spur lookups panic.

**Prevention:** The interner must be owned by the top-level compilation context (in eaml-cli) and passed by reference to each phase. Never clone the interner -- pass it by reference.

### Pitfall 8: Snapshot Test Fragility

**What goes wrong:** insta snapshot tests break on every formatting change, creating noisy diffs.

**Prevention:** Use insta::assert_yaml_snapshot!() for AST snapshots (structured). Use insta::assert_snapshot!() for codegen output (exact string match desired). Run cargo insta review after intentional changes.

### Pitfall 9: Provider API Version Drift

**What goes wrong:** The anthropic/openai Python SDKs release breaking changes. Generated code that worked last month stops working.

**Prevention:** The runtime adapter layer abstracts provider APIs. Pin minimum SDK versions in pyproject.toml. Keep generated code calling eaml_runtime.call_prompt(), never provider SDKs directly.

### Pitfall 10: Error Code Duplication or Gaps

**What goes wrong:** Two compiler phases emit the same error code for different conditions, or a spec-defined error code is never emitted.

**Prevention:** Error codes defined in eaml-errors as an enum. Each variant maps to exactly one condition. Exhaustive tests verifying every error code in spec/ERRORS.md has at least one triggering test.

## Minor Pitfalls

### Pitfall 11: Optional Semicolons and Newline Ambiguity

**What goes wrong:** Since semicolons are optional, the parser may have trouble distinguishing statement boundaries.

**Prevention:** EAML declarations start with keywords (model, schema, etc.), so the parser can always determine boundaries by peeking. Non-issue if parser is structured correctly.

### Pitfall 12: Contextual Keywords Conflicting with User Identifiers

**What goes wrong:** A user names a schema field "temperature" or "system", which are contextual keywords in prompt bodies.

**Prevention:** Contextual keywords are matched by string comparison only within their specific productions. Outside those productions, they are valid identifiers. Grammar already handles this per Layer 5.

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| Lexer implementation | Template string mode switching (#1) | Wrap logos with brace-depth counter, extensive snapshot tests |
| Lexer implementation | Python bridge delimiter (#2) | Document limitation, add hint diagnostic |
| Parser AST design | Mutable AST coupling (#3) | Use NodeId + side tables from the start |
| Parser error recovery | Infinite loop on bad input | Track parser position; if no progress after synchronize(), advance one token |
| Semantic name resolution | Forward references (#5) | Two-sub-pass: collect declarations, then resolve references |
| Codegen Python emission | Indentation bugs (#4) | CodeWriter struct, snapshot tests on all output |
| Codegen type mapping | Bool/int confusion (#6) | isinstance check excluding bool |
| Runtime provider adapters | SDK version drift (#9) | Adapter abstraction, pinned versions |
| Testing | Snapshot fragility (#8) | YAML for AST, string for codegen, cargo insta review workflow |

## Sources

- spec/PYTHON_BRIDGE.md -- }% delimiter edge case (EG-02), bool subclass issue
- spec/ERRORS.md -- error code architecture and ranges
- Layer 5 design decisions -- contextual keywords, optional semicolons
- [Resilient LL Parsing Tutorial (matklad)](https://matklad.github.io/2023/05/21/resilient-ll-parsing-tutorial.html)
- [Logos Handbook](https://logos.maciej.codes/)
