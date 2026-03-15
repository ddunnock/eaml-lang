# ARCHITECTURE.md — EAML Compiler Architecture

## 1. Overview

**Architecture Pattern: Pipeline Compiler with Strict Layer Boundaries**

EAML implements a classic multi-stage compiler pipeline:

```
Source Code (.eaml)
    ↓
[LEXER]      (eaml-lexer)     → TokenStream
    ↓
[PARSER]     (eaml-parser)    → Program (AST)
    ↓
[SEMANTIC]   (eaml-semantic)  → AnalyzedProgram
    ├─ Pass 1: Name Resolution
    ├─ Pass 2: Type Checking
    └─ Pass 3: Capability Checking
    ↓
[CODEGEN]    (eaml-codegen)   → Python source code
    ↓
Output (.py with Pydantic v2 + eaml_runtime)
```

## 2. Compiler Phases

### Phase 1: Lexical Analysis (`eaml-lexer`)
- **Input**: Raw `.eaml` source text (UTF-8)
- **Output**: TokenStream
- **Key responsibilities**:
  - Token identification (identifiers, keywords, literals, operators)
  - Comment skipping (`//`, `/* */`, `///`)
  - String interpolation tracking with brace-depth counting for `{expr}`
  - Python bridge block detection (`python %{ ... }%`) — content passed opaque
  - String interning via `lasso` for identifier deduplication
- **Error codes**: SYN001–SYN039
- **Public API**: `lex(source: &str) -> TokenStream`

### Phase 2: Syntactic Analysis (`eaml-parser`)
- **Input**: TokenStream
- **Output**: Program (AST)
- **Key responsibilities**:
  - Hand-written recursive descent parsing (LL(1), one documented LL(2) point in argList)
  - AST construction for 84 grammar productions
  - 7 top-level declarations: `model`, `schema`, `prompt`, `tool`, `agent`, `import`, `let`
- **Error codes**: SYN040–SYN049
- **Public API**: `parse(tokens: TokenStream) -> Program`

### Phase 3: Semantic Analysis (`eaml-semantic`)

Three-pass analysis:

1. **Name Resolution**: Build symbol table, detect duplicates (RES010), pre-populate primitives
2. **Type Checking**: Resolve references, validate annotations, check bounded types, validate literal unions
3. **Capability Checking**: Extract `requires` clauses, perform subset check (`prompt_requires ⊆ model_caps`), emit CAP010 FATAL on mismatch

- **Error codes**: RES001–019, TYP001–049, SEM001–070, CAP001–020
- **Public API**: `analyze(program: &Program) -> AnalyzedProgram`

### Phase 4: Code Generation (`eaml-codegen`)
- **Input**: AnalyzedProgram
- **Output**: Python source code string
- **Key responsibilities**:
  - Pydantic v2 `BaseModel` for each schema
  - Async functions for prompts/tools with `eaml_runtime` adapters
  - Python bridge block embedding as function body
  - Type annotation generation per TYPESYSTEM.md rules
- **Error codes**: PYB001–024
- **Public API**: `generate(program: &AnalyzedProgram) -> String`

## 3. Layer Boundaries

**Strict dependency hierarchy** (enforced via Cargo.toml):

```
eaml-errors     ← zero eaml deps (foundation)
eaml-lexer      ← eaml-errors only
eaml-parser     ← eaml-errors, eaml-lexer
eaml-semantic   ← eaml-errors, eaml-lexer, eaml-parser
eaml-codegen    ← eaml-errors, eaml-parser, eaml-semantic
eaml-cli        ← all crates
```

Each crate has clear ownership — the lexer knows nothing about the parser, the parser knows nothing about semantic analysis.

## 4. Key Abstractions

### Token Types (`eaml-lexer`)
- Named tokens: `IDENT`, `INT`, `FLOAT`, `STRING`, `PYTHON_BLOCK`, `EOF`
- 27 keywords: 7 active, 3 post-MVP reserved, 8 statement/expr, 9 future
- Contextual keywords: `as`, `id`, `provider`, `caps`, `user`, `system`, `temperature`, `max_tokens`, `max_retries`

### AST Node Types (`eaml-parser`)
- **Declarations**: `ModelDecl`, `SchemaDecl`, `PromptDecl`, `ToolDecl`, `AgentDecl`, `ImportDecl`, `LetDecl`
- **Expressions**: `Identifier`, `Literal`, `BinaryOp`, `UnaryOp`, `MemberAccess`, `CallExpr`, `ArrayLiteral`, `ObjectLiteral`
- **Types**: `TypeExpr` with modifiers (optional `?`, array `[]`, bounded `<params>`)

### Semantic Output (`eaml-semantic`)
- Symbol table: declared names → type info
- Type assignments: each identifier → resolved TypeInfo
- Capability graph: prompt → required caps, model → declared caps
- Error accumulator: all diagnostics with source locations

### Error Types (`eaml-errors`)
- `Diagnostic`: code, message, severity (FATAL/ERROR/WARNING), source location, hints
- Display via `codespan-reporting` with colored terminal output
- 38+ error codes across 6 prefixes (SYN, SEM, TYP, CAP, PYB, RES)

## 5. Entry Points

### CLI Binary (`eaml-cli`)
```bash
eamlc compile <file.eaml>        # Compile to Python
eamlc check <file.eaml>           # Type-check only
eamlc --check-python <file.eaml>  # Validate bridge blocks
```

### Library API
Each crate exports via `pub` items in `lib.rs`:
- `eaml_lexer::lex(source) -> Result<TokenStream, Vec<Diagnostic>>`
- `eaml_parser::parse(tokens) -> Result<Program, Vec<Diagnostic>>`
- `eaml_semantic::analyze(program) -> Result<AnalyzedProgram, Vec<Diagnostic>>`
- `eaml_codegen::generate(program) -> Result<String, Vec<Diagnostic>>`

## 6. Error Severity Model

| Severity | Behavior | Example |
|----------|----------|---------|
| FATAL | Halts compilation immediately | CAP010 capability mismatch |
| ERROR | Continues to find more errors | SYN042 multi-dim array |
| WARNING | Does not block compilation | TYP001 type shadowing |

Max accumulated errors: 20 (overridable with `--max-errors N`).