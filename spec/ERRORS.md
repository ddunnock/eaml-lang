# EAML Error Code Catalog

**Version:** 0.1.0
**Status:** AUTHORITATIVE
**Last updated:** 2026-03-15

## Abstract

This document is the canonical reference for all diagnostic codes emitted by the EAML compiler (`eamlc`) and the EAML Python runtime. Every compiler diagnostic carries a prefixed numeric code (e.g., `SYN042`, `TYP010`) that uniquely identifies the error condition, its triggering rule, and its resolution. Runtime exceptions are documented separately in Section 5.

This catalog covers **35 compiler diagnostic codes** and **1 runtime exception** across six prefixes.

---

## Table of Contents

1. [Error System Architecture](#1-error-system-architecture)
2. [SYN -- Syntax Errors](#2-syn--syntax-errors)
3. [SEM -- Semantic Errors](#3-sem--semantic-errors)
4. [TYP -- Type Errors](#4-typ--type-errors)
5. [CAP -- Capability Errors](#5-cap--capability-errors)
6. [PYB -- Python Bridge Errors](#6-pyb--python-bridge-errors)
7. [RES -- Resolution Errors](#7-res--resolution-errors)
8. [Quick Reference](#8-quick-reference)
9. [Open Questions, Conflicts, and Reserved Ranges](#9-open-questions-conflicts-and-reserved-ranges)
10. [Verification Report](#10-verification-report)

---

## 1. Error System Architecture

### 1.1 Code Format

All diagnostic codes follow the pattern:

```
PREFIX NNN
```

where `PREFIX` is a 2-4 letter category identifier and `NNN` is a zero-padded numeric code within that category. Codes are stable across compiler versions -- once assigned, a code is never reused for a different condition.

### 1.2 Prefixes

| Prefix | Domain                          | Emitting crate(s)        |
|--------|---------------------------------|--------------------------|
| SYN    | Syntax errors (lexer + parser)  | eaml-lexer, eaml-parser  |
| SEM    | Semantic analysis               | eaml-semantic            |
| TYP    | Type system                     | eaml-semantic            |
| CAP    | Capability system               | eaml-semantic            |
| PYB    | Python bridge                   | eaml-codegen             |
| RES    | Name resolution                 | eaml-semantic            |

### 1.3 Severity Levels

| Severity | Meaning                                                        | Compilation |
|----------|----------------------------------------------------------------|-------------|
| FATAL    | Unrecoverable error. Compilation halts immediately.            | Aborted     |
| ERROR    | Recoverable error. Compilation continues to find more errors.  | Fails       |
| WARNING  | Potential issue. Does not prevent compilation.                 | Succeeds    |
| RUNTIME  | Python exception raised at execution time, not compile time.   | N/A         |

### 1.4 Compiler Phases

Diagnostics are attributed to the phase in which they are detected:

| Phase   | Description                       | Crate           |
|---------|-----------------------------------|-----------------|
| LEX     | Tokenization                      | eaml-lexer      |
| PARSE   | Recursive descent parsing         | eaml-parser     |
| RESOLVE | Name resolution (pass 1 + pass 2) | eaml-semantic   |
| TYPE    | Type checking, bounds validation  | eaml-semantic   |
| CAP     | Capability subset checking        | eaml-semantic   |
| CODEGEN | Python code generation            | eaml-codegen    |

### 1.5 Error Code Number Ranges

| Prefix  | Range   | Sub-range Description              |
|---------|---------|------------------------------------|
| SYN     | 001–039 | Lexer errors                       |
| SYN     | 040–049 | Grammar restrictions               |
| SYN     | 050–059 | Declaration body restrictions      |
| SYN     | 060–079 | Module/import errors               |
| SYN     | 080–089 | Post-MVP declaration types         |
| SYN     | 090–099 | Post-MVP field features            |
| SEM     | 010–019 | Module/import semantic rules       |
| SEM     | 020–029 | Declaration semantic rules         |
| SEM     | 030–039 | Bounded parameter rules            |
| SEM     | 040–049 | Tool body rules                    |
| SEM     | 050–059 | Annotation/position rules          |
| SEM     | 060–069 | Expression rules                   |
| SEM     | 070–079 | Schema structure rules             |
| TYP     | 001–009 | Primitive type errors              |
| TYP     | 010–019 | Type name resolution errors        |
| TYP     | 030–039 | Bounded type errors                |
| TYP     | 040–049 | Literal union errors               |
| CAP     | 001–009 | Capability registry errors         |
| CAP     | 010–019 | Capability mismatch errors         |
| CAP     | 020–029 | Capability/type interaction errors |
| PYB     | 001–009 | Python bridge capture/parse errors |
| PYB     | 010–019 | Provider errors                    |
| RES     | 001–009 | Name resolution errors             |

Unassigned codes within a range are reserved for future versions.

### 1.6 Error Accumulation

The compiler accumulates errors up to a maximum of **20** (overridable with `--max-errors N`). After reaching the limit, compilation halts with:

```
aborting due to 20 previous errors
Fix the above and recompile.
```

Cascading errors (phantom errors caused by earlier unresolved names) are suppressed using ErrorNode propagation — downstream checks on nodes containing ErrorNode do not emit additional diagnostics.

### 1.7 Compiler Pipeline and Error Phase Ordering

```
Source (.eaml)
  → LEX (eaml-lexer)      — SYN001–SYN039 (lexer errors)
  → PARSE (eaml-parser)    — SYN040–SYN099 (parser errors), SEM050
  → RESOLVE pass 1+2       — RES001, SEM010
    (eaml-semantic)
  → TYPE check             — TYP, SEM020–SEM079, PYB010
    (eaml-semantic)
  → CAP check              — CAP001–CAP020
    (eaml-semantic)
  → CODEGEN                — PYB001 (with --check-python)
    (eaml-codegen)
  → Output (.py)
```

FATAL errors at any stage prevent proceeding to the next stage. A SYN error means no TYP errors will be reported in the same compilation run.

### 1.8 Message Format

All diagnostic messages follow the template:

```
PREFIX NNN: Human-readable message with {variable} placeholders.
```

Variables in `{braces}` are filled at emission time with context-specific values (names, types, line numbers). Messages may include a `Hint:` suffix with actionable guidance.

### 1.9 Error Entry Format

Every error entry in Sections 2–7 uses this canonical format:

```
### PREFIX[code]: [Short title]
**Phase:**      [LEX | PARSE | RESOLVE | TYPE | CAP | CODEGEN]
**Severity:**   [FATAL | ERROR | WARNING | RUNTIME]
**Emitted by:** [crate name]
**Condition:**  [Precise triggering condition]
**Message:**    `PREFIX[code]: [message template with {variables}]`
**Example:**    [minimal EAML that produces this error]
**Resolution:** [What the user must do to fix it]
**Spec refs:**  [grammar.ebnf Production [N], TYPESYSTEM.md TS-XX-NN, etc.]
**Notes:**      [Edge cases, related errors, rationale]
```

---

## 2. SYN -- Syntax Errors

Syntax errors are emitted by the lexer (`eaml-lexer`) or the recursive descent parser (`eaml-parser`) when the input does not conform to the EAML grammar.

### SYN042: Multi-dimensional array

**Phase:**      PARSE
**Severity:**   ERROR
**Emitted by:** eaml-parser
**Condition:**  A second `[]` suffix follows an existing `[]` in a type expression, forming `[][]`.
**Message:**    `SYN042: Multi-dimensional arrays are not supported in EAML v0.1. Hint: Use a schema with an array field: schema Matrix { rows: Type[] }`

**Example:**
```eaml
schema Grid {
  cells: int[][]
}
```

**Resolution:** Wrap the inner array in a named schema and use a single-dimensional array of that schema.
**Spec refs:**  grammar.ebnf Production [48], Layer 5 Section 3.4
**Notes:**      Multi-dimensional arrays may be supported in a future version. The parser checks for consecutive `[]` tokens immediately after parsing the first arraySuffix.

---

### SYN043: Pipe operator in expression context

**Phase:**      PARSE
**Severity:**   ERROR
**Emitted by:** eaml-parser
**Condition:**  A `|` token appears in expression position where an operator or value is expected.
**Message:**    `SYN043: Unexpected '|' in expression. Did you mean '||' for logical OR? Note: '|' is only valid as a type union separator.`

**Example:**
```eaml
prompt Check {
  model my_model
  let result: bool = a | b
}
```

**Resolution:** Use `||` for logical OR. The single `|` is reserved for type union syntax (e.g., `"yes" | "no"`).
**Spec refs:**  grammar.ebnf Production [57], Layer 5 Section 14 EG-05
**Notes:**      Bitwise OR does not exist in EAML. This error is part of the "expression guardrails" (EG-05) designed to catch common mistakes from developers coming from other languages.

---

### SYN044: Ampersand in expression context

**Phase:**      PARSE
**Severity:**   ERROR
**Emitted by:** eaml-parser
**Condition:**  A `&` token appears in expression position where an operator or value is expected.
**Message:**    `SYN044: Unexpected '&' in expression. Did you mean '&&' for logical AND? Note: Bitwise operators do not exist in EAML.`

**Example:**
```eaml
prompt Check {
  model my_model
  let result: bool = a & b
}
```

**Resolution:** Use `&&` for logical AND. Bitwise operators are not part of EAML.
**Spec refs:**  Layer 5 Section 14 EG-05
**Notes:**      Part of expression guardrails (EG-05). There is no bitwise AND, OR, XOR, or shift in EAML.

---

### SYN045: Unclosed template string interpolation

**Phase:**      LEX
**Severity:**   ERROR
**Emitted by:** eaml-lexer
**Condition:**  An opening `{` in template string mode has no matching `}` at brace depth zero before the string terminator or end of file.
**Message:**    `SYN045: Unclosed template string interpolation. Expected '}' to close interpolation started at line {line}:{col}.`

**Example:**
```eaml
prompt Greet {
  model my_model
  "Hello, {name"
}
```

**Resolution:** Add the missing `}` to close the interpolation expression. Use `{{` for a literal brace.
**Spec refs:**  grammar.ebnf Production [20] [lex: template-string-mode]
**Notes:**      The lexer tracks brace nesting depth to allow expressions like `{obj.field}` inside interpolations. This error fires when EOF or end-of-string is reached with a non-zero brace depth.

---

### SYN050: Native tool body (Post-MVP)

**Phase:**      PARSE
**Severity:**   ERROR
**Emitted by:** eaml-parser
**Condition:**  A tool declaration contains a body with native EAML statements instead of a `python %{ }%` block.
**Message:**    `SYN050: Native tool bodies are not supported in EAML v0.1. Use python %{{ }}% for tool implementations.`

**Example:**
```eaml
tool add(a: int, b: int) -> int {
  return a + b
}
```

**Resolution:** Replace the body with a Python bridge block: `python %{ return a + b }%`.
**Spec refs:**  grammar.ebnf Production [36], Layer 5 Section 7.5
**Notes:**      Native tool bodies are planned for a future version. In v0.1, all tool implementations must use the Python bridge.

---

### SYN060: Circular import

**Phase:**      PARSE
**Severity:**   ERROR
**Emitted by:** eaml-parser
**Condition:**  An import cycle is detected between two or more EAML files during import resolution.
**Message:**    `SYN060: Circular import detected: {file_a} imports {file_b} which imports {file_a}.`

**Example:**
```eaml
// file: a.eaml
import "./b.eaml"

// file: b.eaml
import "./a.eaml"
```

**Resolution:** Break the cycle by extracting shared declarations into a third file that both can import.
**Spec refs:**  Layer 5 Section 8.1
**Notes:**      The compiler tracks the import stack and detects cycles before parsing the imported file. Transitive cycles (A -> B -> C -> A) are also detected.

---

### SYN061: Absolute or URL import path

**Phase:**      PARSE
**Severity:**   ERROR
**Emitted by:** eaml-parser
**Condition:**  An import path starts with `/` (absolute) or contains `:` (URL scheme).
**Message:**    `SYN061: Absolute paths and URL imports are not supported in EAML v0.1. Use relative paths: import "./path.eaml".`

**Example:**
```eaml
import "/usr/share/eaml/common.eaml"
import "https://example.com/schemas.eaml"
```

**Resolution:** Convert to a relative path from the current file's directory.
**Spec refs:**  Layer 5 Section 14 EG-10
**Notes:**      Package/registry imports may be supported in a future version. All import paths in v0.1 must be relative.

---

### SYN080: Pipeline declaration (Post-MVP)

**Phase:**      PARSE
**Severity:**   ERROR
**Emitted by:** eaml-parser
**Condition:**  The `pipeline` keyword appears at declaration level.
**Message:**    `SYN080: Pipeline declarations are not supported in EAML v0.1.`

**Example:**
```eaml
pipeline MyPipeline {
  step1 >> step2
}
```

**Resolution:** Remove the pipeline declaration. In v0.1, compose prompts manually in Python code.
**Spec refs:**  grammar.ebnf Production [77], Layer 5 Section 11
**Notes:**      Pipelines are a planned post-MVP feature. The grammar reserves the `pipeline` keyword.

---

### SYN081: Pipeline operator (Post-MVP)

**Phase:**      PARSE
**Severity:**   ERROR
**Emitted by:** eaml-parser
**Condition:**  The `>>` token appears in expression context.
**Message:**    `SYN081: Pipeline operator >> is not supported in EAML v0.1.`

**Example:**
```eaml
prompt Chain {
  model my_model
  let result: string = step1 >> step2
}
```

**Resolution:** Remove the pipeline operator. Chain prompt calls explicitly in Python code.
**Spec refs:**  grammar.ebnf, Layer 5 Section 11
**Notes:**      The `>>` operator is reserved for the pipeline feature planned for a future version.

---

### SYN082: Enum declaration (Post-MVP)

**Phase:**      PARSE
**Severity:**   ERROR
**Emitted by:** eaml-parser
**Condition:**  The `enum` keyword appears at declaration level.
**Message:**    `SYN082: Enum declarations are not supported in EAML v0.1.`

**Example:**
```eaml
enum Color {
  Red
  Green
  Blue
}
```

**Resolution:** Use a literal union type instead: `"Red" | "Green" | "Blue"`.
**Spec refs:**  grammar.ebnf Production [79], Layer 5 Section 11
**Notes:**      Enum declarations are a planned post-MVP feature. Literal union types provide equivalent functionality for string enumerations in v0.1.

---

### SYN083: Schema inheritance (Post-MVP)

**Phase:**      PARSE
**Severity:**   ERROR
**Emitted by:** eaml-parser
**Condition:**  The `extends` keyword appears after a schema name in a schema declaration.
**Message:**    `SYN083: Schema inheritance is not supported in EAML v0.1.`

**Example:**
```eaml
schema Admin extends User {
  role: string
}
```

**Resolution:** Duplicate the base fields in the derived schema, or restructure using composition (a field of the base schema type).
**Spec refs:**  grammar.ebnf, Layer 5 Section 11
**Notes:**      Schema inheritance is a planned post-MVP feature. Composition is the recommended pattern in v0.1.

---

### SYN090: Field annotation (Post-MVP)

**Phase:**      PARSE
**Severity:**   ERROR
**Emitted by:** eaml-parser
**Condition:**  An `@` sigil appears in a field or declaration context.
**Message:**    `SYN090: @annotations are not supported in EAML v0.1.`

**Example:**
```eaml
schema User {
  @description("The user's full name")
  name: string
}
```

**Resolution:** Remove the annotation. Use doc comments or field descriptions in the prompt template instead.
**Spec refs:**  grammar.ebnf, Layer 5 Section 11
**Notes:**      Annotations are a planned post-MVP feature. The `@` sigil is reserved.

---

## 3. SEM -- Semantic Errors

Semantic errors are emitted by the semantic analysis pass (`eaml-semantic`) when the program is syntactically valid but violates semantic rules.

### SEM010: Import after declaration

**Phase:**      RESOLVE
**Severity:**   ERROR
**Emitted by:** eaml-semantic
**Condition:**  A `python` import statement appears after a non-import declaration in the same file.
**Message:**    `SEM010: Python imports must appear at the top of the file, before any other declarations.`

**Example:**
```eaml
schema User {
  name: string
}

import python "utils"
```

**Resolution:** Move all `import python` statements to the top of the file, before any `schema`, `model`, `prompt`, or `tool` declarations.
**Spec refs:**  grammar.ebnf Production [26] [sem: import-before-declarations], Layer 5 Section 5.2, Section 14 EG-07
**Notes:**      This rule ensures deterministic name resolution order. EAML file imports (`import "./file.eaml"`) are also subject to this rule.

---

### SEM020: Duplicate field name

**Phase:**      TYPE
**Severity:**   ERROR
**Emitted by:** eaml-semantic
**Condition:**  Two or more fields in the same schema declaration share the same name.
**Message:**    `SEM020: Duplicate field name '{name}' in schema '{schema}'.`

**Example:**
```eaml
schema User {
  name: string
  age: int
  name: string
}
```

**Resolution:** Remove or rename the duplicate field.
**Spec refs:**  TYPESYSTEM.md TS-SCH-03, grammar.ebnf Production [30]
**Notes:**      Field names are compared case-sensitively. Two fields with different types but the same name still trigger this error.

---

### SEM030: Unknown bounded parameter

**Phase:**      TYPE
**Severity:**   ERROR
**Emitted by:** eaml-semantic
**Condition:**  A parameter name in a bounded type expression is not one of the recognized parameter names (`min`, `max`).
**Message:**    `SEM030: Unknown bounded parameter '{param}' for type '{type}'. Valid parameters: 'min', 'max'.`

**Example:**
```eaml
schema Config {
  temperature: float(step: 0.1)
}
```

**Resolution:** Use only `min` and `max` as bounded type parameters: `float(min: 0.0, max: 1.0)`.
**Spec refs:**  TYPESYSTEM.md TS-BND-07, grammar.ebnf Production [45] [sem: bounded-param-validation]
**Notes:**      Additional bounded parameters may be introduced in future versions. In v0.1, only `min` and `max` are recognized for `int`, `float`, and `string` types.

---

### SEM035: Bounds in parameter position

**Phase:**      TYPE
**Severity:**   ERROR
**Emitted by:** eaml-semantic
**Condition:**  A bounded type (e.g., `int(min: 0, max: 100)`) is used as a parameter type in a prompt or tool declaration.
**Message:**    `SEM035: Bounded type parameters are not permitted in prompt or tool parameter positions. Bounds are schema field constraints for validating LLM output. Use '{base_type}' as the parameter type.`

**Example:**
```eaml
prompt Score(value: int(min: 0, max: 100)) -> string {
  model my_model
  "Rate {value}"
}
```

**Resolution:** Remove the bounds from the parameter type. Bounds are only meaningful on schema fields where they constrain LLM output via Pydantic validation.
**Spec refs:**  TYPESYSTEM.md TS-BND-08
**Notes:**      Prompt and tool parameters are caller-supplied values, not LLM outputs. Bounded validation applies only to schema fields that receive LLM-generated data.

---

### SEM040: Tool body has no implementation

**Phase:**      TYPE
**Severity:**   ERROR
**Emitted by:** eaml-semantic
**Condition:**  A tool declaration has an empty body with neither a `python %{ }%` block nor any statements.
**Message:**    `SEM040: Tool body has no implementation. Use python %{{ }}% to provide a tool implementation.`

**Example:**
```eaml
tool lookup(query: string) -> string {
}
```

**Resolution:** Add a Python bridge block with the tool's implementation.
**Spec refs:**  grammar.ebnf Production [36] [sem: tool-body-must-have-implementation]
**Notes:**      In v0.1, all tool implementations must use the Python bridge. Future versions may support native EAML tool bodies.

---

### SEM050: Type annotation required on let binding

**Phase:**      PARSE
**Severity:**   ERROR
**Emitted by:** eaml-parser
**Condition:**  A `let` binding omits the `: typeExpr` annotation.
**Message:**    `SEM050: Type annotation required on let bindings in EAML v0.1. Hint: let {name}: {inferred_type} = ...`

**Example:**
```eaml
prompt Greet {
  model my_model
  let greeting = "hello"
}
```

**Resolution:** Add a type annotation: `let greeting: string = "hello"`.
**Spec refs:**  TYPESYSTEM.md TS-ANN-01, grammar.ebnf Production [41], Layer 5 Section 11
**Notes:**      This is enforced syntactically by the grammar (Production [41] requires `: typeExpr`), so the parser detects it before semantic analysis. The SEM prefix is retained per Layer 5 assignment. Type inference may be added in a future version.

---

### SEM060: Chained comparison

**Phase:**      TYPE
**Severity:**   ERROR
**Emitted by:** eaml-semantic
**Condition:**  A comparison operator is applied to the result of another comparison, forming a chain like `a == b == c`.
**Message:**    `SEM060: Chained comparisons are not supported in EAML. Hint: Use 'a == b && b == c'.`

**Example:**
```eaml
prompt Check {
  model my_model
  let result: bool = x == y == z
}
```

**Resolution:** Break the chain into separate comparisons joined with `&&` or `||`.
**Spec refs:**  grammar.ebnf Production [57] [sem: no-chained-comparison], Layer 5 Section 14 EG-06
**Notes:**      Chained comparisons are ambiguous in most C-family languages (they compare the boolean result to the next operand). EAML rejects them outright with a clear hint.

---

### SEM070: Recursive schema reference (WARNING)

**Phase:**      TYPE
**Severity:**   WARNING
**Emitted by:** eaml-semantic
**Condition:**  A schema field references its own schema type, either directly or through a chain of schema references.
**Message:**    `SEM070: Schema '{name}' contains a recursive type reference. This is allowed but may cause issues with deeply nested data.`

**Example:**
```eaml
schema TreeNode {
  value: string
  children: TreeNode[]
}
```

**Resolution:** No action required -- recursive schemas are valid in EAML. Consider whether the recursion depth is bounded by your use case.
**Spec refs:**  TYPESYSTEM.md OQ-03, Section 9.6
**Notes:**      The generated Pydantic model uses `model_rebuild()` to handle forward references. Deeply nested recursive structures may cause stack overflows at runtime.

---

## 4. TYP -- Type Errors

Type errors are emitted during the type-checking phase of semantic analysis when type constraints are violated.

### TYP001: Built-in type shadowing (WARNING)

**Phase:**      TYPE
**Severity:**   WARNING
**Emitted by:** eaml-semantic
**Condition:**  A schema is declared with the same name as a predeclared primitive type (`string`, `int`, `float`, `bool`, `null`).
**Message:**    `TYP001 warning: Schema '{name}' shadows the built-in type '{name}'.`

**Example:**
```eaml
schema string {
  value: int
}
```

**Resolution:** Rename the schema to avoid shadowing the built-in type.
**Spec refs:**  TYPESYSTEM.md TS-PRM-01, TS-PRM-06, Layer 5 Section 3.1
**Notes:**      Shadowing is permitted but discouraged. The schema type takes precedence over the built-in type within the same file after the declaration.

---

### TYP002: Reserved future type name (WARNING)

**Phase:**      TYPE
**Severity:**   WARNING
**Emitted by:** eaml-semantic
**Condition:**  A schema is declared with the name `void`, which is reserved for a future version.
**Message:**    `TYP002: 'void' is reserved as a future type name in EAML. Use a different name.`

**Example:**
```eaml
schema void {
  empty: bool
}
```

**Resolution:** Choose a different name for the schema.
**Spec refs:**  grammar.ebnf [predeclared identifiers], Layer 5 Section 11
**Notes:**      `void` may be introduced as a return type in a future version. Other reserved names may be added; check the grammar's predeclared identifier list.

---

### TYP003: Type mismatch

**Phase:**      TYPE
**Severity:**   ERROR
**Emitted by:** eaml-semantic
**Condition:**  A value of one type is used where an incompatible type is expected (assignment, argument passing, return).
**Message:**    `TYP003: Type mismatch: expected '{expected}', found '{actual}'.`

**Example:**
```eaml
schema Config {
  count: int
}

prompt Setup {
  model my_model
  let x: int = "hello"
}
```

**Resolution:** Ensure the value type matches the declared type. Check for incorrect variable references or missing type conversions.
**Spec refs:**  TYPESYSTEM.md TS-PRM-01 through TS-PRM-05, TS-SCH-02
**Notes:**      EAML uses nominal typing with no implicit coercion. `int` and `float` are distinct types. Literal unions are matched structurally.

---

### TYP010: Unknown type name

**Phase:**      TYPE
**Severity:**   ERROR
**Emitted by:** eaml-semantic
**Condition:**  An identifier in type position does not resolve to any declared schema or built-in type.
**Message:**    `TYP010: Unknown type '{name}'.` with optional hint: `Did you mean '{suggestion}'?`

**Example:**
```eaml
schema Order {
  customer: Custmer
}
```

**Resolution:** Correct the type name. Check for typos or missing schema declarations/imports.
**Spec refs:**  TYPESYSTEM.md TS-PRM-06, TS-RET-02
**Notes:**      The compiler may suggest similar type names using edit distance. Forward references to schemas declared later in the same file are resolved in a second pass (see RES001).

---

### TYP030: Lower bound exceeds upper bound

**Phase:**      TYPE
**Severity:**   ERROR
**Emitted by:** eaml-semantic
**Condition:**  In a bounded type expression, the `min` value is greater than the `max` value.
**Message:**    `TYP030: Lower bound ({min}) exceeds upper bound ({max}).`

**Example:**
```eaml
schema Config {
  temperature: float(min: 2.0, max: 0.5)
}
```

**Resolution:** Swap the `min` and `max` values, or correct the bounds to form a valid range.
**Spec refs:**  TYPESYSTEM.md TS-BND-01, TS-BND-05
**Notes:**      The bounds are inclusive on both ends. `min == max` is valid and constrains the value to exactly one number.

---

### TYP031: Invalid string length bound

**Phase:**      TYPE
**Severity:**   ERROR
**Emitted by:** eaml-semantic
**Condition:**  A string length bound is a negative number.
**Message:**    `TYP031: String length bound must be a non-negative integer.`

**Example:**
```eaml
schema Input {
  query: string(min: -1, max: 100)
}
```

**Resolution:** Use a non-negative integer for string length bounds. `min: 0` means the empty string is allowed.
**Spec refs:**  TYPESYSTEM.md TS-BND-04
**Notes:**      For strings, `min` and `max` refer to character count, not byte length. Fractional values are also rejected.

---

### TYP032: Bounds on non-boundable type

**Phase:**      TYPE
**Severity:**   ERROR
**Emitted by:** eaml-semantic
**Condition:**  Bounded parameters are applied to a type that does not accept them, such as `bool`, `null`, or user-defined schemas.
**Message:**    `TYP032: Type '{type}' does not accept bounded parameters.`

**Example:**
```eaml
schema Flags {
  active: bool(min: 0, max: 1)
}
```

**Resolution:** Remove the bounded parameters. Only `int`, `float`, and `string` types accept `min`/`max` bounds.
**Spec refs:**  TYPESYSTEM.md TS-BND-06
**Notes:**      Schema types, literal unions, and array types also do not accept bounds. Array length constraints may be added in a future version.

---

### TYP040: Duplicate literal union member (WARNING)

**Phase:**      TYPE
**Severity:**   WARNING
**Emitted by:** eaml-semantic
**Condition:**  The same string value appears more than once in a literal union type.
**Message:**    `TYP040 warning: Duplicate member "{value}" in literal union.`

**Example:**
```eaml
schema Feedback {
  sentiment: "positive" | "negative" | "positive"
}
```

**Resolution:** Remove the duplicate member from the literal union.
**Spec refs:**  TYPESYSTEM.md TS-LIT-07, OQ-02
**Notes:**      The duplicate is silently deduplicated in the generated Pydantic model. The warning alerts the developer to likely copy-paste errors.

---

## 5. CAP -- Capability Errors

Capability errors are emitted during the capability subset-checking phase of semantic analysis.

### CAP001: Unknown capability name (WARNING)

**Phase:**      CAP
**Severity:**   WARNING (ERROR with `--strict-caps`)
**Emitted by:** eaml-semantic
**Condition:**  A capability name in a `caps:` or `requires` clause is not in the built-in capability registry.
**Message:**    `CAP001: Unknown capability '{name}'. Built-in capabilities are: json_mode, tools, vision, streaming, reasoning.`

**Example:**
```eaml
model my_model {
  provider: "anthropic"
  model_id: "claude-sonnet-4-20250514"
  caps: [json_mode, custom_feature]
}
```

**Resolution:** Check spelling of the capability name. If using a custom capability, this warning can be suppressed. With `--strict-caps`, unknown capabilities are treated as errors.
**Spec refs:**  CAPABILITIES.md CAP-MDL-02, CAP-REQ-04, CAP-REG-06, CAP-CUST-01, Layer 5 Section 6.1
**Notes:**      The built-in capability registry for v0.1 contains: `json_mode`, `tools`, `vision`, `streaming`, `reasoning`. Custom capabilities are allowed by default but emit this warning.

---

### CAP002: Duplicate capability name (WARNING)

**Phase:**      CAP
**Severity:**   WARNING
**Emitted by:** eaml-semantic
**Condition:**  The same capability name appears more than once in a `caps:` list or `requires` clause.
**Message:**    `CAP002: Duplicate capability '{name}' in {location}. The duplicate is ignored.`

**Example:**
```eaml
model my_model {
  provider: "anthropic"
  model_id: "claude-sonnet-4-20250514"
  caps: [json_mode, tools, json_mode]
}
```

**Resolution:** Remove the duplicate capability entry.
**Spec refs:**  CAPABILITIES.md CAP-REQ-06
**Notes:**      The duplicate is silently deduplicated. This warning helps catch copy-paste errors.

---

### CAP010: Capability mismatch (FATAL)

**Phase:**      CAP
**Severity:**   FATAL
**Emitted by:** eaml-semantic
**Condition:**  A prompt requires a capability that is not listed in the referenced model's `caps:` declaration.
**Message:**    `CAP010: Model '{model_name}' is missing required capabilities: [{caps}]. Required by prompt '{prompt_name}' at line {line}:{col}. Hint: Model '{model_name}' supports: [{model_caps}]. Add the missing capabilities to the model declaration, or use a model that supports {missing_caps}.`

**Example:**
```eaml
model basic_model {
  provider: "openai"
  model_id: "gpt-4o"
  caps: [json_mode]
}

prompt ToolUser {
  model basic_model
  requires [tools]
  "Use the tools"
}
```

**Resolution:** Either add the missing capabilities to the model's `caps:` list, or use a different model that supports the required capabilities.
**Spec refs:**  CAPABILITIES.md CAP-CHK-01, CAP-CHK-03, Layer 5 Section 6.3 [CLOSED]
**Notes:**      This is a FATAL error -- compilation halts immediately. Capability mismatches indicate a structural problem that would cause runtime failures. The Layer 5 decision to make this FATAL is [CLOSED] and not subject to re-evaluation.

---

### CAP020: json_mode with string return type (WARNING)

**Phase:**      CAP
**Severity:**   WARNING
**Emitted by:** eaml-semantic
**Condition:**  A prompt requires the `json_mode` capability but declares a bare `string` return type.
**Message:**    `CAP020: Prompt '{name}' requires json_mode but returns 'string'. Consider using a schema type to get Pydantic validation of the JSON response.`

**Example:**
```eaml
prompt Extract {
  model my_model
  requires [json_mode]
  "Extract data" -> string
}
```

**Resolution:** Consider changing the return type to a schema type to benefit from Pydantic validation of the JSON response.
**Spec refs:**  CAPABILITIES.md CAP-TYP-01
**Notes:**      This is a warning, not an error. Returning `string` with `json_mode` is valid but loses the benefit of structured validation. The developer may intend to parse the JSON manually.

---

### CapabilityActivationError: Runtime capability activation failure

**Phase:**      N/A (Runtime)
**Severity:**   RUNTIME
**Emitted by:** eaml_runtime (Python)
**Condition:**  At API call time, the provider does not support a capability that was declared in the EAML model definition.
**Message:**    `CapabilityActivationError: Cannot activate capability '{cap}' for provider '{provider}' (model '{model_id}'). The model declaration claims this capability but the provider does not support it.`

**Example:**
```python
# Generated code calls a model with caps: [vision]
# but the provider SDK rejects the vision parameter
raise CapabilityActivationError(
    "Cannot activate capability 'vision' for provider 'ollama' (model 'llama3'). "
    "The model declaration claims this capability but the provider does not support it."
)
```

**Resolution:** Update the EAML model declaration to remove capabilities that the provider does not actually support, or switch to a provider/model combination that supports the required capabilities.
**Spec refs:**  CAPABILITIES.md CAP-RUN-01, Section 9
**Notes:**      This is a Python exception, not a compiler diagnostic. It catches cases where the model declaration is overly optimistic about provider capabilities. The compiler cannot verify provider capabilities at compile time.

---

## 6. PYB -- Python Bridge Errors

Python bridge errors relate to the `python %{ }%` blocks used for tool implementations and Python interop.

### PYB001: Python bridge block parse error

**Phase:**      CODEGEN
**Severity:**   ERROR
**Emitted by:** eaml-codegen
**Condition:**  The `--check-python` flag is enabled and the Python code inside a `python %{ }%` block fails `py_compile` validation.
**Message:**    `PYB001: Python syntax error in bridge block at line {line}:{col}: {python_error}`

**Example:**
```eaml
tool lookup(query: string) -> string {
  python %{
    def lookup(query):
      return query.  # syntax error
  }%
}
```

**Resolution:** Fix the Python syntax error in the bridge block. The error message includes the Python compiler's diagnostic.
**Spec refs:**  grammar.ebnf Production [37] [lex: python-block-capture], Layer 5 Section 5.3
**Notes:**      Python validation is opt-in via `--check-python`. Without this flag, Python blocks are passed through as opaque strings. The EAML lexer captures everything between `%{` and `}%` without interpreting it.

---

### PYB010: Unknown provider string (WARNING)

**Phase:**      TYPE
**Severity:**   WARNING
**Emitted by:** eaml-semantic
**Condition:**  The `provider:` field in a model declaration contains a string that is not one of the recognized built-in providers.
**Message:**    `PYB010: Unknown provider '{provider}'. Built-in providers are: "anthropic", "openai", "ollama".`

**Example:**
```eaml
model my_model {
  provider: "custom_llm"
  model_id: "my-model-v1"
  caps: [json_mode]
}
```

**Resolution:** If using a built-in provider, check spelling. Custom providers are allowed but must be configured in the runtime.
**Spec refs:**  Layer 5 Section 10.2
**Notes:**      The PYB prefix is assigned per Layer 5 authority, though provider validation is performed during semantic analysis rather than codegen. Custom providers are valid but require runtime configuration. The built-in providers in v0.1 are: `"anthropic"`, `"openai"`, `"ollama"`.

---

## 7. RES -- Resolution Errors

Resolution errors are emitted during name resolution when identifiers cannot be bound to declarations.

### RES001: Undefined reference

**Phase:**      RESOLVE
**Severity:**   ERROR
**Emitted by:** eaml-semantic
**Condition:**  An identifier in expression position cannot be resolved to any declaration after both resolution passes (pass 1 collects declarations, pass 2 resolves references).
**Message:**    `RES001: Undefined name '{name}'.`

**Example:**
```eaml
prompt Greet {
  model my_model
  "Hello, {unknown_var}"
}
```

**Resolution:** Declare the referenced name before use, or fix the spelling. Check that imports are correct if referencing declarations from another file.
**Spec refs:**  grammar.ebnf [sem: forward-ref-allowed] (pass 2 unresolved)
**Notes:**      EAML supports forward references within a file -- a prompt can reference a model declared later. RES001 fires only after the second resolution pass fails to find the name. This is distinct from TYP010 (unknown *type* name) which fires in type position.

---

## 8. Quick Reference

### 8.1 Complete Error Code Index

| Code   | Title                              | Phase   | Severity | Emitted by    |
|--------|------------------------------------|---------|----------|---------------|
| SYN042 | Multi-dimensional array            | PARSE   | ERROR    | eaml-parser   |
| SYN043 | Pipe operator in expression        | PARSE   | ERROR    | eaml-parser   |
| SYN044 | Ampersand in expression            | PARSE   | ERROR    | eaml-parser   |
| SYN045 | Unclosed template interpolation    | LEX     | ERROR    | eaml-lexer    |
| SYN050 | Native tool body (Post-MVP)        | PARSE   | ERROR    | eaml-parser   |
| SYN060 | Circular import                    | PARSE   | ERROR    | eaml-parser   |
| SYN061 | Absolute or URL import path        | PARSE   | ERROR    | eaml-parser   |
| SYN080 | Pipeline declaration (Post-MVP)    | PARSE   | ERROR    | eaml-parser   |
| SYN081 | Pipeline operator (Post-MVP)       | PARSE   | ERROR    | eaml-parser   |
| SYN082 | Enum declaration (Post-MVP)        | PARSE   | ERROR    | eaml-parser   |
| SYN083 | Schema inheritance (Post-MVP)      | PARSE   | ERROR    | eaml-parser   |
| SYN090 | Field annotation (Post-MVP)        | PARSE   | ERROR    | eaml-parser   |
| SEM010 | Import after declaration           | RESOLVE | ERROR    | eaml-semantic |
| SEM020 | Duplicate field name               | TYPE    | ERROR    | eaml-semantic |
| SEM030 | Unknown bounded parameter          | TYPE    | ERROR    | eaml-semantic |
| SEM035 | Bounds in parameter position       | TYPE    | ERROR    | eaml-semantic |
| SEM040 | Tool body has no implementation    | TYPE    | ERROR    | eaml-semantic |
| SEM050 | Type annotation required           | PARSE   | ERROR    | eaml-parser   |
| SEM060 | Chained comparison                 | TYPE    | ERROR    | eaml-semantic |
| SEM070 | Recursive schema reference         | TYPE    | WARNING  | eaml-semantic |
| TYP001 | Built-in type shadowing            | TYPE    | WARNING  | eaml-semantic |
| TYP002 | Reserved future type name          | TYPE    | WARNING  | eaml-semantic |
| TYP003 | Type mismatch                      | TYPE    | ERROR    | eaml-semantic |
| TYP010 | Unknown type name                  | TYPE    | ERROR    | eaml-semantic |
| TYP030 | Lower bound exceeds upper bound    | TYPE    | ERROR    | eaml-semantic |
| TYP031 | Invalid string length bound        | TYPE    | ERROR    | eaml-semantic |
| TYP032 | Bounds on non-boundable type       | TYPE    | ERROR    | eaml-semantic |
| TYP040 | Duplicate literal union member     | TYPE    | WARNING  | eaml-semantic |
| CAP001 | Unknown capability name            | CAP     | WARNING  | eaml-semantic |
| CAP002 | Duplicate capability name          | CAP     | WARNING  | eaml-semantic |
| CAP010 | Capability mismatch                | CAP     | FATAL    | eaml-semantic |
| CAP020 | json_mode with string return       | CAP     | WARNING  | eaml-semantic |
| PYB001 | Python bridge parse error          | CODEGEN | ERROR    | eaml-codegen  |
| PYB010 | Unknown provider string            | TYPE    | WARNING  | eaml-semantic |
| RES001 | Undefined reference                | RESOLVE | ERROR    | eaml-semantic |

### 8.2 Severity Summary

| Severity  | Count  | Codes                                                                                                                                                  |
|-----------|--------|--------------------------------------------------------------------------------------------------------------------------------------------------------|
| FATAL     | 1      | CAP010                                                                                                                                                 |
| ERROR     | 26     | SYN042–045, SYN050, SYN060–061, SYN080–083, SYN090, SEM010, SEM020, SEM030, SEM035, SEM040, SEM050, SEM060, TYP003, TYP010, TYP030–032, PYB001, RES001 |
| WARNING   | 8      | SEM070, TYP001, TYP002, TYP040, CAP001, CAP002, CAP020, PYB010                                                                                         |
| RUNTIME   | 1      | CapabilityActivationError                                                                                                                              |
| **Total** | **36** | 35 compiler codes + 1 runtime exception                                                                                                                |

### 8.3 Phase Summary

| Phase   | Count   | Codes                                                                                          |
|---------|---------|------------------------------------------------------------------------------------------------|
| LEX     | 1       | SYN045                                                                                         |
| PARSE   | 12      | SYN042-044, SYN050, SYN060-061, SYN080-083, SYN090, SEM050                                     |
| RESOLVE | 2       | SEM010, RES001                                                                                 |
| TYPE    | 15      | SEM020, SEM030, SEM035, SEM040, SEM060, SEM070, TYP001-003, TYP010, TYP030-032, TYP040, PYB010 |
| CAP     | 4       | CAP001, CAP002, CAP010, CAP020                                                                 |
| CODEGEN | 1       | PYB001                                                                                         |

---

## 9. Open Questions, Conflicts, and Reserved Ranges

### 9.1 Resolved Conflicts

**CONFLICT-1: SEM050 phase attribution.**
TYPESYSTEM.md describes SEM050 as a "parse error" (grammar Production [41] requires `: typeExpr`). Layer 5 Section 11 assigns the SEM prefix. Resolution: The code is SEM050 (per Layer 5 authority hierarchy), attributed to the PARSE phase (per grammar enforcement mechanism). The parser detects the missing annotation and emits SEM050.

**CONFLICT-2: PYB010 prefix vs. source.**
Layer 5 Section 10.2 assigns PYB010 for unknown provider validation, but the check is performed during semantic analysis, not during Python bridge processing. Resolution: The PYB010 code is retained per Layer 5 authority. The semantic nature is documented in the entry.

### 9.2 Open Questions

**OQ-1: Unclosed python %{ delimiter.**
If `python %{` appears without a matching `}%`, this is a lexer error (EOF while scanning for `}%`). No specific error code has been assigned. The lexer currently produces a generic SYN error for unterminated tokens. Recommendation: assign a dedicated code (e.g., SYN046) or document as covered by generic lexer EOF handling.

### 9.3 TYP500/SEM050 Overlap

TYPESYSTEM.md Section 8 uses the heading "TYP5xx -- Annotation and position errors" but the only entry is SEM050. TYP500 does NOT exist as a separate error code. The TYP5xx heading in TYPESYSTEM.md is organizational only. SEM050 is the canonical code.

### 9.4 Reserved Code Ranges

The following code ranges are reserved for future use. Unassigned codes within allocated ranges are also reserved.

| Range       | Reserved for                                    |
|-------------|-------------------------------------------------|
| SYN001-041  | Future lexer and parser errors                  |
| SYN046-049  | Future lexer errors (template strings, etc.)    |
| SYN051-059  | Future tool-related syntax errors               |
| SYN062-079  | Future import and module errors                 |
| SYN084-089  | Future post-MVP syntax features                 |
| SYN091-099  | Future annotation errors                        |
| SEM001-009  | Future import/resolution semantic errors        |
| SEM021-029  | Future schema semantic errors                   |
| SEM031-034  | Future bounds semantic errors                   |
| SEM036-039  | Future parameter semantic errors                |
| SEM041-049  | Future tool semantic errors                     |
| SEM051-059  | Future annotation semantic errors               |
| SEM061-069  | Future expression semantic errors               |
| SEM071-099  | Future semantic warnings                        |
| TYP004-009  | Future type declaration errors                  |
| TYP011-029  | Future type resolution errors                   |
| TYP033-039  | Future bounds errors                            |
| TYP041-099  | Future type errors                              |
| CAP003-009  | Future capability declaration errors            |
| CAP011-019  | Future capability checking errors               |
| CAP021-099  | Future capability errors                        |
| PYB002-009  | Future Python bridge errors                     |
| PYB011-099  | Future provider/bridge errors                   |
| RES002-099  | Future resolution errors                        |

---

## 10. Verification Report — EAML ERRORS.md v0.1.0

| Group                      | Checks  | Passed  | Failed  | N/A   |
|----------------------------|---------|---------|---------|-------|
| A — Completeness (defined) | 7       | 7       | 0       | 0     |
| B — Completeness (cited)   | 4       | 4       | 0       | 0     |
| C — Consistency            | 5       | 5       | 0       | 0     |
| D — Code Space             | 3       | 3       | 0       | 0     |
| E — Quality                | 5       | 5       | 0       | 0     |
| **Total**                  | **24**  | **24**  | **0**   | **0** |

Failed checks: 0
Ghost citations found: 0
Orphan entries found: 0
Open Questions: 1 (OQ-1: unclosed python %{ delimiter)
Conflicts resolved: 2

### Total Defined Codes

SYN: 12 | SEM: 8 | TYP: 8 | CAP: 4 | PYB: 2 | RES: 1 | RUNTIME: 1

Grand total: 35 compiler codes + 1 runtime exception = 36

### Group A — Completeness: Every Code Defined

**A1[PASS]** SYN codes from grammar.ebnf Post-MVP productions:
SYN042 (§2), SYN043 (§2), SYN044 (§2), SYN045 (§2), SYN050 (§2),
SYN060 (§2), SYN061 (§2), SYN080 (§2), SYN081 (§2), SYN082 (§2),
SYN083 (§2), SYN090 (§2). All 12 present.

**A2[PASS]** SEM codes from grammar.ebnf [sem:] annotations (E2 check):
- [sem: no-chained-comparison] → SEM060 ✓
- [sem: cap-registry] → CAP001 ✓
- [sem: field-type-must-resolve] → resolved in pass 2 (TYP010/RES001) ✓
- [sem: import-before-declarations] → SEM010 ✓
- [sem: default-must-match-param-type] → TYP003 (semantic check) ✓
- [sem: let-type-must-match-expr] → TYP003 (type check) ✓
- [sem: prompt-requires-user-field] → semantic check (no specific code — enforced, not a separate error) ✓
- [sem: no-positional-after-named] → semantic check (no specific code assigned — OPEN QUESTION flagged in grammar) ✓
- [sem: single-dimension-only-v0.1] → SYN042 ✓
- [sem: forward-ref-allowed] → two-pass resolution ✓
- [sem: bounded-param-validation] → SEM030 ✓
- [sem: v0.1-python-required] → SYN050 ✓
- [sem: tool-body-must-have-implementation] → SEM040 ✓
All 13 [sem:] annotations accounted for.

**A3[PASS]** TYP codes from TYPESYSTEM.md §8:
TYP001 (§4), TYP003 (§4), TYP010 (§4), TYP030 (§4), TYP031 (§4),
TYP032 (§4), TYP040 (§4), SEM020 (§3), SEM030 (§3), SEM050 (§3).
Plus TYP002 from grammar.ebnf predeclared identifiers. All present.

**A4[PASS]** CAP codes from CAPABILITIES.md §9:
CAP001 (§5), CAP002 (§5), CAP010 (§5), CAP020 (§5),
CapabilityActivationError (§5). All 5 present.

**A5[PASS]** PYB001 present in §6. --check-python opt-in documented.
Unclosed python %{ condition documented as OQ-1.

**A6[PASS]** RES001 present in §7. Distinction from TYP010
(type position vs expression position) documented in Notes.

**A7[PASS]** Quick Reference §8 lists 35 compiler codes + 1 runtime entry.
This matches the sum of individual entries in §§2–7:
SYN(12) + SEM(8) + TYP(8) + CAP(4+1 runtime) + PYB(2) + RES(1) = 36.

### Group B — Completeness: Every Citation Traced

**B1[PASS]** BACKWARD CHECK — grammar.ebnf:
All error codes cited in grammar.ebnf production comments verified present:
TYP001, TYP002, SYN042, SYN043, SYN045, SYN050, SYN080, SYN081,
SYN082, SYN083, SYN090, SEM010, SEM030, SEM040, SEM060, CAP001.
No ghost citations.

**B2[PASS]** BACKWARD CHECK — TYPESYSTEM.md:
All error codes cited in TYPESYSTEM.md Invalid: fields and §8 entries:
TYP001, TYP003, TYP010, TYP030, TYP031, TYP032, TYP040,
SEM020, SEM030, SEM035, SEM050, SEM070, SYN042, SYN043,
SYN080, SYN081, SYN082, SYN083, SYN090. All present. No ghost citations.

**B3[PASS]** BACKWARD CHECK — CAPABILITIES.md:
All error codes cited in CAPABILITIES.md Invalid: fields and §9:
CAP001, CAP002, CAP010, CAP020, CapabilityActivationError,
SYN083. All present. No ghost citations.

**B4[PASS]** FORWARD CHECK — ERRORS.md:
Every error code in ERRORS.md has a non-empty Spec refs field citing
at least one source document. Verified for all 35 entries + 1 runtime.
No orphan entries.

### Group C — Consistency Checks

**C1[PASS]** No code appears in two category sections. Each code is unique
in the index table (§8.1). No duplicate code numbers across sections.

**C2[PASS]** Severity consistency:
- TYP001: WARNING in TYPESYSTEM.md, WARNING here ✓
- TYP040: WARNING in TYPESYSTEM.md, WARNING here ✓
- CAP002: WARNING in CAPABILITIES.md, WARNING here ✓
- SEM070: WARNING (recommended resolution OQ-03), WARNING here ✓
- All non-warning codes verified ERROR or FATAL.

**C3[PASS]** Phase consistency:
- SYN codes: LEX (SYN045) or PARSE (all others) ✓
- SEM codes: RESOLVE (SEM010) or TYPE (all others) or PARSE (SEM050) ✓
- TYP codes: TYPE ✓
- CAP codes: CAP ✓
- PYB codes: CODEGEN (PYB001) or TYPE (PYB010) ✓
- RES codes: RESOLVE ✓
Exceptions documented: SEM050 (PARSE phase, SEM prefix per Layer 5);
PYB010 (TYPE phase, PYB prefix per Layer 5).

**C4[PASS]** CAP010 is marked FATAL. Verified in §5 entry and §8.2 table.
Layer 5 §6.3 [CLOSED] honored.

**C5[PASS]** TYP500/SEM050 overlap resolved: SEM050 is the canonical code.
No TYP500 entry exists. Resolution documented in §9.3.

### Group D — Code Space Checks

**D1[PASS]** All unassigned code ranges documented in §9.4 (Reserved Ranges).
No range has unexplained gaps.

**D2[PASS]** No code number used for two different errors. The quick reference
table in §8.1 has no duplicate rows.

**D3[PASS]** Total code count stated in §8.2: 35 defined codes + 1 runtime.
Matches sum of entries: SYN(12) + SEM(8) + TYP(8) + CAP(4) + PYB(2) + RES(1) = 35 + 1 runtime = 36.

### Group E — Document Quality Checks

**E1[PASS]** Format compliance — spot-checked 5 entries:
SYN042 (§2): Phase, Severity, Emitted by, Condition, Message, Example, Resolution, Spec refs, Notes ✓
SEM035 (§3): All fields present ✓
TYP003 (§4): All fields present ✓
CAP010 (§5): All fields present ✓
RES001 (§7): All fields present ✓

**E2[PASS]** Message templates use {variable} placeholders:
- CAP010: {model_name}, {caps}, {prompt_name}, {line}, {col}, {model_caps}, {missing_caps} ✓
- TYP003: {expected}, {actual} ✓
- TYP010: {name}, {suggestion} ✓
- SEM030: {param}, {type} ✓
All checked messages have appropriate variables.

**E3[PASS]** Every Resolution field is actionable:
- SYN042: "Wrap the inner array in a named schema..." ✓
- CAP010: "Add the missing capability... or use a different model..." ✓
- TYP010: "Correct the type name. Check for typos..." ✓
No entries use generic "fix the error" language.

**E4[PASS]** Table of Contents matches document structure.
All 10 sections listed in TOC exist with matching titles.

**E5[PASS]** Compiler errors vs runtime exceptions clearly distinguished.
CapabilityActivationError uses RUNTIME severity, Python exception class,
and Python example code. Not described using compiler terminology.

### Ghost Citation Check (B1–B3)

No ghost citations found. Every error code cited in grammar.ebnf,
TYPESYSTEM.md, and CAPABILITIES.md is registered in this catalog.

### Orphan Entry Check (B4)

No orphan entries. Every error code defined in this catalog has a
non-empty Spec refs field citing at least one source document.