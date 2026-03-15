# EAML Grammar Design Decisions — Layer 5
## Authoritative Design Record: All Deliberate EAML-Specific Decisions

---

## Document Purpose and Usage Instructions

This document is **Layer 5** — the final and most authoritative layer of the
EAML grammar reference stack. Every decision recorded here was made deliberately
by the EAML language designer in a structured design session on 2026-03-14.

**This document is the ground truth for all EAML grammar work.**

Where Layers 1–4 provide notation standards, patterns, prior art, and compiler
theory from external sources, Layer 5 is uniquely EAML's. No reference book,
no prior art grammar, and no AI training data can tell you what is in this
document. An AI helping write EAML grammar productions MUST load this layer
first and treat every entry as a closed decision that is NOT subject to
re-evaluation unless explicitly overridden by the designer.

**How to use this document:**
- Decisions marked `[CLOSED]` are final for v0.1. Do not suggest alternatives.
- Decisions marked `[POST-MVP]` are deferred but reserved — syntax is
  grammatically blocked even in v0.1 to prevent future breaking changes.
- Decisions marked `[GRAMMAR IMPACT]` require a specific production rule
  consequence described inline.
- When a decision conflicts with a suggestion from Layers 1–4 or prior art,
  Layer 5 always wins.

**Load order for complete grammar sessions:**
```
1. eaml-layer1-notation-reference.md    (W3C EBNF operators)
2. eaml-layer2-grammar-patterns.md      (XPath, SPARQL patterns)
3. eaml-layer3-prior-art.md             (Lox, BAML patterns)
4. eaml-layer4-compiler-theory.md       (FIRST/FOLLOW, Pratt, LL(1))
5. eaml-layer5-design-decisions.md      (THIS DOCUMENT — load last)
```

---

## Section 1 — Identity and Scope

### 1.1 Language Identity

| Property                 | Decision                       | Notes                                                      |
|--------------------------|--------------------------------|------------------------------------------------------------|
| **Full name**            | Engineering AI Markup Language | `[CLOSED]`                                                 |
| **Abbreviation**         | EAML                           | `[CLOSED]`                                                 |
| **File extension**       | `.eaml`                        | `[CLOSED]` — confirmed clear, no conflicts                 |
| **Current version**      | `0.1.0`                        | Semantic versioning: `MAJOR.MINOR.PATCH`                   |
| **Version scheme**       | Semantic versioning            | `[CLOSED]` — v0.x = unstable API, v1.0.0 = stable contract |
| **Compiler binary name** | `eamlc`                        | Conventional: language abbrev + `c`                        |

### 1.2 Target Audience

**Dual audience — both of the following, equally:**

1. **Defense/aerospace systems engineers** — DoD/USSF context, SAFe Agile
   workflows, auditability requirements, formal change control. Expect: formal
   language, precise error messages with codes, conservative MVP scope,
   traceability language.

2. **General software engineers building LLM integrations** — Python ecosystem,
   fast iteration, familiar conventions. Expect: ergonomic defaults, Python
   f-string familiarity, minimal boilerplate.

**Implication for grammar design:** When a syntax choice serves one audience
at the expense of the other, document the tradeoff. When both audiences can be
served equally, prefer the more ergonomic choice.

### 1.3 Design Philosophy

EAML's core value proposition is moving LLM integration failures **left** —
from runtime surprises to compile-time errors. Every grammar decision that
supports this philosophy (explicit type annotations, capability checking,
interpolation validation) takes precedence over ergonomic convenience.

---

## Section 2 — Lexical Decisions

### 2.1 Character Encoding and Identifiers

```
[CLOSED] File encoding:    UTF-8
[CLOSED] Identifiers:      ASCII letters, digits, underscore only
                           Pattern: [a-zA-Z_][a-zA-Z0-9_]*
[CLOSED] Case sensitivity: FULLY case-sensitive
                           Keywords are lowercase only.
                           'Schema' is a valid user identifier, 'schema' is a keyword.
                           There are no case-insensitive keywords.
```

**`[GRAMMAR IMPACT]`** The identifier production is:
```ebnf
IDENT ::= [a-zA-Z_] [a-zA-Z0-9_]*
          /* minus all reserved keywords */
```

### 2.2 Semicolons

```
[CLOSED] Semicolons are OPTIONAL statement and declaration terminators.
         A semicolon is accepted if present and silently ignored.
         A semicolon is never required.
         A semicolon is never forbidden.
```

**`[GRAMMAR IMPACT]`** Every statement and declaration production ends with `";"?`
(optional semicolon). The parser never emits an error for a missing or present
semicolon at statement boundaries.

### 2.3 Comment Styles

```
[CLOSED] EAML supports three comment styles:

  // text          Single-line comment — extends to end of line.
                   IMPLEMENTED in v0.1.

  /* text */       Block comment — may span multiple lines.
                   IMPLEMENTED in v0.1.

  /// text         Doc comment — reserved for Post-MVP tooling.
                   PARSED but IGNORED in v0.1.
                   Reserved now to prevent /// becoming a valid
                   operator or token that would break Post-MVP.

  # text           NOT supported. Hash is not a comment character in EAML.
                   (Avoids confusion with Python convention in mixed files.)
```

**`[GRAMMAR IMPACT]`** The lexer skips all three comment forms between tokens:
```ebnf
COMMENT ::= "//" ( AnyChar - NL )*         /* single-line */
           | "/*" ( AnyChar* - "*/") "*/"  /* block */
           | "///" ( AnyChar - NL )*       /* doc — skipped in v0.1 */
```

### 2.4 Numeric Literals

```
[CLOSED] Integer literals:  Decimal only. No leading zeros except lone 0.
                            Pattern: '0' | [1-9][0-9]*
                            No hex, octal, binary in v0.1.

[CLOSED] Float literals:    Decimal point required. Digit required on both
                            sides of the decimal point.
                            Pattern: ('0' | [1-9][0-9]*) '.' [0-9]+
                            '.5' is NOT a valid EAML float — require leading digit.

[CLOSED] Negative numbers:  The '-' is always the unary minus operator, never
                            part of a numeric literal token. The lexer never
                            emits a negative number token.
                            '-1.0' lexes as [MINUS, FLOAT(1.0)], not [FLOAT(-1.0)].
```

### 2.5 Reserved Keywords — Complete Inventory

```
[CLOSED] COMPLETE KEYWORD LIST FOR v0.1:

DECLARATION KEYWORDS (active in v0.1):
  model   schema   prompt   tool   agent   import   let

DECLARATION KEYWORDS (Post-MVP — reserved, blocked as identifiers):
  pipeline   enum   extends

STATEMENT/EXPRESSION KEYWORDS:
  if   else   return   await   true   false   null   python

FUTURE-RESERVED (blocked as identifiers even though not yet used):
  pipeline   enum   extends   override   interface   type   where
  for   while   match   async   yield

NOTE: 'python' is a FULL KEYWORD — not contextual.
      Engineers cannot name schema fields, variables, or any identifier 'python'.
      This was a deliberate choice (simpler lexer) over contextual keyword treatment.

NOTE: 'void' is NOT a keyword in v0.1.
      Tools with no meaningful return value omit the return type or use '-> null'.
      'void' is reserved for Post-MVP.
```

---

## Section 3 — Type System Decisions

### 3.1 Primitive Type Names

```
[CLOSED] Built-in primitive types use LOWERCASE names:

  string    float    int    bool    null

These are PREDECLARED IDENTIFIERS in the type registry, not keywords.
A user may shadow them with a schema of the same name — this emits a
warning (TYP001) but is not a compile error.

  void — NOT in v0.1. Tools with no return value use a different syntax.
         Reserved as a future predeclared identifier.
```

**`[GRAMMAR IMPACT]`** The `typeExpr` production references these as `namedType`
(looked up in symbol table), not as keyword terminals. The type symbol table
is pre-populated with `string`, `float`, `int`, `bool`, `null` before any
user declarations are processed.

### 3.2 Typing Discipline

```
[CLOSED] EAML uses NOMINAL typing for schemas.

  schema A { x: string }
  schema B { x: string }

A and B are DIFFERENT types even though they have identical shape.
They are NOT interchangeable at call sites or let binding annotations.
Structural typing is Post-MVP.
```

### 3.3 Type Expression Modifier Rules

```
[CLOSED] Optional modifier '?' and array modifier '[]' are POSTFIX operators
         applied to type expressions. POSITION DETERMINES MEANING:

  Tag[]?    = Optional(Array(Tag))  — the whole array is optional
              (may be absent entirely)

  Tag?[]    = Array(Optional(Tag))  — array of possibly-null Tags
              (array is always present, but elements may be null)

Both forms are valid EAML. They produce DIFFERENT types and DIFFERENT
Pydantic output. The grammar must encode the distinction structurally.
```

**`[GRAMMAR IMPACT]`** The type expression precedence hierarchy:
```ebnf
typeExpr       ::= baseType boundedSuffix? arraySuffix? optionalSuffix?
                 | baseType optionalSuffix? arraySuffix?
                 /* Second form produces Tag?[] — optional element, required array */
```

Both orderings are legal. The AST records which modifiers appeared in which
order. Codegen produces different Pydantic output for each.

### 3.4 Array Dimensionality

```
[CLOSED] Single-dimension arrays ONLY in v0.1.

  Tag[]      — valid
  Tag[][]    — PARSE ERROR: SYN042

  Error message: "Multi-dimensional arrays are not supported in EAML v0.1.
  Hint: Use a schema with an array field: schema TagMatrix { rows: Tag[] }"
```

### 3.5 Bounded Type Parameters

```
[CLOSED] Bounded type syntax: BaseType<params>

  float<0.0, 1.0>        — positional, min and max, both required
  float<min: 0.0>        — named, only min specified
  float<max: 1.0>        — named, only max specified
  float<min: 0.0, max: 1.0>  — both named

  string<max: 200>       — named, max length
  string<min: 1>         — named, min length
  string<min: 1, max: 200>   — both

  int<min: 0>            — named
  int<min: 0, max: 100>  — both

Bounded parameters for float: 'min', 'max' (both optional, both float)
Bounded parameters for string: 'min', 'max' (both optional, both int)
Bounded parameters for int: 'min', 'max' (both optional, both int)

Validation of parameter names against the base type is a SEMANTIC check
(SEM030), not a grammar check. The grammar accepts any named or positional
bounded parameters and delegates validation to semantic analysis.
```

### 3.6 Literal Union Types

```
[CLOSED] Literal unions require MINIMUM TWO members.
         A single string in a type position is a string type annotation,
         not a single-member union.

  "positive" | "negative" | "neutral"   — valid literal union (3 members)
  "positive"                            — string type annotation, not a union
  "positive" | "negative"              — valid literal union (2 members)

The pipe operator '|' in a TYPE expression position is a union separator.
The pipe operator '|' is NOT valid in an EXPRESSION position — there is
no bitwise OR in EAML. '|' in expression context is a parse error with
hint: "Did you mean '||' for logical OR?"
```

---

## Section 4 — Template String Decisions

### 4.1 Interpolation Syntax

```
[CLOSED] Template string interpolation uses SINGLE BRACES: {varName}
         Matches Python f-string convention — familiar to target audience.

         Literal brace characters use DOUBLE-BRACE ESCAPE:
           {{   produces literal {
           }}   produces literal }

         Examples:
           "Analyze this text: {text}"         — {text} is interpolated
           "JSON format: {{ \"key\": \"value\" }}"  — {{ and }} are literal braces
           "Score range: {{ 0.0 to 1.0 }}"     — literal braces
```

### 4.2 Interpolation Content

```
[CLOSED] FULL EXPRESSIONS are supported inside interpolation slots in v0.1.

  {paramName}              — identifier
  {result.field}           — member access
  {result.field > 0.5}     — comparison expression
  {score * 100}            — arithmetic expression

LEXER IMPLICATION: The lexer must perform BRACE-DEPTH COUNTING inside
interpolation slots to find the closing '}'. This means the closing brace
of an interpolation is the first '}' at depth zero. Nested braces are
allowed: {obj.method({"key": "val"})} requires depth tracking.

SEMANTIC IMPLICATION: Every interpolation expression must be type-checked
against the prompt's parameter scope. Identifier-only interpolations are
resolved via symbol table lookup. Member access and expressions require
full expression type-checking.

This is the MOST COMPLEX choice in the template string design. It is
deliberate and fully accepted by the language designer.
```

### 4.3 Multi-line Template Strings

```
[CLOSED] Multi-line template strings ARE allowed.
         Newlines inside a template string are PRESERVED VERBATIM.
         No dedent or whitespace normalization is applied by the compiler.
         What the engineer writes is what the LLM receives.

         Example:
           system: "You are an expert analyst.
           Respond in {lang}.
           Be concise."

         The newlines between the lines are preserved in the emitted Python
         as a multi-line string. This is consistent with WYSIWYG prompt
         authoring — important for auditable DoD workflows.
```

---

## Section 5 — Python Bridge Decisions

### 5.1 Python Block Delimiter

```
[CLOSED] Python blocks use LEX/YACC-STYLE DELIMITERS: python %{ ... }%

         The opening delimiter is: python %{
         The closing delimiter is: }%

         LEXER ALGORITHM:
           On seeing keyword 'python' followed by '%{':
             Enter PYTHON_BLOCK mode.
             Scan forward until the two-character sequence '}%' is found.
             Emit PYTHON_BLOCK(captured_text) containing everything between
             the delimiters (not including the delimiters themselves).
             Return to normal EAML lexer mode.

         RATIONALE:
           - Inspired by lex/yacc convention (familiar to systems engineers)
           - No brace-depth counting required
           - No Python string scanner required
           - The sequence '}%' cannot appear in real Python code accidentally
           - Zero known limitations — all Python code is safely capturable

         Example:
           tool Analyze(path: string) -> DataSummary {
             python %{
               import pandas as pd
               df = pd.read_csv(path)
               # dict literals with } work fine: {"key": "val"}
               return {"mean": float(df.mean()), "count": int(len(df))}
             }%
           }
```

### 5.2 Python Import Location

```
[CLOSED] Python import declarations MUST appear at FILE-LEVEL, BEFORE any
         other declarations. They are the first items in an EAML file.

         VALID:
           import python "pandas" as pd
           import python "numpy" as np

           schema DataResult { ... }
           tool Analyze(...) -> DataResult { python %{ ... }% }

         INVALID (semantic error SEM010):
           schema DataResult { ... }
           import python "pandas" as pd  ← after a declaration

         RATIONALE:
           All imports visible at the top — no hunting through the file.
           Consistent with Python's own import convention.
           Simplifies static analysis — all available modules known before
           processing any declaration body.
```

### 5.3 Python Syntax Validation

```
[CLOSED] Python block syntax validation is FLAG-CONTROLLED, OFF BY DEFAULT.

         --check-python    Enables python -m py_compile on extracted blocks.
                           Requires Python to be in PATH.
                           If Python is not found: warning, not error.

         Default (no flag): Python blocks are opaque. Syntax errors in
                            python %{ }% blocks appear at Python runtime only.

         RATIONALE:
           Keeps the EAML compiler self-contained with no Python dependency.
           CI/CD containers may not have Python available.
           Engineers who want validation can opt in.
```

### 5.4 Python Return Type Contract

```
[CLOSED] The tool's declared return type creates a CONTRACT with the Python block.

         The EAML compiler does not verify that the Python block returns
         the correct type — this is enforced at runtime by Pydantic validation.

         The emitted Python wraps the block's return value in:
           ReturnType.model_validate(result)

         This is documented as a SEMANTIC RULE, not a grammar rule:
           "The return value of a python %{ }% block must be compatible
            with the tool's declared return type. [sem: python-return-contract]"
```

---

## Section 6 — Capability System Decisions

### 6.1 Capability Name Registry

```
[CLOSED] Capability names are OPEN IDENTIFIERS validated by semantic analysis.

         ANY valid identifier is grammatically acceptable as a capability name.
         The semantic analysis pass validates against a CAPABILITY REGISTRY.

         BUILT-IN REGISTERED CAPABILITIES (v0.1 registry):
           json_mode       Model can return structured JSON output
           tools           Model supports tool/function calling
           vision          Model accepts image inputs
           streaming       Model supports token streaming
           reasoning       Model supports extended reasoning chains

         REGISTRY IS EXTENSIBLE: New capabilities can be added to the registry
         without grammar changes. This is a deliberate design choice allowing
         the language to track the rapidly evolving LLM capability landscape.

         Unknown capability: CAP001 warning (not error) in v0.1.
         This allows engineers to declare capabilities before the registry
         is updated, using the --strict-caps flag to promote to error.
```

### 6.2 Requires Clause Syntax

```
[CLOSED] EITHER form is valid:

         Single capability — NO brackets:
           requires json_mode

         Multiple capabilities — WITH brackets:
           requires [json_mode, tools]
           requires [json_mode, tools, vision]

         Empty brackets:
           requires []       — equivalent to no requires clause (zero requirements)

         Both forms parse to the same AST node:
           RequiresClause { capabilities: Vec<Symbol> }

[GRAMMAR IMPACT]:
  requiresClause ::= "requires" ( IDENT
                                | "[" ( IDENT ( "," IDENT )* )? "]" )
```

### 6.3 Capability Mismatch Severity

```
[CLOSED] Capability mismatch at a call site is a FATAL COMPILE-TIME ERROR.

         ERROR CODE: CAP010
         MESSAGE:    "Model '{model_name}' is missing required capabilities: [{caps}]
                     Required by prompt '{prompt_name}' at line {line}:{col}
                     Hint: Model '{model_name}' supports: [{model_caps}]
                     Add the missing capabilities to the model declaration, or
                     use a model that supports {missing_caps}."

         DEFENSE IN DEPTH: Even though the capability check is compile-time,
         the compiler also emits a RUNTIME GUARD in the generated Python:
           if not model.has_caps(prompt.requires):
               raise CapabilityError(...)

         This catches cases where EAML-compiled code is called from
         non-EAML code that bypasses the type system.
```

---

## Section 7 — Syntax Decisions: Declarations

### 7.1 Schema Body

```
[CLOSED] Schema fields are separated by EITHER newline OR comma.
         Trailing separator (final comma or trailing newline) is ALLOWED.

         Both of these are valid:

           schema SentimentResult {
             sentiment:  "positive" | "negative" | "neutral"
             confidence: float<0.0, 1.0>
             reasoning:  string<max: 200>
             flags:      Tag[]?
           }

           schema SentimentResult {
             sentiment:  "positive" | "negative" | "neutral",
             confidence: float<0.0, 1.0>,
             reasoning:  string<max: 200>,
             flags:      Tag[]?,
           }

[GRAMMAR IMPACT]:
  schemaBody ::= "{" ( fieldDef ( "," | NL ) )* fieldDef? "}"
               | "{" NL? "}"  /* empty schema */
```

### 7.2 Forward References

```
[CLOSED] FORWARD REFERENCES ARE ALLOWED.

         A prompt may reference a schema declared later in the same file.
         A tool may reference a schema declared before or after it.
         Declaration order within a file is irrelevant to correctness.

         IMPLEMENTATION: Two-pass name resolution.
           Pass 1: Collect all declaration names into the global symbol table.
           Pass 2: Resolve all references against the populated symbol table.

         This is the standard approach in all modern languages (Go, Rust, Swift).
```

### 7.3 Prompt Body Required and Optional Fields

```
[CLOSED] PROMPT BODY FIELD RULES:

         REQUIRED:
           user: templateStr      — must be present

         OPTIONAL:
           system: templateStr    — LLM system message
           temperature: float     — generation temperature
           max_tokens: int        — maximum output tokens
           max_retries: int       — validation retry count (default: 2)

         FIELD ORDER: Any order is accepted. Validated in semantic analysis.

         DEFAULT VALUES:
           max_retries  — 2 when not specified  [CLOSED]
           temperature  — not set (uses provider default) when not specified
           max_tokens   — not set (uses provider default) when not specified

[GRAMMAR IMPACT]:
  promptField ::= ( "user"        ":" templateStr )
                | ( "system"      ":" templateStr )
                | ( "temperature" ":" FLOAT )
                | ( "max_tokens"  ":" INT )
                | ( "max_retries" ":" INT )
```

### 7.4 Tool Return Type

```
[CLOSED] Tools with NO MEANINGFUL RETURN VALUE use '-> null'.
         'void' is NOT a keyword in v0.1.

         tool LogEvent(message: string) -> null {
           python %{
             logger.info(message)
           }%
         }

         Post-MVP: 'void' will be added as a predeclared identifier synonym
         for null in the return type position.
```

### 7.5 Tool Native Body

```
[POST-MVP] Tool bodies with native EAML statement implementations.

           tool Add(a: int, b: int) -> int {
             return a + b
           }

           In v0.1: ALL tool implementations must use python %{ }%.
           A tool body without 'python %{' is a parse error in v0.1:
           SYN050: "Native tool bodies are not supported in EAML v0.1.
           Use python %{ }% for tool implementations."
```

---

## Section 8 — Module System Decisions

### 8.1 Multi-File Import (v0.1)

```
[CLOSED] Multi-file imports ARE SUPPORTED in v0.1.

         SYNTAX:
           import "./shared-schemas.eaml"
           import "./models.eaml" as models

         RULES:
           - Import path is relative to the current file's location
           - Circular imports are a compile error: SYN060
           - All imported declarations are merged into the same global namespace
           - No namespace prefixing in v0.1 (Post-MVP: 'as models' may add prefix)
           - Python imports in imported files are merged with the importing file's
             Python imports for the emitted output

         This syntax is DISTINCT from Python bridge imports:
           import "./shared.eaml"          — EAML file import
           import python "pandas" as pd    — Python library import

[GRAMMAR IMPACT]:
  importDecl ::= "import" ( STRING                    /* EAML file import */
                           | "python" STRING ( "as" IDENT )? )  /* Python import */
                 ";"?
```

---

## Section 9 — Error System Decisions

### 9.1 Error Code Format

```
[CLOSED] Error codes use CATEGORY PREFIX format:

         CATEGORY  RANGE    DESCRIPTION
         --------  -----    -----------
         SYN       001-099  Syntax errors — lexer and parser
         SEM       001-099  Semantic analysis errors — name resolution,
                            type checking, annotation consistency
         CAP       001-099  Capability system errors
         TYP       001-099  Type system errors — type mismatches,
                            unknown types, shadowing warnings
         PYB       001-099  Python bridge errors — import issues,
                            block delimiter problems
         RES       001-099  Name resolution errors — undefined names,
                            duplicate declarations

         Examples:
           SYN042   Parse error — unexpected token
           SEM010   Python import after declaration
           CAP010   Capability mismatch at call site
           TYP001   Built-in type shadowing warning
           PYB001   Python block never closed
           RES001   Undefined name reference

         Error format in output:
           error[CAP010]: Model 'BasicModel' missing required capabilities: [json_mode]
             --> sentiment.eaml:28:32
             |
          28 |   let result = BasicModel.call(AnalyzeSentiment(text: "..."))
             |                ^^^^^^^^^^
             = note: 'BasicModel' supports: [tools]
             = help: Add 'json_mode' to BasicModel's caps list, or use a compatible model
```

### 9.2 Compiler Error Accumulation

```
[CLOSED] Compiler accumulates errors up to a MAXIMUM of 20.

         After 20 errors, compilation halts with:
           "aborting due to 20 previous errors
            Fix the above and recompile."

         This matches Rust/Clang behavior. Engineers who hit the limit
         during large refactors can use --max-errors N to override.

         Cascading errors (phantom errors caused by an earlier unresolved name)
         are suppressed using ErrorNode propagation — downstream checks on
         nodes containing ErrorNode do not emit additional diagnostics.
```

---

## Section 10 — Runtime and Codegen Decisions

### 10.1 Target Runtime

```
[CLOSED] Generated code targets:
           Python 3.11+   minimum version
           Pydantic v2    only (no v1 support or dual-version codegen)

         Pydantic v2 APIs used in generated output:
           BaseModel                              schema classes
           model_json_schema()                   JSON Schema emission
           model_validate_json(raw)              response parsing
           Annotated[float, Field(ge=0, le=1)]   bounded types
           Literal["a", "b", "c"]                literal unions
           Optional[T]                            optional types
           list[T]                                array types
```

### 10.2 Provider-Agnostic Model Architecture

```
[CLOSED] EAML is PROVIDER-AGNOSTIC. Model declarations specify a provider
         string that the runtime resolves to the appropriate SDK.

         SYNTAX:
           model Claude = Model(
             id: "claude-sonnet-4-20250514",
             provider: "anthropic",
             caps: [json_mode, tools, vision]
           )

           model GPT4o = Model(
             id: "gpt-4o",
             provider: "openai",
             caps: [json_mode, tools, vision]
           )

           model Local = Model(
             id: "llama3.2",
             provider: "ollama",
             caps: [tools]
           )

         BUILT-IN PROVIDER STRINGS (v0.1):
           "anthropic"    Anthropic SDK (anthropic-sdk-python)
           "openai"       OpenAI SDK (openai-python)
           "ollama"       Ollama local API (httpx direct)

         Unknown provider string: PYB010 warning at compile time,
         runtime error if called.

         RATIONALE: Capability declarations abstract over provider specifics.
         Adding new providers (Gemini, Bedrock) requires no grammar changes —
         only a new runtime resolver entry.

[GRAMMAR IMPACT]:
  modelDecl ::= "model" IDENT "=" "Model" "("
                "id" ":" STRING ","
                "provider" ":" STRING ","
                "caps" ":" "[" capList? "]"
                ")" ";"?
```

### 10.3 Validation Retry Behavior

```
[CLOSED] Runtime validation retry configuration:

         DEFAULT: max_retries = 2 (when not specified in prompt body)
         MINIMUM: max_retries = 0 (hard fail — no retry)
         MAXIMUM: No upper limit enforced by compiler. Values > 10 emit
                  warning TYP040: "max_retries value {n} is unusually high"

         RETRY MECHANISM:
           On Pydantic ValidationError:
             1. Append error message to conversation:
                "Your previous response failed validation: {error}.
                 Please correct and respond again."
             2. Retry the API call
             3. After max_retries exhausted: raise LLMLValidationError

         This behavior is emitted verbatim by codegen via the
         validateOrRetry() runtime function.
```

---

## Section 11 — Post-MVP Reserved Syntax Inventory

The following constructs are **grammatically reserved** in v0.1 — they are
blocked as identifiers or produce specific Post-MVP error messages — but are
not implemented.

```
[POST-MVP] pipeline declaration:
           grammar keyword 'pipeline' is reserved.
           pipeline Foo { ... }  → SYN080: "pipeline declarations are Post-MVP"

[POST-MVP] >> pipeline operator:
           '>>' token is reserved.
           a >> b  → SYN081: "pipeline operator >> is Post-MVP"

[POST-MVP] enum declaration:
           keyword 'enum' is reserved.
           enum Status { ... }  → SYN082: "enum declarations are Post-MVP"

[POST-MVP] schema inheritance:
           keyword 'extends' is reserved.
           schema B extends A { ... }  → SYN083: "schema inheritance is Post-MVP"

[POST-MVP] type inference on let bindings:
           Required in v0.1: let result: SentimentResult = ...
           Without annotation: SEM050: "Type annotation required on let bindings
           in EAML v0.1. Hint: let {name}: {inferred_type} = ..."
           (compiler attempts type inference to provide the hint, even in v0.1)

[POST-MVP] void keyword:
           'void' is reserved as a future predeclared type identifier.
           A schema named 'void' emits TYP002: "void is reserved as a future
           type name in EAML. Use a different name."

[POST-MVP] tool native bodies (non-Python):
           tool bodies without 'python %{' → SYN050 (see Section 7.5)

[POST-MVP] /// doc comments:
           Parsed and silently discarded in v0.1. No tooling support yet.

[POST-MVP] @description and @alias field annotations:
           '@' sigil is reserved. Any '@' in a non-import context →
           SYN090: "@annotations are Post-MVP in EAML v0.1"
```

---

## Section 12 — Ambiguity Resolution Record

All known grammar conflict points and their official resolutions.

```
[CLOSED] CONFLICT: '<' as type parameter opener vs. comparison operator

  RESOLUTION: Context flag 'parsing_type_expr' in parser state.
    - Set to TRUE when parser enters a typeExpr production
    - '<' is a type parameter opener ONLY when flag is TRUE
      AND the preceding token is a type name IDENT
    - '<' is a comparison operator in all other positions
    - Annotated: [lex: angle-bracket-disambiguation]

[CLOSED] CONFLICT: '{' as tool body vs. python %{ }% block start

  RESOLUTION: After consuming '{' in toolBody, peek at next token.
    - If next token is 'python': enter python %{ }% branch
    - Otherwise: enter statement list branch
    - Left-factored in grammar. LL(1) after factoring.

[CLOSED] CONFLICT: '{' as interpolation vs. '{{' as literal brace escape

  RESOLUTION: LEXER-LEVEL. In TEMPLATE_STRING mode:
    - '{{' → emit TMPL_TEXT("{")
    - '}}' → emit TMPL_TEXT("}")
    - '{' followed by expression → emit TMPL_INTERP(expr)
    - '{' followed by invalid → lexer error PYB020
    The parser never sees raw '{' tokens inside template strings.

[CLOSED] CONFLICT: '-' as subtraction vs. unary negation

  RESOLUTION: Pratt parser — '-' in prefix position (no left-hand
  expression) → unary negation. '-' in infix position → subtraction.
  The lexer always emits MINUS token regardless of position.

[CLOSED] CONFLICT: 'requires' clause optional — ε/FOLLOW safety

  RESOLUTION: Verified safe.
    FIRST(requiresClause) = { "requires" }
    FOLLOW(requiresClause in promptDecl) = { "->" }
    Intersection = ∅. LL(1). No conflict.

[CLOSED] CONFLICT: '|' as type union separator vs. potential operator

  RESOLUTION: '|' has NO meaning in expression position.
    In type expression context: '|' is a union separator
    In expression context: '|' is a SYNTAX ERROR with hint:
      "Did you mean '||' for logical OR?"
    Bitwise OR does not exist in EAML.

[CLOSED] CONFLICT: 'import' for EAML file vs. Python library

  RESOLUTION: Second token disambiguates:
    import "..."         → EAML file import (STRING follows)
    import python "..."  → Python library import (keyword 'python' follows)
    LL(2) point — requires two tokens to decide.
```

---

## Section 13 — Design Decisions Summary Table

Quick-reference for all closed decisions. For AI use: treat every row as
a constraint that cannot be changed without explicit designer override.

| Category         | Decision           | Value                                                 |
|------------------|--------------------|-------------------------------------------------------|
| Identity         | Full name          | Engineering AI Markup Language                        |
| Identity         | Abbreviation       | EAML                                                  |
| Identity         | File extension     | `.eaml`                                               |
| Identity         | Compiler binary    | `eamlc`                                               |
| Identity         | Version scheme     | Semantic (v0.1.0)                                     |
| Identity         | Current version    | 0.1.0                                                 |
| Lexical          | Encoding           | UTF-8                                                 |
| Lexical          | Identifier charset | ASCII `[a-zA-Z_][a-zA-Z0-9_]*`                        |
| Lexical          | Case sensitivity   | Fully case-sensitive                                  |
| Lexical          | Semicolons         | Optional                                              |
| Lexical          | Comments           | `//` `/* */` `///` (reserved)                         |
| Lexical          | Integer literals   | Decimal, no leading zeros                             |
| Lexical          | Float literals     | Requires digit before `.`                             |
| Lexical          | Negative numbers   | Always unary minus operator, never token              |
| Type system      | Primitive names    | lowercase: `string float int bool null`               |
| Type system      | Typing discipline  | Nominal                                               |
| Type system      | `Tag[]?` meaning   | Optional(Array(Tag)) — whole array optional           |
| Type system      | `Tag?[]` meaning   | Array(Optional(Tag)) — elements optional              |
| Type system      | Multi-dim arrays   | Single-dimension only in v0.1                         |
| Type system      | `void`             | Not in v0.1 — use `-> null`                           |
| Type system      | Literal union min  | Two members required                                  |
| Template strings | Interpolation      | `{expr}` single braces                                |
| Template strings | Literal brace      | `{{` and `}}` escapes                                 |
| Template strings | Slot content       | Full expressions in v0.1                              |
| Template strings | Multi-line         | Allowed, verbatim                                     |
| Python bridge    | Delimiter          | `python %{ ... }%`                                    |
| Python bridge    | Import location    | File-level only, before declarations                  |
| Python bridge    | Syntax validation  | `--check-python` flag, off by default                 |
| Capabilities     | Name system        | Open identifiers, semantic validation                 |
| Capabilities     | Requires syntax    | Either: bare or `[list]`                              |
| Capabilities     | Mismatch severity  | Fatal compile-time error (CAP010)                     |
| Capabilities     | Runtime guard      | Always emitted as defense-in-depth                    |
| Error system     | Code format        | Category prefix: SYN, SEM, CAP, TYP, PYB, RES         |
| Error system     | Max errors         | 20 (override with `--max-errors N`)                   |
| Error system     | Error recovery     | Accumulate + ErrorNode propagation                    |
| Schema           | Field separator    | Newline OR comma, trailing allowed                    |
| Schema           | Empty schema       | Valid — `schema Foo {}` compiles                      |
| Prompt           | Required field     | `user:`                                               |
| Prompt           | Optional fields    | `system:` `temperature:` `max_tokens:` `max_retries:` |
| Prompt           | Default retries    | 2                                                     |
| Prompt           | Retry mechanism    | validateOrRetry() with error-appended prompt          |
| Forward refs     | Within file        | Allowed — two-pass name resolution                    |
| Module system    | v0.1 scope         | Multi-file with explicit `import "./path.eaml"`       |
| Runtime          | Python floor       | 3.11+                                                 |
| Runtime          | Pydantic           | v2 only                                               |
| Runtime          | Provider model     | Provider-agnostic — runtime resolves by string        |
| Runtime          | Built-in providers | `"anthropic"` `"openai"` `"ollama"`                   |

---

## Section 14 — Grammar Rules Specific to EAML

Rules that apply ONLY to EAML and are not derivable from any Layer 1–4
reference. Treat these as axioms.

```
RULE EG-01: 'python' is a full keyword, not a contextual keyword.
            No identifier named 'python' is valid anywhere in EAML.

RULE EG-02: '}%' is the ONLY valid closing delimiter for python blocks.
            The lexer scans for this two-character sequence only.
            There are no known cases where real Python code contains '}%'.

RULE EG-03: The '->' arrow appears ONLY in declaration return type positions.
            There are no function types (String -> Int) in EAML v0.1.
            '->' in expression context is a parse error.

RULE EG-04: Object literals { } do NOT exist in EAML.
            All '{' characters in expression context open a block statement.
            Structured data uses schema declarations, not inline object syntax.

RULE EG-05: Bitwise operators (&, |, ^, ~, <<, >>) do NOT exist in EAML.
            '|' is exclusively a type union separator.
            '&' is exclusively part of '&&' (logical AND).
            Any single '|' in expression context is: SYN043 — use '||'
            Any single '&' in expression context is: SYN044 — use '&&'

RULE EG-06: Chained comparisons are a SEMANTIC ERROR.
            'a == b == c' parses to a tree but is rejected with:
            SEM060: "Chained comparisons are not supported in EAML.
            Hint: Use: a == b && b == c"

RULE EG-07: Python import declarations PRECEDE all other declarations.
            Any 'import python' appearing after a non-import declaration
            is SEM010: "Python imports must appear at the top of the file."

RULE EG-08: Schema field names are validated as identifiers, not keywords.
            A field named 'model' inside a schema is valid EAML because
            field names are parsed in a non-keyword context.
            Exception: 'python' (EG-01) — a field named 'python' is invalid.

RULE EG-09: The EAML capability registry is case-sensitive.
            'json_mode' and 'JSON_MODE' are different capability names.
            The built-in registry uses snake_case only.

RULE EG-10: 'import "./path.eaml"' uses RELATIVE PATHS only in v0.1.
            Absolute paths and URL-scheme paths are Post-MVP.
            Paths starting with '/' or containing ':' are SYN061.
```

---

*EAML Layer 5 Design Decisions — Version 0.1.0 — 2026-03-14*
*Compiled from designer Q&A session.*
*This document supersedes any conflicting guidance in Layers 1–4.*
*All decisions marked [CLOSED] are final for EAML v0.1.0.*