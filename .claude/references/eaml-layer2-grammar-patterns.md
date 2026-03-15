# EAML Grammar Patterns Reference — Layer 2
## Real W3C EBNF Grammars — Pattern Extraction for EAML Development

---

## Document Purpose and Usage Instructions

This document is **Layer 2** of the EAML grammar reference stack. Where Layer 1
defined the notation operators, Layer 2 shows those operators applied to real,
production W3C grammars that solve the same structural problems EAML faces.

**Two sources, two different lessons:**

| Source                     | Primary EAML Lesson                                                                                                        | URL                                   |
|----------------------------|----------------------------------------------------------------------------------------------------------------------------|---------------------------------------|
| XPath 2.0 (Second Edition) | How to encode operator precedence as grammar structure — the **stratified hierarchy pattern**                              | https://www.w3.org/TR/xpath20/        |
| SPARQL 1.1 Query Language  | How to mix top-level declaration forms with expression-level queries in one grammar — the **declaration dispatch pattern** | https://www.w3.org/TR/sparql11-query/ |

**How to use this document:**
- Do not copy these grammars into EAML. Extract the *patterns* they demonstrate.
- Each section identifies the pattern, shows it in its source grammar, and maps
  it to the corresponding EAML problem it solves.
- Sections 5 and 6 identify what these grammars do that EAML must NOT replicate.

**Prerequisite:** Layer 1 (`eaml-layer1-notation-reference.md`) must be loaded
before this document in any AI grammar session. The notation operators used here
are defined there.

---

## Section 1 — Why These Two Grammars

### 1.1 XPath 2.0

XPath 2.0 is the gold standard for expression grammar in a W3C EBNF language.
It has:

- A complete, rigorously tested **operator precedence hierarchy** expressed
  purely through grammar structure — no precedence tables, no Pratt magic
  in the grammar itself
- A clean **type expression** sub-grammar (`SequenceType`) embedded within
  expression rules — directly analogous to EAML's `typeExpr`
- Explicit **function call syntax** with optional argument lists
- **Member access** (postfix `.` navigation) chained from call results
- A **whitespace-implicit** design identical to what EAML uses

XPath uses the same basic EBNF notation used in XML 1.0.
Each kind of expression is defined in terms of other expressions whose operators
have higher precedence. In this way, the precedence of operators is represented
explicitly in the grammar.

This is the most important sentence in either reference document. It is the
complete theory behind the stratified hierarchy pattern.

### 1.2 SPARQL 1.1

SPARQL is relevant because it solves a structural problem XPath doesn't have:
it must handle **multiple top-level declaration forms** (`PREFIX`, `SELECT`,
`CONSTRUCT`, `ASK`, `DESCRIBE`) dispatched from a single root production,
each with their own body grammar — exactly the pattern EAML uses for
`model`, `schema`, `prompt`, `tool`, `agent`, `pipeline`.

The SPARQL grammar is LL(1) when the rules with uppercased
names are used as terminals. There are two entry points into the grammar:
`QueryUnit` for SPARQL queries, and `UpdateUnit` for SPARQL Update requests.
White space (production `WS`) is used to separate two terminals which would
otherwise be mis-recognized as one terminal.

SPARQL's explicit LL(1) design and its `WS`-as-separator convention are
directly applicable to EAML.

---

## Section 2 — The Stratified Hierarchy Pattern (from XPath 2.0)

### 2.1 What It Is

The stratified hierarchy is the standard technique for encoding operator
precedence in a context-free grammar without ambiguity. Each precedence
level becomes its own non-terminal. A rule at level N references only
rules at level N+1 (the next higher precedence), plus its own operator.

The result: the grammar structure **is** the precedence table. No external
priority numbers are needed. No Pratt parsing in the grammar specification
(though Pratt is fine in the implementation).

### 2.2 The XPath 2.0 Hierarchy — Verbatim

Source: XPath 2.0 Second Edition, Appendix A.1 EBNF
URL: https://www.w3.org/TR/xpath20/#id-grammar

The complete expression precedence hierarchy from lowest to highest:

```ebnf
/* [14] Lowest precedence — or */
OrExpr ::= AndExpr ( "or" AndExpr )*

/* [15] */
AndExpr ::= ComparisonExpr ( "and" ComparisonExpr )*

/* [16] Comparison — non-associative (note: ? not *) */
ComparisonExpr ::= RangeExpr ( ( ValueComp
                               | GeneralComp
                               | NodeComp ) RangeExpr )?

/* [17] */
RangeExpr ::= AdditiveExpr ( "to" AdditiveExpr )?

/* [18] */
AdditiveExpr ::= MultiplicativeExpr
                 ( ( "+" | "-" ) MultiplicativeExpr )*

/* [19] */
MultiplicativeExpr ::= UnionExpr
                       ( ( "*" | "div" | "idiv" | "mod" ) UnionExpr )*

/* [20] */
UnionExpr ::= IntersectExceptExpr
              ( ( "union" | "|" ) IntersectExceptExpr )*

/* [21] */
IntersectExceptExpr ::= InstanceofExpr
                        ( ( "intersect" | "except" ) InstanceofExpr )*

/* [22] */
InstanceofExpr ::= TreatExpr ( "instance" "of" SequenceType )?

/* [23] */
TreatExpr ::= CastableExpr ( "treat" "as" SequenceType )?

/* [24] */
CastableExpr ::= CastExpr ( "castable" "as" SingleType )?

/* [25] */
CastExpr ::= UnaryExpr ( "cast" "as" SingleType )?

/* [26] — Unary — right to left, prefix */
UnaryExpr ::= ( "-" | "+" )* ValueExpr

/* [27] — Postfix — highest binary precedence */
ValueExpr ::= PathExpr

/* [28] — Function call and member access — highest */
PrimaryExpr ::= Literal
              | VarRef
              | ParenthesizedExpr
              | ContextItemExpr
              | FunctionCall

/* Function call */
FunctionCall ::= QName "(" ( ExprSingle ( "," ExprSingle )* )? ")"

/* Parenthesized grouping */
ParenthesizedExpr ::= "(" Expr? ")"
```

### 2.3 The Three Key Structural Patterns

Study these three patterns. They appear repeatedly in the hierarchy above
and you will write each one in EAML.

---

**Pattern A — Left-Associative Binary Operator**

Used for `+`, `-`, `*`, `/`, `&&`, `||` — operators that chain left to right
and may appear zero or more times at that precedence level.

```ebnf
/* Template */
LevelN ::= LevelN1 ( op LevelN1 )*

/* XPath example: additive */
AdditiveExpr ::= MultiplicativeExpr
                 ( ( "+" | "-" ) MultiplicativeExpr )*

/* EAML equivalent: additive (same pattern) */
additiveExpr ::= multiplicativeExpr
                 ( ( "+" | "-" ) multiplicativeExpr )*
```

The `( op LevelN1 )*` suffix is key: it allows zero or more chained
operations at this precedence level, each referencing the *next higher*
level for its operands.

---

**Pattern B — Non-Associative Binary Operator**

Used for comparison operators (`==`, `!=`, `<`, `>`, `<=`, `>=`) that must
NOT chain. `a < b < c` is a grammar error in languages that use this pattern.

```ebnf
/* Template */
LevelN ::= LevelN1 ( op LevelN1 )?    /* NOTE: ? not * */

/* XPath example: comparison */
ComparisonExpr ::= RangeExpr ( ( ValueComp
                               | GeneralComp
                               | NodeComp ) RangeExpr )?

/* EAML equivalent: comparison */
comparisonExpr ::= additiveExpr
                   ( ( "==" | "!=" | "<" | ">" | "<=" | ">=" )
                     additiveExpr )?
```

The `?` instead of `*` is the only difference from Pattern A, but it
completely changes the semantics. With `?`, the operator can appear at
most once — chaining is structurally impossible.

---

**Pattern C — Unary Prefix Operator**

Used for `!`, unary `-`, `await` — operators that apply to a single
following operand.

```ebnf
/* Template */
UnaryExpr ::= op* NextHigherLevel      /* zero or more prefix ops */

/* XPath example */
UnaryExpr ::= ( "-" | "+" )* ValueExpr

/* EAML equivalent */
unaryExpr ::= ( "!" | "-" )* awaitExpr

awaitExpr ::= "await"? postfixExpr     /* await is optional prefix */
```

The `op*` prefix allows stacking (e.g., `!!x`, `--x`) without left recursion.

---

### 2.4 Complete EAML Expression Hierarchy Derived from XPath Pattern

Apply the three patterns to EAML's operator set:

```ebnf
/* ── EAML Expression Grammar (derived from XPath 2.0 pattern) ── */

/* Level 1: Lowest precedence — logical or */
/* Pattern A: left-associative */
orExpr ::= andExpr ( "||" andExpr )*

/* Level 2 */
/* Pattern A: left-associative */
andExpr ::= comparisonExpr ( "&&" comparisonExpr )*

/* Level 3 — comparison */
/* Pattern B: non-associative — use ? not * */
/* [sem: no-chained-comparison] */
comparisonExpr ::= additiveExpr
                   ( ( "==" | "!=" | "<" | ">" | "<=" | ">=" )
                     additiveExpr )?

/* Level 4 */
/* Pattern A: left-associative */
additiveExpr ::= multiplicativeExpr
                 ( ( "+" | "-" ) multiplicativeExpr )*

/* Level 5 */
/* Pattern A: left-associative */
multiplicativeExpr ::= unaryExpr
                       ( ( "*" | "/" ) unaryExpr )*

/* Level 6 — unary prefix */
/* Pattern C: prefix */
unaryExpr ::= ( "!" | "-" )* awaitExpr

/* Level 7 — await prefix (optional) */
awaitExpr ::= "await"? postfixExpr

/* Level 8 — postfix: member access, call, index */
/* Pattern A variant: left-associative suffix chain */
postfixExpr ::= primaryExpr suffix*

suffix ::= ( "." IDENT )             /* member access:  x.field   */
         | ( "(" argList? ")" )       /* function call:  f()       */
         | ( "[" expr "]" )           /* index access:   a[i]      */

/* Level 9: Highest — primary expressions */
primaryExpr ::= literal
              | IDENT
              | "(" expr ")"          /* grouping */
              | callExpr

/* Call expression: ident followed by ( = function call   */
/* Context: distinguished from bare ident by lookahead    */
callExpr ::= IDENT "(" argList? ")"

argList ::= expr ( "," expr )*

/* Root expression entry point */
expr ::= orExpr

/* ── Literals ── */
literal ::= INT | FLOAT | STRING | boolLit | nullLit
boolLit ::= "true" | "false"
nullLit ::= "null"
```

### 2.5 The Type Expression Sub-Grammar (XPath `SequenceType` Pattern)

XPath embeds type annotations inside expression rules using a `SequenceType`
non-terminal. This is exactly what EAML needs for `let result: SentimentResult`.

```ebnf
/* XPath pattern (simplified) */
/* [55] */
SequenceType ::= ( "empty-sequence" "(" ")" )
               | ( ItemType OccurrenceIndicator? )

/* [57] */
OccurrenceIndicator ::= "?" | "*" | "+"

/* [58] */
ItemType ::= KindTest
           | ( "item" "(" ")" )
           | AtomicType      /* <- this is the analog of EAML's namedType */
```

**EAML analog** — the `typeExpr` grammar mirrors this structure:

```ebnf
/* EAML type expression — follows XPath SequenceType pattern */

typeExpr ::= baseType boundedSuffix? arraySuffix? optionalSuffix?

baseType ::= namedType
           | literalUnion
           | "(" typeExpr ")"         /* grouping */

/* Named type: schema name or primitive */
namedType ::= IDENT ( "<" boundParams ">" )?

/* Bounded primitive: Float<0.0, 1.0> or String<max: 200> */
boundedSuffix ::= "<" boundParams ">"
boundParams   ::= boundParam ( "," boundParam )*
boundParam    ::= ( IDENT ":" )? ( FLOAT | INT )

/* Array postfix */
arraySuffix ::= "[]"             /* [sem: single-dimension-only-v0.1] */

/* Optional postfix — applies to entire preceding type */
optionalSuffix ::= "?"

/* Literal union — minimum two members */
literalUnion ::= STRING ( "|" STRING )+

/* EAML OccurrenceIndicator: ? applies AFTER [] */
/* Tag[]? = optional array of Tag  (follows XPath postfix pattern)   */
/* Tag?[] = array of optional Tag  (? before [] — not the same)      */
```

---

## Section 3 — The Declaration Dispatch Pattern (from SPARQL 1.1)

### 3.1 What It Is

SPARQL's root production dispatches to different declaration forms based on
a leading keyword. This is exactly the EAML `Program` structure.

Source: SPARQL 1.1 Query Language, Section 19 Grammar
URL: https://www.w3.org/TR/sparql11-query/#grammar

### 3.2 SPARQL Root Dispatch — Verbatim

```ebnf
/* [1] */
QueryUnit ::= Query

/* [2] */
Query ::= Prologue
          ( SelectQuery | ConstructQuery | DescribeQuery | AskQuery )
          ValuesClause

/* [3] Prologue — declaration section before the query body */
Prologue ::= ( BaseDecl | PrefixDecl )*

/* [4] */
BaseDecl ::= "BASE" IRIREF

/* [5] */
PrefixDecl ::= "PREFIX" PNAME_NS IRIREF
```

The structural lesson: `Query` dispatches to four mutually exclusive query
forms. The dispatcher works because each alternative begins with a distinct
leading keyword (`SELECT`, `CONSTRUCT`, `DESCRIBE`, `ASK`). This is
LL(1) — one token of lookahead is sufficient.

### 3.3 EAML Root Dispatch — Derived Pattern

Apply the SPARQL dispatch pattern to EAML:

```ebnf
/* EAML root — follows SPARQL Prologue + Query dispatch */

/* [1] Root production */
Program ::= declaration* EOF

/* [2] Declaration dispatch — LL(1): each alternative starts with  */
/*     a distinct keyword. One token lookahead is always sufficient. */
declaration ::= importDecl        /* "import"   */
              | modelDecl         /* "model"    */
              | schemaDecl        /* "schema"   */
              | promptDecl        /* "prompt"   */
              | toolDecl          /* "tool"     */
              | agentDecl         /* "agent"    */
              | pipelineDecl      /* "pipeline" — Post-MVP */
              | letDecl           /* "let"      */

/* [3] Import declaration */
importDecl ::= "import" "python" STRING ( "as" IDENT )? ";"?
               /* MVP */

/* [4] Model declaration */
modelDecl ::= "model" IDENT "=" "Model" "(" STRING "," "caps" ":"
              "[" capList "]" ")" ";"?
              /* MVP */

capList ::= IDENT ( "," IDENT )*  /* [sem: cap-registry] */

/* [5] Schema declaration */
schemaDecl ::= "schema" IDENT "{" fieldDef* "}" ";"?
               /* MVP */

/* [6] Field definition */
fieldDef ::= IDENT ":" typeExpr ( "," | NL )?
             /* [sem: field-type-must-resolve] */

/* [7] Prompt declaration */
promptDecl ::= "prompt" IDENT "(" paramList? ")"
               requiresClause?
               "->" typeExpr
               promptBody
               /* MVP */

/* [8] Requires clause */
requiresClause ::= "requires" ( IDENT
                              | ( "[" IDENT ( "," IDENT )* "]" ) )
                   /* [sem: cap-registry] */

/* [9] Prompt body */
promptBody ::= "{" promptField* "}"

promptField ::= ( "system" ":" templateStr )
              | ( "user"   ":" templateStr )
              | ( "temperature" ":" FLOAT )
              | ( "max_tokens"  ":" INT )
              /* order-independent [sem: prompt-body-fields] */

/* [10] Tool declaration */
toolDecl ::= "tool" IDENT "(" paramList? ")"
             "->" typeExpr
             toolBody
             /* MVP */

toolBody ::= "{" "description" ":" STRING toolImpl "}"
           | "{" toolImpl "}"

toolImpl ::= pythonBody                       /* Python bridge tool */
           | statement*                       /* Native tool — Post-MVP */

pythonBody ::= "python" "%{" PYTHON_BLOCK "}%"
               /* [lex: python-block-capture]                              */
               /* Delimiter: python %{ ... }% (lex/yacc-style)            */
               /* Lexer scans for two-character sequence '}%' to close.   */
               /* No brace-depth counting required.                       */
               /* Reconciled with Layer 5 §5.1 [CLOSED] decision.        */

/* [11] Agent declaration */
agentDecl ::= "agent" IDENT "{" agentField* "}" ";"?
              /* MVP */

agentField ::= ( "model"     ":" IDENT )
             | ( "tools"     ":" "[" IDENT ( "," IDENT )* "]" )
             | ( "system"    ":" templateStr )
             | ( "max_turns" ":" INT )
             | ( "on_error"  ":" errorPolicy )

errorPolicy ::= "fail"
              | ( "retry" "(" INT ")" "then" "fail" )

/* [12] Pipeline declaration — Post-MVP */
/* pipelineDecl ::= "pipeline" IDENT "{" pipelineBody "}" */

/* [13] Let binding */
letDecl ::= "let" IDENT ":" typeExpr "=" expr ";"?
            /* [sem: let-type-must-match-expr] */
            /* MVP: type annotation required; inference is Post-MVP */
```

### 3.4 The LL(1) Property of the EAML Dispatch

The dispatch works because every declaration begins with a unique keyword:

| First Token  | Declaration    | Lookahead Needed  |
|--------------|----------------|-------------------|
| `"import"`   | `importDecl`   | LL(1)             |
| `"model"`    | `modelDecl`    | LL(1)             |
| `"schema"`   | `schemaDecl`   | LL(1)             |
| `"prompt"`   | `promptDecl`   | LL(1)             |
| `"tool"`     | `toolDecl`     | LL(1)             |
| `"agent"`    | `agentDecl`    | LL(1)             |
| `"pipeline"` | `pipelineDecl` | LL(1) — Post-MVP  |
| `"let"`      | `letDecl`      | LL(1)             |
| `EOF`        | end of program | LL(1)             |

No two declarations share a leading keyword. This is a deliberate design
constraint for EAML — new keywords must preserve this property.

---

## Section 4 — The Whitespace Design (from Both Sources)

Both XPath and SPARQL declare whitespace implicit and then handle explicit
whitespace only as an exception.

### 4.1 XPath Approach

Unless otherwise noted, whitespace is not significant
in expressions. XPath uses a special `/* ws: explicit */` annotation
in the few productions where whitespace IS significant. All other productions
treat whitespace as skipped.

### 4.2 SPARQL Approach

White space (production `WS`) is used to separate two
terminals which would otherwise be mis-recognized as one terminal.
SPARQL only mentions `WS` where it prevents tokenization ambiguity.

### 4.3 EAML Application

Adopt the XPath/SPARQL approach verbatim:

```ebnf
/* EAML whitespace policy — follows XPath default whitespace handling */
/*                                                                      */
/* DEFAULT: WS is skipped between ALL token pairs in ALL productions.  */
/*          Do not write WS? or WS* between elements in any rule.      */
/*          The lexer handles this.                                     */
/*                                                                      */
/* EXCEPTIONS — marked [lex: ws-preserve-X]:                          */
/*   1. Inside PYTHON_BLOCK — preserved verbatim [lex: ws-preserve-python] */
/*   2. Inside template string text segments     [lex: ws-preserve-template] */
/*   3. NL as a field separator in schemaBody and agentBody            */
/*      — NL appears EXPLICITLY in those productions when significant  */
```

The NL-as-separator case is worth expanding:

```ebnf
/* NL as explicit separator — only where semantically significant */

/* Schema fields may be separated by comma OR newline */
schemaBody ::= "{" ( fieldDef ( "," | NL ) )* fieldDef? "}"

/* Agent fields — same pattern */
agentBody ::= "{" ( agentField NL? )* "}"

/* All other productions: NL is WS, WS is skipped, not written */
```

---

## Section 5 — Template String Grammar (XPath Parallel)

XPath has no template strings, but its `StringLiteral` grammar shows the
correct pattern for bounded string content with escape sequences.

```ebnf
/* XPath 2.0 StringLiteral */
StringLiteral ::= ( '"' (EscapeQuot | [^"])* '"' )
                | ( "'" (EscapeApos | [^'])* "'" )

EscapeQuot ::= '""'
EscapeApos ::= "''"
```

**EAML template string** — extends the XPath pattern with interpolation:

```ebnf
/* EAML template string grammar */
/* The lexer emits: TMPL_START (TMPL_TEXT | TMPL_INTERP)* TMPL_END   */

templateStr ::= TMPL_START tmplPart* TMPL_END

tmplPart ::= TMPL_TEXT                  /* raw text segment           */
           | TMPL_INTERP                /* { expr } interpolation     */

/* Lexer definitions for template string tokens:                       */
/*   TMPL_TEXT   ::= ( AnyChar - ["{] | "{{" | "}}" )+               */
/*                   double-brace = literal brace (Python f-str style) */
/*   TMPL_INTERP ::= "{" expr "}"                                     */
/*                   single-brace = interpolation slot                 */
/*                   v0.1: FULL EXPRESSIONS supported                  */
/*                   Lexer performs brace-depth counting to find the   */
/*                   closing '}' at depth zero.                        */
/*                   [lex: tmpl-interp-brace-depth]                    */
/*                   Reconciled with Layer 5 §4.2 [CLOSED] decision.  */
```

---

## Section 6 — Parameter and Argument Grammar (XPath Pattern)

XPath's `FunctionCall` production shows the standard pattern for typed
parameter lists. EAML `paramList` and `argList` follow this exactly.

```ebnf
/* XPath 2.0 FunctionCall */
FunctionCall ::= QName "(" ( ExprSingle ( "," ExprSingle )* )? ")"
```

**EAML parameter and argument lists — derived:**

```ebnf
/* Parameter declaration: in prompt/tool signatures */
paramList ::= param ( "," param )*

param ::= IDENT ":" typeExpr ( "=" literal )?
          /* literal defaults only for MVP          */
          /* [sem: default-must-match-param-type]   */

/* Argument list: at call sites */
argList ::= arg ( "," arg )*

/* Named arguments (keyword style) — both positional and keyword allowed */
arg ::= ( IDENT ":" expr )            /* named:     text: "hello"    */
      | expr                          /* positional: "hello"         */
      /* [sem: no-positional-after-named]                            */
```

---

## Section 7 — Patterns NOT to Replicate

### 7.1 XPath-Specific Constructs — Do Not Use in EAML

| XPath Construct                           | Why Not in EAML                                                                                                         |
|-------------------------------------------|-------------------------------------------------------------------------------------------------------------------------|
| Path steps with `/` and `//`              | XML tree navigation — irrelevant to EAML                                                                                |
| `@` attribute access                      | XML attribute syntax — EAML uses field access                                                                           |
| `::` axis specifiers                      | XPath axes (`child::`, `descendant::`) — no analog                                                                      |
| `$varName` variable syntax                | XPath prefixes variables with `$` — EAML uses bare identifiers                                                          |
| `(: ... :)` comment syntax                | XPath/XQuery comment delimiter — EAML uses `/* */`                                                                      |
| `SequenceType` occurrence (`?`, `*`, `+`) | EAML type occurrence is different — `?` is optional postfix on the whole type, not a cardinality on items in a sequence |
| `instance of`, `treat as`, `cast as`      | XPath type-system operators — EAML has no runtime type casting                                                          |

### 7.2 SPARQL-Specific Constructs — Do Not Use in EAML

| SPARQL Construct                | Why Not in EAML                                                                       |
|---------------------------------|---------------------------------------------------------------------------------------|
| `?varName` variable syntax      | SPARQL prefixes variables with `?` — EAML uses bare IDENT                             |
| `IRI` / `<http://...>` literals | RDF-specific — no analog in EAML                                                      |
| Triple patterns `?s ?p ?o`      | RDF graph patterns — irrelevant to EAML                                               |
| `FILTER` inline with patterns   | Query-time filtering — EAML uses if/else in tool bodies                               |
| Case-insensitive keywords       | SPARQL keywords are case-insensitive — EAML keywords are lowercase and case-sensitive |
| `PREFIX` namespace declarations | RDF-specific — EAML uses `import python` instead                                      |

### 7.3 Both Sources — Implicit `WS` Between Tokens

Both grammars write `WS` explicitly in a small number of ambiguous places
and rely on implicit whitespace skipping everywhere else. **Do not** write
`WS?` between elements in EAML productions. The EAML lexer skips whitespace
before every token, making it unnecessary in grammar rules.

---

## Section 8 — LL(1) Conflict Points Found in the Source Grammars

Both XPath and SPARQL document known LL(k) conflict points. These are the
same classes of conflict EAML will encounter.

### 8.1 XPath — The Function Call vs. Keyword Ambiguity

There are various strategies that can be used by an
implementation to disambiguate token symbol choices. Among the choices are
lexical look-ahead and look-behind, a two-pass lexical evaluation, and a
single recursive descent lexical evaluation and parse.

XPath's solution: certain names (`if`, `for`, `some`, `every`, `typeswitch`)
are **reserved function names** — they cannot be used as user-defined
function names even though they look syntactically identical to function calls.

**EAML application:** Exactly the same problem exists for `model`, `schema`,
`prompt`, etc. appearing in expression context. EAML's solution is identical:
these are reserved keywords, not contextual. A user cannot name a variable
`prompt` or `schema`. The parser always treats them as keyword tokens.

### 8.2 SPARQL — The Signed Number Ambiguity

In signed numbers, no white space is allowed between
the sign and the number. The AdditiveExpression grammar rule allows for this
by covering the two cases of an expression followed by a signed number.

**EAML application:** Same issue. `x - 1` is subtraction but `x -1` could
also be subtraction. Resolution: negative number literals are always
`unaryExpr ::= "-"* primaryExpr`. The lexer never emits negative number
tokens. A `-` is always the unary minus operator. The parser handles
`x - 1` as `AdditiveExpr("x", "-", "1")` regardless of whitespace.

### 8.3 XPath — The `<` Angle Bracket Ambiguity

XPath's approach to the less-than operator vs. XML comparison operators is
to use distinct token names. EAML has the same problem with `Float<0.0, 1.0>`
vs. `x < y`.

**EAML resolution** (not from XPath but grounded by it):
- The EAML parser uses a **context flag**: when in a `typeExpr` parsing
  state that has just consumed a `IDENT` (type name), an immediately
  following `<` is a type parameter opener. In all other states, `<` is
  a comparison operator.
- This is LL(2): on seeing `IDENT` followed by `<`, the parser looks at
  what follows the `<`. If it's a numeric literal or named parameter,
  it's a type constraint. If it's another expression, it's a comparison.
- Document this as `[lex: angle-bracket-disambiguation]`.

---

## Section 9 — Pattern Summary for EAML Grammar Sessions

When an AI is helping write EAML grammar productions, apply these rules
derived from the source grammars:

1. **Use stratified hierarchy for all expression grammar.**
   Every binary operator gets its own non-terminal level. No exceptions.
   The hierarchy IS the precedence table.

2. **Left-associative operators use `( op Next )*`.**
   `+`, `-`, `*`, `/`, `||`, `&&` all use the `A ( op A )*` pattern.

3. **Non-associative operators use `( op Next )?`.**
   `==`, `!=`, `<`, `>`, `<=`, `>=` use the `A ( op A )?` pattern.
   Chained comparisons are structurally impossible with `?`.

4. **Postfix chains use `Primary suffix*`.**
   Member access `.field`, call `()`, and index `[]` all use the
   `PrimaryExpr suffix*` pattern where `suffix` is any postfix operator.

5. **Declaration dispatch is LL(1) by keyword.**
   Every declaration form begins with a unique keyword. Any new declaration
   added to EAML must not share a leading keyword with an existing one.

6. **Type expressions are a separate non-terminal hierarchy.**
   `typeExpr` is never mixed with `expr`. They are always in mutually
   exclusive grammar positions. This is what eliminates the `<`/`>` ambiguity.

7. **Optional suffix for type constraints follows XPath SequenceType.**
   `?` postfix means optional type. `[]` postfix means array type. Both
   are postfix on `baseType`, not mixed into the base type expression.

8. **Whitespace is never written in productions.**
   Follows XPath/SPARQL implicit whitespace handling. `WS` only appears
   in EAML productions when it is semantically significant (NL as separator).

---

## Section 10 — Source Attribution

| Content                      | Source                                 | URL                                              | Status             |
|------------------------------|----------------------------------------|--------------------------------------------------|--------------------|
| Expression hierarchy pattern | XPath 2.0 Second Edition, Appendix A.1 | https://www.w3.org/TR/xpath20/#id-grammar        | W3C Recommendation |
| Whitespace handling design   | XPath 2.0 Second Edition, Appendix A.2 | https://www.w3.org/TR/xpath20/#lexical-structure | W3C Recommendation |
| Declaration dispatch pattern | SPARQL 1.1 Query Language, Section 19  | https://www.w3.org/TR/sparql11-query/#grammar    | W3C Recommendation |
| LL(1) constraint analysis    | SPARQL 1.1 grammar notes               | https://www.w3.org/TR/sparql11-query/#grammar    | W3C Recommendation |
| EAML-specific derivations    | EAML specification (this document)     | N/A — original                                   | EAML Draft 0.1     |

---

*EAML Layer 2 Grammar Patterns Reference — Version 0.1 — 2026-03-14*
*Load after `eaml-layer1-notation-reference.md` in every grammar session.*
*Combine with Layer 3 (prior art: BAML, Lox) and Layer 4 (compiler theory)*
*for complete grammar development context.*