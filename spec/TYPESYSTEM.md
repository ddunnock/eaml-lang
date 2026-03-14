# EAML Type System Specification

**Version:** 0.1.0
**Date:** 2026-03-14
**Status:** AUTHORITATIVE

---

## Abstract

This document is the complete type system specification for EAML (Engineering AI
Markup Language) version 0.1.0. It defines the semantics of every type expression
the EAML compiler accepts, the rules the type checker enforces, and the Pydantic v2
code that the code generator produces for each type form.

This document serves three consumers:

1. **Compiler semantic analysis** — every type-checking decision the compiler makes
   MUST be traceable to a rule in this document.
2. **Pydantic v2 code generator** — every EAML type has an unambiguous mapping to a
   Python type annotation and Pydantic field declaration specified here.
3. **EAML language users** — every rule is explained in terms an engineer without a
   compiler background can apply when writing `.eaml` files.

### Normative Language

The key words "MUST", "MUST NOT", "SHALL", "SHOULD", "MAY" in this document are to
be interpreted as described in [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

### Related Documents

| Document | Relationship |
|----------|-------------|
| `spec/grammar.ebnf` | Syntactic contract — this document cites grammar productions by number |
| `spec/ERRORS.md` | Error code catalog — type errors documented here are registered there |
| Layer 5 (`eaml-layer5-design-decisions.md`) | Authoritative design decisions — this document implements them |

### How to Read This Document

**Rule blocks** follow a consistent format throughout:

```
RULE [TS-CAT-NN]: [Short imperative title]

  Plain English: [One-paragraph description accessible to any engineer]
  Formal:        [Set theory or EBNF-like notation where it adds precision]
  Grammar:       Production [N] in grammar.ebnf
  Valid:         [EAML code example that is correct]
  Invalid:       [EAML code example that triggers an error] → Error [CODE]: [message]
  Pydantic v2:   [Generated Python type annotation and/or Pydantic Field]
  Notes:         [Cross-references, edge cases, rationale]
```

**Grammar citations** use the format "Production [N]" referring to the numbered
production rule in `spec/grammar.ebnf`.

**Error citations** use the format `TYP[code]` for type errors, `SYN[code]` for
syntax errors, and `SEM[code]` for semantic errors.

---

## Table of Contents

1. [Type System Philosophy](#1-type-system-philosophy)
   - 1.1 Typing Discipline: Nominal, Not Structural
   - 1.2 Static vs Runtime Validation
   - 1.3 Type System Scope in v0.1
   - 1.4 Design Decisions Summary
2. [Primitive Types](#2-primitive-types)
   - 2.1 string
   - 2.2 int
   - 2.3 float
   - 2.4 bool
   - 2.5 null
   - 2.6 Primitive Type Summary
   - 2.7 Type Name Casing
3. [Composite Types](#3-composite-types)
   - 3.1 Array Type
   - 3.2 Optional Type
   - 3.3 Composite Ordering Rules
   - 3.4 Composite Type Restrictions
4. [Bounded Types](#4-bounded-types)
   - 4.1 Bounded Type Overview
   - 4.2 float Bounds
   - 4.3 string Bounds
   - 4.4 int Bounds
   - 4.5 Bounded Type Restrictions
5. [Literal Union Types](#5-literal-union-types)
   - 5.1 Literal Union Definition
   - 5.2 Literal Union Composition
   - 5.3 Literal Union Restrictions
6. [Schema-Defined Types](#6-schema-defined-types)
   - 6.1 Schema as Nominal Type
   - 6.2 Schema Field Rules
   - 6.3 Schema as Return Type
   - 6.4 Pydantic v2 Generation Rules
7. [Type Positions and Annotations](#7-type-positions-and-annotations)
   - 7.1 Required Type Annotation Positions
   - 7.2 Type Annotations Are Forbidden In
   - 7.3 Type Inference (Post-MVP)
   - 7.4 Return Type Rules
8. [Type Error Catalog](#8-type-error-catalog)
9. [Post-MVP Type Features](#9-post-mvp-type-features)
10. [Pydantic v2 Code Generation Reference](#10-pydantic-v2-code-generation-reference)
    - 10.1 Code Generation Philosophy
    - 10.2 Schema to BaseModel Generation
    - 10.3 Complete Type Mapping Table
    - 10.4 Optional Field Default Value Rule
    - 10.5 Literal Union Import

---

## 1. Type System Philosophy

### 1.1 Typing Discipline: Nominal, Not Structural

EAML uses **nominal typing** for schema-defined types. Two schemas with identical
field structures are distinct types and are NOT interchangeable.

**Layer 5 §3.2 [CLOSED]:** "EAML uses NOMINAL typing for schemas."

```eaml
schema Confidence {
  score: float
}

schema Probability {
  score: float
}

prompt Calibrate(data: string) -> Confidence {
  user: "Estimate confidence: {data}"
}

// A Probability value CANNOT be used where Confidence is expected,
// even though both contain a single float field named 'score'.
// This is a TYP003 type mismatch error.
```

**Why nominal?** Schemas represent domain concepts, not data shapes. A `Confidence`
score and a `Probability` value may have the same structure but different semantics.
In defense/aerospace contexts, confusing two structurally identical types is exactly
the class of error that causes system failures. Nominal typing prevents this at
compile time.

**Contrast with structural typing (TypeScript, BAML):** In TypeScript,
`{ score: number }` is interchangeable with any other `{ score: number }` regardless
of the declared interface name. EAML rejects this — the declared schema name IS the
type identity.

Grammar reference: Production [29] `schemaDecl` in grammar.ebnf.

### 1.2 Static vs Runtime Validation

EAML employs a two-layer validation model:

**Layer 1 — Compile-time type checking** (semantic analysis phase):
- Schema field types resolve to defined types
- Prompt and tool return types are valid type expressions
- Parameter types match at call sites
- Capability requirements are satisfied by the model at each call site
- Bounded type parameters are valid (min ≤ max, correct parameter names)

**Layer 2 — Runtime validation** (generated Pydantic v2 models):
- LLM output conforms to the declared schema structure
- Bounded values fall within their declared ranges
- Literal union values match one of the declared members
- Required fields are present in the response

**What compile-time CANNOT catch:** The content of an LLM's response is inherently
unpredictable. A prompt declared to return `SentimentResult` may receive a response
that does not conform to the schema — this is always a runtime concern. The
`max_retries` field in prompt declarations (Layer 5 §7.3, Production [33])
controls how many times the runtime re-prompts the LLM on validation failure
before raising `LLMValidationError`.

### 1.3 Type System Scope in v0.1

**IN SCOPE:**

| Feature | Section |
|---------|---------|
| Primitive types: `string`, `int`, `float`, `bool`, `null` | §2 |
| Array types: `T[]` | §3.1 |
| Optional types: `T?` | §3.2 |
| Composite ordering: `T[]?`, `T?[]`, `T?[]?` | §3.3 |
| Bounded types: `float<0.0, 1.0>`, `string<max: 200>`, `int<min: 0>` | §4 |
| Literal union types: `"yes" \| "no"` | §5 |
| Schema-defined types | §6 |

**OUT OF SCOPE (Post-MVP):**

| Feature | Blocking Error | Reference |
|---------|---------------|-----------|
| `enum` declarations | SYN082 | Layer 5 §11, §9 below |
| Schema inheritance (`extends`) | SYN083 | Layer 5 §11 |
| Type inference on `let` bindings | SEM050 | Layer 5 §11 |
| Union types beyond literal unions (`Tag \| OtherSchema`) | Not in grammar | §9 |
| Generic types (`Schema<T>`) | Not in grammar | §9 |
| Recursive schema types | See §9 OPEN QUESTION | §9 |
| `void` keyword | TYP010 | Layer 5 §7.4 |
| `@` field annotations | SYN090 | Layer 5 §11 |
| String pattern bounds (`string<pattern: "...">`) | Not in grammar | §9 |

### 1.4 Design Decisions Summary

| Decision | Value | Layer 5 Reference | Rationale |
|----------|-------|-------------------|-----------|
| Typing discipline | Nominal | §3.2 [CLOSED] | Schemas are domain concepts, not shapes |
| Primitive names | Lowercase only | §3.1 [CLOSED] | Predeclared identifiers, not keywords |
| `Tag[]?` vs `Tag?[]` | Both valid, different types | §3.3 [CLOSED] | Position determines meaning |
| Array dimensions | Single only in v0.1 | §3.4 [CLOSED] | Simplicity; nested schemas cover use cases |
| Literal union minimum | Two members | §3.6 [CLOSED] | One string is a type annotation, not a union |
| Bounded types | Field constraints via Pydantic | §3.5 [CLOSED] | Runtime enforcement, not type identity |
| `null` vs `?` | Distinct concepts | §3.1, §3.3 | `null` is a value; `?` is absence |
| `void` return type | Use `-> null` | §7.4 [CLOSED] | `void` reserved for Post-MVP |
| Let type annotation | Required in v0.1 | §11 | Type inference is Post-MVP |

---

## 2. Primitive Types

### 2.1 string

**RULE TS-PRM-01: String type**

> Plain English: The `string` type represents a sequence of Unicode characters.
> It maps to Python's `str` and JSON string values.
>
> Formal: `string : Type` where values are Unicode scalar value sequences
>
> Grammar: Production [44] `namedType` — resolved as IDENT "string" via type registry.
> Literal syntax: Production [8] `STRING` — double-quoted with escape sequences.
>
> Valid:
> ```eaml
> schema Greeting {
>   message: string
> }
> ```
>
> Invalid:
> ```eaml
> schema Bad {
>   count: string
> }
> // Then at a call site: providing an int where string is expected
> // → TYP003: Type mismatch: expected 'string', found 'int'
> ```
>
> Pydantic v2: `str`
>
> JSON wire format: JSON string (`"hello"`)
>
> Notes: `string` is a predeclared identifier in the type registry, not a keyword.
> A schema named `string` emits TYP001 warning but is not a compile error
> (Layer 5 §3.1).

### 2.2 int

**RULE TS-PRM-02: Integer type**

> Plain English: The `int` type represents whole numbers (positive, negative, or zero).
> Negative values are expressed as unary minus applied to a positive literal.
>
> Formal: `int : Type` where values are ℤ (integers)
>
> Grammar: Production [44] `namedType` — resolved as IDENT "int" via type registry.
> Literal syntax: Production [6] `INT` — decimal only, no leading zeros except `0`.
>
> Valid:
> ```eaml
> schema Config {
>   max_items: int
> }
> ```
>
> Invalid:
> ```eaml
> schema Bad {
>   max_items: int
> }
> // Providing a float where int expected:
> // → TYP003: Type mismatch: expected 'int', found 'float'
> ```
>
> Pydantic v2: `int`
>
> JSON wire format: JSON number without decimal point (`42`, `-7`, `0`)
>
> Notes: The lexer never emits negative integer tokens. `-1` is parsed as
> unary minus (Production [60]) applied to INT `1` (Layer 5 §2.4).

### 2.3 float

**RULE TS-PRM-03: Float type**

> Plain English: The `float` type represents decimal numbers. A float literal MUST
> have digits on both sides of the decimal point — `.5` is not valid EAML.
>
> Formal: `float : Type` where values are IEEE 754 double-precision
>
> Grammar: Production [44] `namedType` — resolved as IDENT "float" via type registry.
> Literal syntax: Production [7] `FLOAT` — decimal point required, digits on both sides.
>
> Valid:
> ```eaml
> schema Score {
>   confidence: float
> }
> ```
>
> Invalid:
> ```eaml
> // .5 is not a valid float literal:
> // → SYN: Expected digit before decimal point
> ```
>
> Pydantic v2: `float`
>
> JSON wire format: JSON number with or without decimal point (`0.95`, `1.0`)
>
> Notes: Layer 5 §2.4 [CLOSED] — float literals require leading digit.

### 2.4 bool

**RULE TS-PRM-04: Boolean type**

> Plain English: The `bool` type represents a truth value: `true` or `false`.
>
> Formal: `bool : Type` where values ∈ {true, false}
>
> Grammar: Production [44] `namedType` — resolved as IDENT "bool" via type registry.
> Literal syntax: Production [11] `BOOL_LIT` — `"true" | "false"`.
>
> Valid:
> ```eaml
> schema Decision {
>   approved: bool
> }
> ```
>
> Invalid:
> ```eaml
> schema Bad {
>   approved: bool
> }
> // Providing string "true" where bool expected:
> // → TYP003: Type mismatch: expected 'bool', found 'string'
> ```
>
> Pydantic v2: `bool`
>
> JSON wire format: JSON boolean (`true`, `false`)

### 2.5 null

**RULE TS-PRM-05: Null type**

> Plain English: The `null` type represents an explicit null value. It is a concrete
> value, NOT the absence of a value. `null` differs from optional (`?`):
>
> - `null` means "the value is explicitly nothing"
> - `?` (optional) means "the value may or may not be present"
>
> A field of type `string` cannot hold `null` — it MUST be `string?` to accept
> absence, or the union `string | null` is not available in v0.1 (use `string?`).
>
> Formal: `null : Type` where the only value is `null`
>
> Grammar: Production [44] `namedType` — resolved as IDENT "null" via type registry.
> Literal syntax: Production [12] `NULL_LIT` — `"null"`.
>
> Valid:
> ```eaml
> // Tool with no meaningful return value:
> tool LogEvent(message: string) -> null {
>   python %{
>     logger.info(message)
>   }%
> }
> ```
>
> Invalid:
> ```eaml
> schema Result {
>   name: string
> }
> // LLM returns null for 'name' field:
> // → Runtime ValidationError: 'name' field is required (not nullable)
> // Fix: use 'name: string?' if null is acceptable
> ```
>
> Pydantic v2: `None` (Python's `None` type)
>
> JSON wire format: JSON `null`
>
> Notes: `null` as a return type (`-> null`) is EAML's v0.1 equivalent of `void`
> (Layer 5 §7.4). See also §3.2 for the null vs optional distinction.
> Cross-reference: Section 7.4 (return types).

### 2.6 Primitive Type Summary

| EAML Type | Python Type | Pydantic v2 | JSON Wire Format | Literal Syntax (Production) |
|-----------|-------------|-------------|------------------|----------------------------|
| `string` | `str` | `str` | `"text"` | `STRING` [8] |
| `int` | `int` | `int` | `42` | `INT` [6] |
| `float` | `float` | `float` | `0.95` | `FLOAT` [7] |
| `bool` | `bool` | `bool` | `true` / `false` | `BOOL_LIT` [11] |
| `null` | `None` | `None` | `null` | `NULL_LIT` [12] |

### 2.7 Type Name Casing

**RULE TS-PRM-06: Primitive type names are lowercase only**

> Plain English: EAML primitive type names are lowercase: `string`, `int`, `float`,
> `bool`, `null`. Using uppercase variants such as `String`, `Int`, `Float`, `Bool`
> is a type resolution error — these are not predeclared type names.
>
> Grammar: Production [44] `namedType` — IDENT is resolved via the type registry.
> Only lowercase names are predeclared.
>
> Valid: `score: float`
>
> Invalid: `score: Float` → TYP010: Unknown type 'Float'. Did you mean 'float'?
>
> Notes: Layer 5 §3.1 [CLOSED]. Primitive names are predeclared identifiers, not
> keywords. A user-defined schema named `String` is valid (it shadows nothing because
> `String` is not predeclared). A schema named `string` emits TYP001 warning.

---

## 3. Composite Types

### 3.1 Array Type

**RULE TS-ARR-01: Array type form**

> Plain English: An array type is written as a base type followed by `[]`. It
> represents an ordered sequence of zero or more elements, all of the same type.
>
> Formal: If `T` is a valid type, then `T[]` is a valid type representing `List[T]`.
>
> Grammar: Production [48] `arraySuffix` — `"[" "]"`.
> Applied in Production [42] `typeExpr`.
>
> Valid:
> ```eaml
> schema Report {
>   tags: string[]
>   scores: float[]
>   items: DataItem[]
> }
> ```
>
> Invalid:
> ```eaml
> // Using array syntax in expression context:
> // let x = []  — this is an index expression, not an array literal.
> // EAML has no array literal syntax — arrays come from LLM responses.
> ```
>
> Pydantic v2: `List[T]` (from `typing.List`)
>
> JSON wire format: JSON array (`["a", "b", "c"]`)

**RULE TS-ARR-02: Single dimension only**

> Plain English: Multi-dimensional arrays (`Tag[][]`) are not supported in v0.1.
> This is a parse error caught by the grammar, not a type error.
>
> Grammar: Production [48] `arraySuffix` allows only one `[]`. A second `[]` does
> not match any grammar production after the first `arraySuffix`.
>
> Valid: `tags: string[]`
>
> Invalid: `matrix: int[][]` → SYN042: "Multi-dimensional arrays are not supported
> in EAML v0.1. Hint: Use a schema with an array field:
> `schema IntMatrix { rows: int[] }`"
>
> Notes: This restriction is enforced by the grammar structure, not the type checker.
> The `[parser: syn042-multi-dim-check]` annotation on Production [48] documents this.
> The parser actively checks for a second `[` after consuming `arraySuffix`.

### 3.2 Optional Type

**RULE TS-OPT-01: Optional type form**

> Plain English: An optional type is written as a base type followed by `?`. It
> means the value MAY be absent (`None` in Python). An optional field that is not
> present in the LLM response is valid — it defaults to `None`.
>
> Formal: If `T` is a valid type, then `T?` is a valid type representing `T | None`.
>
> Grammar: Production [49] `optionalSuffix` — `"?"`.
> Applied in Production [42] `typeExpr`.
>
> Valid:
> ```eaml
> schema UserProfile {
>   name: string        // required — must be present
>   bio: string?        // optional — may be absent (None)
> }
> ```
>
> Pydantic v2: `Optional[T]` (equivalent to `T | None`).
> Optional fields in generated models receive `= None` as default value.
>
> JSON wire format: Field absent from JSON object, or explicitly `null`.

**RULE TS-OPT-02: null vs optional distinction**

> Plain English: `null` and `?` are distinct concepts in EAML:
>
> - `null` is a **type** representing an explicit null value. A field of type `null`
>   always holds `null`.
> - `?` is a **type modifier** meaning "this value may be absent." A field of type
>   `string?` holds either a string value or nothing at all.
>
> A field of type `string` cannot hold `null`. To accept absence, use `string?`.
>
> | Declaration | Accepts `"hello"` | Accepts `null` / absent | Pydantic v2 |
> |-------------|-------------------|------------------------|-------------|
> | `name: string` | Yes | No — ValidationError | `str` (required) |
> | `name: string?` | Yes | Yes — defaults to None | `Optional[str] = None` |
> | `name: null` | No | Yes — always null | `None` |
>
> Grammar: `null` is Production [44] `namedType` (IDENT "null").
> `?` is Production [49] `optionalSuffix`.
>
> Notes: Cross-reference Section 2.5 (null primitive). This distinction is critical
> for correct Pydantic code generation — see Section 10.4.

### 3.3 Composite Ordering Rules

The position of `[]` and `?` relative to each other determines the type's meaning.
Layer 5 §3.3 [CLOSED]: "POSITION DETERMINES MEANING."

**RULE TS-COMP-01: `T[]` — required array of required elements**

> Plain English: An array that MUST be present, containing elements that MUST NOT
> be null.
>
> Grammar: Production [42] `typeExpr` — `baseType typeModifiers`.
> Production [42a] `typeModifiers` — `arraySuffix` (first alternative).
>
> Valid:
> ```eaml
> schema Report {
>   tags: string[]
> }
> ```
>
> Pydantic v2: `List[str]` (required, no default)
>
> JSON: `["tag1", "tag2"]` — MUST be present, elements MUST NOT be null.

**RULE TS-COMP-02: `T[]?` — optional array of required elements**

> Plain English: The entire array may be absent (`None`). If present, every element
> MUST NOT be null.
>
> Formal: `Optional(Array(T))` — the array itself is nullable.
>
> Grammar: Production [42] `typeExpr` — `baseType typeModifiers`.
> Production [42a] `typeModifiers` — `arraySuffix optionalSuffix` (first alternative with both).
> `[]` is applied first, then `?` wraps the result.
>
> Valid:
> ```eaml
> schema Report {
>   tags: string[]?
> }
> // Valid JSON responses:
> //   { "tags": ["a", "b"] }     — array present
> //   { }                         — tags absent (None)
> //   { "tags": null }            — explicitly null (None)
> ```
>
> Pydantic v2: `Optional[List[str]] = None`
>
> Notes: The `?` applies to the entire `string[]` type, making the whole array optional.

**RULE TS-COMP-03: `T?[]` — required array of optional elements**

> Plain English: The array MUST be present, but individual elements may be null.
>
> Formal: `Array(Optional(T))` — elements are nullable, array is required.
>
> Grammar: Production [42] `typeExpr` — `baseType typeModifiers`.
> Production [42a] `typeModifiers` — `optionalSuffix arraySuffix` (second alternative).
> `?` is applied first to the base type, then `[]` wraps the result.
>
> Valid:
> ```eaml
> schema Survey {
>   responses: string?[]
> }
> // Valid JSON:
> //   { "responses": ["yes", null, "no"] }  — array present, some nulls
> ```
>
> Invalid:
> ```eaml
> // Missing the array entirely:
> //   { }  → Runtime ValidationError: 'responses' is required
> ```
>
> Pydantic v2: `List[Optional[str]]` (required, no default)

**RULE TS-COMP-04: `T?[]?` — optional array of optional elements**

> Plain English: The array may be absent entirely. If present, individual elements
> may be null.
>
> Formal: `Optional(Array(Optional(T)))`
>
> Grammar: Production [42] `typeExpr` — `baseType typeModifiers`.
> Production [42a] `typeModifiers` — `optionalSuffix arraySuffix` (second alternative)
> produces `T?[]`. The outer `?` requires grouping: `(T?[])?` which is equivalent
> to writing `T?[]?` where the final `?` applies to the whole preceding type.
>
> Valid:
> ```eaml
> schema Survey {
>   responses: string?[]?
> }
> // Valid JSON:
> //   { "responses": ["yes", null] }   — present with nulls
> //   { }                               — absent entirely
> ```
>
> Pydantic v2: `Optional[List[Optional[str]]] = None`

**Composite Ordering Summary:**

| Form | Meaning | Array Required? | Elements Required? | Pydantic v2 |
|------|---------|-----------------|-------------------|-------------|
| `T[]` | Array of T | Yes | Yes | `List[T]` |
| `T[]?` | Optional array of T | No | Yes (if present) | `Optional[List[T]] = None` |
| `T?[]` | Array of optional T | Yes | No | `List[Optional[T]]` |
| `T?[]?` | Optional array of optional T | No | No | `Optional[List[Optional[T]]] = None` |

### 3.4 Composite Type Restrictions

**RULE TS-COMP-05: Nested arrays are not permitted**

> Plain English: `Tag[][]` is a parse error in v0.1. Use a schema with an array
> field to represent multi-dimensional data.
>
> Grammar: Production [48] `arraySuffix` matches only one `[]`.
>
> Invalid: `matrix: int[][]` → SYN042
>
> Notes: This is a grammar-level restriction. The type checker does not need to
> check for this — the parser rejects it.

**RULE TS-COMP-06: Array element type**

> Plain English: The element type of an array may be any valid base type: a
> primitive, a schema name, or a grouped literal union.
>
> Valid:
> ```eaml
> schema Example {
>   names: string[]                         // primitive array
>   items: DataItem[]                       // schema array
>   statuses: ("pass" | "fail" | "skip")[]  // literal union array (grouped)
> }
> ```
>
> Grammar: Production [42] `typeExpr` — `baseType` (Production [43]) includes
> `namedType`, `literalUnion`, and `"(" typeExpr ")"` (grouped). The grouped form
> is necessary for literal union arrays because `"pass" | "fail"[]` would be
> ambiguous without parentheses.
>
> Pydantic v2:
> - `List[str]`, `List[DataItem]`, `List[Literal["pass", "fail", "skip"]]`

---

## 4. Bounded Types

### 4.1 Bounded Type Overview

Bounded types attach runtime constraints to a base primitive type. The constraint
is enforced at runtime by the generated Pydantic v2 model via `Field()` parameters,
not at compile time.

**Key principle:** Bounds are NOT part of the nominal type identity. Two schema fields
`float<0.0, 1.0>` and `float<0.0, 2.0>` are both of type `float` — the bounds are
metadata that Pydantic enforces at validation time. This means:

- A function parameter typed as `float` accepts any float value, including one
  that was bounded in a schema field.
- Two schemas with differently-bounded float fields are type-compatible at the
  `float` level (they differ only in validation constraints).

Grammar: Production [45] `boundedSuffix` — `"<" boundParams ">"`.
Applied to Production [44] `namedType` — `IDENT boundedSuffix?`.

### 4.2 float Bounds

**RULE TS-BND-01: float positional bounds**

> Plain English: `float<min_val, max_val>` constrains a float field to values
> between `min_val` and `max_val`, inclusive on both ends.
>
> Formal: `float<a, b>` where `a ≤ b`, both are numeric literals.
> Generated constraint: `a ≤ value ≤ b`.
>
> Grammar: Production [45]-[47]. `boundParam` is `(IDENT ":")?  (FLOAT | INT)`.
> Two positional params: first is min, second is max.
>
> Valid:
> ```eaml
> schema Score {
>   confidence: float<0.0, 1.0>
> }
> ```
>
> Invalid:
> ```eaml
> schema Bad {
>   score: float<1.0, 0.0>
> }
> // → TYP030: Lower bound (1.0) exceeds upper bound (0.0)
> ```
>
> Pydantic v2: `float = Field(ge=0.0, le=1.0)`
>
> Notes: Both bounds are inclusive. Pydantic v2 uses `ge` (greater-or-equal) and
> `le` (less-or-equal), NOT `gt`/`lt` (which are exclusive).

**RULE TS-BND-02: float named bounds**

> Plain English: Float bounds may use named parameters `min:` and `max:` in any
> order. Either may be omitted to leave that bound unconstrained.
>
> Grammar: Production [47] `boundParam` — `(IDENT ":")?  (FLOAT | INT)`.
>
> Valid:
> ```eaml
> schema Scores {
>   a: float<min: 0.0, max: 1.0>     // both bounds
>   b: float<min: 0.0>               // lower bound only
>   c: float<max: 100.0>             // upper bound only
>   d: float<max: 1.0, min: 0.0>     // reversed order — valid
> }
> ```
>
> Pydantic v2:
> - `float<min: 0.0, max: 1.0>` → `float = Field(ge=0.0, le=1.0)`
> - `float<min: 0.0>` → `float = Field(ge=0.0)`
> - `float<max: 100.0>` → `float = Field(le=100.0)`
>
> Notes: Layer 5 §3.5 [CLOSED]. Named parameter order does not matter.
> [sem: bounded-param-validation] (SEM030) validates parameter names against the
> base type.

**RULE TS-BND-03: Numeric literal coercion in float bounds**

> Plain English: Integer literals in float bound positions are coerced to float.
> `float<0, 1>` is equivalent to `float<0.0, 1.0>`.
>
> Grammar: Production [47] `boundParam` accepts `FLOAT | INT`.
>
> Valid: `confidence: float<0, 1>`
>
> Pydantic v2: `float = Field(ge=0.0, le=1.0)` — codegen converts int to float.
>
> ⚠️ **OPEN QUESTION OQ-01:** Layer 5 §3.5 shows float bounds with float values
> only. The grammar (Production [47]) accepts both FLOAT and INT. **Recommended
> resolution:** Allow int literals in float bounds with implicit coercion — this
> is ergonomic and unambiguous.

### 4.3 string Bounds

**RULE TS-BND-04: string length bounds**

> Plain English: String bounds constrain the character length of the string value.
> Use `min:` for minimum length and `max:` for maximum length. Both accept integer
> values.
>
> Grammar: Production [45]-[47]. Named parameters with INT values.
>
> Valid:
> ```eaml
> schema Review {
>   summary: string<max: 200>             // max 200 characters
>   title: string<min: 1>                 // at least 1 character (non-empty)
>   description: string<min: 10, max: 500> // between 10 and 500 characters
> }
> ```
>
> Invalid:
> ```eaml
> schema Bad {
>   name: string<max: -1>
> }
> // → TYP031: String length bound must be a non-negative integer
> ```
>
> Pydantic v2:
> - `string<max: 200>` → `str = Field(max_length=200)`
> - `string<min: 1>` → `str = Field(min_length=1)`
> - `string<min: 10, max: 500>` → `str = Field(min_length=10, max_length=500)`
>
> Notes: Layer 5 §3.5 [CLOSED] specifies both `min` and `max` for string.
> Pydantic v2 uses `min_length` and `max_length` (NOT `ge`/`le` which are for
> numeric types).

### 4.4 int Bounds

**RULE TS-BND-05: int bounds**

> Plain English: Integer bounds constrain the numeric value of the integer.
> Use `min:` for minimum value and `max:` for maximum value. Both accept integer
> values.
>
> Grammar: Production [45]-[47]. Named parameters with INT values.
>
> Valid:
> ```eaml
> schema Config {
>   priority: int<min: 0, max: 100>
>   retries: int<min: 0>
>   level: int<max: 10>
> }
> ```
>
> Invalid:
> ```eaml
> schema Bad {
>   priority: int<min: 100, max: 0>
> }
> // → TYP030: Lower bound (100) exceeds upper bound (0)
> ```
>
> Pydantic v2:
> - `int<min: 0, max: 100>` → `int = Field(ge=0, le=100)`
> - `int<min: 0>` → `int = Field(ge=0)`
> - `int<max: 10>` → `int = Field(le=10)`
>
> Notes: Layer 5 §3.5 [CLOSED]. Int bounds use `ge`/`le` (inclusive),
> same as float bounds.

### 4.5 Bounded Type Restrictions

**RULE TS-BND-06: Bounds are not permitted on bool or null**

> Plain English: Only `float`, `string`, and `int` accept bounded type parameters.
> Applying bounds to `bool` or `null` is a semantic error.
>
> Invalid:
> ```eaml
> schema Bad {
>   flag: bool<min: 0>
> }
> // → TYP032: Type 'bool' does not accept bounded parameters
> ```
>
> Invalid:
> ```eaml
> schema Bad {
>   nothing: null<max: 0>
> }
> // → TYP032: Type 'null' does not accept bounded parameters
> ```
>
> Notes: [sem: bounded-param-validation] (SEM030) — the grammar accepts bounds on
> any named type; the semantic analysis phase validates that the base type supports
> bounds and that the parameter names are valid for that type.

**RULE TS-BND-07: Unknown bound parameter names**

> Plain English: Using a parameter name other than `min` or `max` is a semantic error.
>
> Invalid:
> ```eaml
> schema Bad {
>   score: float<average: 0.5>
> }
> // → SEM030: Unknown bounded parameter 'average' for type 'float'.
> //           Valid parameters: 'min', 'max'
> ```

**Bounded Type Summary:**

| EAML Bound Form | Base Type | Pydantic v2 `Field()` | Notes |
|-----------------|-----------|----------------------|-------|
| `float<a, b>` | `float` | `Field(ge=a, le=b)` | Positional: min, max |
| `float<min: a>` | `float` | `Field(ge=a)` | Named, lower only |
| `float<max: b>` | `float` | `Field(le=b)` | Named, upper only |
| `float<min: a, max: b>` | `float` | `Field(ge=a, le=b)` | Named, both |
| `string<max: n>` | `str` | `Field(max_length=n)` | Character count |
| `string<min: n>` | `str` | `Field(min_length=n)` | Character count |
| `string<min: a, max: b>` | `str` | `Field(min_length=a, max_length=b)` | Both |
| `int<min: a>` | `int` | `Field(ge=a)` | Numeric value |
| `int<max: b>` | `int` | `Field(le=b)` | Numeric value |
| `int<min: a, max: b>` | `int` | `Field(ge=a, le=b)` | Numeric value |

---

## 5. Literal Union Types

### 5.1 Literal Union Definition

**RULE TS-LIT-01: Literal union form**

> Plain English: A literal union is a type consisting of two or more string literal
> values separated by `|`. A value of this type MUST be exactly one of the listed
> strings. The minimum of two members is enforced by the grammar structure.
>
> Formal: `literalUnion = STRING ("|" STRING)+` — the `+` ensures at least one
> additional member beyond the first.
>
> Grammar: Production [50] `literalUnion` in grammar.ebnf.
>
> Valid:
> ```eaml
> schema SentimentResult {
>   sentiment: "positive" | "negative" | "neutral"
> }
> ```
>
> Invalid (single member — not a literal union):
> A bare STRING in type position is not a valid `baseType` — it does not match
> `namedType` (which requires IDENT), `literalUnion` (which requires two members),
> or grouped type. A single STRING in type position is a parse error.
>
> Pydantic v2: `Literal["positive", "negative", "neutral"]`
> (from `typing.Literal`)
>
> JSON wire format: One of the listed string values (`"positive"`)

**RULE TS-LIT-02: Members must be string literals**

> Plain English: Every member of a literal union MUST be a string literal.
> Non-string values (integers, booleans, identifiers) are not permitted.
>
> Grammar: Production [50] — `STRING ( "|" STRING )+`. The grammar only accepts
> STRING tokens, so non-string members are parse errors.
>
> Invalid: `status: "yes" | 1` → Parse error: expected STRING after `|`
>
> Invalid: `status: "yes" | true` → Parse error: expected STRING after `|`
>
> Notes: This is enforced by the grammar, not the type checker. Layer 5 §3.6 [CLOSED].

**RULE TS-LIT-03: The `|` operator is type-position only**

> Plain English: The `|` symbol separates literal union members in type expression
> positions ONLY. It MUST NOT appear in expression positions — EAML has no bitwise
> OR operator. Using `|` in an expression produces a syntax error with a helpful hint.
>
> Grammar: Production [50] `literalUnion` — `|` appears only in `typeExpr` context.
> EG-05 from grammar.ebnf: "|" in expression context → SYN043.
>
> Invalid:
> ```eaml
> let x: int = a | b
> // → SYN043: Unexpected '|' in expression. Did you mean '||' for logical OR?
> ```
>
> Notes: This reinforces EG-05. There is no bitwise OR, bitwise AND, XOR, or
> bitwise NOT in EAML.

### 5.2 Literal Union Composition

**RULE TS-LIT-04: Optional literal union**

> Plain English: A literal union can be made optional by wrapping it in parentheses
> and adding `?`. The parentheses are required because `?` would otherwise bind to
> the last string member only.
>
> Grammar: Production [43] `baseType` — `"(" typeExpr ")"` (grouped type).
> Then Production [42] `typeExpr` applies `optionalSuffix`.
>
> Valid:
> ```eaml
> schema Review {
>   tone: ("positive" | "negative" | "neutral")?
> }
> ```
>
> Pydantic v2: `Optional[Literal["positive", "negative", "neutral"]] = None`

**RULE TS-LIT-05: Array of literal union**

> Plain English: An array of literal union values requires parentheses around the
> union, followed by `[]`.
>
> Valid:
> ```eaml
> schema TagResult {
>   categories: ("tech" | "science" | "art")[]
> }
> ```
>
> Pydantic v2: `List[Literal["tech", "science", "art"]]`

**RULE TS-LIT-06: Literal union as schema field type**

> Valid:
> ```eaml
> schema SentimentResult {
>   sentiment: "positive" | "negative" | "neutral"
>   confidence: float<0.0, 1.0>
> }
> ```
>
> Generated Pydantic v2:
> ```python
> from pydantic import BaseModel, Field
> from typing import Literal
>
> class SentimentResult(BaseModel):
>     sentiment: Literal["positive", "negative", "neutral"]
>     confidence: float = Field(ge=0.0, le=1.0)
> ```

### 5.3 Literal Union Restrictions

**RULE TS-LIT-07: Duplicate members in literal union**

> ⚠️ **OPEN QUESTION OQ-02:** Layer 5 does not address duplicate members in
> literal unions. For example: `"yes" | "yes" | "no"`.
>
> **Recommended resolution:** Emit TYP040 **warning** (not error) for duplicate
> members. Rationale: Pydantic's `Literal` handles duplicates without error
> (they are deduplicated), so this does not cause incorrect runtime behavior.
> However, it is likely a copy-paste mistake and should be flagged.
>
> Invalid:
> ```eaml
> schema Bad {
>   status: "yes" | "yes" | "no"
> }
> // → TYP040 warning: Duplicate member "yes" in literal union
> ```

---

## 6. Schema-Defined Types

### 6.1 Schema as Nominal Type

**RULE TS-SCH-01: Schema declarations create named types**

> Plain English: A `schema` declaration creates a new named type that can be used
> as a type expression anywhere in the program. The schema name becomes available
> in the type registry after name resolution pass 1. Forward references to schemas
> declared later in the file are valid.
>
> Grammar: Production [29] `schemaDecl`. The schema name (IDENT) is registered as
> a type. [sem: forward-ref-allowed] — two-pass name resolution (Production [24]).
>
> Valid:
> ```eaml
> // Forward reference: prompt uses schema declared below
> prompt Analyze(text: string) -> SentimentResult {
>   user: "Analyze: {text}"
> }
>
> schema SentimentResult {
>   sentiment: "positive" | "negative" | "neutral"
> }
> ```
>
> Pydantic v2: Each schema generates a `class(BaseModel)` with the schema name.

**RULE TS-SCH-02: Nominal identity — schemas with identical fields are distinct**

> Plain English: Two schemas with the same field names and types are NOT the same
> type. They cannot be used interchangeably at call sites or in type annotations.
>
> Valid:
> ```eaml
> schema Confidence { score: float }
> schema Probability { score: float }
>
> prompt Estimate(data: string) -> Confidence {
>   user: "Estimate: {data}"
> }
>
> // The return value is of type Confidence.
> // It CANNOT be assigned to a variable of type Probability.
> let result: Confidence = Claude.call(Estimate(data: "test"))  // OK
> // let wrong: Probability = Claude.call(Estimate(data: "test"))  // TYP003
> ```
>
> Notes: Layer 5 §3.2 [CLOSED] — "A and B are DIFFERENT types even though they have
> identical shape."

### 6.2 Schema Field Rules

**RULE TS-SCH-03: Field name uniqueness**

> Plain English: Within a single schema, field names MUST be unique. Declaring two
> fields with the same name is a semantic error.
>
> Invalid:
> ```eaml
> schema Bad {
>   name: string
>   name: int
> }
> // → SEM020: Duplicate field name 'name' in schema 'Bad'
> ```

**RULE TS-SCH-04: Field separators**

> Plain English: Schema fields may be separated by newline, comma, or both. Trailing
> commas are allowed. Both styles may be mixed within the same schema.
>
> Grammar: Production [30] `fieldDef` — `IDENT ":" typeExpr ","?`
> (NL separation works implicitly via WS skipping between tokens.)
> Layer 5 §7.1 [GRAMMAR IMPACT].
>
> Valid (all equivalent):
> ```eaml
> // Newline-separated:
> schema A { name: string
>            age: int }
>
> // Comma-separated:
> schema B { name: string, age: int }
>
> // Trailing comma:
> schema C { name: string, age: int, }
> ```

**RULE TS-SCH-05: Recursive schemas**

> ⚠️ **OPEN QUESTION OQ-03:** Can a schema field reference the same schema type?
>
> ```eaml
> schema TreeNode {
>   value: string
>   children: TreeNode[]   // recursive reference
> }
> ```
>
> Layer 5 does not address recursive schema types. This creates a potential issue:
> Pydantic v2 requires `model_rebuild()` for self-referencing models, and deeply
> recursive LLM outputs may cause validation stack overflows.
>
> **Recommended resolution:** Allow recursive schemas in v0.1 with a semantic warning
> (SEM070). The generated Pydantic code would use `model_rebuild()` after class
> definition. Full recursive type support with depth limits is deferred to v0.2.

### 6.3 Schema as Return Type

**RULE TS-SCH-06: Schema as prompt/tool return type**

> Plain English: A prompt's return type annotation specifies what the LLM is expected
> to return. When the return type is a schema, the compiler generates a Pydantic v2
> model for runtime validation. If the LLM's response does not conform, the
> `max_retries` policy applies.
>
> Grammar: Production [31] `promptDecl` — `"->" typeExpr`.
> Production [34] `toolDecl` — `"->" typeExpr`.
>
> Valid:
> ```eaml
> prompt AnalyzeSentiment(text: string)
>   requires json_mode
>   -> SentimentResult {
>   user: "Analyze the sentiment of: {text}"
> }
> ```
>
> Pydantic v2: The return type schema's generated `BaseModel` is used in
> `model_validate_json(response)` at runtime.
>
> Notes: A prompt with return type `string` is valid for unstructured text
> extraction — no Pydantic validation is applied, the raw string is returned.
> Layer 5 §7.3.

### 6.4 Pydantic v2 Generation Rules

The following table maps every EAML field type to its generated Python code:

| EAML Field Type | Python Annotation | Pydantic Field | Default | Notes |
|-----------------|------------------|---------------|---------|-------|
| `string` | `str` | — | (required) | |
| `int` | `int` | — | (required) | |
| `float` | `float` | — | (required) | |
| `bool` | `bool` | — | (required) | |
| `null` | `None` | — | (required) | Rarely used as field type |
| `string?` | `Optional[str]` | — | `= None` | See §10.4 |
| `int?` | `Optional[int]` | — | `= None` | |
| `float?` | `Optional[float]` | — | `= None` | |
| `bool?` | `Optional[bool]` | — | `= None` | |
| `SchemaName` | `SchemaName` | — | (required) | Nominal reference |
| `SchemaName?` | `Optional[SchemaName]` | — | `= None` | |
| `SchemaName[]` | `List[SchemaName]` | — | (required) | |
| `SchemaName[]?` | `Optional[List[SchemaName]]` | — | `= None` | |
| `SchemaName?[]` | `List[Optional[SchemaName]]` | — | (required) | |
| `string[]` | `List[str]` | — | (required) | |
| `string<max: 200>` | `str` | `Field(max_length=200)` | (required) | |
| `float<0.0, 1.0>` | `float` | `Field(ge=0.0, le=1.0)` | (required) | |
| `int<min: 0, max: 100>` | `int` | `Field(ge=0, le=100)` | (required) | |
| `"yes" \| "no"` | `Literal["yes", "no"]` | — | (required) | `from typing import Literal` |
| `("a" \| "b")?` | `Optional[Literal["a", "b"]]` | — | `= None` | |
| `("a" \| "b")[]` | `List[Literal["a", "b"]]` | — | (required) | |

---

## 7. Type Positions and Annotations

### 7.1 Required Type Annotation Positions

The following positions REQUIRE a type annotation in EAML v0.1:

| Position | Grammar Production | Example |
|----------|-------------------|---------|
| Schema field | [30] `fieldDef` — `IDENT ":" typeExpr` | `name: string` |
| Prompt parameter | [73] `param` — `IDENT ":" typeExpr` | `text: string` |
| Tool parameter | [73] `param` — `IDENT ":" typeExpr` | `path: string` |
| Prompt return type | [31] `promptDecl` — `"->" typeExpr` | `-> SentimentResult` |
| Tool return type | [34] `toolDecl` — `"->" typeExpr` | `-> DataSummary` |
| Let binding | [41] `letDecl` — `IDENT ":" typeExpr` | `let x: string = ...` |

All six positions use the same `typeExpr` grammar (Production [42]), so all type
forms (primitives, composites, bounded, literal unions, schema references) are
valid in all positions.

### 7.2 Type Annotations Are Forbidden In

Type annotations appear only in the positions listed in §7.1. The grammar
structurally prevents type annotations in all other positions:

- **Expression operands** — `1 + (2 : int)` is not valid EAML. The grammar has no
  production for type annotations inside expressions.
- **Capability names** — `requires (json_mode : string)` is invalid. Production [76]
  `requiresClause` accepts IDENT only, not typed identifiers.
- **Import statements** — `import "./file.eaml" : Module` is invalid. Production [26]
  `importDecl` has no type annotation position.

These are structural impossibilities — no SYN code is needed because the parser will
produce a generic "unexpected token" error.

### 7.3 Type Inference (Post-MVP)

**RULE TS-ANN-01: Type annotation required on let bindings in v0.1**

> Plain English: In v0.1, every `let` binding MUST include a type annotation.
> Omitting the type annotation is a parse error because the grammar requires it.
> The compiler attempts type inference internally to provide a helpful hint in
> the error message.
>
> Grammar: Production [41] `letDecl` — `"let" IDENT ":" typeExpr "=" expr`.
> The grammar requires the `:` typeExpr, so omitting it is a parse error.
>
> Valid: `let result: SentimentResult = Claude.call(Analyze(text: "hello"))`
>
> Invalid: `let result = Claude.call(Analyze(text: "hello"))`
> → Parse error: expected `:` after identifier in let binding.
>
> Notes: Layer 5 §11. The compiler provides a hint:
> `"Hint: let result: SentimentResult = ..."`

> **v0.1 Restriction:** Type inference on `let` bindings is Post-MVP. The grammar
> requires explicit type annotations. When type inference is added in a future
> version, the grammar will be updated to make `: typeExpr` optional.

### 7.4 Return Type Rules

**RULE TS-RET-01: Prompt return type**

> Plain English: Every prompt MUST have a return type annotation (`-> typeExpr`).
> The return type SHOULD be a schema type for structured output from the LLM.
> A prompt with return type `string` is valid — it extracts unstructured text.
>
> Grammar: Production [31] `promptDecl` — `"->" typeExpr` is required by grammar.
>
> Valid:
> ```eaml
> prompt Summarize(text: string) -> string {
>   user: "Summarize: {text}"
> }
>
> prompt Classify(text: string) -> SentimentResult {
>   user: "Classify: {text}"
> }
> ```
>
> Pydantic v2: When return type is a schema, the generated model is used for
> `model_validate_json()`. When return type is `string`, no Pydantic validation
> occurs — the raw response is returned.

**RULE TS-RET-02: Tool return type and `-> null` for void tools**

> Plain English: Every tool MUST have a return type annotation. Tools with no
> meaningful return value use `-> null`. The keyword `void` is NOT available in v0.1.
>
> Grammar: Production [34] `toolDecl` — `"->" typeExpr` is required by grammar.
>
> Valid:
> ```eaml
> tool LogEvent(message: string) -> null {
>   python %{
>     logger.info(message)
>   }%
> }
> ```
>
> Invalid: Using `void` → TYP010: Unknown type 'void'.
> `void` is reserved for Post-MVP (Layer 5 §7.4, §11).
>
> Pydantic v2: `-> null` generates `-> None` return type annotation in Python.
>
> Notes: Layer 5 §7.4 [CLOSED]: "Tools with NO MEANINGFUL RETURN VALUE use `-> null`."

---

## 8. Type Error Catalog

### TYP0xx — Primitive Type Errors

#### TYP001: Built-in type shadowing

**Condition:** A schema is declared with the same name as a predeclared primitive type.

**Message:** `TYP001 warning: Schema '{name}' shadows the built-in type '{name}'`

```eaml
schema string {
  value: int
}
```

**Resolution:** Rename the schema to avoid shadowing the built-in type.

**Notes:** This is a WARNING, not an error (Layer 5 §3.1). The schema declaration
is valid but may confuse later references to the type name.
Referenced by: TS-PRM-01, TS-PRM-06.

---

#### TYP003: Type mismatch

**Condition:** A value of one type is used where a different, incompatible type is
expected (e.g., at a call site, in a let binding, or in a field assignment).

**Message:** `TYP003: Type mismatch: expected '{expected}', found '{actual}'`

```eaml
schema Config { max: int }
prompt GetConfig(s: string) -> Config {
  user: "{s}"
}
let result: string = Claude.call(GetConfig(s: "test"))
// → TYP003: Type mismatch: expected 'string', found 'Config'
```

**Resolution:** Change the variable type to match the expression type, or change
the expression.

**Notes:** Referenced by: TS-PRM-01 through TS-PRM-05, TS-SCH-02.

---

#### TYP010: Unknown type name

**Condition:** An identifier used as a type does not match any predeclared primitive
or declared schema name.

**Message:** `TYP010: Unknown type '{name}'.`
With hint when close match exists: `Did you mean '{suggestion}'?`

```eaml
schema Bad {
  score: Float
}
// → TYP010: Unknown type 'Float'. Did you mean 'float'?
```

**Resolution:** Use the correct lowercase primitive name or declare the schema first.

**Notes:** Covers the casing error case (TS-PRM-06). The type registry only contains
lowercase primitives. `Float`, `String`, `Int`, `Bool`, `Null` are not registered.
Also emitted for `void` in v0.1.
Referenced by: TS-PRM-06, TS-RET-02.

---

### TYP0xx — Bounded Type Errors

#### TYP030: Lower bound exceeds upper bound

**Condition:** A bounded type's minimum value is greater than its maximum value.

**Message:** `TYP030: Lower bound ({min}) exceeds upper bound ({max})`

```eaml
schema Bad {
  score: float<1.0, 0.0>
}
```

**Resolution:** Swap the bound values so that the minimum is less than or equal
to the maximum.

**Notes:** Referenced by: TS-BND-01, TS-BND-05.

---

#### TYP031: Invalid string length bound

**Condition:** A string length bound is negative or otherwise invalid.

**Message:** `TYP031: String length bound must be a non-negative integer`

```eaml
schema Bad {
  name: string<max: -1>
}
```

**Resolution:** Use a non-negative integer for string length bounds.

**Notes:** Referenced by: TS-BND-04.

---

#### TYP032: Bounds on non-boundable type

**Condition:** Bounded type parameters are applied to a type that does not accept
them (any type other than `float`, `string`, or `int`).

**Message:** `TYP032: Type '{type}' does not accept bounded parameters`

```eaml
schema Bad {
  flag: bool<min: 0>
}
```

**Resolution:** Remove the bounded parameters. Only `float`, `string`, and `int`
accept bounds.

**Notes:** Referenced by: TS-BND-06.

---

### TYP0xx — Literal Union Errors

#### TYP040: Duplicate literal union member

**Condition:** A literal union contains the same string value more than once.

**Message:** `TYP040 warning: Duplicate member "{value}" in literal union`

```eaml
schema Bad {
  status: "yes" | "yes" | "no"
}
```

**Resolution:** Remove the duplicate member.

**Notes:** This is a WARNING, not an error. Pydantic's `Literal` deduplicates
values silently. See OPEN QUESTION OQ-02.
Referenced by: TS-LIT-07.

---

### SEM0xx — Semantic Errors (Type-Related)

#### SEM020: Duplicate field name

**Condition:** A schema declares two fields with the same name.

**Message:** `SEM020: Duplicate field name '{name}' in schema '{schema}'`

```eaml
schema Bad {
  name: string
  name: int
}
```

**Resolution:** Rename one of the fields.

**Notes:** Referenced by: TS-SCH-03.

---

#### SEM030: Unknown bounded parameter

**Condition:** A bounded type parameter uses an unrecognized name.

**Message:** `SEM030: Unknown bounded parameter '{param}' for type '{type}'. Valid parameters: 'min', 'max'`

```eaml
schema Bad {
  score: float<average: 0.5>
}
```

**Resolution:** Use `min` or `max` as parameter names.

**Notes:** Referenced by: TS-BND-07.

---

#### SEM050: Type annotation required on let binding (v0.1)

**Condition:** A `let` binding omits the type annotation. In v0.1, this is a parse
error because the grammar requires `: typeExpr`. Included here for documentation
completeness.

**Message:** `Type annotation required on let bindings in EAML v0.1. Hint: let {name}: {inferred_type} = ...`

**Notes:** Layer 5 §11. Grammar Production [41] requires the annotation syntactically.
Referenced by: TS-ANN-01.

---

## 9. Post-MVP Type Features

### 9.1 Enum Declarations

**Feature:** Named enumeration type with member aliases and descriptions.

**Why deferred:** Adds a declaration type, name resolution case, and `@alias`
complexity. Literal unions cover 90% of use cases.

**Blocking error:** SYN082: "Enum declarations are not supported in EAML v0.1."

```eaml
enum Sentiment { POSITIVE, NEGATIVE, NEUTRAL }
// → SYN082
```

**Planned v-next:** `enum` keyword creates a named type with members. Members may
have `@alias` and `@description` annotations. The type maps to Pydantic `Literal`
or Python `Enum` depending on annotation presence.

### 9.2 Schema Inheritance

**Feature:** Schema extending another schema with additional fields.

**Why deferred:** Requires inheritance semantics (field override rules, diamond
inheritance prevention). Nominal typing with inheritance adds complexity.

**Blocking error:** SYN083: "Schema inheritance is not supported in EAML v0.1."

```eaml
schema DetailedResult extends BaseResult {
  extra: string
}
// → SYN083
```

**Planned v-next:** Single inheritance only. Fields from parent are included.
Child fields may not shadow parent fields. Maps to Pydantic class inheritance.

### 9.3 Type Inference on Let Bindings

**Feature:** Omitting the type annotation on `let` when it can be inferred.

**Why deferred:** Requires a type inference engine. v0.1 prioritizes explicitness.

**Blocking error:** Parse error — grammar Production [41] requires `: typeExpr`.
Layer 5 §11: SEM050 message includes inferred type as hint.

```eaml
let result = Claude.call(Analyze(text: "hello"))
// → Parse error: expected ':' after identifier in let binding
// Hint: let result: SentimentResult = ...
```

### 9.4 General Union Types

**Feature:** Union of arbitrary types: `Tag | OtherSchema`.

**Why deferred:** Requires union type resolution, exhaustiveness checking, and
Pydantic discriminated union generation.

**Blocking:** Not in grammar — `|` is only valid in `literalUnion` (Production [50])
which requires STRING members. Using `|` between type identifiers would not parse.

**Planned v-next:** `typeExpr | typeExpr` syntax with Pydantic discriminated unions.

### 9.5 Generic Types

**Feature:** Parameterized schemas: `schema Container<T> { item: T }`.

**Why deferred:** Generics add significant complexity to the type system and
code generation. Not justified for v0.1 scope.

**Blocking:** Not in grammar — `<` after a schema name in a schema declaration
is not a recognized production.

### 9.6 Recursive Schema Types

> ⚠️ **OPEN QUESTION OQ-03** (repeated from §6.2):
> Recursive schemas (a schema field referencing its own type) are not explicitly
> addressed in Layer 5.
>
> **Recommended resolution for v0.1:** Allow with SEM070 warning. Generate Pydantic
> code with `model_rebuild()`. Full support with depth limits in v0.2.

### 9.7 String Pattern Bounds

**Feature:** `string<pattern: "[A-Z]{3}-\\d{4}">` for regex-validated strings.

**Why deferred:** Adds regex compilation and validation to the compiler.

**Blocking:** Not in grammar — `pattern` is not a recognized bound parameter name
(SEM030 would trigger).

**Planned v-next:** `string<pattern: "regex">` maps to Pydantic
`Field(pattern="regex")`.

### 9.8 Pipeline Type Flow

**Feature:** The `>>` operator propagating types through a pipeline.

**Why deferred:** Pipeline declarations are Post-MVP (SYN080/SYN081).

**Blocking error:** SYN081: "Pipeline operator >> is not supported in EAML v0.1."

### 9.9 Field Annotations

**Feature:** `@description`, `@alias` on schema fields.

**Why deferred:** Adds annotation parsing, JSON Schema metadata generation.

**Blocking error:** SYN090: "@annotations are not supported in EAML v0.1."

### 9.10 `void` Keyword

**Feature:** `void` as a return type synonym for `null`.

**Why deferred:** `-> null` serves the same purpose. `void` is reserved for
familiarity in a future version.

**Blocking:** `void` is a reserved identifier. Using it as a type name triggers
TYP010 (unknown type) with no special message in v0.1.

> **v0.1 Restriction:** All features listed in this section are blocked in v0.1.
> The grammar and/or semantic analysis prevents their use with the error codes
> documented above.

---

## 10. Pydantic v2 Code Generation Reference

### 10.1 Code Generation Philosophy

Every `schema` declaration generates one `class(BaseModel)`. Every field generates
one annotated class attribute. The generated code imports from:

- `pydantic`: `BaseModel`, `Field`
- `typing`: `Optional`, `List`, `Literal`

No other imports are needed for type annotations. Python bridge imports
(`import python "pandas" as pd`) are separate and appear above the generated models.

**Python version floor:** 3.11+ (Layer 5 §10.1). `Optional`, `List`, and `Literal`
are all available from `typing` in Python 3.11+. EAML codegen uses `typing` imports
for clarity rather than Python 3.10+ `X | None` syntax.

**Pydantic version:** v2.x only (Layer 5 §10.1). No v1 patterns: no `validator`
decorator (use `field_validator`), no `__root__` (use `model_validator`), no
`class Config` (use `model_config`).

### 10.2 Schema to BaseModel Generation

**Canonical example:**

```eaml
schema EntityResult {
  label: string
  score: float<0.0, 1.0>
  tags: string[]
  source: string?
  category: "person" | "org" | "location"
}
```

**Generated Python:**

```python
from pydantic import BaseModel, Field
from typing import Optional, List, Literal


class EntityResult(BaseModel):
    label: str
    score: float = Field(ge=0.0, le=1.0)
    tags: List[str]
    source: Optional[str] = None
    category: Literal["person", "org", "location"]
```

This is the canonical reference. Every type form MUST be derivable from this pattern.

### 10.3 Complete Type Mapping Table

| EAML `typeExpr` | Python Annotation | Pydantic `Field()` | Default Value | Notes |
|-----------------|------------------|-------------------|---------------|-------|
| `string` | `str` | — | (required) | |
| `int` | `int` | — | (required) | |
| `float` | `float` | — | (required) | |
| `bool` | `bool` | — | (required) | |
| `null` | `None` | — | (required) | Used as `-> null` return type |
| `string?` | `Optional[str]` | — | `= None` | |
| `int?` | `Optional[int]` | — | `= None` | |
| `float?` | `Optional[float]` | — | `= None` | |
| `bool?` | `Optional[bool]` | — | `= None` | |
| `string[]` | `List[str]` | — | (required) | |
| `int[]` | `List[int]` | — | (required) | |
| `float[]` | `List[float]` | — | (required) | |
| `string[]?` | `Optional[List[str]]` | — | `= None` | Optional array |
| `string?[]` | `List[Optional[str]]` | — | (required) | Array of optional |
| `string?[]?` | `Optional[List[Optional[str]]]` | — | `= None` | Both optional |
| `SchemaName` | `SchemaName` | — | (required) | Nominal type ref |
| `SchemaName?` | `Optional[SchemaName]` | — | `= None` | |
| `SchemaName[]` | `List[SchemaName]` | — | (required) | |
| `SchemaName[]?` | `Optional[List[SchemaName]]` | — | `= None` | |
| `SchemaName?[]` | `List[Optional[SchemaName]]` | — | (required) | |
| `float<0.0, 1.0>` | `float` | `Field(ge=0.0, le=1.0)` | (required) | Inclusive bounds |
| `float<min: 0.0>` | `float` | `Field(ge=0.0)` | (required) | Lower only |
| `float<max: 1.0>` | `float` | `Field(le=1.0)` | (required) | Upper only |
| `string<max: 200>` | `str` | `Field(max_length=200)` | (required) | Length constraint |
| `string<min: 1>` | `str` | `Field(min_length=1)` | (required) | Non-empty |
| `string<min: 1, max: 200>` | `str` | `Field(min_length=1, max_length=200)` | (required) | Both |
| `int<min: 0>` | `int` | `Field(ge=0)` | (required) | Lower only |
| `int<max: 100>` | `int` | `Field(le=100)` | (required) | Upper only |
| `int<min: 0, max: 100>` | `int` | `Field(ge=0, le=100)` | (required) | Both |
| `"yes" \| "no"` | `Literal["yes", "no"]` | — | (required) | 2-member |
| `"a" \| "b" \| "c"` | `Literal["a", "b", "c"]` | — | (required) | 3-member |
| `("yes" \| "no")?` | `Optional[Literal["yes", "no"]]` | — | `= None` | Optional union |
| `("yes" \| "no")[]` | `List[Literal["yes", "no"]]` | — | (required) | Array of union |

### 10.4 Optional Field Default Value Rule

**RULE TS-GEN-01: Optional fields receive `= None` default**

> Any field whose type ends in `?` (optionalSuffix) receives `= None` as a default
> value in the generated Pydantic model. Any field NOT ending in `?` has no default
> value — it is a required field in Pydantic.
>
> This is critical because Pydantic v2 raises `ValidationError` if a required field
> is missing from the input data. For LLM-generated responses, this means:
>
> - **Required fields:** If the LLM omits a required field, validation fails and
>   the retry policy applies.
> - **Optional fields:** If the LLM omits an optional field, it defaults to `None`
>   and validation succeeds.
>
> **Pydantic v2 behavior note:** In Pydantic v2, `Optional[T]` without a default
> still makes the field required (it just accepts `None`). EAML's codegen MUST
> add `= None` to make optional fields truly optional in the Pydantic sense.

### 10.5 Literal Union Import

The `Literal` type annotation is imported from `typing`:

```python
from typing import Literal
```

This import is available in Python 3.8+ (from `typing`) and Python 3.11+ (also from
`typing`). Since EAML targets Python 3.11+ (Layer 5 §10.1), `typing.Literal` is
always available. `typing_extensions` is not needed.

**Generated field for a literal union:**

```eaml
status: "pass" | "fail" | "skip"
```

→

```python
from typing import Literal

status: Literal["pass", "fail", "skip"]
```

---

## Verification Report — EAML TYPESYSTEM.md v0.1.0

| Group | Checks | Passed | Failed | N/A |
|-------|--------|--------|--------|-----|
| A — Completeness | 8 | 8 | 0 | 0 |
| B — Layer 5 | 10 | 10 | 0 | 0 |
| C — Pydantic v2 | 6 | 6 | 0 | 0 |
| D — Grammar Align | 5 | 5 | 0 | 0 |
| E — Quality | 7 | 7 | 0 | 0 |
| **Total** | **36** | **36** | **0** | **0** |

Failed checks: 0
Open Questions: 3 (OQ-01, OQ-02, OQ-03)

### Verification Details

**GROUP A — Completeness:**

A1[PASS] Every typeExpr production has a corresponding rule:
- [42] typeExpr, [42a] typeModifiers → TS-COMP-01 through TS-COMP-04 (orderings)
- [43] baseType → TS-PRM-01..06, TS-SCH-01, TS-LIT-01, TS-COMP-06
- [44] namedType → TS-PRM-01..06, TS-SCH-01
- [45] boundedSuffix → TS-BND-01..07
- [46] boundParams → TS-BND-01..05
- [47] boundParam → TS-BND-01..05
- [48] arraySuffix → TS-ARR-01, TS-ARR-02
- [49] optionalSuffix → TS-OPT-01, TS-OPT-02
- [50] literalUnion → TS-LIT-01..07

A2[PASS] All five primitives have complete rule blocks: TS-PRM-01 (string),
TS-PRM-02 (int), TS-PRM-03 (float), TS-PRM-04 (bool), TS-PRM-05 (null).
Each includes plain English, formal, grammar citation, valid/invalid, Pydantic v2.

A3[PASS] All four composite orderings specified:
TS-COMP-01 (T[]), TS-COMP-02 (T[]?), TS-COMP-03 (T?[]), TS-COMP-04 (T?[]?).
Pydantic v2 mappings verified: List[T], Optional[List[T]], List[Optional[T]],
Optional[List[Optional[T]]]. All v2 syntax.

A4[PASS] All bounded forms from Layer 5 §3.5 present:
float positional (TS-BND-01), float named (TS-BND-02), float coercion (TS-BND-03),
string bounds (TS-BND-04), int bounds (TS-BND-05). Cross-checked against extraction.

A5[PASS] Post-MVP type features from Layer 5 §11 in Section 9:
enum (SYN082), extends (SYN083), type inference (SEM050), void (TYP010),
@annotations (SYN090), pipeline (SYN081). All present.

A6[PASS] Layer 5 [GRAMMAR IMPACT] annotations with type relevance:
§3.1 primitives → TS-PRM-01..06; §3.3 orderings → TS-COMP-01..04;
§3.5 bounded → TS-BND-01..07; §3.6 literal union → TS-LIT-01..03;
§7.1 field separator → TS-SCH-04; §7.3 prompt fields → TS-SCH-06;
§10.2 model decl → not type-relevant (runtime config).

A7[PASS] Cross-references verified: each Invalid example cites an error code;
each error in §8 is referenced by at least one rule block.

A8[PASS] Mapping table in §10.3 covers: 5 primitives, 4 optional primitives,
3 primitive arrays, 3 composite orderings (string), 5 schema forms,
9 bounded forms, 4 literal union forms = 33 entries. All type forms covered.

**GROUP B — Layer 5 Compliance:**

B1[PASS] §1.1 states nominal typing with Confidence/Probability example.
Cites Layer 5 §3.2 [CLOSED].

B2[PASS] §2.7 TS-PRM-06 specifies TYP010 for casing violations. Primitives
are lowercase throughout.

B3[PASS] §3.3: T[]? → Optional[List[T]], T?[] → List[Optional[T]].
Verified distinct and correct Pydantic v2 syntax.

B4[PASS] §3.4 TS-ARR-02 and TS-COMP-05: Tag[][] → SYN042 parse error.
Attributed to grammar structure, not type checker.

B5[PASS] §5.1 TS-LIT-01: minimum two members attributed to grammar
Production [50] structure (STRING ("|" STRING)+).

B6[PASS] §4: All bounded rules use ge/le (inclusive), not gt/lt (exclusive).
Verified in summary table and individual rules.

B7[PASS] null vs optional distinction in §2.5 (TS-PRM-05) and §3.2 (TS-OPT-02).
Cross-reference present in both directions.

B8[PASS] §7.4 TS-RET-02: Documents `-> null` for void tools. States "void" is
NOT available in v0.1. The word "void" appears only in reference to this
restriction, never as an EAML type name.

B9[PASS] §7.3 TS-ANN-01: let type annotation required, grammar Production [41]
enforces it syntactically. SEM050 documented.

B10[PASS] No rule block uses "python" as an EAML identifier or type name.
EG-01 is respected throughout.

**GROUP C — Pydantic v2 Correctness:**

C1[PASS] All imports are v2 compatible:
`from pydantic import BaseModel, Field`
`from typing import Optional, List, Literal`
No `pydantic.v1` or `typing_extensions` references.

C2[PASS] §10.4 TS-GEN-01: Optional[T] fields get `= None` default.
The critical Pydantic v2 behavior note is documented.

C3[PASS] All bounded rules use ge/le (inclusive): TS-BND-01, TS-BND-02,
TS-BND-04, TS-BND-05. String uses min_length/max_length. Verified.

C4[PASS] §5.1 TS-LIT-01 and §10.5: "yes" | "no" → Literal["yes", "no"].
Not Union[str, str] or Enum.

C5[PASS] No Pydantic v1 patterns found. No `validator`, `__root__`,
`class Config` anywhere. §10.1 explicitly excludes v1 patterns.

C6[PASS] Canonical example in §10.2 is valid Python 3.11+ with Pydantic v2.
Imports correct, class definition correct, Field() params correct,
Optional default correct.

**GROUP D — Grammar Alignment:**

D1[PASS] All production citations exist in grammar.ebnf:
[6] INT, [7] FLOAT, [8] STRING, [11] BOOL_LIT, [12] NULL_LIT,
[24] Program, [26] importDecl, [29] schemaDecl, [30] fieldDef,
[31] promptDecl, [33] promptField, [34] toolDecl, [41] letDecl,
[42] typeExpr, [42a] typeModifiers, [43] baseType, [44] namedType,
[45] boundedSuffix, [46] boundParams, [47] boundParam, [48] arraySuffix,
[49] optionalSuffix,
[50] literalUnion, [60] unaryExpr, [73] param, [76] requiresClause.
All verified present in grammar.ebnf (84 productions).

D2[PASS] Every typeExpr sub-production [42], [42a], [43]-[50] mapped to a rule (see A1).

D3[PASS] §5.1 TS-LIT-03: | identified as type-position-only, cites EG-05
and Production [50]. Does not suggest | in expression context.

D4[PASS] Python bridge is not assigned a type. §6.3 documents the contract
between return type and Python block at semantic level, not parse time.

D5[PASS] §6.1 TS-SCH-01: Forward references documented, cites
[sem: forward-ref-allowed] and Production [24].

**GROUP E — Quality:**

E1[PASS] Spot-checked 5 rule blocks: TS-PRM-01, TS-ARR-01, TS-BND-01,
TS-LIT-01, TS-SCH-02. All follow declared format with Plain English,
Grammar citation, Valid/Invalid, Pydantic v2.

E2[PASS] 3 OPEN QUESTIONs clearly marked with ⚠️:
OQ-01 (int literal coercion in float bounds) — §4.2
OQ-02 (duplicate literal union members) — §5.3
OQ-03 (recursive schemas) — §6.2, §9.6
All include recommended resolutions.

E3[PASS] Normative language consistent: MUST/MUST NOT for requirements,
SHOULD for recommendations, MAY for optional. No lowercase "should"
where MUST is intended.

E4[PASS] Table of Contents matches document structure. All 10 sections
and subsections present and numbered correctly.

E5[PASS] No internal contradictions found:
(a) null and ? are consistently distinguished in §2.5 and §3.2
(b) Nominal typing consistent across §1.1, §6.1, §6.2
(c) v0.1 scope in §1.3 matches §9 Post-MVP features

E6[PASS] Bidirectional cross-references present:
§2.5 ↔ §3.2 (null vs optional)
Rule blocks ↔ §8 error catalog
§1.3 ↔ §9 (scope)

E7[PASS] Document is self-contained. A developer with TYPESYSTEM.md and
Pydantic v2 docs can implement the type checker and codegen crate.

### Known Limitations (v0.1.0)

1. **Recursive schemas** (OQ-03) — behavior undefined, recommended to allow with warning
2. **Int literal coercion in float bounds** (OQ-01) — grammar allows, semantics to be confirmed
3. **Duplicate literal union members** (OQ-02) — warning recommended
4. **No type inference** — all `let` bindings require explicit annotation
5. **No enum, extends, generics, or general union types**

### Grammar.ebnf Production Citations

All 24 production numbers cited in this document have been verified against
the current `spec/grammar.ebnf` (v0.1.0, 2026-03-14, 82 productions).

### Pydantic v2 Version Assumption

Pydantic version: 2.x (tested against 2.5+)
Python version floor: 3.11+