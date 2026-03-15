     [1mSTDIN[0m
[38;5;247m   1[0m [38;5;254m# Architecture Patterns[0m
[38;5;247m   2[0m 
[38;5;247m   3[0m [38;5;254m**Domain:** Rust compiler for LLM integration DSL targeting Python/Pydantic[0m
[38;5;247m   4[0m [38;5;254m**Researched:** 2026-03-15[0m
[38;5;247m   5[0m 
[38;5;247m   6[0m [38;5;254m## Recommended Architecture[0m
[38;5;247m   7[0m 
[38;5;247m   8[0m [38;5;254mEAML follows a classic multi-pass compiler pipeline with strict crate boundaries already defined. This document specifies the internal architecture of each compiler phase and the Python runtime that generated code calls into.[0m
[38;5;247m   9[0m 
[38;5;247m  10[0m [38;5;254m```[0m
[38;5;247m  11[0m [38;5;254mSource (.eaml)[0m
[38;5;247m  12[0m [38;5;254m    |[0m
[38;5;247m  13[0m [38;5;254m    v[0m
[38;5;247m  14[0m [38;5;254m[eaml-errors]  <-- shared Diagnostic, Span, ErrorCode types[0m
[38;5;247m  15[0m [38;5;254m    |[0m
[38;5;247m  16[0m [38;5;254m    v[0m
[38;5;247m  17[0m [38;5;254m[eaml-lexer]   --> Token stream (logos-derived, lasso-interned)[0m
[38;5;247m  18[0m [38;5;254m    |[0m
[38;5;247m  19[0m [38;5;254m    v[0m
[38;5;247m  20[0m [38;5;254m[eaml-parser]  --> AST (typed enum tree, arena-optional)[0m
[38;5;247m  21[0m [38;5;254m    |[0m
[38;5;247m  22[0m [38;5;254m    v[0m
[38;5;247m  23[0m [38;5;254m[eaml-semantic] --> AnalyzedProgram (symbol table + type info + capability graph)[0m
[38;5;247m  24[0m [38;5;254m    |   Pass 1: Name Resolution (populate symbol table)[0m
[38;5;247m  25[0m [38;5;254m    |   Pass 2: Type Checking (resolve types, validate bounds)[0m
[38;5;247m  26[0m [38;5;254m    |   Pass 3: Capability Checking (subset check)[0m
[38;5;247m  27[0m [38;5;254m    |[0m
[38;5;247m  28[0m [38;5;254m    v[0m
[38;5;247m  29[0m [38;5;254m[eaml-codegen] --> Python source string (builder pattern, not templates)[0m
[38;5;247m  30[0m [38;5;254m    |[0m
[38;5;247m  31[0m [38;5;254m    v[0m
[38;5;247m  32[0m [38;5;254m[eaml-cli]     --> eamlc binary (clap, orchestrates pipeline)[0m
[38;5;247m  33[0m [38;5;254m    |[0m
[38;5;247m  34[0m [38;5;254m    v[0m
[38;5;247m  35[0m [38;5;254mGenerated .py  --> imports eaml_runtime (provider adapters, validation, telemetry)[0m
[38;5;247m  36[0m [38;5;254m```[0m
[38;5;247m  37[0m 
[38;5;247m  38[0m [38;5;254m### Component Boundaries[0m
[38;5;247m  39[0m 
[38;5;247m  40[0m [38;5;254m| Component | Responsibility | Communicates With |[0m
[38;5;247m  41[0m [38;5;254m|-----------|---------------|-------------------|[0m
[38;5;247m  42[0m [38;5;254m| `eaml-errors` | Diagnostic, Span, Severity, ErrorCode enums, codespan-reporting display | All crates depend on it |[0m
[38;5;247m  43[0m [38;5;254m| `eaml-lexer` | Tokenization, string interning, comment skipping, mode switching (template strings, python blocks) | Produces `TokenStream` consumed by parser |[0m
[38;5;247m  44[0m [38;5;254m| `eaml-parser` | Recursive descent parsing, AST construction, syntactic error recovery | Consumes `TokenStream`, produces `Program` (AST root) |[0m
[38;5;247m  45[0m [38;5;254m| `eaml-semantic` | Name resolution, type checking, capability checking, symbol table construction | Consumes `Program`, produces `AnalyzedProgram` with type annotations |[0m
[38;5;247m  46[0m [38;5;254m| `eaml-codegen` | Python/Pydantic code emission from analyzed AST | Consumes `AnalyzedProgram`, produces Python source `String` |[0m
[38;5;247m  47[0m [38;5;254m| `eaml-cli` | CLI argument parsing, pipeline orchestration, file I/O, error display | Orchestrates all crates |[0m
[38;5;247m  48[0m [38;5;254m| `eaml-runtime` (Python) | Provider adapters, retry logic, Pydantic validation at runtime, telemetry hooks | Called by generated Python code |[0m
[38;5;247m  49[0m 
[38;5;247m  50[0m [38;5;254m### Data Flow[0m
[38;5;247m  51[0m 
[38;5;247m  52[0m [38;5;254m```[0m
[38;5;247m  53[0m [38;5;254m                    Source text (&str)[0m
[38;5;247m  54[0m [38;5;254m                         |[0m
[38;5;247m  55[0m [38;5;254m                    eaml-lexer::lex()[0m
[38;5;247m  56[0m [38;5;254m                         |[0m
[38;5;247m  57[0m [38;5;254m                    TokenStream {[0m
[38;5;247m  58[0m [38;5;254m                      tokens: Vec<Token>,   // kind + span[0m
[38;5;247m  59[0m [38;5;254m                      interner: ThreadedRodeo,  // lasso string interner[0m
[38;5;247m  60[0m [38;5;254m                      diagnostics: Vec<Diagnostic>,[0m
[38;5;247m  61[0m [38;5;254m                    }[0m
[38;5;247m  62[0m [38;5;254m                         |[0m
[38;5;247m  63[0m [38;5;254m                    eaml-parser::parse()[0m
[38;5;247m  64[0m [38;5;254m                         |[0m
[38;5;247m  65[0m [38;5;254m                    Program {[0m
[38;5;247m  66[0m [38;5;254m                      declarations: Vec<Declaration>,  // model, schema, prompt, tool, agent, let[0m
[38;5;247m  67[0m [38;5;254m                      diagnostics: Vec<Diagnostic>,[0m
[38;5;247m  68[0m [38;5;254m                    }[0m
[38;5;247m  69[0m [38;5;254m                         |[0m
[38;5;247m  70[0m [38;5;254m                    eaml-semantic::analyze()[0m
[38;5;247m  71[0m [38;5;254m                         |[0m
[38;5;247m  72[0m [38;5;254m                    AnalyzedProgram {[0m
[38;5;247m  73[0m [38;5;254m                      program: Program,                // original AST (borrowed or owned)[0m
[38;5;247m  74[0m [38;5;254m                      symbol_table: SymbolTable,       // name -> SymbolInfo[0m
[38;5;247m  75[0m [38;5;254m                      type_env: TypeEnvironment,       // node_id -> ResolvedType[0m
[38;5;247m  76[0m [38;5;254m                      capability_graph: CapabilityGraph, // prompt -> caps, model -> caps[0m
[38;5;247m  77[0m [38;5;254m                      diagnostics: Vec<Diagnostic>,[0m
[38;5;247m  78[0m [38;5;254m                    }[0m
[38;5;247m  79[0m [38;5;254m                         |[0m
[38;5;247m  80[0m [38;5;254m                    eaml-codegen::generate()[0m
[38;5;247m  81[0m [38;5;254m                         |[0m
[38;5;247m  82[0m [38;5;254m                    String  (Python source code)[0m
[38;5;247m  83[0m [38;5;254m```[0m
[38;5;247m  84[0m 
[38;5;247m  85[0m [38;5;254m## Patterns to Follow[0m
[38;5;247m  86[0m 
[38;5;247m  87[0m [38;5;254m### Pattern 1: Typed AST with Enum Variants (not trait objects)[0m
[38;5;247m  88[0m 
[38;5;247m  89[0m [38;5;254m**What:** Represent AST nodes as Rust enums with struct variants. Each declaration type, expression type, and type expression gets its own variant. Use `Box<T>` for recursive nodes.[0m
[38;5;247m  90[0m 
[38;5;247m  91[0m [38;5;254m**When:** Always -- this is the standard pattern for small-to-medium language compilers in Rust.[0m
[38;5;247m  92[0m 
[38;5;247m  93[0m [38;5;254m**Why:** Enum dispatch is exhaustive (compiler catches missing match arms), zero-cost (no vtable), and pattern-matchable. Trait objects would add unnecessary indirection and lose exhaustiveness checking.[0m
[38;5;247m  94[0m 
[38;5;247m  95[0m [38;5;254m**Example:**[0m
[38;5;247m  96[0m [38;5;254m```rust[0m
[38;5;247m  97[0m [38;5;254m/// Top-level declaration[0m
[38;5;247m  98[0m [38;5;254mpub enum Declaration {[0m
[38;5;247m  99[0m [38;5;254m    Model(ModelDecl),[0m
[38;5;247m 100[0m [38;5;254m    Schema(SchemaDecl),[0m
[38;5;247m 101[0m [38;5;254m    Prompt(PromptDecl),[0m
[38;5;247m 102[0m [38;5;254m    Tool(ToolDecl),[0m
[38;5;247m 103[0m [38;5;254m    Agent(AgentDecl),[0m
[38;5;247m 104[0m [38;5;254m    Import(ImportDecl),[0m
[38;5;247m 105[0m [38;5;254m    Let(LetDecl),[0m
[38;5;247m 106[0m [38;5;254m}[0m
[38;5;247m 107[0m 
[38;5;247m 108[0m [38;5;254m/// A schema declaration: `schema Foo { ... }`[0m
[38;5;247m 109[0m [38;5;254mpub struct SchemaDecl {[0m
[38;5;247m 110[0m [38;5;254m    pub name: Spanned<Spur>,        // interned identifier + span[0m
[38;5;247m 111[0m [38;5;254m    pub fields: Vec<SchemaField>,[0m
[38;5;247m 112[0m [38;5;254m    pub span: Span,                 // full declaration span[0m
[38;5;247m 113[0m [38;5;254m}[0m
[38;5;247m 114[0m 
[38;5;247m 115[0m [38;5;254mpub struct SchemaField {[0m
[38;5;247m 116[0m [38;5;254m    pub name: Spanned<Spur>,[0m
[38;5;247m 117[0m [38;5;254m    pub ty: TypeExpr,[0m
[38;5;247m 118[0m [38;5;254m    pub span: Span,[0m
[38;5;247m 119[0m [38;5;254m}[0m
[38;5;247m 120[0m 
[38;5;247m 121[0m [38;5;254m/// Type expressions[0m
[38;5;247m 122[0m [38;5;254mpub enum TypeExpr {[0m
[38;5;247m 123[0m [38;5;254m    Primitive(PrimitiveType, Span),[0m
[38;5;247m 124[0m [38;5;254m    Named(Spanned<Spur>),                          // schema reference[0m
[38;5;247m 125[0m [38;5;254m    Array(Box<TypeExpr>, Span),                    // T[][0m
[38;5;247m 126[0m [38;5;254m    Optional(Box<TypeExpr>, Span),                 // T?[0m
[38;5;247m 127[0m [38;5;254m    Bounded(Box<TypeExpr>, Vec<BoundParam>, Span), // float<0.0, 1.0>[0m
[38;5;247m 128[0m [38;5;254m    LiteralUnion(Vec<Literal>, Span),              // "a" | "b" | "c"[0m
[38;5;247m 129[0m [38;5;254m}[0m
[38;5;247m 130[0m [38;5;254m```[0m
[38;5;247m 131[0m 
[38;5;247m 132[0m [38;5;254m**Confidence:** HIGH -- this is the dominant pattern in rustc, rust-analyzer, ruff, oxc, and every serious Rust compiler project.[0m
[38;5;247m 133[0m 
[38;5;247m 134[0m [38;5;254m### Pattern 2: Spanned Everything[0m
[38;5;247m 135[0m 
[38;5;247m 136[0m [38;5;254m**What:** Every AST node and sub-node carries a `Span` (byte offset range into source). Wrap identifiers in `Spanned<T>` which pairs the value with its span.[0m
[38;5;247m 137[0m 
[38;5;247m 138[0m [38;5;254m**When:** Always. Spans are essential for error reporting via codespan-reporting.[0m
[38;5;247m 139[0m 
[38;5;247m 140[0m [38;5;254m**Example:**[0m
[38;5;247m 141[0m [38;5;254m```rust[0m
[38;5;247m 142[0m [38;5;254mpub struct Span {[0m
[38;5;247m 143[0m [38;5;254m    pub start: usize,  // byte offset[0m
[38;5;247m 144[0m [38;5;254m    pub end: usize,    // byte offset (exclusive)[0m
[38;5;247m 145[0m [38;5;254m}[0m
[38;5;247m 146[0m 
[38;5;247m 147[0m [38;5;254mpub struct Spanned<T> {[0m
[38;5;247m 148[0m [38;5;254m    pub value: T,[0m
[38;5;247m 149[0m [38;5;254m    pub span: Span,[0m
[38;5;247m 150[0m [38;5;254m}[0m
[38;5;247m 151[0m [38;5;254m```[0m
[38;5;247m 152[0m 
[38;5;247m 153[0m [38;5;254m**Confidence:** HIGH -- universal in production compilers. codespan-reporting requires byte offsets.[0m
[38;5;247m 154[0m 
[38;5;247m 155[0m [38;5;254m### Pattern 3: NodeId for Cross-referencing (Semantic Phase)[0m
[38;5;247m 156[0m 
[38;5;247m 157[0m [38;5;254m**What:** Assign a unique `NodeId` to each AST node during parsing. The semantic phase uses these IDs as keys into side tables (type environment, resolution map) rather than mutating the AST.[0m
[38;5;247m 158[0m 
[38;5;247m 159[0m [38;5;254m**When:** When the semantic phase needs to annotate nodes with resolved type information without modifying the parser's AST types.[0m
[38;5;247m 160[0m 
[38;5;247m 161[0m [38;5;254m**Why:** Avoids making the AST mutable or adding generic type parameters. Side tables are simpler and allow the semantic crate to own its data independently.[0m
[38;5;247m 162[0m 
[38;5;247m 163[0m [38;5;254m**Example:**[0m
[38;5;247m 164[0m [38;5;254m```rust[0m
[38;5;247m 165[0m [38;5;254m// In eaml-parser[0m
[38;5;247m 166[0m [38;5;254mpub struct NodeId(u32);[0m
[38;5;247m 167[0m 
[38;5;247m 168[0m [38;5;254mpub struct SchemaDecl {[0m
[38;5;247m 169[0m [38;5;254m    pub id: NodeId,[0m
[38;5;247m 170[0m [38;5;254m    pub name: Spanned<Spur>,[0m
[38;5;247m 171[0m [38;5;254m    pub fields: Vec<SchemaField>,[0m
[38;5;247m 172[0m [38;5;254m    pub span: Span,[0m
[38;5;247m 173[0m [38;5;254m}[0m
[38;5;247m 174[0m 
[38;5;247m 175[0m [38;5;254m// In eaml-semantic[0m
[38;5;247m 176[0m [38;5;254mpub struct TypeEnvironment {[0m
[38;5;247m 177[0m [38;5;254m    types: HashMap<NodeId, ResolvedType>,[0m
[38;5;247m 178[0m [38;5;254m}[0m
[38;5;247m 179[0m [38;5;254m```[0m
[38;5;247m 180[0m 
[38;5;247m 181[0m [38;5;254m**Confidence:** HIGH -- used by rustc (DefId), TypeScript (node IDs), and rust-analyzer (syntax node pointers).[0m
[38;5;247m 182[0m 
[38;5;247m 183[0m [38;5;254m### Pattern 4: Multi-Pass Semantic Analysis with Separate Concerns[0m
[38;5;247m 184[0m 
[38;5;247m 185[0m [38;5;254m**What:** Three distinct passes over the AST, each with a single responsibility:[0m
[38;5;247m 186[0m [38;5;254m1. **Name Resolution:** Walk all declarations, populate symbol table with names and their declaration sites. Detect duplicate names (RES010). Pre-populate primitive types.[0m
[38;5;247m 187[0m [38;5;254m2. **Type Checking:** Walk expressions and type annotations. Resolve named types to their definitions. Validate bounded types, literal unions, composite type ordering. Emit TYP/SEM errors.[0m
[38;5;247m 188[0m [38;5;254m3. **Capability Checking:** Extract `requires` clauses from prompts, `caps` from models. Perform subset check. Emit CAP010 on mismatch.[0m
[38;5;247m 189[0m 
[38;5;247m 190[0m [38;5;254m**When:** Always -- the spec mandates this three-pass structure.[0m
[38;5;247m 191[0m 
[38;5;247m 192[0m [38;5;254m**Why:** Each pass can assume the previous pass completed successfully (or at least populated partial results). Name resolution must complete before type checking can resolve schema references.[0m
[38;5;247m 193[0m 
[38;5;247m 194[0m [38;5;254m**Confidence:** HIGH -- matches spec/ERRORS.md phase definitions and Layer 5 decisions.[0m
[38;5;247m 195[0m 
[38;5;247m 196[0m [38;5;254m### Pattern 5: Error Accumulation with Continuation[0m
[38;5;247m 197[0m 
[38;5;247m 198[0m [38;5;254m**What:** Never abort on the first error. Each compiler phase accumulates `Vec<Diagnostic>` and continues processing. The CLI decides when to stop based on error count and severity.[0m
[38;5;247m 199[0m 
[38;5;247m 200[0m [38;5;254m**When:** Always. The spec defines a max-errors limit (default 20) and three severity levels (FATAL, ERROR, WARNING).[0m
[38;5;247m 201[0m 
[38;5;247m 202[0m [38;5;254m**Why:** Users want to see all errors at once, not fix one error at a time. Only FATAL errors (like CAP010 capability mismatch) should halt immediately.[0m
[38;5;247m 203[0m 
[38;5;247m 204[0m [38;5;254m**Example:**[0m
[38;5;247m 205[0m [38;5;254m```rust[0m
[38;5;247m 206[0m [38;5;254mpub struct CompileResult<T> {[0m
[38;5;247m 207[0m [38;5;254m    pub value: Option<T>,       // None if fatal error occurred[0m
[38;5;247m 208[0m [38;5;254m    pub diagnostics: Vec<Diagnostic>,[0m
[38;5;247m 209[0m [38;5;254m}[0m
[38;5;247m 210[0m 
[38;5;247m 211[0m [38;5;254m// Each phase returns CompileResult[0m
[38;5;247m 212[0m [38;5;254mpub fn lex(source: &str) -> CompileResult<TokenStream> { ... }[0m
[38;5;247m 213[0m [38;5;254mpub fn parse(tokens: &TokenStream) -> CompileResult<Program> { ... }[0m
[38;5;247m 214[0m [38;5;254m```[0m
[38;5;247m 215[0m 
[38;5;247m 216[0m [38;5;254m**Confidence:** HIGH -- directly specified in ERRORS.md severity model.[0m
[38;5;247m 217[0m 
[38;5;247m 218[0m [38;5;254m### Pattern 6: String Builder for Python Code Generation[0m
[38;5;247m 219[0m 
[38;5;247m 220[0m [38;5;254m**What:** Use a structured `CodeWriter` with explicit indentation tracking rather than template strings or the `genco` crate. The writer tracks current indentation level, handles Python's whitespace sensitivity, and provides methods like `write_line()`, `indent()`, `dedent()`, `write_block()`.[0m
[38;5;247m 221[0m 
[38;5;247m 222[0m [38;5;254m**When:** For all Python code emission in eaml-codegen.[0m
[38;5;247m 223[0m 
[38;5;247m 224[0m [38;5;254m**Why not genco:** genco requires Rust 1.88+ for span information (EAML targets Rust 1.75+). genco adds complexity for a code generator that emits a single target language. A hand-rolled `CodeWriter` is simpler, has zero dependencies, and gives full control over output formatting.[0m
[38;5;247m 225[0m 
[38;5;247m 226[0m [38;5;254m**Why not raw format!/write!:** Python is whitespace-sensitive. Raw string formatting makes it too easy to get indentation wrong and produces unreadable codegen code.[0m
[38;5;247m 227[0m 
[38;5;247m 228[0m [38;5;254m**Example:**[0m
[38;5;247m 229[0m [38;5;254m```rust[0m
[38;5;247m 230[0m [38;5;254mpub struct CodeWriter {[0m
[38;5;247m 231[0m [38;5;254m    output: String,[0m
[38;5;247m 232[0m [38;5;254m    indent_level: usize,[0m
[38;5;247m 233[0m [38;5;254m    indent_str: &'static str,  // "    " (4 spaces for Python)[0m
[38;5;247m 234[0m [38;5;254m}[0m
[38;5;247m 235[0m 
[38;5;247m 236[0m [38;5;254mimpl CodeWriter {[0m
[38;5;247m 237[0m [38;5;254m    pub fn write_line(&mut self, line: &str) { ... }[0m
[38;5;247m 238[0m [38;5;254m    pub fn indent(&mut self) { self.indent_level += 1; }[0m
[38;5;247m 239[0m [38;5;254m    pub fn dedent(&mut self) { self.indent_level -= 1; }[0m
[38;5;247m 240[0m [38;5;254m    pub fn blank_line(&mut self) { ... }[0m
[38;5;247m 241[0m 
[38;5;247m 242[0m [38;5;254m    /// Write a block with automatic indent/dedent[0m
[38;5;247m 243[0m [38;5;254m    pub fn write_block(&mut self, header: &str, f: impl FnOnce(&mut Self)) {[0m
[38;5;247m 244[0m [38;5;254m        self.write_line(header);[0m
[38;5;247m 245[0m [38;5;254m        self.indent();[0m
[38;5;247m 246[0m [38;5;254m        f(self);[0m
[38;5;247m 247[0m [38;5;254m        self.dedent();[0m
[38;5;247m 248[0m [38;5;254m    }[0m
[38;5;247m 249[0m [38;5;254m}[0m
[38;5;247m 250[0m [38;5;254m```[0m
[38;5;247m 251[0m 
[38;5;247m 252[0m [38;5;254m**Confidence:** HIGH -- standard approach in compilers targeting Python. genco's Rust 1.88 requirement makes it incompatible.[0m
[38;5;247m 253[0m 
[38;5;247m 254[0m [38;5;254m### Pattern 7: Symbol Table as Flat HashMap with Scope Tracking[0m
[38;5;247m 255[0m 
[38;5;247m 256[0m [38;5;254m**What:** Use a single `HashMap<Spur, SymbolInfo>` for the symbol table since EAML v0.1 has flat scope (no nested scopes, no imports crossing files). The interner key (`Spur` from lasso) serves as the lookup key.[0m
[38;5;247m 257[0m 
[38;5;247m 258[0m [38;5;254m**When:** For v0.1 specifically. If nested scopes are added post-MVP, this would need to become a scope stack.[0m
[38;5;247m 259[0m 
[38;5;247m 260[0m [38;5;254m**Why:** EAML v0.1 has only top-level declarations (model, schema, prompt, tool, agent, let). There are no nested scopes, no closures, no block-scoped variables. A flat map is the simplest correct solution.[0m
[38;5;247m 261[0m 
[38;5;247m 262[0m [38;5;254m**Example:**[0m
[38;5;247m 263[0m [38;5;254m```rust[0m
[38;5;247m 264[0m [38;5;254mpub struct SymbolTable {[0m
[38;5;247m 265[0m [38;5;254m    symbols: HashMap<Spur, SymbolInfo>,[0m
[38;5;247m 266[0m [38;5;254m}[0m
[38;5;247m 267[0m 
[38;5;247m 268[0m [38;5;254mpub struct SymbolInfo {[0m
[38;5;247m 269[0m [38;5;254m    pub kind: SymbolKind,        // Model, Schema, Prompt, Tool, Agent, Let, Primitive[0m
[38;5;247m 270[0m [38;5;254m    pub declared_at: Span,[0m
[38;5;247m 271[0m [38;5;254m    pub type_info: TypeInfo,     // resolved type information[0m
[38;5;247m 272[0m [38;5;254m}[0m
[38;5;247m 273[0m 
[38;5;247m 274[0m [38;5;254mpub enum SymbolKind {[0m
[38;5;247m 275[0m [38;5;254m    Model,[0m
[38;5;247m 276[0m [38;5;254m    Schema,[0m
[38;5;247m 277[0m [38;5;254m    Prompt,[0m
[38;5;247m 278[0m [38;5;254m    Tool,[0m
[38;5;247m 279[0m [38;5;254m    Agent,[0m
[38;5;247m 280[0m [38;5;254m    Let,[0m
[38;5;247m 281[0m [38;5;254m    Primitive,  // string, int, float, bool, null[0m
[38;5;247m 282[0m [38;5;254m    BuiltinConstructor,  // Model()[0m
[38;5;247m 283[0m [38;5;254m}[0m
[38;5;247m 284[0m [38;5;254m```[0m
[38;5;247m 285[0m 
[38;5;247m 286[0m [38;5;254m**Confidence:** HIGH -- EAML v0.1 scope rules are flat per the grammar and Layer 5.[0m
[38;5;247m 287[0m 
[38;5;247m 288[0m [38;5;254m### Pattern 8: Parser Error Recovery via Synchronization Points[0m
[38;5;247m 289[0m 
[38;5;247m 290[0m [38;5;254m**What:** When the parser encounters an unexpected token, skip tokens until reaching a synchronization point (a token that can start a new declaration or close the current block). Emit an error node in the AST to preserve the span information.[0m
[38;5;247m 291[0m 
[38;5;247m 292[0m [38;5;254m**When:** During parsing of any production that encounters an unexpected token.[0m
[38;5;247m 293[0m 
[38;5;247m 294[0m [38;5;254m**Why:** The grammar has clear synchronization points: declaration keywords (`model`, `schema`, `prompt`, `tool`, `agent`, `import`, `let`) at top level, `}` for block recovery, `)` for argument list recovery.[0m
[38;5;247m 295[0m 
[38;5;247m 296[0m [38;5;254m**Example:**[0m
[38;5;247m 297[0m [38;5;254m```rust[0m
[38;5;247m 298[0m [38;5;254mfn synchronize(&mut self) {[0m
[38;5;247m 299[0m [38;5;254m    // Skip tokens until we find a declaration keyword or EOF[0m
[38;5;247m 300[0m [38;5;254m    while !self.at_end() {[0m
[38;5;247m 301[0m [38;5;254m        match self.peek() {[0m
[38;5;247m 302[0m [38;5;254m            Token::Model | Token::Schema | Token::Prompt |[0m
[38;5;247m 303[0m [38;5;254m            Token::Tool | Token::Agent | Token::Import |[0m
[38;5;247m 304[0m [38;5;254m            Token::Let => return,[0m
[38;5;247m 305[0m [38;5;254m            Token::RBrace => {[0m
[38;5;247m 306[0m [38;5;254m                self.advance(); // consume the closing brace[0m
[38;5;247m 307[0m [38;5;254m                return;[0m
[38;5;247m 308[0m [38;5;254m            }[0m
[38;5;247m 309[0m [38;5;254m            _ => { self.advance(); }[0m
[38;5;247m 310[0m [38;5;254m        }[0m
[38;5;247m 311[0m [38;5;254m    }[0m
[38;5;247m 312[0m [38;5;254m}[0m
[38;5;247m 313[0m [38;5;254m```[0m
[38;5;247m 314[0m 
[38;5;247m 315[0m [38;5;254m**Confidence:** HIGH -- standard recursive descent error recovery. matklad's resilient parsing tutorial confirms this approach works well for LL parsers.[0m
[38;5;247m 316[0m 
[38;5;247m 317[0m [38;5;254m## Anti-Patterns to Avoid[0m
[38;5;247m 318[0m 
[38;5;247m 319[0m [38;5;254m### Anti-Pattern 1: Mutable AST for Semantic Annotations[0m
[38;5;247m 320[0m 
[38;5;247m 321[0m [38;5;254m**What:** Adding `Option<ResolvedType>` fields to AST nodes that get filled in during semantic analysis.[0m
[38;5;247m 322[0m 
[38;5;247m 323[0m [38;5;254m**Why bad:** Couples the parser's data structures to the semantic phase. Makes it impossible to keep the parser crate independent. Forces `RefCell` or `unsafe` for interior mutability during tree walks.[0m
[38;5;247m 324[0m 
[38;5;247m 325[0m [38;5;254m**Instead:** Use `NodeId` keys and side tables in `eaml-semantic`. The AST is immutable after parsing.[0m
[38;5;247m 326[0m 
[38;5;247m 327[0m [38;5;254m### Anti-Pattern 2: Trait Objects for AST Nodes[0m
[38;5;247m 328[0m 
[38;5;247m 329[0m [38;5;254m**What:** Using `dyn AstNode` trait objects for the AST.[0m
[38;5;247m 330[0m 
[38;5;247m 331[0m [38;5;254m**Why bad:** Loses exhaustive matching. Requires heap allocation for every node. Makes pattern matching impossible -- must use downcasting. Adds runtime cost for zero benefit.[0m
[38;5;247m 332[0m 
[38;5;247m 333[0m [38;5;254m**Instead:** Use enums with struct variants. Rust's enum system is purpose-built for this.[0m
[38;5;247m 334[0m 
[38;5;247m 335[0m [38;5;254m### Anti-Pattern 3: Template Files for Code Generation[0m
[38;5;247m 336[0m 
[38;5;247m 337[0m [38;5;254m**What:** Storing Python code templates as `.py.tmpl` files and using string interpolation to fill them in.[0m
[38;5;247m 338[0m 
[38;5;247m 339[0m [38;5;254m**Why bad:** Templates become unreadable as conditional logic grows. Hard to debug generated code. Template engines add dependencies. Python indentation errors are invisible in templates.[0m
[38;5;247m 340[0m 
[38;5;247m 341[0m [38;5;254m**Instead:** Use a `CodeWriter` builder that makes indentation explicit and keeps generation logic in Rust code where the compiler can check it.[0m
[38;5;247m 342[0m 
[38;5;247m 343[0m [38;5;254m### Anti-Pattern 4: Single Error, Single Abort[0m
[38;5;247m 344[0m 
[38;5;247m 345[0m [38;5;254m**What:** Returning `Result<T, Diagnostic>` (single error) and stopping on first failure.[0m
[38;5;247m 346[0m 
[38;5;247m 347[0m [38;5;254m**Why bad:** Users must fix one error at a time, recompile, and discover the next. Extremely poor developer experience.[0m
[38;5;247m 348[0m 
[38;5;247m 349[0m [38;5;254m**Instead:** Accumulate `Vec<Diagnostic>` and continue processing. Only abort on FATAL severity.[0m
[38;5;247m 350[0m 
[38;5;247m 351[0m [38;5;254m### Anti-Pattern 5: Monolithic Semantic Pass[0m
[38;5;247m 352[0m 
[38;5;247m 353[0m [38;5;254m**What:** Doing name resolution, type checking, and capability checking in a single AST walk.[0m
[38;5;247m 354[0m 
[38;5;247m 355[0m [38;5;254m**Why bad:** Name resolution must complete before type checking can resolve references. Interleaving the passes creates ordering bugs where a forward-declared name is not yet in the symbol table when referenced.[0m
[38;5;247m 356[0m 
[38;5;247m 357[0m [38;5;254m**Instead:** Three separate passes, as specified. Each pass can assume the previous pass's data is complete.[0m
[38;5;247m 358[0m 
[38;5;247m 359[0m [38;5;254m### Anti-Pattern 6: Generated Code Calling Provider APIs Directly[0m
[38;5;247m 360[0m 
[38;5;247m 361[0m [38;5;254m**What:** Having codegen emit `anthropic.messages.create(...)` calls directly in generated code.[0m
[38;5;247m 362[0m 
[38;5;247m 363[0m [38;5;254m**Why bad:** Provider API changes break all previously compiled code. No retry logic, no validation, no telemetry. Every provider needs different codegen paths.[0m
[38;5;247m 364[0m 
[38;5;247m 365[0m [38;5;254m**Instead:** Generated code calls `eaml_runtime` functions (e.g., `eaml_runtime.call_prompt(...)`) which handle provider dispatch, retry, and validation internally.[0m
[38;5;247m 366[0m 
[38;5;247m 367[0m [38;5;254m## Python Runtime Architecture[0m
[38;5;247m 368[0m 
[38;5;247m 369[0m [38;5;254mThe `eaml-runtime` Python package is the bridge between generated code and LLM providers. It must be designed as a stable API surface that generated code can depend on.[0m
[38;5;247m 370[0m 
[38;5;247m 371[0m [38;5;254m### Runtime Component Structure[0m
[38;5;247m 372[0m 
[38;5;247m 373[0m [38;5;254m```[0m
[38;5;247m 374[0m [38;5;254meaml_runtime/[0m
[38;5;247m 375[0m [38;5;254m    __init__.py          # Public API: call_prompt, call_tool, validate_output[0m
[38;5;247m 376[0m [38;5;254m    _types.py            # Internal type definitions (PromptConfig, ToolConfig, etc.)[0m
[38;5;247m 377[0m [38;5;254m    _validation.py       # Pydantic validation with retry logic (validate_or_retry)[0m
[38;5;247m 378[0m [38;5;254m    _telemetry.py        # Telemetry hooks (optional, pluggable)[0m
[38;5;247m 379[0m [38;5;254m    providers/[0m
[38;5;247m 380[0m [38;5;254m        __init__.py      # ProviderRegistry, get_provider()[0m
[38;5;247m 381[0m [38;5;254m        _base.py         # Abstract base: BaseProvider protocol[0m
[38;5;247m 382[0m [38;5;254m        _anthropic.py    # Anthropic adapter (anthropic SDK)[0m
[38;5;247m 383[0m [38;5;254m        _openai.py       # OpenAI adapter (openai SDK)[0m
[38;5;247m 384[0m [38;5;254m        _ollama.py       # Ollama adapter (httpx)[0m
[38;5;247m 385[0m [38;5;254m    exceptions.py        # EamlRuntimeError, ProviderError, ValidationError[0m
[38;5;247m 386[0m [38;5;254m```[0m
[38;5;247m 387[0m 
[38;5;247m 388[0m [38;5;254m### Generated Code Contract[0m
[38;5;247m 389[0m 
[38;5;247m 390[0m [38;5;254mGenerated Python code MUST only depend on:[0m
[38;5;247m 391[0m [38;5;254m1. `eaml_runtime` public API (stable)[0m
[38;5;247m 392[0m [38;5;254m2. `pydantic.BaseModel` (for schema classes)[0m
[38;5;247m 393[0m [38;5;254m3. Python stdlib (`typing`, `asyncio`)[0m
[38;5;247m 394[0m 
[38;5;247m 395[0m [38;5;254mGenerated code MUST NOT:[0m
[38;5;247m 396[0m [38;5;254m- Import provider SDKs directly[0m
[38;5;247m 397[0m [38;5;254m- Contain retry logic[0m
[38;5;247m 398[0m [38;5;254m- Contain validation logic beyond Pydantic model definitions[0m
[38;5;247m 399[0m [38;5;254m- Contain hardcoded API keys or URLs[0m
[38;5;247m 400[0m 
[38;5;247m 401[0m [38;5;254m### Example Generated Code Shape[0m
[38;5;247m 402[0m 
[38;5;247m 403[0m [38;5;254mFor the sentiment example, codegen should produce:[0m
[38;5;247m 404[0m 
[38;5;247m 405[0m [38;5;254m```python[0m
[38;5;247m 406[0m [38;5;254mfrom __future__ import annotations[0m
[38;5;247m 407[0m [38;5;254mfrom typing import Literal[0m
[38;5;247m 408[0m [38;5;254mfrom pydantic import BaseModel, Field[0m
[38;5;247m 409[0m [38;5;254mimport eaml_runtime[0m
[38;5;247m 410[0m 
[38;5;247m 411[0m [38;5;254mclass SentimentResult(BaseModel):[0m
[38;5;247m 412[0m [38;5;254m    sentiment: Literal["positive", "neutral", "negative"][0m
[38;5;247m 413[0m [38;5;254m    confidence: float = Field(ge=0.0, le=1.0)[0m
[38;5;247m 414[0m [38;5;254m    explanation: str[0m
[38;5;247m 415[0m 
[38;5;247m 416[0m [38;5;254masync def analyze_sentiment(text: str) -> SentimentResult:[0m
[38;5;247m 417[0m [38;5;254m    return await eaml_runtime.call_prompt([0m
[38;5;247m 418[0m [38;5;254m        model="anthropic/claude-3-5-sonnet-20241022",[0m
[38;5;247m 419[0m [38;5;254m        provider="anthropic",[0m
[38;5;247m 420[0m [38;5;254m        messages=[[0m
[38;5;247m 421[0m [38;5;254m            {"role": "system", "content": "You are a sentiment analysis expert..."},[0m
[38;5;247m 422[0m [38;5;254m            {"role": "user", "content": f"Analyze the sentiment...\n\n{text}"},[0m
[38;5;247m 423[0m [38;5;254m        ],[0m
[38;5;247m 424[0m [38;5;254m        response_model=SentimentResult,[0m
[38;5;247m 425[0m [38;5;254m        temperature=0.2,[0m
[38;5;247m 426[0m [38;5;254m        max_tokens=256,[0m
[38;5;247m 427[0m [38;5;254m        capabilities=["json_mode"],[0m
[38;5;247m 428[0m [38;5;254m    )[0m
[38;5;247m 429[0m [38;5;254m```[0m
[38;5;247m 430[0m 
[38;5;247m 431[0m [38;5;254m### Provider Adapter Pattern[0m
[38;5;247m 432[0m 
[38;5;247m 433[0m [38;5;254m```python[0m
[38;5;247m 434[0m [38;5;254mfrom typing import Protocol, Any[0m
[38;5;247m 435[0m [38;5;254mfrom pydantic import BaseModel[0m
[38;5;247m 436[0m 
[38;5;247m 437[0m [38;5;254mclass ProviderAdapter(Protocol):[0m
[38;5;247m 438[0m [38;5;254m    async def complete([0m
[38;5;247m 439[0m [38;5;254m        self,[0m
[38;5;247m 440[0m [38;5;254m        model_id: str,[0m
[38;5;247m 441[0m [38;5;254m        messages: list[dict[str, str]],[0m
[38;5;247m 442[0m [38;5;254m        response_model: type[BaseModel] | None,[0m
[38;5;247m 443[0m [38;5;254m        temperature: float | None,[0m
[38;5;247m 444[0m [38;5;254m        max_tokens: int | None,[0m
[38;5;247m 445[0m [38;5;254m        capabilities: list[str],[0m
[38;5;247m 446[0m [38;5;254m    ) -> Any: ...[0m
[38;5;247m 447[0m [38;5;254m```[0m
[38;5;247m 448[0m 
[38;5;247m 449[0m [38;5;254m## Scalability Considerations[0m
[38;5;247m 450[0m 
[38;5;247m 451[0m [38;5;254m| Concern | At 10 files | At 100 files | At 1000+ files |[0m
[38;5;247m 452[0m [38;5;254m|---------|-------------|--------------|----------------|[0m
[38;5;247m 453[0m [38;5;254m| Parse time | Negligible | Negligible (logos is very fast) | Still fast -- logos benchmarks at GB/s |[0m
[38;5;247m 454[0m [38;5;254m| Symbol table | Flat HashMap | Flat HashMap (still works) | May need arena allocation for interned strings |[0m
[38;5;247m 455[0m [38;5;254m| Type checking | Single-file, instant | Needs multi-file support (post-MVP) | Incremental checking needed |[0m
[38;5;247m 456[0m [38;5;254m| Code generation | Single output file | One output per input file | Parallel codegen possible |[0m
[38;5;247m 457[0m [38;5;254m| String interning | Single ThreadedRodeo | Share across files | lasso handles this well |[0m
[38;5;247m 458[0m 
[38;5;247m 459[0m [38;5;254mFor v0.1 (single-file compilation), scalability is not a concern. The architecture supports future multi-file compilation by keeping the symbol table and type environment as separate data structures that can be extended.[0m
[38;5;247m 460[0m 
[38;5;247m 461[0m [38;5;254m## Suggested Build Order[0m
[38;5;247m 462[0m 
[38;5;247m 463[0m [38;5;254mBased on crate dependencies and the pipeline architecture:[0m
[38;5;247m 464[0m 
[38;5;247m 465[0m [38;5;254m1. **eaml-errors** (foundation) -- Diagnostic, Span, Severity, ErrorCode. No dependencies. Must be stable before anything else.[0m
[38;5;247m 466[0m [38;5;254m2. **eaml-lexer** -- Token enum, lexer modes, string interning. Depends only on eaml-errors.[0m
[38;5;247m 467[0m [38;5;254m3. **eaml-parser** -- AST types, recursive descent parser, error recovery. Depends on eaml-errors + eaml-lexer.[0m
[38;5;247m 468[0m [38;5;254m4. **eaml-semantic** -- Symbol table, type checker, capability checker (three passes). Depends on eaml-errors + eaml-lexer + eaml-parser.[0m
[38;5;247m 469[0m [38;5;254m5. **eaml-codegen** -- CodeWriter, Python emission. Depends on eaml-errors + eaml-parser + eaml-semantic.[0m
[38;5;247m 470[0m [38;5;254m6. **eaml-cli** -- clap CLI, pipeline orchestration. Depends on all crates.[0m
[38;5;247m 471[0m [38;5;254m7. **eaml-runtime** (Python) -- Can be built in parallel with steps 4-5 since its API contract is defined by the codegen output shape.[0m
[38;5;247m 472[0m 
[38;5;247m 473[0m [38;5;254m**Critical path:** errors -> lexer -> parser -> semantic -> codegen -> cli[0m
[38;5;247m 474[0m [38;5;254m**Parallel path:** eaml-runtime can proceed alongside semantic/codegen work.[0m
[38;5;247m 475[0m 
[38;5;247m 476[0m [38;5;254m## Sources[0m
[38;5;247m 477[0m 
[38;5;247m 478[0m [38;5;254m- [Resilient LL Parsing Tutorial (matklad)](https://matklad.github.io/2023/05/21/resilient-ll-parsing-tutorial.html) -- Error recovery patterns for hand-written parsers[0m
[38;5;247m 479[0m [38;5;254m- [Ruff v0.4.0: hand-written recursive descent parser](https://astral.sh/blog/ruff-v0.4.0) -- Production example of logos + hand-written parser in Rust[0m
[38;5;247m 480[0m [38;5;254m- [Rust Compiler Development Guide: Overview](https://rustc-dev-guide.rust-lang.org/overview.html) -- AST/HIR/MIR pipeline architecture[0m
[38;5;247m 481[0m [38;5;254m- [genco: whitespace-aware quasiquoter](https://github.com/udoprog/genco) -- Evaluated but rejected (requires Rust 1.88+)[0m
[38;5;247m 482[0m [38;5;254m- [BAML Compiler Architecture (DeepWiki)](https://deepwiki.com/BoundaryML/baml) -- Prior art for LLM DSL compiler targeting Python[0m
[38;5;247m 483[0m [38;5;254m- [Logos Handbook](https://logos.maciej.codes/) -- Custom error types and lexer configuration[0m
[38;5;247m 484[0m [38;5;254m- [Build a Compiler: Symbol Table](https://marcauberer.medium.com/build-a-compiler-symbol-table-2d4582234112) -- Symbol table design patterns[0m
[38;5;247m 485[0m [38;5;254m- [Parser patterns (oxc)](https://oxc.rs/docs/learn/parser_in_rust/parser) -- High-performance Rust parser patterns[0m
