# EAML Grammar Prior Art Reference — Layer 3
## DSL Grammar Precedents: Lox (Crafting Interpreters) + BAML

---

## Document Purpose and Usage Instructions

This document is **Layer 3** of the EAML grammar reference stack. It covers
two domain-specific prior art grammars — one general-purpose scripting language
and one production LLM DSL — that solve problems structurally identical to EAML.

**Two sources, two different lessons:**

| Source                                              | Primary EAML Lesson                                                                                                                                           | Format                                                   |
|-----------------------------------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------|----------------------------------------------------------|
| **Lox** — *Crafting Interpreters* by Robert Nystrom | How to write a clean, complete, readable compiled-language grammar with zero ambiguity — the **gold standard for pedagogical grammar clarity**                | BNF-adjacent metasyntax (not W3C EBNF — see Section 1.2) |
| **BAML** — BoundaryML's LLM DSL                     | How the closest existing language to EAML solved declaration structure, type system, template strings, and client configuration — **direct domain prior art** | Pest PEG grammar (not W3C EBNF — see Section 2.2)        |

**Critical prerequisite:** Layer 1 (`eaml-layer1-notation-reference.md`) and
Layer 2 (`eaml-layer2-grammar-patterns.md`) must be loaded before this document.
The notation used in EAML grammar productions is defined in Layer 1 only.

**How to use this document:**
- Extract patterns and design decisions, not syntax.
- Neither Lox nor BAML uses W3C EBNF. Do not copy their rule notation into EAML.
- When you see a Lox or BAML pattern you want to apply, translate it into
  W3C EBNF using the operators defined in Layer 1.
- Section 5 provides explicit translation mappings for the most important patterns.

---

## Section 1 — Lox Grammar (Crafting Interpreters, Appendix I)

### 1.1 Source Attribution

**Author:** Robert Nystrom
**Book:** *Crafting Interpreters* (2021)
**URL:** https://craftinginterpreters.com/appendix-i.html
**License:** Creative Commons Attribution 4.0 — free to reference

Lox is a dynamically-typed scripting language built across the entire book.
It is the most clearly documented, most readable complete grammar of a
real programming language available anywhere online. Its grammar demonstrates
exactly the patterns EAML needs: declaration dispatch, statement hierarchy,
expression precedence, function signatures, and method chaining.

### 1.2 Notation Warning — Lox Uses Its Own Metasyntax

Lox grammar uses Nystrom's own BNF-adjacent notation, **not** W3C EBNF.
The differences are significant:

| Lox Notation       | W3C EBNF Equivalent             | Meaning                             |
|--------------------|---------------------------------|-------------------------------------|
| `→`                | `::=`                           | Production definition               |
| `;` at end of rule | Nothing (W3C has no terminator) | End of production                   |
| `rule*`            | `rule*`                         | Zero or more (same)                 |
| `rule+`            | `rule+`                         | One or more (same)                  |
| `rule?`            | `rule?`                         | Optional (same)                     |
| `( a \| b )`       | `( a \| b )`                    | Grouping + alternation (same)       |
| `"text"`           | `"text"`                        | String literal (same)               |
| `UPPERCASE`        | `ALL_CAPS`                      | Terminal token (same convention)    |
| `lowercase`        | `camelCase` / `PascalCase`      | Non-terminal (different convention) |

**Key difference:** Lox rules end with `;`. W3C EBNF rules have no terminator.
Do not import semicolon terminators into EAML grammar productions.

### 1.3 Complete Lox Grammar — Verbatim from Appendix I

Source: https://craftinginterpreters.com/appendix-i.html

#### Syntactic Grammar

```
/* ── DECLARATIONS ─────────────────────────────────── */

program        → declaration* EOF ;

declaration    → classDecl
               | funDecl
               | varDecl
               | statement ;

classDecl      → "class" IDENTIFIER ( "<" IDENTIFIER )?
                 "{" function* "}" ;

funDecl        → "fun" function ;

varDecl        → "var" IDENTIFIER ( "=" expression )? ";" ;


/* ── STATEMENTS ────────────────────────────────────── */

statement      → exprStmt
               | forStmt
               | ifStmt
               | printStmt
               | returnStmt
               | whileStmt
               | block ;

exprStmt       → expression ";" ;

forStmt        → "for" "(" ( varDecl | exprStmt | ";" )
                   expression? ";"
                   expression? ")" statement ;

ifStmt         → "if" "(" expression ")" statement
                 ( "else" statement )? ;

printStmt      → "print" expression ";" ;

returnStmt     → "return" expression? ";" ;

whileStmt      → "while" "(" expression ")" statement ;

block          → "{" declaration* "}" ;


/* ── EXPRESSIONS (stratified hierarchy) ────────────── */

expression     → assignment ;

assignment     → ( call "." )? IDENTIFIER "=" assignment
               | logic_or ;

logic_or       → logic_and ( "or" logic_and )* ;

logic_and      → equality ( "and" equality )* ;

equality       → comparison ( ( "!=" | "==" ) comparison )* ;

comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;

term           → factor ( ( "-" | "+" ) factor )* ;

factor         → unary ( ( "/" | "*" ) unary )* ;

unary          → ( "!" | "-" ) unary
               | call ;

call           → primary ( "(" arguments? ")" | "." IDENTIFIER )* ;

primary        → "true" | "false" | "nil" | "this"
               | NUMBER | STRING | IDENTIFIER | "(" expression ")"
               | "super" "." IDENTIFIER ;


/* ── HELPER RULES ─────────────────────────────────── */

function       → IDENTIFIER "(" parameters? ")" block ;

parameters     → IDENTIFIER ( "," IDENTIFIER )* ;

arguments      → expression ( "," expression )* ;
```

#### Lexical Grammar

```
/* ── LEXICAL (scanner, not parser) ───────────────── */

NUMBER         → DIGIT+ ( "." DIGIT+ )? ;

STRING         → "\"" <any char except "\"">* "\"" ;

IDENTIFIER     → ALPHA ( ALPHA | DIGIT )* ;

ALPHA          → "a" ... "z" | "A" ... "Z" | "_" ;

DIGIT          → "0" ... "9" ;
```

### 1.4 What Lox Teaches EAML — Pattern Extraction

#### Pattern L1 — Declaration Dispatch with Inherited Statement Fallback

```
declaration    → classDecl | funDecl | varDecl | statement ;
```

The genius of this rule: declarations fall through to `statement` as the
default. Any expression statement is valid at the top level. EAML should
follow this: any expression is valid after the last declaration.

**EAML translation:**
```ebnf
declaration ::= importDecl
              | modelDecl
              | schemaDecl
              | promptDecl
              | toolDecl
              | agentDecl
              | letDecl
              | exprStmt
```

#### Pattern L2 — The `call` Rule for Postfix Chaining

```
call           → primary ( "(" arguments? ")" | "." IDENTIFIER )* ;
```

This single rule handles ALL postfix operators — function calls and member
access — with zero left recursion. The `( ... )*` suffix makes it iterative.
This is exactly the `postfixExpr` rule in EAML's expression grammar (defined
in Layer 2, Section 2.4).

**Nystrom's annotation on this rule:**
The outer while loop corresponds to the `*` in the grammar rule.
We zip along the tokens building up a chain of calls and gets as we find
parentheses and dots.

**EAML translation:**
```ebnf
/* Direct equivalent — already in Layer 2 EAML grammar */
postfixExpr ::= primaryExpr suffix*

suffix ::= ( "." IDENT )
         | ( "(" argList? ")" )
         | ( "[" expr "]" )
```

#### Pattern L3 — The `ifStmt` Dangling-Else Resolution

```
ifStmt         → "if" "(" expression ")" statement
                 ( "else" statement )? ;
```

The optional `( "else" statement )?` makes `else` greedy — it always binds to
the nearest `if`. This is the standard resolution of the dangling-else problem
by grammar structure. EAML uses the same pattern.

**EAML translation:**
```ebnf
ifStmt ::= "if" expr block ( "else" ( ifStmt | block ) )?
           /* [sem: else-binds-nearest-if] */
```

Note: EAML uses `block` (not `statement`) for if/else bodies — this enforces
braces and prevents dangling statements.

#### Pattern L4 — `block` as Reusable Brace-Delimited Body

```
block          → "{" declaration* "}" ;
```

Lox reuses `block` everywhere a braced body is needed: function bodies,
if/else branches, while bodies, for bodies. EAML does the same — one `block`
rule used in every declaration body that contains statements.

**EAML translation:**
```ebnf
block ::= "{" statement* "}"

/* Used by: toolDecl (native body), ifStmt, whileStmt, forStmt */
/* NOT used by: promptBody, schemaBody — those have their own field grammars */
```

#### Pattern L5 — Function Rule Separation

```
funDecl        → "fun" function ;
function       → IDENTIFIER "(" parameters? ")" block ;
```

Lox separates the declaration keyword (`fun`) from the function body
(`function`). This allows reuse — methods in classes use `function` without
repeating the `fun` keyword. EAML applies this same separation between
`promptDecl` (which has `prompt`, `requires`, `->`) and `promptBody` (the
`{ }` block contents).

#### Pattern L6 — The `logic_or` / `logic_and` Naming Convention

Lox names its precedence levels descriptively: `logic_or`, `logic_and`,
`equality`, `comparison`, `term`, `factor`, `unary`. This naming makes the
grammar self-documenting. EAML's Layer 2 names follow the same convention:
`orExpr`, `andExpr`, `comparisonExpr`, `additiveExpr`, `multiplicativeExpr`,
`unaryExpr`.

### 1.5 What Lox Does NOT Have (Do Not Import These)

| Lox Feature                                         | Why Not in EAML                                           |
|-----------------------------------------------------|-----------------------------------------------------------|
| `"class"` declaration with inheritance              | EAML has `schema` — no inheritance in MVP                 |
| `"super"` keyword                                   | OOP concept — no analog in EAML                           |
| `"this"` keyword                                    | OOP concept — EAML tools don't have instance context      |
| `"print"` statement                                 | Baked-in for pedagogy — EAML has no built-in IO statement |
| `"for"` / `"while"` loops                           | EAML tools use Python for iteration logic                 |
| `"var"` variable declaration                        | EAML uses `"let"` with explicit type annotation           |
| Dynamic typing (no type annotations)                | EAML is statically typed at the interface level           |
| String literals with `<any char>` regex description | EAML uses precise `AnyChar - ['"\\]` expression           |
| Semicolons required on statements                   | EAML treats semicolons as optional                        |

---

## Section 2 — BAML Grammar (BoundaryML)

### 2.1 Source Attribution

**Organization:** BoundaryML
**Repository:** https://github.com/BoundaryML/baml
**Grammar file:** `engine/baml-lib/baml-core/src/pest_grammar/baml.pest`
**Documentation:** https://docs.boundaryml.com
**Language Reference:** https://deepwiki.com/BoundaryML/baml/5-baml-language-reference
**License:** Apache 2.0

BAML is the direct domain predecessor to EAML. It is a production LLM DSL
built in Rust, with a Pest PEG parser, a Rust-based runtime, VS Code extension,
and multi-language code generation (Python, TypeScript, Ruby, Go).

BAML's architecture is: Pest grammar → AST → HIR (symbol resolution)
→ codegen / VM execution. This is identical to EAML's planned architecture.

### 2.2 Notation Warning — BAML Uses Pest PEG Notation

BAML uses **Pest** (Parsing Expression Grammar) notation, which is
fundamentally different from W3C EBNF in several ways:

| Pest Notation    | W3C EBNF Equivalent              | Meaning                                      |
|------------------|----------------------------------|----------------------------------------------|
| `rule = { ... }` | `rule ::= ...`                   | Production definition                        |
| `~`              | (space between terms)            | Concatenation                                |
| `\|`             | `\|`                             | Ordered choice (PEG is ordered, EBNF is not) |
| `rule*`          | `rule*`                          | Zero or more                                 |
| `rule+`          | `rule+`                          | One or more                                  |
| `rule?`          | `rule?`                          | Optional                                     |
| `!rule`          | `AnyChar - rule` (approximately) | Negative lookahead                           |
| `&rule`          | (no direct equivalent)           | Positive lookahead                           |
| `ASCII_ALPHA`    | `[a-zA-Z]`                       | Built-in Pest character class                |
| `ANY`            | `AnyChar`                        | Any character                                |
| `WHITESPACE`     | `WS` (implicit)                  | Pest auto-inserts between rules              |

**Critical difference:** Pest `|` is **ordered choice** (first match wins).
W3C EBNF `|` is **unordered alternation** (any match is valid). This means
you cannot directly translate Pest grammar into W3C EBNF without verifying
that the ordering doesn't matter (it usually does in Pest grammars).

### 2.3 BAML Language Constructs — Structure

BAML organizes its grammar around these top-level constructs:

```
// BAML declaration types (from BAML documentation and source)
// File: baml_src/*.baml

// 1. class — structural data type (EAML analog: schema)
class Resume {
  name      string
  email     string?
  skills    string[]
  education Education[]
}

// 2. enum — fixed value set (EAML analog: literal union in typeExpr)
enum Sentiment {
  POSITIVE   @alias("positive")
  NEGATIVE   @alias("negative")
  NEUTRAL    @alias("neutral")
}

// 3. function — typed LLM prompt (EAML analog: prompt)
function ExtractResume(text: string) -> Resume {
  client "openai/gpt-4o"
  prompt #"
    Extract the resume from this text.
    {{ ctx.output_format }}
    {{ _.role('user') }}
    {{ text }}
  "#
}

// 4. client<llm> — model configuration (EAML analog: model)
client<llm> GPT4o {
  provider "openai"
  options {
    model "gpt-4o"
    api_key env.OPENAI_API_KEY
    temperature 0.0
  }
}

// 5. retry_policy — retry configuration (EAML analog: on_error in agent)
retry_policy RetryTwice {
  max_retries 2
  strategy {
    type exponential_backoff
    delay_ms 100
  }
}
```

### 2.4 BAML Type System — Structure

```
// BAML type annotations (from BAML language reference)

// Primitive types
string     int     float     bool     null

// Optional — postfix ?
string?    int?    Resume?

// Array — postfix []
string[]   Resume[]   int[]?

// Union — pipe operator
string | int | Resume

// Literal union (inline in class field or function signature)
"positive" | "negative" | "neutral"

// Map
map<string, int>

// Class reference (declared elsewhere)
Resume     Experience[]

// Generic (BAML uses angle brackets for generics, Post-MVP in EAML)
// map<K, V>
```

**Key observation for EAML:** BAML's type system is structurally identical to
what EAML specifies. The same five patterns appear: primitives, optional `?`,
array `[]`, union `|`, and class references. The difference is that EAML adds
**bounded types** (`Float<0.0, 1.0>`, `String<max: 200>`) which BAML does not have.

### 2.5 BAML Class/Schema Declaration Pattern

```
// BAML class declaration (source: docs.boundaryml.com)

class Resume {
  name      string                           // required, primitive
  email     string?                          // optional (postfix ?)
  skills    string[]                         // array
  education Education[]                      // array of class reference
  score     int?       @description("0-100") // field decorator
  status    "active" | "inactive"            // literal union field type
}
```

**BAML field syntax pattern:**
```
fieldName  typeAnnotation  [decorators]*  [// comment]?
```

Fields are separated by newlines (no commas). The type annotation immediately
follows the field name with a space (no `:` separator).

**EAML difference:** EAML uses `fieldName: typeExpr` with a colon separator,
matching Python/TypeScript/Rust convention. BAML omits the colon. EAML's
colon is a deliberate choice to distinguish field declarations from variable
uses and to match engineer intuition from typed languages.

### 2.6 BAML Function Declaration Pattern

```
// BAML function declaration (source: docs.boundaryml.com)

function ExtractResume(
  text: string,         // parameter with colon — BAML DOES use colon in params
  format: string = "json"  // default value
) -> Resume {
  client "openai/gpt-4o"           // which LLM client to use
  prompt #"                         // raw string prompt body
    Extract the resume from:
    {{ text }}
    {{ ctx.output_format }}
  "#
}
```

**Key structural observations for EAML:**

1. **Parameters use colon:** `text: string` — same as EAML. Consistent with
   general convention even though class fields don't use colon.

2. **`->` return type arrow:** Identical to EAML. A clear, widely-understood
   convention (Rust, Python type hints, TypeScript).

3. **`client` as function-level field:** BAML binds the model inside the
   function body. EAML binds the model at the call site (`Claude.call(prompt)`).
   This is a significant design difference — EAML's approach allows runtime
   model selection; BAML's bakes it in at definition time.

4. **Raw string prompt block `#"..."#`:** BAML's delimiter for multi-line
   prompt content. It avoids all escaping. EAML uses regular double-quoted
   template strings with `{{` escape for literal braces — a different choice.

5. **`{{ variable }}` interpolation inside prompt:** Jinja2-style. EAML uses
   `{variable}` (single braces) with `{{` and `}}` as escape sequences for
   literal braces. This is an EAML-specific convention documented in Layer 1.

### 2.7 BAML Template String Design — The `#"..."#` Pattern

BAML's raw string delimiter is the single most important syntactic decision
for an LLM DSL. Prompts routinely contain JSON examples, code snippets, and
instructions with curly braces. BAML's solution:

```
// BAML raw string — no escaping needed
prompt #"
  Extract the following fields as JSON:
  {
    "name": "...",
    "email": "..."
  }
  From this text: {{ text }}
"#
```

The `#"..."#` delimiter cannot appear inside a prompt without deliberate
construction, because it requires both a `"` and a `#` in sequence with
specific orientation.

**EAML's alternative approach:**
EAML uses `{{` / `}}` as escape sequences for literal braces inside
standard double-quoted strings:

```ebnf
/* EAML template string with brace escaping */
"Here is the JSON schema: {{ \"name\": \"string\" }} and {{ text }}"
/*                         ^^                   ^^               ^^ */
/*                         escaped {             escaped }        interpolation */
```

**Trade-off analysis:**

| Criterion        | BAML `#"..."#`                                | EAML `"...{{...}}"`                     |
|------------------|-----------------------------------------------|-----------------------------------------|
| Familiarity      | Novel delimiter — must be learned             | Matches Python f-strings and Jinja2     |
| Brace handling   | All literal braces work natively              | `{{` and `}}` needed for literal braces |
| Multi-line       | Native                                        | Native                                  |
| JSON in prompts  | Trivial — no escaping                         | Requires `{{` for each JSON brace       |
| Lexer complexity | Simple — capture until `"#`                   | Requires interpolation mode tracking    |
| EAML decision    | Not used — too novel for engineering audience | Used — familiar to the target audience  |

EAML retains the double-quoted template string with `{{`/`}}` escape because
the target audience (engineers) is already familiar with this pattern from
Python f-strings. BAML's `#"..."#` is a valid alternative that EAML
deliberately did not adopt.

### 2.8 BAML Client Declaration — EAML `model` Analog

```
// BAML client declaration (source: docs.boundaryml.com)

client<llm> GPT4o {
  provider "openai"
  options {
    model       "gpt-4o"
    api_key     env.OPENAI_API_KEY
    temperature 0.7
    max_tokens  1000
  }
}

// BAML retry policy (separate concern)
retry_policy MyRetry {
  max_retries 3
  strategy {
    type constant_delay
    delay_ms 200
  }
}
```

**EAML `model` analog:**
```ebnf
/* EAML collapses client + retry into a single model declaration */
model GPT4o = Model("gpt-4o", caps: [json_mode, tools, vision])
```

**Structural difference:** BAML separates the LLM provider configuration
(`client<llm>`) from retry policy (`retry_policy`) into distinct declarations.
EAML consolidates these concerns: capabilities are declared at model definition,
retry policy is declared in `agent` declarations. This is a deliberate EAML
simplification for the MVP.

### 2.9 BAML Enum — EAML Literal Union Analog

```
// BAML enum declaration
enum Sentiment {
  POSITIVE   @alias("positive")   @description("Good feedback")
  NEGATIVE   @alias("negative")
  NEUTRAL    @alias("neutral")
}

// Used as: function Classify(text: string) -> Sentiment
```

**EAML equivalent:** EAML does not have a separate `enum` declaration.
Instead, literal unions appear inline in schema field types:

```ebnf
/* EAML schema field with literal union — no separate enum needed */
schema SentimentResult {
  sentiment: "positive" | "negative" | "neutral"
}
```

**Trade-off:** BAML's named enum is more reusable across multiple schemas.
EAML's inline literal union is more concise for single-use cases. EAML Post-MVP
should add a named enum declaration for reusable value sets.

**EAML Post-MVP enum reservation:**
```ebnf
/* Post-MVP — reserved but not implemented in v0.1 */
/* enumDecl ::= "enum" IDENT "{" enumValue* "}" */
/* enumValue ::= IDENT ( "@alias" "(" STRING ")" )? */
```

### 2.10 BAML Decorator Pattern — EAML `@` Annotation Analog

```
// BAML decorators on class fields
class Resume {
  name  string  @description("Full legal name")
  score int?    @description("0-100 confidence score") @alias("confidence")
}

// BAML class-level decorators
class Candidate {
  name  string
  score int
  @@dynamic                    // allows runtime schema modification
  @@assert(valid_score, {{ this.score >= 0 and this.score <= 100 }})
}
```

**EAML annotation plan (Post-MVP):**
EAML reserves `@` for field-level annotations, following BAML's pattern:

```ebnf
/* EAML field with annotation — Post-MVP */
/* FieldDef ::= Annotation* IDENT ":" typeExpr */
/* Annotation ::= "@" IDENT ( "(" ArgList ")" )? */

/* Reserved @-names for EAML Post-MVP: */
/* @description("...") — field documentation passed to LLM via JSON Schema */
/* @alias("...")        — JSON key override for generated schema */
/* @example(value)     — example value in JSON Schema for LLM guidance */
```

The `@` sigil is **reserved** in EAML v0.1 grammar even though annotations
are Post-MVP. This prevents `@` from becoming a valid operator or identifier
character that would break Post-MVP implementations.

---

## Section 3 — Cross-Source Pattern Comparison

### 3.1 Declaration Structure Comparison

| Construct       | Lox     | BAML           | EAML                |
|-----------------|---------|----------------|---------------------|
| Structural type | `class` | `class`        | `schema`            |
| Function/prompt | `fun`   | `function`     | `prompt`            |
| Model/client    | (none)  | `client<llm>`  | `model`             |
| Retry policy    | (none)  | `retry_policy` | in `agent`          |
| Enum            | (none)  | `enum`         | Post-MVP `enum`     |
| Agent           | (none)  | (none)         | `agent`             |
| Pipeline        | (none)  | (none)         | Post-MVP `pipeline` |
| Tool            | (none)  | (none)         | `tool`              |
| Import          | (none)  | `generator`    | `import python`     |

### 3.2 Type System Comparison

| Feature                        | Lox          | BAML                | EAML                       |
|--------------------------------|--------------|---------------------|----------------------------|
| Type annotations on parameters | No (dynamic) | Yes: `name: string` | Yes: `name: String`        |
| Optional type                  | No           | `type?`             | `typeExpr?`                |
| Array type                     | No           | `type[]`            | `typeExpr[]`               |
| Literal union                  | No           | `"a" \| "b"`        | `"a" \| "b"`               |
| Bounded primitives             | No           | No                  | `Float<0.0, 1.0>`          |
| Enum                           | No           | `enum` decl         | Inline literal union (MVP) |
| Generic types                  | No           | `map<K, V>`         | Post-MVP                   |
| Return type annotation         | No           | `-> Type`           | `-> typeExpr`              |

### 3.3 Template String Comparison

| Feature                     | Lox  | BAML                    | EAML                     |
|-----------------------------|------|-------------------------|--------------------------|
| Multi-line strings          | Yes  | Yes (`#"..."#`)         | Yes (double-quoted)      |
| Interpolation               | No   | `{{ var }}` (Jinja2)    | `{var}` (single brace)   |
| Literal brace escape        | N/A  | Not needed in `#"..."#` | `{{` and `}}`            |
| Interpolation expressions   | N/A  | Yes (Jinja2 full)       | Identifier only (MVP)    |
| Type-checked interpolations | N/A  | No                      | Yes (against param list) |

### 3.4 Comment Syntax Comparison

| Feature     | Lox       | BAML      | EAML             |
|-------------|-----------|-----------|------------------|
| Single-line | `// text` | `// text` | `// text`        |
| Multi-line  | No        | No        | `/* text */`     |
| Doc comment | No        | No        | Post-MVP (`///`) |

EAML is the only one of the three with multi-line comment support, adopted
from the C family because engineers are universally familiar with `/* */`.

---

## Section 4 — BAML Decisions EAML Deliberately Did Not Adopt

Understanding what EAML chose NOT to copy from BAML is as important as
understanding what it did adopt.

### 4.1 Model Binding Inside Function Body

**BAML:** `client "openai/gpt-4o"` is declared inside the `function` body.

**Problem:** The model is baked in at definition time. Swapping models
requires editing the function definition.

**EAML decision:** Model is specified at the **call site**:
`Claude.call(AnalyzeSentiment(...))`. This allows the same prompt to be
called against different models at runtime, and capability checking is
performed against the specific model being used.

### 4.2 No Separate `enum` Declaration (MVP)

**BAML:** `enum Sentiment { POSITIVE NEGATIVE NEUTRAL }` as a top-level
declaration that can be reused across multiple functions and classes.

**Problem for MVP:** Adds a declaration type to implement and a name
resolution case for enum values. Adds `@alias` mapping complexity.

**EAML decision:** Literal unions inline in schema fields for MVP.
`sentiment: "positive" | "negative" | "neutral"` — simpler, sufficient
for 90% of use cases. Named enums are Post-MVP.

### 4.3 `#"..."#` Raw String Delimiter

**BAML:** Raw string `#"..."#` avoids all escaping in prompt bodies.

**Problem:** Novel delimiter that engineers must learn. Not similar to any
existing language convention.

**EAML decision:** Standard double-quoted template strings with `{{`/`}}`
escape sequences — familiar from Python f-strings and Jinja2.

### 4.4 Jinja2 Full Expression in Interpolations

**BAML:** `{{ ctx.output_format }}`, `{{ _.role('user') }}` — full Jinja2
template expressions including filters, method calls, and context variables.

**Problem for MVP:** Jinja2 parsing inside a template string is a full
sub-language. This adds significant lexer and semantic analysis complexity.

**EAML decision:** MVP interpolation accepts identifier only: `{varName}`.
This covers 90% of real use cases. Full expressions in interpolations
are Post-MVP.

### 4.5 No Capability System

**BAML:** No concept of model capabilities. Any function can be called
against any client. Capability mismatches are discovered at runtime when
the API returns an error.

**EAML decision:** Explicit capability declarations on `model` and
`requires` clauses on `prompt`. Capability mismatch is a **compile-time
error** in EAML. This is EAML's primary type-safety differentiator over BAML.

---

## Section 5 — Translation Guide: Prior Art → W3C EBNF

When you see a pattern in Lox or BAML that you want to apply to EAML,
use these translations to write it in W3C EBNF notation (Layer 1).

### 5.1 Lox `→` Productions → W3C `::=` Productions

```
/* LOX (do not use this syntax in EAML) */
declaration    → classDecl | funDecl | varDecl | statement ;

/* W3C EBNF EAML equivalent */
declaration ::= importDecl
              | modelDecl
              | schemaDecl
              | promptDecl
              | toolDecl
              | agentDecl
              | letDecl
              | exprStmt
```

Rules: replace `→` with `::=`, remove `;`, convert lowercase names to
`camelCase`, convert UPPERCASE terminals to `ALL_CAPS`.

### 5.2 Lox `( option | option )*` → W3C Same Pattern

```
/* LOX */
equality   → comparison ( ( "!=" | "==" ) comparison )* ;

/* W3C EBNF — identical structure, different syntax */
equalityExpr ::= comparisonExpr ( ( "!=" | "==" ) comparisonExpr )*
```

### 5.3 BAML Field Syntax → W3C EBNF

```
/* BAML field pattern: fieldName typeAnnotation [decorators]* */
/* name  string?  @description("...") */

/* W3C EBNF EAML equivalent */
fieldDef ::= IDENT ":" typeExpr ( "," | NL )?
             /* [sem: field-type-must-resolve]    */
             /* Post-MVP: annotation* before IDENT */
```

### 5.4 BAML Optional `?` → W3C EBNF Postfix `?`

```
/* BAML */
email     string?

/* W3C EBNF */
/* string? is already W3C EBNF postfix — syntax is identical */
/* In a typeExpr production: */
typeExpr ::= baseType arraySuffix? optionalSuffix?
optionalSuffix ::= "?"
```

### 5.5 BAML Literal Union → W3C EBNF

```
/* BAML inline literal union */
tone: "happy" | "sad" | "neutral"

/* W3C EBNF */
literalUnion ::= STRING ( "|" STRING )+
/* Note: minimum TWO members (one STRING is not a union) */
/* [sem: literal-union-minimum-two-members] */
```

### 5.6 BAML `->` Return Type → W3C EBNF

```
/* BAML */
function Classify(text: string) -> Sentiment { ... }

/* W3C EBNF */
promptDecl ::= "prompt" IDENT "(" paramList? ")"
               requiresClause?
               "->" typeExpr
               promptBody
```

---

## Section 6 — Prior Art Rules for AI Grammar Assistance

When helping write EAML grammar productions, apply these rules derived
from the prior art in this document:

1. **Lox is the readability standard.** If an EAML grammar production cannot
   be read as clearly as the corresponding Lox rule, it needs to be simplified.

2. **BAML validates EAML's structural decisions.** Where EAML and BAML agree
   on syntax (parameter colon, `->` return type, `?` optional, `[]` array),
   the choice is confirmed by independent convergence on the same convention.

3. **BAML divergences from EAML are deliberate.** Never suggest BAML patterns
   that EAML explicitly rejected: `client` inside function body, `#"..."#`
   delimiters, full Jinja2 in interpolations, or no capability system.

4. **Do not import Lox or BAML notation.** Lox uses `→` and `;`. BAML uses
   Pest `= { }`. EAML uses W3C EBNF `::=`. Never use Lox or BAML notation
   in an EAML grammar production.

5. **EAML's bounded types (`Float<0.0, 1.0>`) are unique.** Neither Lox nor
   BAML has them. Do not look to prior art for this grammar — use the XPath
   `SequenceType` pattern from Layer 2 as the structural model instead.

6. **The `tool` / `agent` / `pipeline` constructs have no prior art here.**
   Lox and BAML have no equivalents. Apply the SPARQL declaration dispatch
   pattern from Layer 2 and derive these grammars from first principles.

7. **The Python bridge has no prior art.** `python { }` blocks are unique
   to EAML. The `PYTHON_BLOCK` opaque token approach is defined in Layer 1.
   Do not attempt to find analogies in Lox or BAML.

8. **Capability checking is EAML-unique.** The `requires` clause and
   capability subset checking at compile time has no analog in either
   prior art source. It is a pure semantic analysis concern, not a grammar
   concern — annotate with `[sem: cap-check]` and move on.

---

## Section 7 — Source Attribution

| Content                                    | Source                                              | URL                                                            | License        |
|--------------------------------------------|-----------------------------------------------------|----------------------------------------------------------------|----------------|
| Lox grammar (complete)                     | *Crafting Interpreters*, Appendix I, Robert Nystrom | https://craftinginterpreters.com/appendix-i.html               | CC BY 4.0      |
| Lox grammar patterns (call, ifStmt, block) | *Crafting Interpreters*, various chapters           | https://craftinginterpreters.com/                              | CC BY 4.0      |
| BAML language constructs                   | BAML Language Reference, DeepWiki                   | https://deepwiki.com/BoundaryML/baml/5-baml-language-reference | Apache 2.0     |
| BAML syntax and grammar                    | BAML Syntax and Grammar, DeepWiki                   | https://deepwiki.com/BoundaryML/baml/5.1-syntax-and-grammar    | Apache 2.0     |
| BAML type system                           | BAML Type System, DeepWiki                          | https://deepwiki.com/BoundaryML/baml/5.3-type-system           | Apache 2.0     |
| BAML examples                              | BAML documentation                                  | https://docs.boundaryml.com                                    | Apache 2.0     |
| BAML architecture                          | BoundaryML/baml, CONTRIBUTING.md                    | https://github.com/BoundaryML/baml/blob/canary/CONTRIBUTING.md | Apache 2.0     |
| EAML-specific analysis                     | EAML specification (this document)                  | N/A — original                                                 | EAML Draft 0.1 |

---

*EAML Layer 3 Prior Art Reference — Version 0.1 — 2026-03-14*
*Load after `eaml-layer1-notation-reference.md` and `eaml-layer2-grammar-patterns.md`.*
*Combine with Layer 4 (compiler theory references) for complete grammar sessions.*