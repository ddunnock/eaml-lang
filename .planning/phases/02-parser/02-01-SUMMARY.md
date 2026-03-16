---
phase: 02-parser
plan: 01
subsystem: compiler
tags: [rust, ast, parser, arena, recursive-descent]

# Dependency graph
requires:
  - phase: 01-error-foundation-and-lexer
    provides: Token, TokenKind, Span, Interner, Diagnostic, ErrorCode, LexOutput
provides:
  - Complete AST type system with typed arena allocation and newtype index IDs
  - Parser struct with token cursor, error emission, and synchronization
  - Public parse() API returning ParseOutput (stub program)
affects: [02-parser plans 02-04, 03-semantic]

# Tech tracking
tech-stack:
  added: [lasso (parser crate dep)]
  patterns: [typed-arena-allocation, newtype-index-ids, discriminant-based-token-matching, brace-depth-synchronization]

key-files:
  created:
    - crates/eaml-parser/src/ast.rs
    - crates/eaml-parser/src/parser.rs
    - crates/eaml-parser/tests/parser_infra.rs
  modified:
    - crates/eaml-parser/src/lib.rs
    - crates/eaml-parser/Cargo.toml

key-decisions:
  - "DeclId cannot derive Copy because Span (Range<usize>) does not implement Copy"
  - "Parser expect methods use #[allow(clippy::result_unit_err)] since errors are emitted to diagnostics not returned"

patterns-established:
  - "Typed arena: each AST node kind stored in own Vec, accessed via newtype index ID"
  - "Index<*Id> for Ast: ergonomic ast[expr_id] access pattern"
  - "Parser cursor: peek/advance/at/eat/expect with discriminant-based matching for TokenKind payloads"
  - "Synchronize: skip to declaration keywords at brace depth 0 for error recovery"

requirements-completed: [PAR-09, PAR-08]

# Metrics
duration: 7min
completed: 2026-03-16
---

# Phase 02 Plan 01: Parser Infrastructure Summary

**Complete AST type system with typed arena allocation, parser cursor with peek/advance/expect/synchronize, and public parse() entry point**

## Performance

- **Duration:** 7 min
- **Started:** 2026-03-16T11:52:12Z
- **Completed:** 2026-03-16T11:59:06Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments
- All AST node types defined: 9 declaration types, Expr enum (16 variants), TypeExpr enum (7 variants), TemplateString, BinOp, UnaryOp, Arg, BoundParam
- Typed arena allocation with per-kind Vec and newtype index IDs (ExprId, TypeExprId, ModelDeclId, etc.) preventing cross-arena indexing
- Parser struct with complete cursor API (peek, advance, at, eat, expect, expect_ident, at_contextual, expect_contextual)
- Synchronize function with brace-depth tracking for declaration-level error recovery
- Public parse() API with ParseOutput struct integrating lexer and parser output

## Task Commits

Each task was committed atomically:

1. **Task 1: AST node types, typed arenas, and ID newtypes** - `a586404` (feat)
2. **Task 2: Parser struct with cursor, helpers, and error recovery** - `2c3e12b` (feat)

_Both tasks followed TDD: tests written first (RED), then implementation (GREEN)._

## Files Created/Modified
- `crates/eaml-parser/src/ast.rs` - All AST node types, typed arenas, DeclId enum, Index impls
- `crates/eaml-parser/src/parser.rs` - Parser struct with cursor, diagnostics, synchronize, parse_program stub
- `crates/eaml-parser/src/lib.rs` - Public API: parse(), ParseOutput, re-exports
- `crates/eaml-parser/Cargo.toml` - Added lasso dependency
- `crates/eaml-parser/tests/parser_infra.rs` - 29 tests covering arenas, cursor, and synchronization

## Decisions Made
- DeclId cannot derive Copy because Span (Range<usize>) does not implement Copy -- used Clone only
- Parser expect/expect_ident/expect_contextual return Result<_, ()> with clippy allow since errors are side-effected into diagnostics vec, not returned
- Used std::mem::discriminant for at() token matching to handle Ident(Spur) payload correctly

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Removed Copy derive from DeclId**
- **Found during:** Task 1 (AST node types)
- **Issue:** Plan specified `derive(Debug, Clone, Copy)` for DeclId but Span (Range<usize>) does not implement Copy
- **Fix:** Removed Copy from DeclId derive, using Clone only
- **Files modified:** crates/eaml-parser/src/ast.rs
- **Verification:** cargo check passes
- **Committed in:** a586404

**2. [Rule 1 - Bug] Added clippy allows for Result<_, ()> returns**
- **Found during:** Task 2 (Parser struct)
- **Issue:** Clippy -D warnings rejects Result<_, ()> as error type
- **Fix:** Added #[allow(clippy::result_unit_err)] on expect, expect_ident, expect_contextual
- **Files modified:** crates/eaml-parser/src/parser.rs
- **Verification:** cargo clippy -D warnings passes
- **Committed in:** 2c3e12b

---

**Total deviations:** 2 auto-fixed (2 bugs)
**Impact on plan:** Minor adjustments for Rust type system constraints. No scope creep.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- AST types and parser infrastructure ready for declaration parsing (Plan 02-02)
- parse_program() is a stub returning empty Program -- will be implemented in Plan 02-02
- All 29 tests pass, clippy clean, workspace compiles

---
*Phase: 02-parser*
*Completed: 2026-03-16*
