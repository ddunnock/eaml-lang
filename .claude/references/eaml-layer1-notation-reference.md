# EAML Grammar Notation Reference — Layer 1
## AI Grounding Document for EBNF Grammar Development

---

## Document Purpose and Usage Instructions

This document is the **complete notation reference** for writing the EAML
(Engineering AI Markup Language) formal grammar. It combines two W3C sources
into a single self-contained reference:

1. **W3C XML 1.0 (Fifth Edition) — Section 6: Notation**
   The authoritative definition of every operator used in EAML grammar productions.
   Source: https://www.w3.org/TR/xml/#sec-notation
   Status: W3C Recommendation, 26 November 2008. Stable — use without reservation.

2. **W3C XML 1.0 (Fifth Edition) — Section 2.2: Characters (`Char` production)**
   Defines the legal Unicode character set referenced in Section 6.
   Source: https://www.w3.org/TR/xml/#charsets

3. **EAML Extensions** (defined in this document)
   Productions and annotation conventions that Section 6 does not define but
   that EAML grammar requires: `EOF`, `AnyChar`, `NL`, `WS`, and constraint
   annotation forms.

**How to use this document:**
- Every EAML grammar production MUST use only the operators defined in
  Section 6 below, plus the EAML extensions in Section 4.
- When in doubt about operator meaning or precedence, the definitions in
  Section 2 are authoritative.
- The EAML Extensions in Section 4 take precedence over any XML-specific
  interpretations where they conflict.

---

## Section 1 — Naming Conventions

EAML grammar follows these naming rules throughout:

| Symbol Kind                                     | Convention               | Example                        |
|-------------------------------------------------|--------------------------|--------------------------------|
| Non-terminal (start symbol of regular language) | `PascalCase`             | `Program`, `SchemaDecl`        |
| Non-terminal (all others)                       | `camelCase`              | `fieldDef`, `typeExpr`         |
| Terminal (keyword or literal)                   | `'quoted'` or `"quoted"` | `'schema'`, `"true"`           |
| Terminal (named token)                          | `ALL_CAPS`               | `IDENT`, `EOF`, `PYTHON_BLOCK` |
| Comment                                         | `/* ... */`              | `/* skipped by lexer */`       |
| EAML semantic constraint                        | `[sem: name]`            | `[sem: cap-check]`             |
| EAML lexer constraint                           | `[lex: name]`            | `[lex: python-block-capture]`  |
| MVP production                                  | trailing comment         | `/* MVP */`                    |
| Post-MVP production                             | trailing comment         | `/* Post-MVP */`               |

---

## Section 2 — W3C EBNF Notation (Source: XML 1.0 Fifth Edition, Section 6)

> **Verbatim from:** https://www.w3.org/TR/xml/#sec-notation
> W3C Recommendation 26 November 2008
> Copyright © 2008 W3C® (MIT, ERCIM, Keio). All Rights Reserved.

The formal grammar of XML is given in this specification using a simple
Extended Backus-Naur Form (EBNF) notation. Each rule in the grammar defines
one symbol, in the form:

```
symbol ::= expression
```

Symbols are written with an initial capital letter if they are the start
symbol of a regular language, otherwise with an initial lowercase letter.
Literal strings are quoted.

Within the expression on the right-hand side of a rule, the following
expressions are used to match strings of one or more characters:

---

### 2.1 — Terminal Expressions

#### `#xN`
Where `N` is a hexadecimal integer, the expression matches the character
whose number (code point) in ISO/IEC 10646 is `N`. The number of leading
zeros in the `#xN` form is insignificant.

```ebnf
/* Examples */
tab     ::= #x9
lf      ::= #xA
cr      ::= #xD
space   ::= #x20
```

#### `[a-zA-Z]` and `[#xN-#xN]` — Character Range
Matches any `Char` with a value in the range(s) indicated (inclusive).

```ebnf
/* Examples */
asciiLetter  ::= [a-zA-Z]
hexDigit     ::= [0-9a-fA-F]
latinExtended ::= [#xC0-#xD6]
```

#### `[abc]` and `[#xN#xN#xN]` — Character Enumeration
Matches any `Char` with a value among the characters enumerated.
Enumerations and ranges can be mixed in one set of brackets.

```ebnf
/* Examples */
vowel       ::= [aeiouAEIOU]
octalDigit  ::= [0-7]
mixed       ::= [a-fA-F0-9]    /* ranges and enumerations combined */
```

#### `[^a-z]` and `[^#xN-#xN]` — Negated Character Range
Matches any `Char` with a value **outside** the range indicated.

```ebnf
/* Examples */
notLower    ::= [^a-z]
notControl  ::= [^#x00-#x1F]
```

#### `[^abc]` and `[^#xN#xN#xN]` — Negated Character Enumeration
Matches any `Char` with a value **not** among the characters given.
Enumerations and ranges of forbidden values can be mixed in one set of
brackets.

```ebnf
/* Examples */
notBrace    ::= [^{}]
notQuote    ::= [^"'`]
notSpecial  ::= [^{}"'\\]
```

#### `"string"` — Double-Quoted String Literal
Matches the literal string given inside the double quotes.

```ebnf
/* Examples */
kwSchema    ::= "schema"
kwPrompt    ::= "prompt"
arrow       ::= "->"
```

#### `'string'` — Single-Quoted String Literal
Matches the literal string given inside the single quotes.

```ebnf
/* Examples */
kwModel     ::= 'model'
kwTool      ::= 'tool'
```

> **EAML Convention:** Use double-quoted strings `"..."` for all keyword
> and operator terminals throughout the EAML grammar. Single-quoted strings
> `'...'` are reserved for string literals that themselves contain double
> quotes. Do not mix arbitrarily.

---

### 2.2 — Combination Expressions

Where `A` and `B` represent any valid expressions:

#### `(expression)` — Grouping
The expression is treated as a unit and may be combined with other operators.

```ebnf
/* Examples */
ws       ::= (#x20 | #x9 | #xA | #xD)+
separated ::= "a" ("," "a")*
```

#### `A?` — Optional
Matches `A` or nothing. Zero or one occurrence of `A`.

```ebnf
/* Examples */
optSemi     ::= ";"?
optDefault  ::= ("=" literal)?
```

#### `A B` — Concatenation (Sequencing)
Matches `A` followed by `B`. **This operator has higher precedence than
alternation.** Thus `A B | C D` is identical to `(A B) | (C D)`.

```ebnf
/* Examples */
assignment  ::= IDENT "=" expr
arrowType   ::= ")" "->" typeExpr
```

> **Precedence note:** Concatenation binds more tightly than `|`.
> Always use explicit parentheses `( )` when the grouping is not obvious.

#### `A | B` — Alternation
Matches `A` or `B`. Lowest-precedence operator.

```ebnf
/* Examples */
boolLit     ::= "true" | "false"
separator   ::= "," | NL
```

#### `A - B` — Exclusion
Matches any string that matches `A` but does **not** match `B`.

```ebnf
/* Examples */
/* Any character except a double quote or backslash */
strChar     ::= AnyChar - ['"\\]

/* Any character except the sequence '}' at depth zero */
/* (used in Python block capture description) */
nonCloseBrace ::= AnyChar - "}"
```

> **Important:** The exclusion operator `A - B` is one of the most powerful
> operators in W3C EBNF and is absent from ISO 14977. Use it to express
> "any character except X" cleanly rather than complex negated ranges.

#### `A+` — One or More
Matches one or more occurrences of `A`. **Concatenation has higher
precedence than alternation.** Thus `A+ | B+` is identical to `(A+) | (B+)`.

```ebnf
/* Examples */
digits      ::= [0-9]+
identPart   ::= [a-zA-Z0-9_]+
```

#### `A*` — Zero or More
Matches zero or more occurrences of `A`. **Concatenation has higher
precedence than alternation.** Thus `A* | B*` is identical to `(A*) | (B*)`.

```ebnf
/* Examples */
declarations ::= declaration*
fieldList    ::= (fieldDef ","?)*
```

---

### 2.3 — Comment Notation

#### `/* ... */` — Comment
Inline prose comment. May appear anywhere in a production rule.

```ebnf
/* Examples */
WS  ::= (#x20 | #x9 | NL)+  /* skipped between all tokens */
EOF ::= /* end of input stream */
```

---

### 2.4 — Operator Precedence Summary (Highest to Lowest)

| Precedence   | Operator                      | Description                |
|--------------|-------------------------------|----------------------------|
| 1 (highest)  | `#xN` `[...]` `"..."` `'...'` | Terminal atoms             |
| 2            | `(expression)`                | Grouping                   |
| 3            | `A?` `A+` `A*`                | Postfix quantifiers        |
| 4            | `A - B`                       | Exclusion                  |
| 5            | `A B`                         | Concatenation (sequencing) |
| 6 (lowest)   | `A \| B`                      | Alternation                |

> **Key rule:** Concatenation (`A B`) binds more tightly than alternation
> (`A | B`). When mixing them, use parentheses to make grouping explicit.
> `A B | C` means `(A B) | C`, NOT `A (B | C)`.

---

## Section 3 — W3C XML `Char` Production (Source: XML 1.0 Fifth Edition, Section 2.2)

> **Verbatim from:** https://www.w3.org/TR/xml/#charsets
> Defines the set of legal characters that all other productions reference.

The XML 1.0 `Char` production defines every Unicode code point legal in
an XML document:

```ebnf
/* XML 1.0 Fifth Edition — Production [2] */
/* Any Unicode character, excluding the surrogate blocks,       */
/* U+FFFE, and U+FFFF.                                          */
Char ::= #x9
       | #xA
       | #xD
       | [#x20-#xD7FF]
       | [#xE000-#xFFFD]
       | [#x10000-#x10FFFF]
```

**Excluded ranges explained:**

| Range              | Reason for Exclusion                                     |
|--------------------|----------------------------------------------------------|
| `#x0`–`#x8`        | C0 control characters (non-printable, non-whitespace)    |
| `#xB`–`#xC`        | Vertical tab, form feed — not standard XML whitespace    |
| `#xE`–`#x1F`       | Remaining C0 controls                                    |
| `#xD800`–`#xDFFF`  | UTF-16 surrogate pairs — not valid Unicode scalar values |
| `#xFFFE`, `#xFFFF` | Unicode non-characters — forbidden in interchange        |

> **Permitted C0 controls:** Only `#x9` (tab), `#xA` (LF), and `#xD` (CR)
> are legal. These are the three whitespace characters used in `NL` and `WS`.

---

## Section 4 — EAML Grammar Extensions

These productions are **not** from W3C XML 1.0. They are defined here for
EAML and must be included in every EAML grammar session as foundational
terminals. Every other EAML production derives from these.

### 4.1 — `EOF` — End of Input

```ebnf
/* Emitted by the EAML lexer when the input stream is exhausted.    */
/* Every valid EAML program must be derivable from the root         */
/* production and terminate with EOF.                               */
/* Root production: Program ::= declaration* EOF                    */
EOF ::= /* end of input stream */
```

### 4.2 — `AnyChar` — Any Legal EAML Character

```ebnf
/* Any Unicode scalar value legal in an EAML source file.           */
/* Identical to XML 1.0 Char but named AnyChar to avoid confusion   */
/* with XML-specific terminology.                                    */
/* Used in: string literal bodies, Python block capture, template   */
/* string text segments.                                            */
AnyChar ::= #x9
           | #xA
           | #xD
           | [#x20-#xD7FF]
           | [#xE000-#xFFFD]
           | [#x10000-#x10FFFF]
```

### 4.3 — `NL` — Newline

```ebnf
/* Newline in any of three platform conventions.                     */
/* All forms are normalized to LF (#xA) by the EAML lexer before    */
/* token processing. Productions above the lexer level only see LF. */
NL ::= #xA              /* LF   — Unix, macOS                      */
     | (#xD #xA)        /* CRLF — Windows (matched as single NL)   */
     | #xD              /* CR   — legacy macOS (pre-OSX)           */
```

> **Normalization rule:** The EAML lexer normalizes all `NL` forms to a
> single `#xA` (LF). All parser-level productions that reference `NL`
> operate on the normalized form only.

### 4.4 — `WS` — Whitespace

```ebnf
/* One or more whitespace characters.                                */
/* The EAML lexer SKIPS WS between all tokens, with two exceptions: */
/*   [lex: ws-preserve-python]   Inside PYTHON_BLOCK captures       */
/*   [lex: ws-preserve-template] Inside template string text segs   */
WS ::= (#x20 | #x9 | NL)+
```

### 4.5 — `PYTHON_BLOCK` — Opaque Python Source Capture

```ebnf
/* Emitted by the lexer as a single opaque token when the lexer     */
/* enters Python-block capture mode on seeing the keyword 'python'  */
/* followed by '{'.                                                 */
/*                                                                  */
/* Capture algorithm [lex: python-block-capture]:                   */
/*   1. On 'python' '{': enter PYTHON_BLOCK mode, depth = 1         */
/*   2. For each '{' encountered: depth++                           */
/*   3. For each '}' encountered: depth--                           */
/*   4. When depth == 0: emit PYTHON_BLOCK(captured_text), return   */
/*      to normal EAML lexer mode. The captured_text does NOT       */
/*      include the opening or closing braces.                      */
/*   5. AnyChar is preserved verbatim including NL and WS.          */
/*                                                                  */
/* KNOWN LIMITATION (MVP): Brace characters inside Python string    */
/* literals within the block will affect depth counting. Python     */
/* code in python{} blocks must not contain unbalanced literal      */
/* brace characters outside of braced constructs.                   */
/* This is documented as [lex: python-brace-limitation].            */
PYTHON_BLOCK ::= /* opaque token — AnyChar* captured by lexer mode  */
                 /* [lex: python-block-capture]                     */
```

---

## Section 5 — EAML Constraint Annotation Forms

The W3C notation defines `[wfc: ...]` and `[vc: ...]` XML-specific
constraint annotations. **These are NOT used in EAML.** EAML uses the
following annotation forms instead:

### `[sem: name]` — Semantic Constraint

Marks a constraint that the grammar itself cannot enforce. Enforced by
the EAML semantic analysis pass (not the parser).

```ebnf
/* Example: the grammar allows forward references to undeclared     */
/* names; the semantic pass resolves and validates them.            */
returnType ::= typeExpr  /* [sem: return-type-must-resolve] */
```

### `[lex: name]` — Lexer Constraint

Marks a constraint on lexer behavior that cannot be expressed as a
context-free grammar rule.

```ebnf
/* Example: the lexer must track brace depth for Python blocks      */
toolBody ::= "{" "python" "{" PYTHON_BLOCK "}" "}"
           | "{" statement* "}"
           /* [lex: python-block-capture] */
```

---

## Section 6 — Quick Reference Card

For rapid lookup during grammar writing sessions:

```
TERMINAL ATOMS (highest precedence)
  #xN            Unicode code point literal
  [a-z]          Character range (inclusive)
  [^a-z]         Negated character range
  [abc]          Character enumeration
  [^abc]         Negated enumeration
  "str"          Double-quoted string literal  ← EAML preferred
  'str'          Single-quoted string literal

GROUPING
  (expr)         Group as unit

POSTFIX QUANTIFIERS (bind tighter than exclusion/concat/alt)
  A?             Zero or one  (optional)
  A+             One or more
  A*             Zero or more

EXCLUSION
  A - B          A but not B

CONCATENATION  (higher precedence than alternation)
  A B            A followed by B

ALTERNATION    (lowest precedence)
  A | B          A or B

COMMENTS
  /* ... */      Inline comment anywhere

EAML ANNOTATIONS
  [sem: name]    Semantic analysis constraint
  [lex: name]    Lexer-mode constraint

EAML FOUNDATION TERMINALS
  EOF            End of input
  AnyChar        Any legal Unicode scalar value (see Section 4.2)
  NL             Newline — LF | CRLF | CR, normalized to LF
  WS             Whitespace — skipped between tokens (see exceptions)
  PYTHON_BLOCK   Opaque Python source capture token
  IDENT          Identifier — defined in EAML lexical grammar
  INT            Integer literal — defined in EAML lexical grammar
  FLOAT          Float literal — defined in EAML lexical grammar
  STRING         String literal — defined in EAML lexical grammar
```

---

## Section 7 — Key Rules for AI Grammar Assistance

When helping write EAML grammar productions, apply these rules:

1. **Use only the operators in Section 2.** No ISO 14977 commas, no
   `{expr}` repetition syntax, no semicolon terminators on rules.

2. **All EAML grammar rules end with no terminator.** Unlike ISO 14977
   (which uses `;`) or some other notations (which use `.`), W3C EBNF
   rules have no terminator. The next rule begins a new definition.

3. **Exclusion `A - B` is available and preferred** over complex negated
   character enumerations when expressing "any character except X."

4. **`AnyChar` is the base character set**, not `Char`. Use `AnyChar`
   in all EAML productions. Never use the XML `Char` production directly.

5. **`PYTHON_BLOCK` is opaque.** Never write grammar rules that try to
   parse the content of a Python block. It is captured by the lexer as
   raw text. Grammar rules only reference the token `PYTHON_BLOCK`.

6. **`WS` is implicit between tokens.** Do not insert `WS?` or `WS*`
   between elements in a rule — the lexer skips whitespace automatically.
   The two exceptions (Python block interior, template string text) are
   handled by lexer modes, not grammar productions.

7. **`NL` as a significant separator is explicit.** When a grammar rule
   depends on a newline (e.g., as an optional field separator in a schema
   body), `NL` must appear explicitly in that production.

8. **`[sem: ...]` marks deferred constraints.** Do not make the grammar
   more complex than necessary to handle semantic constraints. Mark them
   with `[sem: name]` and let semantic analysis handle them. Example:
   "capability names must be from the registered set" is `[sem: cap-registry]`,
   not a grammar-level restriction.

9. **MVP vs. Post-MVP.** Productions marked `/* Post-MVP */` are
   syntactically reserved but not implemented in the first release.
   When writing MVP grammar, do not reference Post-MVP productions.

10. **Operator precedence is explicit in the grammar structure.**
    Do not rely on a reader "knowing" precedence. When mixing
    concatenation and alternation, use parentheses to make the
    intended grouping unambiguous.

---

## Section 8 — Source Attribution

| Content                         | Source                             | URL                                     | Status             |
|---------------------------------|------------------------------------|-----------------------------------------|--------------------|
| Section 6 Notation operators    | W3C XML 1.0 Fifth Edition §6       | https://www.w3.org/TR/xml/#sec-notation | W3C Recommendation |
| `Char` production               | W3C XML 1.0 Fifth Edition §2.2     | https://www.w3.org/TR/xml/#charsets     | W3C Recommendation |
| `EOF`, `AnyChar`, `NL`, `WS`    | EAML specification (this document) | N/A — original                          | EAML Draft 0.1     |
| `PYTHON_BLOCK`                  | EAML specification (this document) | N/A — original                          | EAML Draft 0.1     |
| `[sem:]` / `[lex:]` annotations | EAML specification (this document) | N/A — original                          | EAML Draft 0.1     |

---

*EAML Layer 1 Notation Reference — Version 0.1 — 2026-03-14*
*For use as AI context grounding document for EAML grammar development.*
*Combine with Layer 2 (design decisions), Layer 3 (prior art), and*
*Layer 4 (compiler theory references) for complete grammar sessions.*
