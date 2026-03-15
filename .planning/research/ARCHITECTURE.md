# Architecture Patterns

**Domain:** Rust compiler for LLM integration DSL targeting Python/Pydantic
**Researched:** 2026-03-15

## Recommended Architecture

EAML follows a classic multi-pass compiler pipeline with strict crate boundaries already defined. This document specifies the internal architecture of each compiler phase and the Python runtime that generated code calls into.

```
Source (.eaml)
    |
    v
[eaml-errors]  <-- shared Diagnostic, Span, ErrorCode types
    |
    v
[eaml-lexer]   --> Token stream (logos-derived, lasso-interned)
    |
    v
[eaml-parser]  --> AST (typed enum tree, arena-optional)
    |
    v
[eaml-semantic] --> AnalyzedProgram (symbol table + type info + capability graph)
    |   Pass 1: Name Resolution (populate symbol table)
    |   Pass 2: Type Checking (resolve types, validate bounds)
    |   Pass 3: Capability Checking (subset check)
    |
    v
[eaml-codegen] --> Python source string (builder pattern, not templates)
    |
    v
[eaml-cli]     --> eamlc binary (clap, orchestrates pipeline)
    |
    v
Generated .py  --> imports eaml_runtime (provider adapters, validation, telemetry)
```

### Component Boundaries

| Component | Responsibility | Communicates With |
|-----------|---------------|-------------------|
| eaml-errors | Diagnostic, Span, Severity, ErrorCode enums, codespan-reporting display | All crates depend on it |
| eaml-lexer | Tokenization, string interning, comment skipping, mode switching (template strings, python blocks) | Produces TokenStream consumed by parser |
| eaml-parser | Recursive descent parsing, AST construction, syntactic error recovery | Consumes TokenStream, produces Program (AST root) |
| eaml-semantic | Name resolution, type checking, capability checking, symbol table construction | Consumes Program, produces AnalyzedProgram with type annotations |
| eaml-codegen | Python/Pydantic code emission from analyzed AST | Consumes AnalyzedProgram, produces Python source String |
| eaml-cli | CLI argument parsing, pipeline orchestration, file I/O, error display | Orchestrates all crates |
| eaml-runtime (Python) | Provider adapters, retry logic, Pydantic validation at runtime, telemetry hooks | Called by generated Python code |

### Data Flow

```
                    Source text (&str)
                         |
                    eaml-lexer::lex()
                         |
                    TokenStream {
                      tokens: Vec<Token>,   // kind + span
                      interner: ThreadedRodeo,  // lasso string interner
                      diagnostics: Vec<Diagnostic>,
                    }
                         |
                    eaml-parser::parse()
                         |
                    Program {
                      declarations: Vec<Declaration>,
                      diagnostics: Vec<Diagnostic>,
                    }
                         |
                    eaml-semantic::analyze()
                         |
                    AnalyzedProgram {
                      program: Program,                // original AST
                      symbol_table: SymbolTable,       // name -> SymbolInfo
                      type_env: TypeEnvironment,       // node_id -> ResolvedType
                      capability_graph: CapabilityGraph,
                      diagnostics: Vec<Diagnostic>,
                    }
                         |
                    eaml-codegen::generate()
                         |
                    String  (Python source code)
```

## Patterns to Follow

### Pattern 1: Typed AST with Enum Variants (not trait objects)

**What:** Represent AST nodes as Rust enums with struct variants. Each declaration type, expression type, and type expression gets its own variant. Use Box<T> for recursive nodes.

**When:** Always -- this is the standard pattern for small-to-medium language compilers in Rust.

**Why:** Enum dispatch is exhaustive (compiler catches missing match arms), zero-cost (no vtable), and pattern-matchable. Trait objects would add unnecessary indirection and lose exhaustiveness checking.

**Example:**
```rust
/// Top-level declaration
pub enum Declaration {
    Model(ModelDecl),
    Schema(SchemaDecl),
    Prompt(PromptDecl),
    Tool(ToolDecl),
    Agent(AgentDecl),
    Import(ImportDecl),
    Let(LetDecl),
}

pub struct SchemaDecl {
    pub name: Spanned<Spur>,
    pub fields: Vec<SchemaField>,
    pub span: Span,
}

pub struct SchemaField {
    pub name: Spanned<Spur>,
    pub ty: TypeExpr,
    pub span: Span,
}

pub enum TypeExpr {
    Primitive(PrimitiveType, Span),
    Named(Spanned<Spur>),
    Array(Box<TypeExpr>, Span),
    Optional(Box<TypeExpr>, Span),
    Bounded(Box<TypeExpr>, Vec<BoundParam>, Span),
    LiteralUnion(Vec<Literal>, Span),
}
```

**Confidence:** HIGH -- dominant pattern in rustc, rust-analyzer, ruff, oxc.

### Pattern 2: Spanned Everything

**What:** Every AST node and sub-node carries a Span (byte offset range into source). Wrap identifiers in Spanned<T> which pairs the value with its span.

**When:** Always. Spans are essential for error reporting via codespan-reporting.

**Example:**
```rust
pub struct Span {
    pub start: usize,
    pub end: usize,
}

pub struct Spanned<T> {
    pub value: T,
    pub span: Span,
}
```

**Confidence:** HIGH -- universal in production compilers. codespan-reporting requires byte offsets.

### Pattern 3: NodeId for Cross-referencing (Semantic Phase)

**What:** Assign a unique NodeId to each AST node during parsing. The semantic phase uses these IDs as keys into side tables (type environment, resolution map) rather than mutating the AST.

**When:** When the semantic phase needs to annotate nodes with resolved type information without modifying the parser's AST types.

**Why:** Avoids making the AST mutable or adding generic type parameters. Side tables are simpler and allow the semantic crate to own its data independently.

**Example:**
```rust
// In eaml-parser
pub struct NodeId(u32);

pub struct SchemaDecl {
    pub id: NodeId,
    pub name: Spanned<Spur>,
    pub fields: Vec<SchemaField>,
    pub span: Span,
}

// In eaml-semantic
pub struct TypeEnvironment {
    types: HashMap<NodeId, ResolvedType>,
}
```

**Confidence:** HIGH -- used by rustc (DefId), TypeScript (node IDs), rust-analyzer.

### Pattern 4: Multi-Pass Semantic Analysis with Separate Concerns

**What:** Three distinct passes over the AST, each with a single responsibility:
1. Name Resolution: Walk all declarations, populate symbol table. Detect duplicate names (RES010). Pre-populate primitive types.
2. Type Checking: Walk expressions and type annotations. Resolve named types. Validate bounded types, literal unions, composite type ordering.
3. Capability Checking: Extract requires clauses from prompts, caps from models. Perform subset check. Emit CAP010 on mismatch.

**When:** Always -- the spec mandates this three-pass structure.

**Why:** Each pass can assume the previous pass completed. Name resolution must complete before type checking can resolve schema references.

**Confidence:** HIGH -- matches spec/ERRORS.md phase definitions and Layer 5 decisions.

### Pattern 5: Error Accumulation with Continuation

**What:** Never abort on the first error. Each compiler phase accumulates Vec<Diagnostic> and continues processing. The CLI decides when to stop based on error count and severity.

**When:** Always. The spec defines a max-errors limit (default 20) and three severity levels (FATAL, ERROR, WARNING).

**Example:**
```rust
pub struct CompileResult<T> {
    pub value: Option<T>,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn lex(source: &str) -> CompileResult<TokenStream> { ... }
pub fn parse(tokens: &TokenStream) -> CompileResult<Program> { ... }
```

**Confidence:** HIGH -- directly specified in ERRORS.md severity model.

### Pattern 6: String Builder for Python Code Generation

**What:** Use a structured CodeWriter with explicit indentation tracking rather than template strings or the genco crate.

**When:** For all Python code emission in eaml-codegen.

**Why not genco:** genco requires Rust 1.88+ for span information (EAML targets Rust 1.75+). A hand-rolled CodeWriter is simpler, has zero dependencies, and gives full control.

**Why not raw format!/write!:** Python is whitespace-sensitive. Raw string formatting makes it too easy to get indentation wrong.

**Example:**
```rust
pub struct CodeWriter {
    output: String,
    indent_level: usize,
    indent_str: &'static str,  // "    " (4 spaces for Python)
}

impl CodeWriter {
    pub fn write_line(&mut self, line: &str) { ... }
    pub fn indent(&mut self) { self.indent_level += 1; }
    pub fn dedent(&mut self) { self.indent_level -= 1; }
    pub fn blank_line(&mut self) { ... }

    pub fn write_block(&mut self, header: &str, f: impl FnOnce(&mut Self)) {
        self.write_line(header);
        self.indent();
        f(self);
        self.dedent();
    }
}
```

**Confidence:** HIGH -- standard approach for Python-targeting compilers.

### Pattern 7: Symbol Table as Flat HashMap

**What:** Use a single HashMap<Spur, SymbolInfo> for the symbol table since EAML v0.1 has flat scope (no nested scopes).

**When:** For v0.1 specifically. Post-MVP scope additions would need a scope stack.

**Why:** EAML v0.1 has only top-level declarations. No nested scopes, closures, or block-scoped variables.

**Example:**
```rust
pub struct SymbolTable {
    symbols: HashMap<Spur, SymbolInfo>,
}

pub struct SymbolInfo {
    pub kind: SymbolKind,
    pub declared_at: Span,
    pub type_info: TypeInfo,
}

pub enum SymbolKind {
    Model, Schema, Prompt, Tool, Agent, Let,
    Primitive, BuiltinConstructor,
}
```

**Confidence:** HIGH -- EAML v0.1 scope rules are flat per grammar and Layer 5.

### Pattern 8: Parser Error Recovery via Synchronization Points

**What:** When the parser encounters an unexpected token, skip tokens until reaching a synchronization point. Emit an error diagnostic and continue parsing.

**When:** During parsing of any production that encounters an unexpected token.

**Why:** The grammar has clear synchronization points: declaration keywords at top level, closing braces for block recovery, closing parens for argument list recovery.

**Example:**
```rust
fn synchronize(&mut self) {
    while !self.at_end() {
        match self.peek() {
            Token::Model | Token::Schema | Token::Prompt |
            Token::Tool | Token::Agent | Token::Import |
            Token::Let => return,
            Token::RBrace => {
                self.advance();
                return;
            }
            _ => { self.advance(); }
        }
    }
}
```

**Confidence:** HIGH -- standard recursive descent error recovery.

## Anti-Patterns to Avoid

### Anti-Pattern 1: Mutable AST for Semantic Annotations

**What:** Adding Option<ResolvedType> fields to AST nodes filled during semantic analysis.
**Why bad:** Couples parser to semantic types. Forces RefCell or unsafe. Breaks strict crate boundaries.
**Instead:** Use NodeId + side tables. AST is immutable after parsing.

### Anti-Pattern 2: Trait Objects for AST Nodes

**What:** Using dyn AstNode trait objects.
**Why bad:** Loses exhaustive matching. Requires heap allocation. No pattern matching.
**Instead:** Use enums with struct variants.

### Anti-Pattern 3: Template Files for Code Generation

**What:** Storing Python code templates as .py.tmpl files with string interpolation.
**Why bad:** Templates become unreadable with conditionals. Indentation errors invisible.
**Instead:** Use CodeWriter builder with explicit indentation.

### Anti-Pattern 4: Single Error, Single Abort

**What:** Returning Result<T, Diagnostic> (single error) and stopping on first failure.
**Why bad:** Users must fix one error at a time.
**Instead:** Accumulate Vec<Diagnostic>. Only FATAL aborts immediately.

### Anti-Pattern 5: Monolithic Semantic Pass

**What:** Name resolution, type checking, and capability checking in one AST walk.
**Why bad:** Forward references break single-pass resolution.
**Instead:** Three separate passes per spec.

### Anti-Pattern 6: Generated Code Calling Provider APIs Directly

**What:** Codegen emitting anthropic.messages.create() directly.
**Why bad:** Provider API changes break all compiled code. No retry/validation.
**Instead:** Generated code calls eaml_runtime.call_prompt().

## Python Runtime Architecture

### Runtime Component Structure

```
eaml_runtime/
    __init__.py          # Public API: call_prompt, call_tool, validate_output
    _types.py            # Internal type definitions (PromptConfig, ToolConfig)
    _validation.py       # Pydantic validation with retry (validate_or_retry)
    _telemetry.py        # Telemetry hooks (optional, pluggable)
    providers/
        __init__.py      # ProviderRegistry, get_provider()
        _base.py         # Abstract base: BaseProvider protocol
        _anthropic.py    # Anthropic adapter
        _openai.py       # OpenAI adapter
        _ollama.py       # Ollama adapter (httpx)
    exceptions.py        # EamlRuntimeError, ProviderError, ValidationError
```

### Generated Code Contract

Generated Python code MUST only depend on:
1. eaml_runtime public API (stable)
2. pydantic.BaseModel (for schema classes)
3. Python stdlib (typing, asyncio)

Generated code MUST NOT:
- Import provider SDKs directly
- Contain retry logic
- Contain validation logic beyond Pydantic model definitions
- Contain hardcoded API keys or URLs

### Example Generated Code Shape

```python
from __future__ import annotations
from typing import Literal
from pydantic import BaseModel, Field
import eaml_runtime

class SentimentResult(BaseModel):
    sentiment: Literal["positive", "neutral", "negative"]
    confidence: float = Field(ge=0.0, le=1.0)
    explanation: str

async def analyze_sentiment(text: str) -> SentimentResult:
    return await eaml_runtime.call_prompt(
        model="anthropic/claude-3-5-sonnet-20241022",
        provider="anthropic",
        messages=[
            {"role": "system", "content": "You are a sentiment analysis expert..."},
            {"role": "user", "content": f"Analyze the sentiment...\n\n{text}"},
        ],
        response_model=SentimentResult,
        temperature=0.2,
        max_tokens=256,
        capabilities=["json_mode"],
    )
```

### Provider Adapter Pattern

```python
from typing import Protocol, Any
from pydantic import BaseModel

class ProviderAdapter(Protocol):
    async def complete(
        self,
        model_id: str,
        messages: list[dict[str, str]],
        response_model: type[BaseModel] | None,
        temperature: float | None,
        max_tokens: int | None,
        capabilities: list[str],
    ) -> Any: ...
```

## Scalability Considerations

| Concern | At 10 files | At 100 files | At 1000+ files |
|---------|-------------|--------------|----------------|
| Parse time | Negligible | Negligible | Still fast (logos GB/s) |
| Symbol table | Flat HashMap | Flat HashMap | May need arena allocation |
| Type checking | Single-file, instant | Needs multi-file (post-MVP) | Incremental checking needed |
| Code generation | Single output file | One output per input | Parallel codegen possible |
| String interning | Single ThreadedRodeo | Share across files | lasso handles this well |

## Suggested Build Order

1. **eaml-errors** -- Diagnostic, Span, Severity, ErrorCode. Foundation for all crates.
2. **eaml-lexer** -- Token enum, logos derive, lasso interning, mode switching.
3. **eaml-parser** -- AST types, recursive descent, error recovery.
4. **eaml-semantic** -- Symbol table, type checker, capability checker (three passes).
5. **eaml-codegen** -- CodeWriter, Python/Pydantic emission.
6. **eaml-cli** -- clap CLI, pipeline orchestration.
7. **eaml-runtime** (Python) -- Parallel with steps 4-5.

**Critical path:** errors -> lexer -> parser -> semantic -> codegen -> cli
**Parallel path:** eaml-runtime alongside semantic/codegen work.

## Sources

- [Resilient LL Parsing Tutorial (matklad)](https://matklad.github.io/2023/05/21/resilient-ll-parsing-tutorial.html)
- [Ruff v0.4.0: hand-written recursive descent parser](https://astral.sh/blog/ruff-v0.4.0)
- [Rust Compiler Development Guide: Overview](https://rustc-dev-guide.rust-lang.org/overview.html)
- [genco: whitespace-aware quasiquoter](https://github.com/udoprog/genco)
- [BAML Compiler Architecture (DeepWiki)](https://deepwiki.com/BoundaryML/baml)
- [Logos Handbook](https://logos.maciej.codes/)
- [Build a Compiler: Symbol Table](https://marcauberer.medium.com/build-a-compiler-symbol-table-2d4582234112)
- [Parser patterns (oxc)](https://oxc.rs/docs/learn/parser_in_rust/parser)
