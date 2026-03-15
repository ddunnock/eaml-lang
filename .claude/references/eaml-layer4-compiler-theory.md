# EAML Grammar Theory Reference — Layer 4
## Compiler Theory: LL(1) Analysis, Left Recursion, FIRST/FOLLOW, Pratt Parsing

---

## Document Purpose and Usage Instructions

This document is **Layer 4** — the final layer of the EAML grammar reference
stack. It covers the formal compiler theory needed to **verify** that a grammar
is correct before implementation begins, and the practical techniques for
**implementing** the parser from that grammar.

**Four topics, four practical deliverables:**

| Topic                          | Deliverable for EAML                                                                                                                                                        |
|--------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| **FIRST and FOLLOW sets**      | The mathematical proof that EAML's grammar is LL(1) or LL(2) at each conflict point                                                                                         |
| **LL(1) grammar conditions**   | The checklist to verify every EAML production rule before writing parser code                                                                                               |
| **Left recursion elimination** | The transformation algorithm for fixing any left-recursive rules found during verification                                                                                  |
| **Pratt parsing**              | The implementation technique for EAML's expression grammar — replaces the stratified hierarchy at the code level while keeping the grammar clean at the specification level |

**Critical prerequisite:** Layers 1–3 must be loaded before this document.
Layer 4 references EAML grammar productions defined in Layer 2 (expression
hierarchy) and Layer 3 (Lox/BAML patterns). The theory here is applied to
those specific productions.

**How to use this document:**
- Section 1–3 are **grammar verification** — apply these before writing any parser code.
- Section 4 (Pratt) is **parser implementation** — apply this when building the parser, not when writing the grammar specification.
- Section 5 applies all four topics specifically to known EAML conflict points.

---

## Section 1 — FIRST and FOLLOW Sets

### 1.1 Definitions

**FIRST(X)** is the set of terminals that can appear as the **first symbol** in
any string derivable from grammar symbol `X`.

Formal definition:
```
FIRST(α) = { t | α ⟹* t β, for some string β }

If α ⟹* ε, then ε ∈ FIRST(α)
```

**FOLLOW(A)** is the set of terminals that can appear **immediately after**
non-terminal `A` in any sentential form derivable from the start symbol.

Formal definition:
```
FOLLOW(A) = { t | S ⟹* α A t β, for some strings α, β }

$ ∈ FOLLOW(S), where S is the start symbol and $ represents EOF
```

### 1.2 FIRST Set Computation Algorithm

Apply these rules exhaustively until no new terminals can be added:

```
For every grammar symbol X:

Rule F1: If X is a terminal:
    FIRST(X) = { X }

Rule F2: If X is a non-terminal and X ::= ε is a production:
    add ε to FIRST(X)

Rule F3: If X is a non-terminal and X ::= Y1 Y2 ... Yn is a production:
    add FIRST(Y1) - {ε} to FIRST(X)
    if ε ∈ FIRST(Y1):
        add FIRST(Y2) - {ε} to FIRST(X)
    if ε ∈ FIRST(Y1) and ε ∈ FIRST(Y2):
        add FIRST(Y3) - {ε} to FIRST(X)
    ... continue until either Yi cannot derive ε, or all Yi derive ε
    if ε ∈ FIRST(Y1) ∩ ... ∩ FIRST(Yn):
        add ε to FIRST(X)

Rule F4: If X ::= A | B | C (alternatives):
    FIRST(X) = FIRST(A) ∪ FIRST(B) ∪ FIRST(C)
```

### 1.3 FOLLOW Set Computation Algorithm

Initialize: `FOLLOW(S) = {$}` where `S` is the start symbol.

Apply these rules exhaustively until no changes occur:

```
For every production A ::= α B β (B appears with something after it):
    add FIRST(β) - {ε} to FOLLOW(B)
    if ε ∈ FIRST(β):
        add FOLLOW(A) to FOLLOW(B)

For every production A ::= α B (B at the end of the right-hand side):
    add FOLLOW(A) to FOLLOW(B)
```

### 1.4 Worked Example — EAML Declaration Dispatch

Apply FIRST computation to the EAML `declaration` rule from Layer 2:

```ebnf
declaration ::= importDecl        /* starts with "import"   */
              | modelDecl         /* starts with "model"    */
              | schemaDecl        /* starts with "schema"   */
              | promptDecl        /* starts with "prompt"   */
              | toolDecl          /* starts with "tool"     */
              | agentDecl         /* starts with "agent"    */
              | letDecl           /* starts with "let"      */
              | exprStmt          /* starts with: IDENT, FLOAT,
                                     INT, STRING, "(", "!", "-",
                                     "await", "true", "false", "null" */
```

**Computing FIRST(declaration):**

```
FIRST(importDecl) = { "import" }
FIRST(modelDecl)  = { "model" }
FIRST(schemaDecl) = { "schema" }
FIRST(promptDecl) = { "prompt" }
FIRST(toolDecl)   = { "tool" }
FIRST(agentDecl)  = { "agent" }
FIRST(letDecl)    = { "let" }
FIRST(exprStmt)   = { IDENT, INT, FLOAT, STRING, "(", "!", "-",
                      "await", "true", "false", "null" }

FIRST(declaration) = union of all above
```

**LL(1) check:** Are any of these sets overlapping?

```
{ "import" } ∩ { "model" }  = ∅  ✓
{ "model" }  ∩ { "schema" } = ∅  ✓
... (all keyword sets are disjoint)            ✓
{ "let" }    ∩ FIRST(exprStmt) = ∅  ✓
             ("let" is a keyword — not in IDENT set)
```

**Result: FIRST sets are disjoint. `declaration` is LL(1). ✓**

### 1.5 Worked Example — EAML Expression Hierarchy

Apply FIRST to the expression precedence levels:

```
FIRST(orExpr)          = FIRST(andExpr)          = ...
                       = FIRST(unaryExpr)         = ...
                       = FIRST(primaryExpr)
                       = { IDENT, INT, FLOAT, STRING,
                           "(", "!", "-", "await",
                           "true", "false", "null" }

FIRST(orExpr) = FIRST(andExpr) = FIRST(comparisonExpr) = ...
```

**Why these are all the same:** Each level of the hierarchy has only ONE
non-optional production — it must begin with whatever `primaryExpr` begins
with. The operator choices (`||`, `&&`, etc.) are all in the `*` or `?`
suffix, not the initial required element. The hierarchy has no ε-productions
for the base elements, so FIRST flows straight down.

**LL(1) check on expression grammar:** No conflicts possible because each level
has exactly one leading alternative — the next level. The only decision points
are in the suffix `( op NextLevel )*` iterations, where the parser checks
whether the next token is an operator at that precedence level. This is not
an LL(1) table conflict — it is a lookahead on the iteration continuation.

### 1.6 The LL(1) Condition — Formal Statement

A grammar is **LL(1)** if and only if for every non-terminal `A` with multiple
alternative productions `A ::= α₁ | α₂ | ... | αₙ`:

```
CONDITION 1 (distinct FIRST):
    FIRST(αᵢ) ∩ FIRST(αⱼ) = ∅  for all i ≠ j

CONDITION 2 (ε-production safety):
    If αᵢ ⟹* ε, then:
    FIRST(αⱼ) ∩ FOLLOW(A) = ∅  for all j ≠ i
```

**Informal reading of Condition 2:** If one alternative can produce nothing
(ε), the parser needs to know when to take the ε-path instead of one of the
other paths. The other alternatives must not start with anything that can
legally follow `A` — otherwise the parser cannot decide.

**In EAML:** No MVP production has an ε-alternative mixed with non-ε
alternatives at the same decision point, with one exception:

```ebnf
/* The suffix of iterated productions */
declaration* — zero or more declarations

/* When parsing declaration*, the parser checks: */
/* "Is the next token in FIRST(declaration)?"    */
/* If yes → parse another declaration            */
/* If no  → take the ε path (done)               */
/* FOLLOW(Program) = { EOF }                     */
/* FIRST(declaration) ∩ { EOF } = ∅              ✓ */
```

---

## Section 2 — LL(1) Grammar Analysis Checklist

Apply this checklist to every EAML production rule before writing parser code.
Every item must be confirmed before proceeding to implementation.

### 2.1 The Five LL(1) Violations to Check

#### Violation V1 — Direct Left Recursion

A rule is directly left-recursive if the non-terminal on the left-hand side
appears as the **first** symbol on any right-hand side alternative.

```
Pattern: A ::= A α | β   ← DIRECTLY LEFT-RECURSIVE
```

**Why it fails:** A recursive-descent parser for `A` immediately calls itself
without consuming any input → infinite loop.

**Test:** For every EAML production `A ::= rhs`, check: does `rhs` begin
with `A`?

```
/* EAML CHECK — pass or fail for each production */

orExpr ::= andExpr ( "||" andExpr )*
/* Does orExpr begin with orExpr? */
/* First symbol: andExpr — NOT orExpr. PASS ✓  */

/* WOULD FAIL: */
/* orExpr ::= orExpr "||" andExpr | andExpr  ← FAIL */
```

#### Violation V2 — Indirect Left Recursion

A rule is indirectly left-recursive if it can eventually derive itself as the
first symbol through a chain of other rules.

```
Pattern: A ::= B α
         B ::= A β   ← INDIRECTLY LEFT-RECURSIVE
```

**Test:** For every production `A ::= B ...`, trace whether `B` can derive
a string starting with `A` without consuming any input first.

```
/* EAML CHECK */

postfixExpr ::= primaryExpr suffix*
primaryExpr ::= literal | IDENT | "(" expr ")"
/* Does primaryExpr eventually derive postfixExpr first? */
/* primaryExpr → literal → no. PASS ✓ */
/* primaryExpr → "(" expr ")" — expr leads to orExpr → ... → postfixExpr */
/* BUT "(" is consumed first, so the recursion is bounded. PASS ✓ */
```

#### Violation V3 — Common Prefix Conflict (Needs Left Factoring)

Two alternatives for the same non-terminal begin with the same token.
The parser cannot determine which alternative to take with 1-token lookahead.

```
Pattern: A ::= "if" condition1 block
            | "if" condition2 block   ← COMMON PREFIX "if"
```

**Test:** For every non-terminal `A` with multiple alternatives, check whether
any pair of alternatives share a common first token.

```
/* EAML CHECK on schemaBody field separator */

fieldSep ::= ","
           | NL
/* FIRST(",") = {","}, FIRST(NL) = {NL} — disjoint. PASS ✓ */

/* EAML CHECK on toolBody */
toolBody ::= "{" "python" "{" PYTHON_BLOCK "}" "}"
           | "{" statement* "}"
/* Both alternatives begin with "{". */
/* But after "{" we look at the NEXT token: */
/*   "python" → take first alternative  */
/*   anything else → take second         */
/* This is LL(2) — requires 2-token lookahead. Document as LL(2) point. */
```

#### Violation V4 — ε/FOLLOW Conflict

When a non-terminal has an ε-alternative, another alternative's FIRST set
must not intersect FOLLOW.

```
Pattern: A ::= α | ε    (A can produce nothing)
         context: ... A b ...   where b ∈ FIRST(α)
                               AND b ∈ FOLLOW(A)
```

**Test:** For every EAML production with `?` (optional) or `*` (zero or more),
verify that the optional element's FIRST set does not intersect FOLLOW of the
containing non-terminal.

```
/* EAML CHECK on promptDecl requiresClause? */

promptDecl ::= "prompt" IDENT "(" paramList? ")"
               requiresClause?     ← optional
               "->" typeExpr
               promptBody

/* FIRST(requiresClause) = { "requires" } */
/* FOLLOW(requiresClause in promptDecl) = { "->" } */
/* { "requires" } ∩ { "->" } = ∅  PASS ✓ */
```

#### Violation V5 — Ambiguity

Two different parse trees can be produced for the same input string.

```
Pattern: A ::= A "+" A   ← AMBIGUOUS (both left and right parse valid)
```

**Test:** Try to construct two different parse trees for the same input string
using your grammar rules. If you can, the grammar is ambiguous.

```
/* EAML CHECK on binary expressions */
/* The stratified hierarchy from Layer 2 is unambiguous by construction: */
/* Each operator has exactly ONE production level.                        */
/* a + b * c: only one parse tree possible — * binds tighter than +.     */
/* PASS ✓ by construction of the hierarchy                               */

/* EAML dangling-else check */
ifStmt ::= "if" expr block ( "else" ( ifStmt | block ) )?
/* "else" is optional. Does this create ambiguity? */
/* With nested ifs: if A then if B then X else Y */
/* The "else" binds to the NEAREST if by grammar structure */
/* (because "else" is the optional suffix of the INNER ifStmt) */
/* PASS ✓ — resolved by grammar structure, not semantic rule */
```

### 2.2 EAML LL(1) / LL(2) Inventory

All known conflict points in the EAML grammar with their classification:

| Production                       | Conflict Type                | k     | Resolution                                                                   |
|----------------------------------|------------------------------|-------|------------------------------------------------------------------------------|
| `declaration` alternatives       | None — all distinct keywords | LL(1) | N/A                                                                          |
| `toolBody`                       | Common prefix `"{"`          | LL(2) | Look at 2nd token: `"python"` or not                                         |
| `typeExpr` vs `comparisonExpr`   | `<` ambiguity                | LL(2) | Context flag: in typeExpr state or expr state                                |
| `paramList?` in promptDecl       | ε/FOLLOW                     | LL(1) | `FIRST(paramList) = {IDENT}`, `FOLLOW = {")"}` — disjoint                    |
| `requiresClause?`                | ε/FOLLOW                     | LL(1) | `FIRST = {"requires"}`, `FOLLOW = {"->"}` — disjoint                         |
| `suffix*` in postfixExpr         | ε/FOLLOW                     | LL(1) | `FIRST = {".", "(", "["}`, `FOLLOW(postfixExpr) = expr followers` — disjoint |
| `NL` vs `","` as field separator | None — distinct tokens       | LL(1) | N/A                                                                          |

---

## Section 3 — Left Recursion Elimination

### 3.1 Why Left Recursion Breaks Recursive Descent

A recursive-descent parser is a set of mutually recursive functions. If
function `parse_A()` can call itself as its first action without consuming
any input, it loops forever.

```rust
fn parse_A() {
    parse_A();  // ← infinite recursion — no input consumed
    consume(PLUS);
    parse_B();
}
```

### 3.2 Identifying Direct Left Recursion

A production is **directly left-recursive** if it has the form:

```
A ::= A α₁ | A α₂ | ... | A αₙ | β₁ | β₂ | ... | βₘ

where β₁ ... βₘ do NOT begin with A
```

### 3.3 Direct Left Recursion Elimination — Algorithm

Given a directly left-recursive rule:
```
A ::= A α₁ | A α₂ | β₁ | β₂
```

Replace with:
```
A  ::= β₁ A' | β₂ A'
A' ::= α₁ A' | α₂ A' | ε
```

Where `A'` is a new non-terminal (conventionally named `A` + prime or
`A_tail` or `A_rest`).

**Concrete example — left-recursive additive expression:**
```
/* BEFORE — left-recursive (WRONG for recursive descent) */
additiveExpr ::= additiveExpr "+" multiplicativeExpr
               | additiveExpr "-" multiplicativeExpr
               | multiplicativeExpr

/* AFTER — right-recursive (correct for recursive descent) */
additiveExpr  ::= multiplicativeExpr additiveExprTail
additiveExprTail ::= "+" multiplicativeExpr additiveExprTail
                  | "-" multiplicativeExpr additiveExprTail
                  | ε
```

**Note on associativity:** The right-recursive form produces a right-leaning
parse tree. To recover left-associativity in the AST, the parser must build
the tree iteratively (left-fold) as it recognizes each `additiveExprTail`.
This is exactly what the `( op Next )*` W3C EBNF pattern in Layer 2 encodes.

**The Layer 2 EAML grammar already uses the correct form:**
```ebnf
/* Layer 2 — already left-recursion-free by using the iteration pattern */
additiveExpr ::= multiplicativeExpr ( ( "+" | "-" ) multiplicativeExpr )*
```

The `( ... )*` iteration is the grammar-level expression of the right-recursive
tail form above. They describe the same language. The W3C EBNF form is more
readable; the explicit tail form is what you implement in the parser.

### 3.4 Indirect Left Recursion Elimination — Algorithm

Indirect left recursion: `A →* A α` through a chain of productions.

```
/* Example of indirect left recursion */
A ::= B x
B ::= A y | z

/* A → B x → A y x → A y x (A reappears at the front) */
```

**Elimination algorithm (Paull's algorithm):**

```
1. Order the non-terminals: A₁, A₂, ..., Aₙ
2. For i = 1 to n:
   a. For j = 1 to i-1:
      - For each production Aᵢ ::= Aⱼ γ:
        Replace with: Aᵢ ::= δ₁ γ | δ₂ γ | ... (where Aⱼ ::= δ₁ | δ₂ | ...)
   b. Eliminate direct left recursion from Aᵢ (using Section 3.3 algorithm)
3. Remove unreachable productions
```

**Application to EAML:** The EAML grammar as specified in Layers 2 and 3 has
**no left recursion** by design. Every rule uses either:
- The iteration pattern `( ... )*` — inherently right-recursive
- Named alternatives that all begin with distinct terminals
- Postfix suffix chains `primaryExpr suffix*` — not recursive

Run the V1 and V2 checks from Section 2.1 to confirm. If any left recursion
is found, apply the elimination algorithm and re-verify.

### 3.5 Left Factoring — Eliminating Common Prefix Conflicts

Left factoring resolves LL(1) conflicts where two alternatives share a
common prefix.

**Given:**
```
A ::= α β₁ | α β₂
```

**After left factoring:**
```
A  ::= α A'
A' ::= β₁ | β₂
```

**EAML application — `toolBody` LL(2) conflict:**
```ebnf
/* BEFORE — common prefix "{" creates LL(2) need */
toolBody ::= "{" "python" "{" PYTHON_BLOCK "}" "}"
           | "{" statement* "}"

/* AFTER — left factored */
toolBody  ::= "{" toolBodyInner "}"
toolBodyInner ::= "python" "{" PYTHON_BLOCK "}"
                | statement*
```

After left factoring, the parser consumes `"{"`, then looks at ONE token
(`"python"` or not) to decide the alternative. This converts the LL(2)
point to LL(1). The grammar specification in Layer 2 uses the pre-factored
form for readability; the parser implementation uses the factored form.

---

## Section 4 — Pratt Parsing for EAML Expressions

### 4.1 Why Pratt for Expressions

The stratified hierarchy from Layer 2 is the **correct grammar specification**
for EAML expressions. It is unambiguous, well-understood, and documents
precedence explicitly. However, implementing it as pure recursive descent
requires one function per precedence level — 9 functions for EAML's 9 levels.

Pratt parsing is an enhancement of recursive descent parsing that uses the natural terminology of precedence and associativity for parsing expressions, instead of grammar obfuscation techniques.

If recursive descent is peanut butter, Pratt parsing is the jelly. When you mix the two together, you get a simple, terse, readable parser that can handle any grammar you throw at it.

The combination is:
- **Grammar specification:** Use the stratified hierarchy (Layer 2) — clear, formal, verifiable
- **Parser implementation:** Use Pratt parsing for the expression sub-parser — compact, efficient

### 4.2 Core Pratt Concepts

Pratt parsing was first described by Vaughan Pratt in the 1973 paper "Top Down Operator Precedence", based on recursive descent.

**Binding Power (BP):** A numeric value assigned to each operator token that
determines how tightly it binds to its operands. Higher = tighter binding.

```
/* EAML binding power table */

Operator    Left BP    Right BP    Notes
--------    -------    --------    -----
||          10         11          left-associative (L < R)
&&          20         21          left-associative
==  !=      30         30          non-associative (equal BP → no chaining)
<  >  <=    35         35          non-associative
>=
+   -       40         41          left-associative
*   /       50         51          left-associative
!   - (unary) n/a      70          prefix — no left operand
await        n/a       65          prefix
.  ()  []   80         81          postfix/suffix — highest
```

**Left-associativity:** Left BP = Right BP - 1. The right recursive call
uses right BP, so the next operator with equal left BP is NOT consumed
by the recursive call, leaving it to be consumed by the current level.

**Non-associativity:** Left BP = Right BP. Equal binding powers cause the
recursive call to stop before consuming an operator of the same type.
`a == b == c` fails because after parsing `a == b`, the next `==` has
left BP 30 which equals the right BP 30 → recursive call exits →
outer expression sees `==` with no left-hand expression → error.

### 4.3 Pratt Parser Structure

```rust
/* Pratt parser pseudocode — Rust-style */
/* Source: Aleksey Kladov "Simple but Powerful Pratt Parsing" */
/* URL: https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html */

fn parse_expr(lexer: &mut Lexer, min_bp: u8) -> Expr {
    /* Step 1: Parse the left-hand side (prefix position) */
    let mut lhs = match lexer.next() {
        /* Literals and identifiers — no special handling */
        Token::Int(n)    => Expr::Lit(n),
        Token::Float(f)  => Expr::Lit(f),
        Token::String(s) => Expr::Lit(s),
        Token::Ident(id) => Expr::Ident(id),
        Token::True      => Expr::Lit(true),
        Token::False     => Expr::Lit(false),
        Token::Null      => Expr::Lit(null),

        /* Grouping — "(" expr ")" */
        Token::LParen => {
            let inner = parse_expr(lexer, 0); /* reset bp */
            lexer.expect(Token::RParen);
            inner
        },

        /* Prefix operators */
        Token::Bang  => { let rhs = parse_expr(lexer, 70); Expr::Not(rhs) },
        Token::Minus => { let rhs = parse_expr(lexer, 70); Expr::Neg(rhs) },
        Token::Await => { let rhs = parse_expr(lexer, 65); Expr::Await(rhs) },

        t => panic!("Unexpected token in expression: {:?}", t),
    };

    /* Step 2: Loop consuming infix/postfix operators */
    loop {
        let op = match lexer.peek() {
            Token::Eof      => break,
            Token::RParen   => break,  /* closing group — stop */
            Token::RBracket => break,  /* closing index — stop */
            Token::Comma    => break,  /* arg separator — stop */
            op => op,
        };

        /* Get binding powers for this operator */
        let (l_bp, r_bp) = infix_binding_power(op);

        /* If left bp ≤ min_bp, this operator belongs to the caller */
        if l_bp <= min_bp {
            break;
        }

        lexer.next(); /* consume the operator */

        /* Build the new node based on operator type */
        lhs = match op {
            /* Binary operators */
            Token::Or       => { let rhs = parse_expr(lexer, r_bp); Expr::Or(lhs, rhs) },
            Token::And      => { let rhs = parse_expr(lexer, r_bp); Expr::And(lhs, rhs) },
            Token::EqEq     => { let rhs = parse_expr(lexer, r_bp); Expr::Eq(lhs, rhs) },
            Token::BangEq   => { let rhs = parse_expr(lexer, r_bp); Expr::Ne(lhs, rhs) },
            Token::Lt       => { let rhs = parse_expr(lexer, r_bp); Expr::Lt(lhs, rhs) },
            Token::Gt       => { let rhs = parse_expr(lexer, r_bp); Expr::Gt(lhs, rhs) },
            Token::Plus     => { let rhs = parse_expr(lexer, r_bp); Expr::Add(lhs, rhs) },
            Token::Minus    => { let rhs = parse_expr(lexer, r_bp); Expr::Sub(lhs, rhs) },
            Token::Star     => { let rhs = parse_expr(lexer, r_bp); Expr::Mul(lhs, rhs) },
            Token::Slash    => { let rhs = parse_expr(lexer, r_bp); Expr::Div(lhs, rhs) },

            /* Postfix: member access */
            Token::Dot => {
                let field = lexer.expect_ident();
                Expr::Member(lhs, field)
            },

            /* Postfix: function call */
            Token::LParen => {
                let args = parse_arg_list(lexer);
                lexer.expect(Token::RParen);
                Expr::Call(lhs, args)
            },

            /* Postfix: index access */
            Token::LBracket => {
                let idx = parse_expr(lexer, 0);
                lexer.expect(Token::RBracket);
                Expr::Index(lhs, idx)
            },

            _ => unreachable!(),
        };
    }
    lhs
}

fn infix_binding_power(op: Token) -> (u8, u8) {
    match op {
        Token::Or       => (10, 11),
        Token::And      => (20, 21),
        Token::EqEq |
        Token::BangEq   => (30, 30),   /* non-associative */
        Token::Lt  | Token::Gt  |
        Token::LtEq | Token::GtEq => (35, 35),  /* non-associative */
        Token::Plus  | Token::Minus => (40, 41),
        Token::Star  | Token::Slash => (50, 51),
        Token::Dot   | Token::LParen |
        Token::LBracket             => (80, 81),
        _ => (0, 0),   /* not an infix operator — caller will stop */
    }
}
```

### 4.4 The Grammar / Parser Duality

This is the critical insight for using both Layer 2 and Pratt together:

**The grammar specification (Layer 2) and the Pratt implementation are
two representations of the same language — they must agree.**

```
GRAMMAR SPEC (Layer 2, W3C EBNF):          PRATT IMPL (Section 4.3):
─────────────────────────────────          ─────────────────────────
orExpr ::=                                 parse_expr(min_bp = 0)
  andExpr ( "||" andExpr )*                ← "||" with BP (10, 11)

andExpr ::=                                still parse_expr
  comparisonExpr ( "&&" comparisonExpr )*  ← "&&" with BP (20, 21)

comparisonExpr ::=                         still parse_expr
  additiveExpr                             ← "==" with BP (30, 30)
  ( ("==" | "!=") additiveExpr )?          ← "?" → (30,30) non-assoc

additiveExpr ::=                           still parse_expr
  multiplicativeExpr                       ← "+" with BP (40, 41)
  ( ("+" | "-") multiplicativeExpr )*
```

The stratified hierarchy levels map 1:1 to binding power pairs. The only
difference is that the grammar makes precedence visible as structure, while
Pratt makes it visible as numbers. Use the grammar for specification and
review; use Pratt for implementation.

### 4.5 Non-Associativity in Pratt Parsing

Non-associative operators (comparison) use **equal** left and right binding
power. This causes the recursive call to stop before consuming another
comparison operator, making `a == b == c` a parse error by construction.

```rust
/* Non-associative: l_bp == r_bp */
Token::EqEq => (30, 30)

/* Parsing a == b == c: */
/* 1. parse_expr(0) called */
/* 2. lhs = parse a */
/* 3. op = ==, l_bp=30 > min_bp=0 → consume */
/* 4. rhs = parse_expr(30) → parses b, stops at next == (l_bp=30 ≤ min_bp=30) */
/* 5. lhs = Eq(a, b) */
/* 6. next op = ==, l_bp=30 > min_bp=0 → consume */
/* 7. rhs = parse_expr(30) → parses c */
/* Result: Eq(Eq(a,b), c) — WRONG for non-associative! */
```

Wait — this is still wrong. For true non-associativity, the parse should
**error**, not produce a tree. The fix: after building a comparison node,
check if the next token is also a comparison operator and emit an error:

```rust
/* After step 5, add a check: */
Token::EqEq | Token::BangEq | Token::Lt | ... => {
    let rhs = parse_expr(lexer, r_bp);
    let node = Expr::Cmp(op, lhs, rhs);
    /* Non-associativity check */
    if is_comparison_op(lexer.peek()) {
        return Err("Chained comparisons not supported. Use && to combine.");
    }
    node
}
```

This is a **semantic check**, not a grammar check. The grammar allows it;
the semantic analysis (or a post-parse check) rejects it. Document as
`[sem: no-chained-comparison]`.

### 4.6 Pratt References

| Resource                                                     | URL                                                                                       | Best For                                     |
|--------------------------------------------------------------|-------------------------------------------------------------------------------------------|----------------------------------------------|
| Aleksey Kladov — "Simple but Powerful Pratt Parsing"         | https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html               | Rust implementation, binding power concept   |
| Bob Nystrom — "Pratt Parsers: Expression Parsing Made Easy"  | https://journal.stuffwithstuff.com/2011/03/19/pratt-parsers-expression-parsing-made-easy/ | Java, clear terminology, Bantam toy language |
| Eli Bendersky — "Top-Down Operator Precedence"               | https://eli.thegreenplace.net/2010/01/02/top-down-operator-precedence-parsing             | Python, thorough explanation of nud/led      |
| Pratt original paper — "Top Down Operator Precedence" (1973) | ACM Digital Library (paywall)                                                             | Historical reference                         |

---

## Section 5 — Applying Theory to EAML's Known Conflict Points

This section applies all four topics to the specific EAML grammar problems
documented in previous layers.

### 5.1 The `<`/`>` Angle Bracket Problem

**Conflict:** `Float<0.0, 1.0>` (type parameter) vs. `x < y` (comparison).

**FIRST/FOLLOW analysis:**

```
Context A: after a type name in typeExpr position
  Input:   Float < 0.0 , 1.0 >
  FIRST(typeExpr following typeNameIdent) = { "<", "[]", "?", ... }
  Decision: "<" → parse as type parameter opener

Context B: in expression position
  Input:   x < y
  FIRST(comparisonExpr suffix) = { "<", ">", "<=", ">=" }
  Decision: "<" → parse as comparison operator
```

**Resolution:** The contexts are mutually exclusive **in the grammar** because
`typeExpr` and `expr` never appear in the same production position. The parser
maintains a **context flag** (`parsing_type_expr` boolean). The flag is set
when the parser enters a typeExpr production and cleared when it returns.
Document as `[lex: angle-bracket-disambiguation]`.

**LL(k) classification:** This is not an LL(k) conflict at all — it is
**context-sensitive disambiguation**. The grammar is unambiguous because
typeExpr and expr are different non-terminals. The parser implementation
must track which non-terminal it is currently expanding.

### 5.2 The `toolBody` LL(2) Point

**Conflict:** Both `toolBody` alternatives begin with `"{"`.

```ebnf
toolBody ::= "{" "python" "{" PYTHON_BLOCK "}" "}"  /* alternative 1 */
           | "{" statement* "}"                      /* alternative 2 */
```

**FIRST analysis:**

```
FIRST( "python" "{" PYTHON_BLOCK "}" "}" ) = { "python" }
FIRST( statement* "}" )                     = { ...all statement firsts..., "}" }
```

After factoring out the common `"{"`:

```ebnf
toolBody      ::= "{" toolBodyInner "}"
toolBodyInner ::= "python" "{" PYTHON_BLOCK "}"
                | statement*
```

**FIRST(toolBodyInner):**
```
FIRST( "python" ... ) = { "python" }
FIRST( statement* )   = { IDENT, INT, FLOAT, STRING, "(", "!", "-",
                          "await", "true", "false", "null", "let", "if" }
                        ∪ { ε }    /* statement* can be empty */
```

**LL(1) check post-factoring:**
```
{ "python" } ∩ FIRST(statement*) = ∅   ✓
(because "python" is a keyword not in the expression FIRST set,
 and it is not a statement-level keyword)
```

**Result after factoring:** LL(1). The parser: consume `"{"`, peek at next
token — if `"python"` → Python branch; otherwise → statement list branch.

### 5.3 The Template String `{` Disambiguation

**Conflict:** `{var}` interpolation vs. `{{` literal brace.

**This is a LEXER-level disambiguation, not a parser-level conflict.**

The lexer handles this in template string mode:
```
State: TEMPLATE_STRING_MODE
  See "{{" → emit TMPL_TEXT("{"), stay in TMPL mode
  See "}}" → emit TMPL_TEXT("}"), stay in TMPL mode
  See "{" followed by IDENT "}" → emit TMPL_INTERP(ident), stay in TMPL mode
  See "{" followed by anything else → lexer error (unclosed or invalid interp)
```

The parser sees only `TMPL_TEXT` and `TMPL_INTERP` tokens — the `{`/`}`
disambiguation has already been resolved by the lexer. No parser-level
conflict exists.

### 5.4 The Signed Number Problem

**Conflict:** `x - 1` (subtraction) vs. `-1` (negative literal).

**Analysis:** The EAML grammar uses `unaryExpr ::= ("!" | "-")* awaitExpr`.
The `-` is always a **unary prefix operator**, never part of a numeric literal.

```
Lexer always emits: Token::Minus for "-"
                    Token::Int(1) for "1"
                    Token::Float(1.0) for "1.0"
                    (Never: Token::Int(-1))

Parser sees:
  x - 1    →  [IDENT(x), MINUS, INT(1)]
  -1       →  [MINUS, INT(1)]
  x + -1   →  [IDENT(x), PLUS, MINUS, INT(1)]
```

In the Pratt parser, `MINUS` in prefix position (no left-hand expression
available) triggers the prefix/nud handler — it parses as unary negation.
In infix position (with a left-hand expression), it triggers the infix/led
handler — it parses as subtraction.

**FOLLOW analysis:** No conflict. The Pratt parser naturally resolves this
because prefix position and infix position are structurally different.

### 5.5 The `requires` Clause Optional Element

**Conflict:** Is `requiresClause?` (optional) safe in promptDecl?

```ebnf
promptDecl ::= "prompt" IDENT "(" paramList? ")"
               requiresClause?
               "->" typeExpr
               promptBody
```

**ε/FOLLOW analysis:**

```
FIRST(requiresClause) = { "requires" }
FOLLOW(requiresClause in promptDecl) = { "->" }

FIRST(requiresClause) ∩ FOLLOW(requiresClause in promptDecl)
  = { "requires" } ∩ { "->" }
  = ∅   PASS ✓
```

The parser: after consuming `)`, peek at next token.
- If `"requires"` → parse requiresClause
- If `"->"` → no requiresClause, proceed to return type

**Result:** LL(1) — no conflict. Safe to implement as optional.

---

## Section 6 — Theory Application Rules for AI Grammar Assistance

When helping write, verify, or debug EAML grammar productions, apply
these rules derived from the compiler theory in this document:

1. **Verify FIRST sets before declaring LL(1) compliance.** Do not assume a
   grammar is unambiguous — compute the FIRST sets and check for disjointness.
   Any intersection is a bug, not a feature.

2. **Check for left recursion in every new production.** Before writing a
   parser function, verify that the production's first symbol (after optional
   elements) is not the production itself or any production reachable from it
   without consuming input.

3. **The stratified hierarchy (Layer 2) is left-recursion-free by design.**
   The `( op Next )*` iteration pattern is the LL-compatible form. Never
   revert to `A ::= A op B` form in EAML grammar productions.

4. **Left factor before implementing, not after.** If two alternatives share
   a common prefix, apply left factoring to the grammar specification before
   writing the parser code. The implementation follows the grammar; fixing
   the grammar is easier than fixing the parser.

5. **Use Pratt for the expression parser, stratified hierarchy for the spec.**
   The grammar document specifies levels (Layer 2 style). The Rust/Python
   implementation uses Pratt with binding powers. Both must be consistent —
   verify that binding power pairs match the precedence levels in the spec.

6. **Non-associativity is a semantic check, not a grammar check.**
   `a == b == c` parses to a tree in Pratt but should be rejected. Add a
   post-parse semantic check rather than trying to encode non-associativity
   in the grammar rules.

7. **Context flags are valid for EAML's `<`/`>` disambiguation.**
   This is not a grammar defect — it is a known LL(1)-compatible resolution
   used by many real compilers (including Rust's). Document the flag location
   and the two states it can be in.

8. **FOLLOW sets confirm that optional elements are safe.** Before using `?`
   in any production, compute FIRST(optional_element) and FOLLOW(containing_rule)
   and verify they are disjoint. If they intersect, the optional element
   cannot be safely parsed without backtracking.

9. **`$` (EOF) is always in FOLLOW(Program).** Every grammar's start symbol
   has `$` in its FOLLOW set. EAML's `Program ::= declaration* EOF` terminates
   when the parser sees `EOF` and `FIRST(declaration)` would not match.

10. **Document every LL(2) point explicitly.** Each place where 2-token
    lookahead is needed must be named, justified, and confirmed as the
    minimum necessary lookahead. LL(3+) is a design error — restructure
    the grammar before proceeding.

---

## Section 7 — Source Attribution

| Content                              | Source                                                                             | URL                                                                                       |
|--------------------------------------|------------------------------------------------------------------------------------|-------------------------------------------------------------------------------------------|
| FIRST/FOLLOW algorithms              | Dragon Book (Aho, Lam, Sethi, Ullman) — *Compilers: Principles, Techniques, Tools* | Reference text — standard CS curriculum                                                   |
| FIRST/FOLLOW worked examples         | University compiler course notes                                                   | https://www.cs.uaf.edu/~cs331/notes/FirstFollow.pdf                                       |
| FIRST/FOLLOW algorithm notation      | hypertextbookshop.com compiler notes                                               | http://hypertextbookshop.com/transPL/Contents/                                            |
| LL(1) grammar conditions             | tutorialspoint.com compiler design                                                 | https://www.tutorialspoint.com/compiler_design/compiler_design_ll1_grammar.htm            |
| Left recursion formal definition     | Wikipedia — Left Recursion                                                         | https://en.wikipedia.org/wiki/Left_recursion                                              |
| Left recursion elimination algorithm | Paull's algorithm, via ScienceDirect                                               | https://www.sciencedirect.com/topics/computer-science/left-recursion                      |
| Pratt parsing concept                | Vaughan Pratt, "Top Down Operator Precedence" (1973)                               | ACM (paywall)                                                                             |
| Pratt parsing — binding power        | Aleksey Kladov, "Simple but Powerful Pratt Parsing"                                | https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html               |
| Pratt parsing — practical guide      | Bob Nystrom, "Pratt Parsers: Expression Parsing Made Easy"                         | https://journal.stuffwithstuff.com/2011/03/19/pratt-parsers-expression-parsing-made-easy/ |
| Pratt = precedence climbing proof    | Andy Chu, oilshell.org                                                             | https://www.oilshell.org/blog/2016/11/01.html                                             |
| EAML-specific applications           | EAML specification (this document)                                                 | N/A — original                                                                            |

---

*EAML Layer 4 Compiler Theory Reference — Version 0.1 — 2026-03-14*
*Final layer in the EAML grammar reference stack.*
*Load after Layers 1, 2, and 3 for complete grammar development context.*
*Layer 4 is for grammar verification and parser implementation — not grammar specification.*