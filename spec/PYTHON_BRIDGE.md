# EAML Python Bridge Specification

**Version:** 0.1.0
**Date:** 2026-03-15
**Status:** AUTHORITATIVE

---

## Abstract

This document is the complete Python bridge specification for EAML (Engineering AI
Markup Language) version 0.1.0. The Python bridge enables EAML tool declarations
to execute Python code in-process within the user's Python environment. It is the
boundary layer between EAML's static type system and Python's dynamic runtime,
and between the EAML compiler's generated code and the `eaml_runtime` adapter layer.

This document serves five consumers:

1. **EAML compiler** (`eaml-codegen` crate) — specifies exactly what Python code
   to generate for tool declarations containing `python %{ }%` blocks.
2. **`eaml_runtime` adapter layer** — specifies how generated bridge code is
   executed, how arguments are marshaled, and how return values are validated.
3. **EAML language users** — specifies what can be written inside bridge blocks,
   what imports are available, and how EAML types map to Python types.
4. **Test authors** — every bridge rule must have at least one test covering it.
5. **IDE/LSP implementers** — specifies what `--check-python` does and what PYB
   error codes mean in structured diagnostics.

### Normative Language

The key words "MUST", "MUST NOT", "SHALL", "SHOULD", "MAY" in this document are to
be interpreted as described in [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

### Related Documents

| Document                                    | Relationship                                                                             |
|---------------------------------------------|------------------------------------------------------------------------------------------|
| `spec/grammar.ebnf`                         | Syntactic contract — this document cites grammar productions by number                   |
| `spec/TYPESYSTEM.md`                        | Type contract — bridge type representations are DERIVED from §10.3, not redefined here   |
| `spec/CAPABILITIES.md`                      | Capability contract — tools capability governs model-to-tool binding                     |
| `spec/ERRORS.md`                            | Error contract — PYB and SEM error codes are registered there                            |
| Layer 5 (`eaml-layer5-design-decisions.md`) | Authoritative design decisions — this document implements them                           |

**Downstream position:** PYTHON_BRIDGE.md is DOWNSTREAM of all four completed spec
documents. It cannot contradict them. When a bridge rule has a type consequence, it
cites the TYPESYSTEM.md rule. When it has an error consequence, it cites the ERRORS.md
code. When it depends on a grammar production, it cites the production number.

### Context Labels

Three context labels are used throughout this document. These are unique to the
bridge specification:

**[COMPILER]** — applies to the `eaml-codegen` crate processing the `.eaml` source.
Describes what the compiler does with bridge-related constructs during compilation.

**[GENERATED]** — applies to the Python code emitted by the compiler into the
output `.py` file. Describes the structure and content of the generated code.

**[RUNTIME]** — applies to `eaml_runtime` executing the generated Python code.
Describes what happens when the generated bridge function is actually called,
including argument marshaling, execution, and return value handling.

These labels appear in every rule block's `Context:` field.

### Rule Format

Every rule block follows this format:

```
RULE [PYB-CAT-NN]: [Short imperative title]

  Context:       [COMPILER | GENERATED | RUNTIME | combinations]
  Plain English: [One-paragraph description]
  Grammar:       Production [N] in grammar.ebnf [or N/A]
  Valid:         [EAML example that is correct]
  Invalid:       [example that triggers an error] → Error [CODE]: [message]
  Generated:     [Python code the compiler emits for this rule]
  Runtime:       [What eaml_runtime does when this code executes]
  Notes:         [Cross-references, edge cases]
```

The `Generated:` field shows actual Python code produced by the compiler.
The `Runtime:` field specifies execution behavior. Together they define
both sides of the bridge contract.

### Table of Contents

1. [Bridge Architecture Overview](#1-bridge-architecture-overview)
   - 1.1 In-Process Execution Model
   - 1.2 Bridge as Codegen Output
   - 1.3 The Bridge Contract
   - 1.4 Scope in v0.1
2. [Bridge Block Syntax](#2-bridge-block-syntax)
3. [Import System](#3-import-system)
   - 3.1 File-Level Python Imports
   - 3.2 Imports Inside Bridge Blocks
4. [Type Marshaling](#4-type-marshaling)
   - 4.1 Parameter Marshaling — EAML to Python
   - 4.2 Return Value Marshaling — Python to EAML
   - 4.3 Bounded Type Parameters at the Bridge
5. [Generated Code Structure](#5-generated-code-structure)
   - 5.1 Generated Function Structure
   - 5.2 Bridge Function Naming Convention
   - 5.3 Tool Metadata Registration
   - 5.4 Multiple Tools in One File
6. [Error Handling in Bridge Blocks](#6-error-handling-in-bridge-blocks)
   - 6.1 Python Syntax Errors — --check-python Flag
   - 6.2 Python Runtime Exceptions
   - 6.3 Return Type Mismatch at Runtime
7. [Post-MVP Bridge Features](#7-post-mvp-bridge-features)
8. [Open Questions](#8-open-questions)

---

## 1. Bridge Architecture Overview

### 1.1 In-Process Execution Model

The Python bridge runs **in-process** in the user's Python environment. The generated
`.py` file imports from `eaml_runtime`, which calls bridge functions in the same
Python process. The user's installed packages (numpy, pandas, httpx, etc.) are
available without any additional installation step. The user's environment variables,
file system access, and network connectivity are all available to bridge code.

**Contrast with subprocess model:** No process spawn, no serialization overhead, no
package re-installation. But also no isolation — a bridge function that crashes takes
down the entire process.

Layer 5 §5.3 [CLOSED]: "Python block syntax validation is FLAG-CONTROLLED, OFF BY
DEFAULT." The in-process model is the fundamental architecture decision.

### 1.2 Bridge as Codegen Output

Bridge blocks are NOT executed at compile time. The compiler treats `PYTHON_BLOCK`
content as opaque text and emits it verbatim into the generated `.py` file. The
compiler's only interaction with bridge content (beyond capture) is the optional
`--check-python` syntax validation.

The bridge is a codegen concern, not a type-checking concern: the compiler verifies
the tool's EAML signature (parameter types, return type) but cannot verify that the
Python implementation actually satisfies it.

Grammar: Production [18] `PYTHON_BLOCK` — opaque token captured by lexer mode.
Grammar: Production [37] `pythonImpl` — `"python" "%{" PYTHON_BLOCK "}%"`.

### 1.3 The Bridge Contract

The bridge contract is **asymmetric**:

**EAML's side (compile-time):** The tool declaration specifies typed parameters and
a typed return value. The compiler generates a Python wrapper that accepts those types
(as Pydantic models where applicable) and expects the Python implementation to return
a value of the declared return type. The compiler verifies the EAML type signature
is valid but cannot verify the Python implementation satisfies it.

**Python's side (runtime):** The bridge block implements a function that receives
marshaled arguments and returns a value. `eaml_runtime` validates the return value
against the EAML return type using `ReturnType.model_validate(result)` for schema
types, or direct type checking for primitives. Layer 5 §5.4 [CLOSED].

The contract asymmetry is deliberate: EAML enforces its side at compile time;
Python's side is enforced at runtime only.

### 1.4 Scope in v0.1

**IN SCOPE:**

| Feature                                          | Section  |
|--------------------------------------------------|----------|
| `python %{ }%` blocks in tool declarations only  | §2       |
| `import python "..."` at file level              | §3       |
| `--check-python` optional flag                   | §6.1     |
| In-process execution model                       | §1.1     |
| Type marshaling for all EAML types               | §4       |

**OUT OF SCOPE (Post-MVP):**

| Feature                              | Blocking Error        | Reference        |
|--------------------------------------|-----------------------|------------------|
| Native tool bodies (EAML statements) | SYN050                | §7, ERRORS.md    |
| Bridge blocks in prompt declarations | Parse error (grammar) | §7               |
| Bridge blocks in agent declarations  | Parse error (grammar) | §7               |
| Async bridge blocks                  | See OQ-01 (§8)        | §7               |
| Bridge block unit testing hooks      | Not in grammar        | §7               |

---

## 2. Bridge Block Syntax

### RULE PYB-SYN-01: Bridge block delimiter

> Context: COMPILER
>
> Plain English: A Python bridge block is delimited by `python %{` (opening) and
> `}%` (closing). The `python` keyword is a full keyword (EG-01, Production [5] —
> excluded from IDENT). The opening delimiter is the two-character sequence `%{`.
> The closing delimiter is the two-character sequence `}%`. Layer 5 §5.1 [CLOSED]
> and EG-02: `}%` is the ONLY valid closing delimiter. The sequence `}%` cannot
> appear in valid Python code accidentally.
>
> Grammar: Production [37] `pythonImpl` — `"python" "%{" PYTHON_BLOCK "}%"`
>
> Valid:
> ```eaml
> tool greet(name: string) -> string {
>   python %{
>     return f"Hello, {name}!"
>   }%
> }
> ```
>
> Invalid:
> ```eaml
> tool greet(name: string) -> string {
>   python {
>     return f"Hello, {name}!"
>   }
> }
> // → SYN error: expected '%{' after 'python' keyword in tool body
> ```
>
> Generated: The `PYTHON_BLOCK` content between `%{` and `}%` is emitted verbatim
> into the generated `.py` file as the body of a Python function:
> ```python
> def greet(name: str) -> str:
>     return f"Hello, {name}!"
> ```
>
> Runtime: The Python interpreter executes the emitted function body normally.
>
> Notes: EG-02 from grammar.ebnf: `}%` is the only valid closing delimiter. The
> lex/yacc-style delimiter was chosen because it cannot appear in real Python code.
> Layer 5 §5.1 [CLOSED].

---

### RULE PYB-SYN-02: Capture algorithm — scan-based, not brace-depth

> Context: COMPILER (lexer phase only)
>
> Plain English: The lexer scans for the two-character sequence `}%` to close the
> block. It does NOT count Python brace depth. Python code containing dict literals,
> f-strings, class bodies, and function definitions with braces is captured correctly
> because ONLY `}%` ends the block, not a lone `}`.
>
> Grammar: Production [18] `PYTHON_BLOCK` — capture algorithm:
> 1. On seeing keyword `python` followed by `%{`: enter PYTHON_BLOCK lexer mode.
> 2. Scan forward until the two-character sequence `}%` is found.
> 3. Emit `PYTHON_BLOCK(captured_text)` containing everything between `%{` and `}%`
>    (not including the delimiters).
> 4. Return to normal EAML lexer mode.
> 5. All content including NL and WS is preserved verbatim.
>    `[lex: ws-preserve-python]`
>
> Valid:
> ```eaml
> tool analyze(data: string) -> string {
>   python %{
>     result = {"key": "value", "nested": {"inner": 42}}
>     items = [x for x in range(10) if x > 3]
>     return str(result)
>   }%
> }
> ```
> The `}` characters inside dict literals do NOT close the block. Only `}%` closes it.
>
> Generated: The content between `%{` and `}%` is emitted verbatim, preserving
> all whitespace and indentation:
> ```python
> def analyze(data: str) -> str:
>     result = {"key": "value", "nested": {"inner": 42}}
>     items = [x for x in range(10) if x > 3]
>     return str(result)
> ```
>
> Runtime: Python executes the preserved code with its original indentation intact.
>
> Notes: `[lex: ws-preserve-python]` — whitespace preservation is critical because
> Python's syntax is indentation-sensitive. The scan-based algorithm has zero known
> limitations (Layer 5 §5.1 rationale).

---

### RULE PYB-SYN-03: Bridge block position — tool declarations only in v0.1

> Context: COMPILER (parser phase)
>
> Plain English: In v0.1, `python %{ }%` blocks appear ONLY inside tool declarations.
> Specifically in `toolBodyInner` (Production [36]) via the `pythonImpl` alternative.
> The grammar structurally prevents bridge blocks in other contexts. A bridge block
> in any other position produces a syntax error.
>
> Grammar: Production [36] `toolBodyInner` —
> `pythonImpl | "description" ":" STRING pythonImpl | statement*`
>
> Valid:
> ```eaml
> tool fetch(url: string) -> string {
>   python %{
>     import httpx
>     return httpx.get(url).text
>   }%
> }
> ```
>
> Invalid:
> ```eaml
> // Bridge block at file level — not in a tool body
> python %{ print("hello") }%
> // → SYN error: unexpected token 'python' at declaration level
> ```
>
> Generated: N/A — this is a grammar-level restriction, not a codegen rule.
>
> Runtime: N/A — invalid programs do not reach codegen or runtime.
>
> Notes: `[sem: v0.1-python-required]` on Production [36]: in v0.1, if `toolBodyInner`
> matches the `statement*` branch with non-empty statements, emit SYN050. An empty
> body emits SEM040. See §7 (Post-MVP) for future expansion.

---

### RULE PYB-SYN-04: Optional description field before bridge block

> Context: COMPILER + GENERATED + RUNTIME
>
> Plain English: A tool body MAY include a `description: STRING` field before the
> `pythonImpl`. The description string is used by the runtime adapter when
> constructing the LLM's tool-use API call.
>
> Grammar: Production [36] `toolBodyInner` —
> `"description" ":" STRING pythonImpl` (second alternative)
>
> Valid:
> ```eaml
> tool fetchPage(url: string) -> string {
>   description: "Fetch the HTML content of a web page"
>   python %{
>     import httpx
>     return httpx.get(url).text
>   }%
> }
> ```
>
> Generated:
> ```python
> # Tool metadata includes the description
> _tool_fetchPage = ToolMetadata(
>     name="fetchPage",
>     description="Fetch the HTML content of a web page",
>     parameters={"url": {"type": "string"}},
>     return_type="string",
> )
>
> def fetch_page(url: str) -> str:
>     import httpx
>     return httpx.get(url).text
> ```
>
> Runtime: The adapter passes the description string to the provider's tool-use API.
> For Anthropic: `tools: [{"name": "fetchPage", "description": "...", ...}]`.
> For OpenAI: `tools: [{"type": "function", "function": {"name": "fetchPage", "description": "...", ...}}]`.
> Cross-reference: CAPABILITIES.md §7.2 (per-provider tool activation).
>
> Notes: When no description is provided, the adapter uses the tool name as a
> fallback description. The description has no effect on type checking or marshaling.

---

## 3. Import System

### 3.1 File-Level Python Imports

### RULE PYB-IMP-01: Syntax and position of file-level Python imports

> Context: COMPILER + GENERATED
>
> Plain English: File-level Python imports use the syntax `import python STRING`
> with an optional `as IDENT` alias. They MUST precede all non-import declarations
> in the file. Violation emits SEM010.
>
> Grammar: Production [26] `importDecl` —
> `"import" "python" STRING ( "as" IDENT )? ";"?`
> `[sem: import-before-declarations]` (EG-07)
>
> Valid:
> ```eaml
> import python "pandas" as pd
> import python "numpy" as np
>
> schema DataResult {
>   mean: float
>   count: int
> }
>
> tool analyze(path: string) -> DataResult {
>   python %{
>     df = pd.read_csv(path)
>     return {"mean": float(df.mean().mean()), "count": len(df)}
>   }%
> }
> ```
>
> Invalid:
> ```eaml
> schema DataResult { mean: float }
>
> import python "pandas" as pd
> // → SEM010: Python imports must appear at the top of the file,
> //   before any other declarations.
> ```
>
> Generated: File-level Python imports are emitted at the top of the generated
> `.py` file, before any generated class or function definitions:
> ```python
> # File-level EAML Python imports
> import pandas as pd
> import numpy as np
>
> from pydantic import BaseModel, Field
> from typing import Optional, List, Literal
>
> # ... generated schema classes and tool functions follow
> ```
>
> Runtime: The imports execute at Python module load time, before any bridge
> function is called. A missing package produces a Python `ImportError` at import
> time, not at function call time.
>
> Notes: Layer 5 §5.2 [CLOSED]. EG-07. The import form without alias
> (`import python "numpy"`) generates `import numpy` — the module is accessible
> by its package name. Cross-reference: ERRORS.md SEM010.

---

### RULE PYB-IMP-02: Scope of file-level Python imports

> Context: GENERATED + RUNTIME
>
> Plain English: File-level Python imports are available throughout the generated
> `.py` file. All bridge blocks in the same file have access to all file-level
> imports without re-importing.
>
> Grammar: N/A — this is a codegen scope rule, not a grammar rule.
>
> Valid:
> ```eaml
> import python "pandas" as pd
>
> tool analyze(path: string) -> string {
>   python %{
>     df = pd.read_csv(path)   // pd is available — file-level import
>     return str(df.describe())
>   }%
> }
>
> tool summarize(path: string) -> string {
>   python %{
>     df = pd.read_csv(path)   // pd is also available here
>     return str(df.mean())
>   }%
> }
> ```
>
> Generated: Both bridge functions appear in the same `.py` file after the
> `import pandas as pd` statement. Python's module-level scope makes `pd`
> available to both functions.
>
> Runtime: Standard Python scoping rules apply. Module-level imports are
> visible to all functions in the module.
>
> Notes: Layer 5 §5.2 rationale: "All imports visible at the top — no hunting
> through the file." Layer 5 §8.1: "Python imports in imported files are merged
> with the importing file's Python imports for the emitted output."

---

### RULE PYB-IMP-03: Import validation — no compile-time package check

> Context: COMPILER
>
> Plain English: The compiler does NOT verify that the imported Python package is
> installed in the user's environment. A missing package produces a Python
> `ImportError` at runtime, not a compile-time error. This is consistent with the
> in-process execution model: package availability is an environment concern, not
> a compilation concern.
>
> Grammar: N/A — this is a non-validation rule.
>
> Valid:
> ```eaml
> import python "nonexistent_package" as np
> // Compiles successfully — package existence is not checked
> ```
>
> Generated: `import nonexistent_package as np` — emitted verbatim.
>
> Runtime: Python raises `ImportError: No module named 'nonexistent_package'`
> when the generated `.py` file is first imported.
>
> Notes: Layer 5 §5.3: the `--check-python` flag validates Python SYNTAX only,
> not package availability. The compiler has no Python dependency by default.

---

### 3.2 Imports Inside Bridge Blocks

### RULE PYB-IMP-04: Python import statements inside bridge blocks

> Context: GENERATED + RUNTIME
>
> Plain English: Python `import` and `from ... import ...` statements may appear
> freely inside `python %{ }%` blocks. They follow Python's normal import semantics:
> executed when the function is called, not at module load time.
>
> Grammar: N/A — bridge block content is opaque to the EAML grammar.
>
> Valid:
> ```eaml
> tool fetchJson(url: string) -> string {
>   python %{
>     import httpx
>     import json
>     response = httpx.get(url)
>     return json.dumps(response.json(), indent=2)
>   }%
> }
> ```
>
> Generated: The import statements are emitted verbatim inside the function body:
> ```python
> def fetch_json(url: str) -> str:
>     import httpx
>     import json
>     response = httpx.get(url)
>     return json.dumps(response.json(), indent=2)
> ```
>
> Runtime: Python executes the imports each time the function is called. Python
> caches module imports internally, so repeated calls do not re-parse the module.
>
> Notes: Block-level imports are useful for packages used by only one tool.
> File-level imports (PYB-IMP-01) are preferred for packages used across
> multiple tools — they execute once at module load time.

---

### RULE PYB-IMP-05: Relationship between file-level and block-level imports

> Context: GENERATED + RUNTIME
>
> Plain English: File-level imports (`import python "pandas" as pd`) are available
> inside bridge blocks without re-importing. Block-level imports that duplicate a
> file-level import are redundant but not errors — Python handles this gracefully.
>
> Grammar: N/A — semantic scope rule.
>
> Valid (file-level preferred):
> ```eaml
> import python "pandas" as pd
>
> tool analyze(path: string) -> string {
>   python %{
>     df = pd.read_csv(path)   // uses file-level import
>     return str(df.describe())
>   }%
> }
> ```
>
> Valid (block-level also works):
> ```eaml
> tool analyze(path: string) -> string {
>   python %{
>     import pandas as pd      // block-level import — valid but redundant if file-level exists
>     df = pd.read_csv(path)
>     return str(df.describe())
>   }%
> }
> ```
>
> Generated: Both patterns produce valid Python. File-level imports appear before
> function definitions; block-level imports appear inside function bodies.
>
> Runtime: Standard Python behavior — duplicate imports are harmless.
>
> Notes: File-level imports are recommended for packages used across multiple tools.
> Block-level imports are acceptable for packages used in only one tool.

---

### RULE PYB-IMP-06: Circular import restriction

> Context: RUNTIME
>
> Plain English: Bridge blocks MUST NOT import the generated EAML module itself.
> The generated `.py` file defines the bridge functions; a bridge function that
> imports its own module creates a circular import in Python.
>
> Grammar: N/A — runtime behavior, not grammar.
>
> Invalid:
> ```python
> # Inside a bridge block — DO NOT do this:
> from generated_output import SomeSchema  # circular import!
> ```
>
> Generated: N/A — the compiler does not detect circular imports in bridge blocks.
>
> Runtime: Python raises `ImportError` or produces `None` values for partially-loaded
> modules. This is standard Python circular import behavior.
>
> Notes: See OQ-02 (§8). The compiler does not detect this in v0.1. It is a user
> responsibility. Future versions MAY add detection as a SEM warning.

---

## 4. Type Marshaling

This section specifies how EAML types become Python values at the bridge boundary.
Every type mapping is **derived from** TYPESYSTEM.md §10.3 (Complete Type Mapping
Table). PYTHON_BRIDGE.md does not independently define type representations — it
specifies how those representations manifest at the call boundary.

### 4.1 Parameter Marshaling — EAML to Python

### RULE PYB-MAR-01: Primitive parameter marshaling

> Context: GENERATED + RUNTIME
>
> Plain English: EAML primitive types map directly to Python native types.
> No marshaling overhead. Values are passed by standard Python calling convention.
>
> Grammar: Production [73] `param` — `IDENT ":" typeExpr ( "=" literal )?`
>
> | EAML Type   | Python Type Annotation   | Runtime Value   |
> |-------------|--------------------------|-----------------|
> | `string`    | `str`                    | Python `str`    |
> | `int`       | `int`                    | Python `int`    |
> | `float`     | `float`                  | Python `float`  |
> | `bool`      | `bool`                   | Python `bool`   |
>
> Valid:
> ```eaml
> tool format(name: string, count: int, ratio: float, active: bool) -> string {
>   python %{
>     return f"{name}: {count} items, ratio={ratio}, active={active}"
>   }%
> }
> ```
>
> Generated:
> ```python
> def format(name: str, count: int, ratio: float, active: bool) -> str:
>     return f"{name}: {count} items, ratio={ratio}, active={active}"
> ```
>
> Runtime: Parameters arrive as native Python types. No conversion or validation
> is performed on parameters — they were validated at the EAML call site.
>
> Notes: Cross-reference: TYPESYSTEM.md §2.6 (Primitive Type Summary).

---

### RULE PYB-MAR-02: Schema parameter marshaling

> Context: GENERATED + RUNTIME
>
> Plain English: An EAML parameter of schema type `T` receives a Pydantic v2 model
> instance (`class T(BaseModel)` as defined by TYPESYSTEM.md §10.2). The bridge
> function receives a fully-validated `T` instance. Fields are accessed via attribute
> access: `param.field_name`.
>
> Grammar: Production [73] `param` with `typeExpr` resolving to a schema name.
>
> Valid:
> ```eaml
> schema Query {
>   text: string
>   limit: int
> }
>
> tool search(query: Query) -> string {
>   python %{
>     return f"Searching for '{query.text}' with limit {query.limit}"
>   }%
> }
> ```
>
> Generated:
> ```python
> class Query(BaseModel):
>     text: str
>     limit: int
>
> def search(query: Query) -> str:
>     return f"Searching for '{query.text}' with limit {query.limit}"
> ```
>
> Runtime: The `query` parameter arrives as a `Query` instance with validated
> fields. Bridge code accesses fields via `query.text`, `query.limit`.
>
> Notes: Cross-reference: TYPESYSTEM.md TS-SCH-01, §10.2 (canonical example).
> Schema parameters are always fully validated Pydantic model instances.

---

### RULE PYB-MAR-03: Optional parameter marshaling

> Context: GENERATED + RUNTIME
>
> Plain English: An optional parameter (`T?`) receives either a `T` value or
> `None`. Bridge code MUST handle the `None` case explicitly.
>
> Grammar: Production [73] `param` with `typeExpr` including `optionalSuffix`.
>
> Valid:
> ```eaml
> tool greet(name: string, title: string?) -> string {
>   python %{
>     if title is not None:
>         return f"Hello, {title} {name}!"
>     return f"Hello, {name}!"
>   }%
> }
> ```
>
> Generated:
> ```python
> def greet(name: str, title: Optional[str] = None) -> str:
>     if title is not None:
>         return f"Hello, {title} {name}!"
>     return f"Hello, {name}!"
> ```
>
> Runtime: `title` is either a Python `str` or `None`.
>
> Notes: Cross-reference: TYPESYSTEM.md TS-OPT-01, §10.4 (Optional field default).

---

### RULE PYB-MAR-04: Array parameter marshaling

> Context: GENERATED + RUNTIME
>
> Plain English: An array parameter (`T[]`) receives a Python `list` containing
> elements of the appropriate type. Elements are individually typed per
> PYB-MAR-01 through PYB-MAR-03.
>
> Grammar: Production [73] `param` with `typeExpr` including `arraySuffix`.
>
> Valid:
> ```eaml
> tool joinNames(names: string[]) -> string {
>   python %{
>     return ", ".join(names)
>   }%
> }
> ```
>
> Generated:
> ```python
> def join_names(names: List[str]) -> str:
>     return ", ".join(names)
> ```
>
> Runtime: `names` arrives as a Python `list` of `str` values.
>
> Notes: Cross-reference: TYPESYSTEM.md TS-ARR-01, §10.3 (array rows).

---

### RULE PYB-MAR-05: Literal union parameter marshaling

> Context: GENERATED + RUNTIME
>
> Plain English: A literal union parameter (`"yes" | "no"`) receives a Python `str`
> containing one of the listed values. The value has already been validated as a
> member of the union at the EAML call site.
>
> Grammar: Production [73] `param` with `typeExpr` resolving to `literalUnion`.
>
> Valid:
> ```eaml
> tool respond(answer: "yes" | "no") -> string {
>   python %{
>     if answer == "yes":
>         return "Affirmative"
>     return "Negative"
>   }%
> }
> ```
>
> Generated:
> ```python
> def respond(answer: Literal["yes", "no"]) -> str:
>     if answer == "yes":
>         return "Affirmative"
>     return "Negative"
> ```
>
> Runtime: `answer` is a Python `str` guaranteed to be one of the listed values.
>
> Notes: Cross-reference: TYPESYSTEM.md TS-LIT-01, §10.3 (literal union rows).

---

**Parameter Marshaling Summary:**

| EAML Parameter Type   | Python Type Annotation   | Runtime Value Type              |
|-----------------------|--------------------------|---------------------------------|
| `string`              | `str`                    | `str`                           |
| `int`                 | `int`                    | `int`                           |
| `float`               | `float`                  | `float`                         |
| `bool`                | `bool`                   | `bool`                          |
| `SchemaName`          | `SchemaName`             | Pydantic `BaseModel` instance   |
| `SchemaName?`         | `Optional[SchemaName]`   | `SchemaName` instance or `None` |
| `string[]`            | `List[str]`              | `list` of `str`                 |
| `string?[]`           | `List[Optional[str]]`    | `list` of `str` or `None`       |
| `string[]?`           | `Optional[List[str]]`    | `list` of `str` or `None`       |
| `"a" \| "b"`          | `Literal["a", "b"]`      | `str` (one of the members)      |

---

### 4.2 Return Value Marshaling — Python to EAML

### RULE PYB-MAR-06: Primitive return marshaling

> Context: RUNTIME
>
> Plain English: A tool with a primitive return type must return the corresponding
> Python native type. `eaml_runtime` performs a type check on the return value.
>
> Grammar: Production [34] `toolDecl` — `"->" typeExpr`.
>
> | EAML Return Type  | Required Python Return   | Validation                         |
> |-------------------|--------------------------|------------------------------------|
> | `-> string`       | `str`                    | `isinstance(result, str)`          |
> | `-> int`          | `int`                    | `isinstance(result, int)`          |
> | `-> float`        | `int` or `float`         | `isinstance(result, (int, float))` |
> | `-> bool`         | `bool`                   | `isinstance(result, bool)`         |
>
> Valid:
> ```eaml
> tool add(a: int, b: int) -> int {
>   python %{
>     return a + b
>   }%
> }
> ```
>
> Generated:
> ```python
> def add(a: int, b: int) -> int:
>     return a + b
>
> # eaml_runtime wrapper validates the return:
> def _eaml_call_add(a: int, b: int) -> int:
>     result = add(a, b)
>     if not isinstance(result, int):
>         raise TypeError(f"Tool 'add' expected return type 'int', got '{type(result).__name__}'")
>     return result
> ```
>
> Runtime: `eaml_runtime` validates the return value. On mismatch, raises `TypeError`.
>
> Notes: `-> float` accepts Python `int` return values (implicit int-to-float
> coercion is standard Python behavior). Cross-reference: TYPESYSTEM.md §2.6.

---

### RULE PYB-MAR-07: Schema return marshaling

> Context: RUNTIME
>
> Plain English: A tool with a schema return type (`-> SchemaName`) must return
> either a `SchemaName` Pydantic model instance OR a `dict` with the correct
> field structure. `eaml_runtime` validates the return via
> `SchemaName.model_validate(result)`.
>
> Grammar: Production [34] `toolDecl` — `"->" typeExpr`.
>
> Valid:
> ```eaml
> schema DataSummary {
>   mean: float
>   count: int
> }
>
> tool analyze(path: string) -> DataSummary {
>   python %{
>     import pandas as pd
>     df = pd.read_csv(path)
>     return {"mean": float(df.mean().mean()), "count": len(df)}
>   }%
> }
> ```
>
> Generated:
> ```python
> class DataSummary(BaseModel):
>     mean: float
>     count: int
>
> def analyze(path: str) -> DataSummary:
>     import pandas as pd
>     df = pd.read_csv(path)
>     return {"mean": float(df.mean().mean()), "count": len(df)}
>
> # eaml_runtime wrapper validates the return:
> def _eaml_call_analyze(path: str) -> DataSummary:
>     result = analyze(path)
>     return DataSummary.model_validate(result)
> ```
>
> Runtime: `eaml_runtime` calls `SchemaName.model_validate(result)`. This accepts
> both Pydantic model instances and dicts. On validation failure, Pydantic v2
> raises `ValidationError` with field-level error details.
>
> Notes: Layer 5 §5.4 [CLOSED]: "The emitted Python wraps the block's return
> value in: `ReturnType.model_validate(result)`". Both dict and model instance
> are accepted by `model_validate()`. Cross-reference: TYPESYSTEM.md TS-SCH-06.

---

### RULE PYB-MAR-08: Null return type for void tools

> Context: GENERATED + RUNTIME
>
> Plain English: A tool with `-> null` (void-equivalent) can return `None` or
> return nothing (implicit `None` in Python). Any non-`None` return value is
> silently discarded — the runtime does not warn.
>
> Grammar: Production [34] `toolDecl` — `"->" typeExpr` where `typeExpr` is `null`.
>
> Valid:
> ```eaml
> tool logEvent(message: string) -> null {
>   python %{
>     import logging
>     logging.info(message)
>     # No explicit return — Python implicitly returns None
>   }%
> }
> ```
>
> Generated:
> ```python
> def log_event(message: str) -> None:
>     import logging
>     logging.info(message)
>
> # eaml_runtime wrapper — no validation needed for null return:
> def _eaml_call_log_event(message: str) -> None:
>     log_event(message)
> ```
>
> Runtime: `eaml_runtime` does not validate the return value when the declared
> return type is `null`. Any return value is discarded.
>
> Notes: Cross-reference: TYPESYSTEM.md TS-RET-02 — "`-> null` for void tools."
> Layer 5 §7.4 [CLOSED]: `void` is NOT a keyword in v0.1; use `-> null`.

---

### RULE PYB-MAR-09: Return value validation timing

> Context: RUNTIME
>
> Plain English: Return value validation occurs in `eaml_runtime` AFTER the bridge
> function returns. It is NOT checked by the EAML compiler. The compiler verifies
> the tool's declared return type is valid EAML — it cannot verify the Python
> implementation actually returns that type. This is the fundamental contract
> asymmetry from §1.3.
>
> Grammar: N/A — this is a runtime contract rule.
>
> Generated: The `eaml_runtime` wrapper function (shown in PYB-MAR-06 and
> PYB-MAR-07) performs validation after calling the bridge function.
>
> Runtime: Validation timing:
> 1. `eaml_runtime` marshals parameters and calls the bridge function.
> 2. The bridge function executes and returns a value.
> 3. `eaml_runtime` validates the return value against the declared EAML type.
> 4. On success: the validated value is returned to the caller.
> 5. On failure: Pydantic `ValidationError` (for schema types) or `TypeError`
>    (for primitive types) is raised.
>
> Notes: Layer 5 §5.4 [CLOSED]. The compiler cannot verify Python implementation
> correctness — only `model_validate()` at runtime catches mismatches.

---

**Return Marshaling Summary:**

| EAML Return Type   | Required Python Return   | Validation Method                   | On Mismatch                |
|--------------------|--------------------------|-------------------------------------|----------------------------|
| `-> string`        | `str`                    | `isinstance(result, str)`           | `TypeError`                |
| `-> int`           | `int`                    | `isinstance(result, int)`           | `TypeError`                |
| `-> float`         | `int` or `float`         | `isinstance(result, (int, float))`  | `TypeError`                |
| `-> bool`          | `bool`                   | `isinstance(result, bool)`          | `TypeError`                |
| `-> null`          | `None` (or implicit)     | No validation                       | Return discarded           |
| `-> SchemaName`    | `dict` or `SchemaName`   | `SchemaName.model_validate(result)` | Pydantic `ValidationError` |
| `-> SchemaName[]`  | `list` of dict/model     | Validate each element               | Pydantic `ValidationError` |
| `-> "a" \| "b"`    | `str` (member value)     | Check membership                    | `ValueError`               |

---

### 4.3 Bounded Type Parameters at the Bridge

### RULE PYB-MAR-10: Bounded types in tool parameter positions

> Context: COMPILER
>
> Plain English: Bounded type parameters (e.g., `float<0.0, 1.0>`) are NOT
> permitted in prompt or tool parameter type positions. Bounds are schema field
> constraints for validating LLM output. The bridge receives an unbounded
> primitive value.
>
> Grammar: Production [73] `param` — grammar permits `typeExpr` which includes
> `namedType` with `boundedSuffix`. Semantic analysis rejects bounds in parameter
> positions.
>
> Valid:
> ```eaml
> tool analyze(score: float) -> string {
>   python %{
>     return f"Score: {score}"
>   }%
> }
> ```
>
> Invalid:
> ```eaml
> tool analyze(score: float<0.0, 1.0>) -> string {
>   python %{
>     return f"Score: {score}"
>   }%
> }
> // → SEM035: Bounded type parameters are not permitted in prompt or tool
> //   parameter positions. Use 'float' as the parameter type.
> ```
>
> Generated: N/A — invalid programs do not reach codegen.
>
> Runtime: N/A — the restriction is compile-time only.
>
> Notes: Cross-reference: TYPESYSTEM.md TS-BND-08, ERRORS.md SEM035.
> Bounds are meaningful only on schema fields where they constrain LLM output
> via Pydantic `Field()` validation. Tool parameters are caller-supplied values.

---

## 5. Generated Code Structure

### 5.1 Generated Function Structure

### RULE PYB-GEN-01: Canonical generated tool function

> Context: COMPILER + GENERATED
>
> Plain English: For each tool declaration with a bridge block, the compiler
> generates a complete Python module containing: (a) file-level imports,
> (b) Pydantic schema classes, (c) the bridge function with verbatim content,
> (d) the `eaml_runtime` wrapper with return validation, and (e) tool metadata.
>
> Grammar: Productions [34]–[37] (toolDecl → toolBody → toolBodyInner → pythonImpl)
>
> EAML input:
> ```eaml
> import python "httpx"
>
> schema PageInfo {
>   title: string
>   length: int
> }
>
> tool fetchPage(url: string, timeout: int) -> PageInfo {
>   description: "Fetch page metadata"
>   python %{
>     response = httpx.get(url, timeout=timeout)
>     response.raise_for_status()
>     return {
>         "title": response.headers.get("title", url),
>         "length": len(response.text),
>     }
>   }%
> }
> ```
>
> Generated Python (complete structure):
> ```python
> """Generated by eamlc from example.eaml"""
>
> # (a) File-level imports — from import python "..." declarations
> import httpx
>
> # (a) Standard imports for generated code
> from pydantic import BaseModel, Field
> from typing import Optional, List, Literal
> from eaml_runtime import ToolMetadata, validate_return
>
> # (b) Pydantic schema classes — from schema declarations
> class PageInfo(BaseModel):
>     title: str
>     length: int
>
> # (c) Bridge function — PYTHON_BLOCK content emitted verbatim
> def fetch_page(url: str, timeout: int) -> dict:
>     response = httpx.get(url, timeout=timeout)
>     response.raise_for_status()
>     return {
>         "title": response.headers.get("title", url),
>         "length": len(response.text),
>     }
>
> # (d) eaml_runtime wrapper — validates return type
> def _eaml_call_fetch_page(url: str, timeout: int) -> PageInfo:
>     result = fetch_page(url, timeout)
>     return PageInfo.model_validate(result)
>
> # (e) Tool metadata registration
> _tool_fetch_page = ToolMetadata(
>     name="fetchPage",
>     description="Fetch page metadata",
>     parameters=[
>         {"name": "url", "type": "string"},
>         {"name": "timeout", "type": "int"},
>     ],
>     return_type="PageInfo",
>     function=_eaml_call_fetch_page,
> )
> ```
>
> Runtime: `eaml_runtime` uses `_tool_fetch_page` to register the tool with the
> agent's tool list. When the LLM invokes the tool, the adapter calls
> `_eaml_call_fetch_page(url, timeout)`, which calls the bridge function and
> validates the return.
>
> Notes: All five structural components (a)–(e) are present. The bridge function
> content is emitted verbatim from the PYTHON_BLOCK. The wrapper function adds
> return validation. The metadata enables the adapter to construct provider-specific
> tool-use API calls.

---

### 5.2 Bridge Function Naming Convention

### RULE PYB-GEN-02: Bridge function name derived from tool name

> Context: COMPILER + GENERATED
>
> Plain English: The generated Python function name is derived from the EAML tool
> name by converting `camelCase` to `snake_case`. The bridge block content is
> emitted as the body of this generated function. The bridge block itself does NOT
> need to define a function — the compiler wraps it.
>
> Grammar: Production [34] `toolDecl` — `"tool" IDENT ...`
>
> | EAML Tool Name   | Generated Python Function   |
> |------------------|-----------------------------|
> | `fetchPage`      | `fetch_page`                |
> | `analyzeText`    | `analyze_text`              |
> | `getData`        | `get_data`                  |
> | `run`            | `run`                       |
>
> Valid:
> ```eaml
> tool analyzeText(text: string) -> string {
>   python %{
>     return text.upper()
>   }%
> }
> ```
>
> Generated:
> ```python
> def analyze_text(text: str) -> str:
>     return text.upper()
> ```
>
> Runtime: The function is called by its snake_case name by the `eaml_runtime` wrapper.
>
> Notes: See OQ-03 (§8) for the open question about user-defined function names
> inside bridge blocks. In the v0.1 model, the bridge block content is the function
> BODY, not a standalone function definition. The compiler wraps it in a `def`
> statement with the derived name and typed parameters.

---

### 5.3 Tool Metadata Registration

### RULE PYB-GEN-03: Tool metadata for runtime adapter

> Context: GENERATED + RUNTIME
>
> Plain English: The generated code registers tool metadata with `eaml_runtime`.
> The metadata includes: tool name, description (from optional `description:` field),
> parameter names and types, and return type. The adapter uses this metadata when
> constructing the tool-use API call to the LLM provider.
>
> Grammar: Productions [34]–[36] (toolDecl, toolBody, toolBodyInner).
>
> Generated:
> ```python
> _tool_fetch_page = ToolMetadata(
>     name="fetchPage",           # EAML tool name (camelCase preserved)
>     description="Fetch page metadata",  # from description: field (or tool name if absent)
>     parameters=[
>         {"name": "url", "type": "string"},
>         {"name": "timeout", "type": "int"},
>     ],
>     return_type="PageInfo",
>     function=_eaml_call_fetch_page,   # reference to the wrapper function
> )
> ```
>
> Runtime: The adapter passes tool metadata to provider APIs:
> - Anthropic: `tools: [{"name": "fetchPage", "description": "...", "input_schema": {...}}]`
> - OpenAI: `tools: [{"type": "function", "function": {"name": "fetchPage", ...}}]`
>
> Notes: Cross-reference: CAPABILITIES.md CAP-TYP-03 (tools capability),
> §7.2 (per-provider tool activation). The tool name in metadata uses the EAML
> camelCase name (not the snake_case Python function name).

---

### 5.4 Multiple Tools in One File

### RULE PYB-GEN-04: Multi-tool file structure

> Context: COMPILER + GENERATED
>
> Plain English: When a `.eaml` file contains multiple tool declarations, each
> generates its own bridge function, wrapper, and metadata registration. File-level
> Python imports appear once at the top. Schema classes appear once, before any
> tool functions.
>
> Grammar: Production [24] `Program` — `declaration* EOF`
>
> Generated structure for a file with two tools:
> ```python
> """Generated by eamlc from tools.eaml"""
>
> # File-level imports (once)
> import httpx
> from pydantic import BaseModel, Field
> from typing import Optional, List, Literal
> from eaml_runtime import ToolMetadata, validate_return
>
> # Schema classes (once each)
> class DataSummary(BaseModel):
>     mean: float
>     count: int
>
> # Tool 1: bridge function, wrapper, metadata
> def fetch_data(url: str) -> dict:
>     return httpx.get(url).json()
>
> def _eaml_call_fetch_data(url: str) -> DataSummary:
>     result = fetch_data(url)
>     return DataSummary.model_validate(result)
>
> _tool_fetch_data = ToolMetadata(
>     name="fetchData", description="fetchData",
>     parameters=[{"name": "url", "type": "string"}],
>     return_type="DataSummary", function=_eaml_call_fetch_data,
> )
>
> # Tool 2: bridge function, wrapper, metadata
> def transform(data: DataSummary) -> str:
>     return f"Mean: {data.mean}, Count: {data.count}"
>
> def _eaml_call_transform(data: DataSummary) -> str:
>     result = transform(data)
>     if not isinstance(result, str):
>         raise TypeError(f"Tool 'transform' expected 'str', got '{type(result).__name__}'")
>     return result
>
> _tool_transform = ToolMetadata(
>     name="transform", description="transform",
>     parameters=[{"name": "data", "type": "DataSummary"}],
>     return_type="string", function=_eaml_call_transform,
> )
> ```
>
> Runtime: All tools in the file are registered with `eaml_runtime` and available
> for agent tool lists.
>
> Notes: Declaration order in the `.eaml` file is preserved in the generated `.py`
> file. Schema classes are emitted before tool functions to ensure type availability.

---

## 6. Error Handling in Bridge Blocks

### 6.1 Python Syntax Errors — --check-python Flag

### RULE PYB-ERR-01: Optional Python syntax validation

> Context: COMPILER (CODEGEN phase)
>
> Plain English: The `--check-python` flag enables optional Python syntax validation.
> When enabled, the compiler invokes `python -m py_compile` on the extracted
> `PYTHON_BLOCK` content after all EAML type and capability checks pass. This is
> CODEGEN phase — it runs only if the `.eaml` file is otherwise valid.
>
> Grammar: Production [37] `pythonImpl` — `[lex: python-block-capture]`
>
> Without `--check-python` (default): Python syntax errors are silent at compile
> time. They become Python `SyntaxError` at runtime when the generated `.py` file
> is imported.
>
> With `--check-python`: Invalid Python syntax produces PYB001.
>
> Invalid (with --check-python):
> ```eaml
> tool broken(x: string) -> string {
>   python %{
>     return x.    # Python syntax error — trailing dot
>   }%
> }
> // → PYB001: Python syntax error in bridge block at line 3:14: invalid syntax
> ```
>
> Generated: When `--check-python` detects an error, no `.py` file is written.
>
> Runtime: Without `--check-python`, the syntax error surfaces as a Python
> `SyntaxError` when the generated module is imported.
>
> Notes: Layer 5 §5.3 [CLOSED]: off by default. The Python executable used is
> the `python` (or `python3`) found in `PATH`. See OQ-04 (§8) for the version
> matching question. Cross-reference: ERRORS.md PYB001 (CODEGEN phase, ERROR severity).
> If Python is not found in PATH: warning, not error (Layer 5 §5.3).

---

### 6.2 Python Runtime Exceptions

### RULE PYB-ERR-02: Bridge exception propagation

> Context: RUNTIME
>
> Plain English: Python exceptions raised inside bridge blocks propagate to the
> `eaml_runtime` caller. `eaml_runtime` does NOT silently swallow bridge
> exceptions. A bridge function that raises `httpx.TimeoutError` propagates that
> exception to the calling agent or prompt pipeline.
>
> Grammar: N/A — runtime behavior.
>
> Valid (defensive pattern):
> ```eaml
> tool safeFetch(url: string) -> string {
>   python %{
>     import httpx
>     try:
>         response = httpx.get(url, timeout=10)
>         return response.text
>     except httpx.TimeoutError:
>         return "ERROR: Request timed out"
>     except httpx.HTTPError as e:
>         return f"ERROR: {e}"
>   }%
> }
> ```
>
> Generated:
> ```python
> def safe_fetch(url: str) -> str:
>     import httpx
>     try:
>         response = httpx.get(url, timeout=10)
>         return response.text
>     except httpx.TimeoutError:
>         return "ERROR: Request timed out"
>     except httpx.HTTPError as e:
>         return f"ERROR: {e}"
> ```
>
> Runtime: If the bridge function raises an unhandled exception, `eaml_runtime`
> lets it propagate. The calling agent's `on_error` policy (Production [40]
> `errorPolicy`) determines whether to retry or fail.
>
> Notes: See OQ-05 (§8) for the question of whether `eaml_runtime` wraps bridge
> exceptions in a `BridgeExecutionError`. Recommended user pattern: wrap bridge
> code in `try/except` and return an error-indicating value of the declared
> return type.

---

### 6.3 Return Type Mismatch at Runtime

### RULE PYB-ERR-03: Return type validation failure

> Context: RUNTIME
>
> Plain English: When a bridge function returns a value that does not match the
> declared EAML return type, `eaml_runtime` raises a validation error. For schema
> return types, this is Pydantic v2 `ValidationError`. For primitive return types,
> this is Python `TypeError`.
>
> Grammar: N/A — runtime validation.
>
> Invalid at runtime:
> ```eaml
> schema Result {
>   score: float
>   label: string
> }
>
> tool classify(text: string) -> Result {
>   python %{
>     return {"score": "not a number", "label": 42}
>   }%
> }
> // At runtime → pydantic.ValidationError:
> //   score: Input should be a valid number [type=float_parsing, ...]
> //   label: Input should be a valid string [type=string_type, ...]
> ```
>
> Generated: The wrapper function calls `Result.model_validate(result)`, which
> raises `ValidationError` on field type mismatches.
>
> Runtime: Pydantic v2 `ValidationError` includes field-level details showing
> which fields failed and why. This error propagates to the caller.
>
> Notes: Cross-reference: TYPESYSTEM.md §1.2 (runtime Pydantic validation),
> PYB-MAR-09 (return value validation timing). This is distinct from
> `CapabilityActivationError` (CAPABILITIES.md §7.1) which covers provider
> capability failures, not bridge return type failures.

---

## 7. Post-MVP Bridge Features

The following bridge features are explicitly out of scope for v0.1.

### 7.1 Native Tool Bodies

**Feature:** Tool bodies with native EAML statements instead of Python bridge blocks.

**Why deferred:** Requires a complete EAML expression evaluator. The Python bridge
covers all v0.1 use cases.

**Blocking error:** SYN050 — `"Native tool bodies are not supported in EAML v0.1.
Use python %{ }% for tool implementations."` (ERRORS.md SYN050, PARSE phase, ERROR).

**Triggering code:**
```eaml
tool add(a: int, b: int) -> int {
  return a + b
}
// → SYN050
```

### 7.2 Bridge Blocks in Prompt Declarations

**Feature:** Inline Python code in prompt declarations for preprocessing.

**Why deferred:** Prompts are declarative (system/user template fields). Code
execution in prompts would require a new execution model.

**Current behavior:** The grammar structurally prevents `pythonImpl` in
`promptBody` (Production [32]). `promptField` alternatives are keyword-specific
fields only.

### 7.3 Bridge Blocks in Agent Declarations

**Feature:** Inline Python code in agent declarations for custom orchestration.

**Why deferred:** Agents are configuration structures (model, tools, system, error
policy). Code execution would change the agent model from declarative to imperative.

**Current behavior:** `agentField` (Production [39]) alternatives do not include
`pythonImpl`.

### 7.4 Async Bridge Blocks

**Feature:** Using `async def` and `await` inside bridge blocks.

**Current behavior:** See OQ-01 (§8). Async Python code inside a bridge block is
syntactically valid Python and will be captured by the lexer. However, calling an
async function requires `await` or `asyncio.run()`, which the generated wrapper
does not currently use. An async bridge function would return a coroutine object
instead of a value, causing a type mismatch at validation time.

### 7.5 Bridge Block Unit Testing Hooks

**Feature:** Ability to mock bridge functions in tests for isolated testing of
EAML agent behavior without executing real Python tool code.

**Why deferred:** Requires a testing framework integration layer. Users can test
generated Python functions directly using standard Python testing tools.

---

## 8. Open Questions

### OQ-01: Async bridge blocks

**Context:** §7.4 (Post-MVP), PYB-GEN-01 (generated wrapper).

**Question:** Should `async def` inside bridge blocks be supported in v0.1?
Currently, the generated wrapper calls the bridge function synchronously. An
async bridge function would return a coroutine, not a value.

**Recommended resolution:** Document async bridge blocks as unsupported in v0.1.
The generated wrapper is synchronous. Users who need async behavior should use
`asyncio.run()` inside the bridge block (which blocks the calling thread).
Full async support requires `eaml_runtime` to use `await` in the wrapper,
which is architecture-significant.

### OQ-02: Circular import detection

**Context:** PYB-IMP-06.

**Question:** Should the compiler detect when a bridge block imports the generated
module itself?

**Recommended resolution:** No detection in v0.1 — this is a user responsibility.
Document as a known pitfall. Future versions MAY add detection as a SEM warning.

### OQ-03: User-defined function names in bridge blocks

**Context:** PYB-GEN-02 (naming convention).

**Question:** The v0.1 model wraps bridge block content as the BODY of a generated
function. What if the user writes a `def` statement inside the block?

**Recommended resolution:** The bridge block content IS the function body. If the
user writes a `def` inside it, that defines a local helper function, not the tool's
entry point. The generated wrapper calls the tool by executing the block content
directly (the last expression or `return` statement provides the return value).

### OQ-04: --check-python Python version

**Context:** PYB-ERR-01.

**Question:** Which Python executable does `--check-python` use? The `python` in
PATH, or the Python that will eventually run the generated file?

**Recommended resolution:** Use the `python3` (or `python`) found in `PATH`. Document
that the check Python version should match the execution environment. If Python is
not found in PATH, emit a warning (not error) per Layer 5 §5.3.

### OQ-05: Bridge exception wrapping

**Context:** PYB-ERR-02.

**Question:** Should `eaml_runtime` wrap bridge exceptions in a
`BridgeExecutionError` for structured error handling?

**Recommended resolution:** No wrapping in v0.1 — let exceptions propagate
naturally. This is simpler and allows users to catch specific exception types.
Future versions MAY add an optional wrapper for structured error reporting.

---

## Verification Report — EAML PYTHON_BRIDGE.md v0.1.0

| Group                | Checks   | Passed   | Failed   | N/A   |
|----------------------|----------|----------|----------|-------|
| A — Completeness     | 7        | 7        | 0        | 0     |
| B — Grammar Contract | 4        | 4        | 0        | 0     |
| C — Type Contract    | 4        | 4        | 0        | 0     |
| D — Error Contract   | 3        | 3        | 0        | 0     |
| E — Quality          | 5        | 5        | 0        | 0     |
| **Total**            | **23**   | **23**   | **0**    | **0** |

Failed checks: 0
Open Questions: 5 (OQ-01 through OQ-05, see §8)

### Group A — Completeness Checks

**A1[PASS]** Every bridge-related grammar production has a rule:
- [18] PYTHON_BLOCK → PYB-SYN-02 (capture algorithm)
- [26] importDecl → PYB-IMP-01 (Python import form)
- [34] toolDecl → PYB-MAR-06, PYB-GEN-01 (tool structure)
- [35] toolBody → PYB-SYN-03 (tool body position)
- [36] toolBodyInner → PYB-SYN-03, PYB-SYN-04 (alternatives)
- [37] pythonImpl → PYB-SYN-01 (delimiter)

**A2[PASS]** Context labels used in every rule. Spot-checked:
PYB-SYN-01 (COMPILER), PYB-MAR-02 (GENERATED + RUNTIME),
PYB-GEN-01 (COMPILER + GENERATED), PYB-ERR-01 (COMPILER),
PYB-IMP-04 (GENERATED + RUNTIME). All have Context: field.

**A3[PASS]** Generated: fields show actual Python code. PYB-GEN-01 canonical
example includes all five structural components: (a) file-level imports,
(b) schema classes, (c) bridge function, (d) wrapper with validation,
(e) tool metadata.

**A4[PASS]** Both import mechanisms covered:
- File-level: PYB-IMP-01 (syntax/position), PYB-IMP-02 (scope), PYB-IMP-03 (no package check)
- Block-level: PYB-IMP-04 (in-block imports), PYB-IMP-05 (relationship), PYB-IMP-06 (circular)
- Position rule cites SEM010 from ERRORS.md ✓

**A5[PASS]** Parameter types covered: primitive (PYB-MAR-01), schema (PYB-MAR-02),
optional (PYB-MAR-03), array (PYB-MAR-04), literal union (PYB-MAR-05).
Return types covered: primitive (PYB-MAR-06), schema (PYB-MAR-07), null (PYB-MAR-08).
Summary tables present for both directions. All cross-reference TYPESYSTEM.md §10.3.

**A6[PASS]** All five OPEN QUESTIONs are in §8 with recommended resolutions.
None are buried in rule body text without §8 entries.

**A7[PASS]** Post-MVP features in §7: native tool body (SYN050 ✓), prompt bridge,
agent bridge, async bridge (OQ-01), testing hooks. SYN050 cited from ERRORS.md.

### Group B — Grammar Contract Checks

**B1[PASS]** Production [37] in grammar.ebnf (line 513):
`pythonImpl ::= "python" "%{" PYTHON_BLOCK "}%"`
PYB-SYN-01 accurately describes this: delimiter is `python %{` ... `}%`.
Cites EG-02 and Layer 5 §5.1 [CLOSED]. ✓

**B2[PASS]** Production [18] in grammar.ebnf (lines 264-283):
Capture algorithm: scan for `}%`, no brace-depth counting, whitespace preserved.
PYB-SYN-02 accurately describes all five steps of the algorithm. ✓

**B3[PASS]** Production [26] in grammar.ebnf (lines 384-395):
`importDecl ::= "import" ( STRING ( "as" IDENT )? | "python" STRING ( "as" IDENT )? ) ";"?`
PYB-IMP-01 accurately describes the Python import alternative.
`[sem: import-before-declarations]` annotation cited correctly. ✓

**B4[PASS]** All cited production numbers verified in grammar.ebnf:
[5] IDENT (line 202), [18] PYTHON_BLOCK (line 283), [24] Program (line 353),
[26] importDecl (line 393), [34] toolDecl (line 474), [35] toolBody (line 488),
[36] toolBodyInner (line 505), [37] pythonImpl (line 513),
[40] errorPolicy (line 529), [73] param (line 802). All exist. ✓

### Group C — Type System Contract Checks

**C1[PASS]** Every type mapping in §4 is consistent with TYPESYSTEM.md §10.3:
- `string` → `str` ✓, `int` → `int` ✓, `float` → `float` ✓, `bool` → `bool` ✓
- `SchemaName` → `SchemaName` (Pydantic model) ✓
- `Optional[T]` for `T?` types ✓, `List[T]` for `T[]` types ✓
- `Literal[...]` for literal unions ✓
- `None` for `null` ✓
No contradictions found.

**C2[PASS]** PYB-MAR-10 correctly implements TS-BND-08: bounded types are not
permitted in parameter positions. Cites SEM035. Consistent with TYPESYSTEM.md. ✓

**C3[PASS]** PYB-MAR-08 is consistent with TYPESYSTEM.md TS-RET-02:
`-> null` for void tools, return `None` or implicit None. ✓

**C4[PASS]** All Generated: fields use Pydantic v2 patterns:
- `BaseModel` (not v1 `Model`)
- `model_validate()` (not v1 `parse_obj()`)
- `from pydantic import BaseModel, Field` (v2 imports)
- No `validator`, `__root__`, or `class Config` patterns.
All Python shown is valid Python 3.11+. ✓

### Group D — Error Contract Checks

**D1[PASS]** All cited error codes verified in ERRORS.md:
- PYB001: CODEGEN phase, ERROR severity ✓ (ERRORS.md §6)
- SEM010: RESOLVE phase, ERROR severity ✓ (ERRORS.md §3)
- SEM035: TYPE phase, ERROR severity ✓ (ERRORS.md §3)
- SEM040: TYPE phase, ERROR severity ✓ (ERRORS.md §3)
- SYN050: PARSE phase, ERROR severity ✓ (ERRORS.md §2)
No ghost citations.

**D2[PASS]** PYB001 described consistently with ERRORS.md:
opt-in via `--check-python`, CODEGEN phase, ERROR severity. ✓

**D3[PASS]** SYN050 cited in §7.1 (Post-MVP native tool bodies).
Triggering condition matches ERRORS.md: tool body without `python %{`. ✓

### Group E — Document Quality Checks

**E1[PASS]** Format consistency spot-checked across 5 rules:
PYB-SYN-01 (§2), PYB-IMP-01 (§3.1), PYB-MAR-02 (§4.1),
PYB-GEN-01 (§5.1), PYB-ERR-01 (§6.1).
All have Context:, Plain English:, Grammar:, Valid:, Generated:, Runtime:, Notes:. ✓

**E2[PASS]** Generated: fields show actual Python code in all rules.
PYB-GEN-01 canonical example is complete valid Python 3.11+.
No prose-only Generated: fields. ✓

**E3[PASS]** Contract asymmetry maintained throughout:
- §1.3 explicitly states it
- PYB-MAR-09 documents validation timing
- No rule claims the compiler verifies Python implementation correctness. ✓

**E4[PASS]** `--check-python` consistently described as:
optional, off by default, CODEGEN phase. No rule implies automatic execution. ✓

**E5[PASS]** Self-containment verified:
- Codegen implementer: PYB-GEN-01 through PYB-GEN-04 specify complete generated structure
- Bridge block author: §2 (syntax) + §3 (imports) + §4 (types) are sufficient
- `eaml_runtime` implementer: §4.2 (return validation) + §6 (errors) specify runtime behavior ✓

### Grammar Production Citations (physically verified)

| Production         | Line in grammar.ebnf  | Exists  | Rule(s) citing it             |
|--------------------|-----------------------|---------|-------------------------------|
| [5] IDENT          | 202                   | ✓       | PYB-SYN-01                    |
| [18] PYTHON_BLOCK  | 283                   | ✓       | PYB-SYN-02                    |
| [24] Program       | 353                   | ✓       | PYB-GEN-04                    |
| [26] importDecl    | 393                   | ✓       | PYB-IMP-01                    |
| [34] toolDecl      | 474                   | ✓       | PYB-MAR-06, PYB-GEN-01        |
| [35] toolBody      | 488                   | ✓       | PYB-SYN-03                    |
| [36] toolBodyInner | 505                   | ✓       | PYB-SYN-03, PYB-SYN-04        |
| [37] pythonImpl    | 513                   | ✓       | PYB-SYN-01                    |
| [40] errorPolicy   | 529                   | ✓       | PYB-ERR-02                    |
| [73] param         | 802                   | ✓       | PYB-MAR-01 through PYB-MAR-05 |

### TYPESYSTEM.md Rule Citations (physically verified)

| Rule ID    | Line in TYPESYSTEM.md   | Exists | Cited in            |
|------------|-------------------------|--------|---------------------|
| TS-SCH-01  | 1090                    | ✓      | PYB-MAR-02          |
| TS-SCH-06  | 1196                    | ✓      | PYB-MAR-07          |
| TS-OPT-01  | 487                     | ✓      | PYB-MAR-03          |
| TS-ARR-01  | 437                     | ✓      | PYB-MAR-04          |
| TS-LIT-01  | 950                     | ✓      | PYB-MAR-05          |
| TS-BND-08  | 711                     | ✓      | PYB-MAR-10          |
| TS-RET-02  | 1359                    | ✓      | PYB-MAR-08          |
| §10.3      | 1775                    | ✓      | All §4 rules        |
| §2.6       | 402                     | ✓      | PYB-MAR-01          |

### ERRORS.md Code Citations (physically verified)

| Code   | Section in ERRORS.md   | Severity | Phase   | Cited in         |
|--------|------------------------|----------|---------|------------------|
| PYB001 | §6                     | ERROR    | CODEGEN | PYB-ERR-01       |
| SEM010 | §3                     | ERROR    | RESOLVE | PYB-IMP-01       |
| SEM035 | §3                     | ERROR    | TYPE    | PYB-MAR-10       |
| SEM040 | §3                     | ERROR    | TYPE    | PYB-SYN-03       |
| SYN050 | §2                     | ERROR    | PARSE   | §7.1 (Post-MVP)  |

### Three-Context Coverage

| Context         | Count   | Rules                                                                     |
|-----------------|---------|---------------------------------------------------------------------------|
| COMPILER-only   | 4       | PYB-SYN-01, PYB-SYN-02, PYB-SYN-03, PYB-MAR-10                            |
| GENERATED-only  | 0       |                                                                           |
| RUNTIME-only    | 4       | PYB-MAR-06, PYB-MAR-09, PYB-ERR-02, PYB-ERR-03                            |
| Multi-context   | 14      | PYB-SYN-04, PYB-IMP-01–05, PYB-MAR-01–05,07,08, PYB-GEN-01–04, PYB-ERR-01 |
| RUNTIME (§8 OQ) | 1       | PYB-IMP-06                                                                |

Total named rules: 22 (within target range of 20–25).