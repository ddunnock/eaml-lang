---
phase: 02-parser
plan: 02
subsystem: compiler
tags: [rust, parser, pratt, type-expressions, template-strings, expressions]

# Dependency graph
requires:
  - phase: 02-parser plan 01
    provides: AST types (Expr, TypeExpr, TemplateString enums), Parser struct with cursor API, typed arena allocation
provides:
  - Type expression parser: parse_type_expr with bounded types, modifier ordering, literal unions
  - Pratt expression parser: parse_expr with all operator precedences and postfix chains
  - Template string parser: parse_template_string consuming lexer-emitted template token sequences
  - Argument list parser: parse_arg_list with LL(2) named argument detection
affects: [02-parser plans 03-04, 03-semantic]

# Tech tracking
tech-stack:
  added: []
  patterns: [pratt-parsing, save-restore-backtracking, binding-power-table]

key-files:
  created:
    - crates/eaml-parser/src/type_expr.rs
    - crates/eaml-parser/src/expr.rs
    - crates/eaml-parser/src/template.rs
    - crates/eaml-parser/tests/type_exprs.rs
    - crates/eaml-parser/tests/expressions.rs
    - crates/eaml-parser/tests/templates.rs
  modified:
    - crates/eaml-parser/src/lib.rs
    - crates/eaml-parser/src/parser.rs

key-decisions:
  - "Literal union detection uses save/restore_pos backtracking to lookahead past template string tokens for pipe"
  - "Comparisons use left-associative BPs (35,36) per CONTEXT.md -- semantic analysis will reject chained comparisons"
  - "finish_with_interner() added to Parser for tests that need Spur resolution after parsing"

patterns-established:
  - "Pratt BP table: ||=10/11, &&=20/21, ==/!=30/31, comparisons=35/36, +/-=40/41, *//=50/51, prefix=70, await=65, postfix=80"
  - "Template string parsing: match TmplStart, loop on TmplText/TmplInterpStart, parse_expr(0) in interpolation slots"
  - "Type modifier ordering: [] then ? for T[]?, ? then [] for T?[], composable for T?[]?"

requirements-completed: [PAR-02, PAR-03, PAR-04]

# Metrics
duration: 9min
completed: 2026-03-16
---

# Phase 02 Plan 02: Leaf Parser Modules Summary

**Type expression, Pratt expression, and template string parsers with 40 insta snapshot tests covering all operator precedences, type modifiers, and interpolation patterns**

## Performance

- **Duration:** 9 min
- **Started:** 2026-03-16T12:03:39Z
- **Completed:** 2026-03-16T12:13:36Z
- **Tasks:** 2
- **Files modified:** 8

## Accomplishments
- Type expression parser handles all grammar productions [42]-[50]: primitives, named types, bounded types with positional/named params, 5 modifier combinations, literal unions, grouped types, and SYN042 multi-dimensional array detection
- Pratt expression parser with correct binding powers for all 12 operators plus 3 postfix operators (field access, function call, index), await prefix, and unary prefix (!, -)
- Template string parser consuming pre-tokenized template sequences with full expression support in interpolation slots
- 40 tests total (16 type expression + 19 expression + 5 template) all using insta snapshots

## Task Commits

Each task was committed atomically:

1. **Task 1: Type expression parser** - `de7e283` (feat)
2. **Task 2: Pratt expression parser and template string parser** - `466b1e5` (feat)

_Both tasks followed TDD: tests written first (RED), then implementation (GREEN)._

## Files Created/Modified
- `crates/eaml-parser/src/type_expr.rs` - Type expression parser with bounded types, modifiers, literal unions
- `crates/eaml-parser/src/expr.rs` - Pratt expression parser with BP table, postfix chains, arg lists
- `crates/eaml-parser/src/template.rs` - Template string parser consuming lexer template tokens
- `crates/eaml-parser/src/parser.rs` - Added finish_with_interner(), made pos pub(crate) for backtracking
- `crates/eaml-parser/src/lib.rs` - Added module declarations for type_expr, expr, template
- `crates/eaml-parser/tests/type_exprs.rs` - 16 tests for type expressions
- `crates/eaml-parser/tests/expressions.rs` - 19 tests for expressions
- `crates/eaml-parser/tests/templates.rs` - 5 tests for template strings

## Decisions Made
- Literal union detection uses save_pos/restore_pos backtracking -- saves position, skips past template string tokens, checks for Pipe token, then either restores and parses the union or restores and falls through to base type
- Comparisons use left-associative binding powers (35,36) per CONTEXT.md decision -- chained comparisons like `a == b == c` will parse as left-associative BinaryOp tree, semantic analysis (Phase 3) rejects with SEM060
- Added finish_with_interner() to Parser since Interner does not implement Clone and tests need it for Spur resolution after parsing

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Removed redundant let binding for spur in Ident match arm**
- **Found during:** Task 2 (Pratt expression parser)
- **Issue:** `let spur = spur;` flagged as redundant by clippy
- **Fix:** Removed the redundant rebinding, using the pattern variable directly
- **Files modified:** crates/eaml-parser/src/expr.rs
- **Verification:** cargo clippy -D warnings passes
- **Committed in:** 466b1e5

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Trivial clippy fix. No scope creep.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- All three leaf parser modules ready for declaration parsers (Plan 02-03/04)
- Declaration parsers will call parse_type_expr() for field types, parse_expr() for let RHS/conditions, parse_template_string() for prompt/model string fields
- 40 tests pass, clippy clean, workspace compiles

---
*Phase: 02-parser*
*Completed: 2026-03-16*
