---
phase: 02-parser
plan: 03
subsystem: parser
tags: [recursive-descent, declaration-parsing, requires-clause, python-bridge, error-recovery]

# Dependency graph
requires:
  - phase: 02-parser/02-02
    provides: "Leaf parsers: type_expr, expr (Pratt), template string"
  - phase: 02-parser/02-01
    provides: "Parser infrastructure: cursor, expect/peek, synchronize, AST arena"
provides:
  - "All 7 declaration parsers (model, schema, prompt, tool, agent, import, let)"
  - "parse_program() entry point with keyword dispatch loop"
  - "Parameter list parsing with optional defaults"
  - "Requires clause parsing (bare, bracketed, empty)"
  - "Error policy parsing (fail, retry then fail)"
  - "Post-MVP reserved syntax detection (SYN080, 082, 083, 090)"
affects: [02-parser/02-04, 03-semantic]

# Tech tracking
tech-stack:
  added: []
  patterns: [declaration-level-sync-recovery, contextual-keyword-parsing, left-factored-tool-body]

key-files:
  created:
    - crates/eaml-parser/src/decl.rs
    - crates/eaml-parser/tests/declarations.rs
  modified:
    - crates/eaml-parser/src/parser.rs
    - crates/eaml-parser/src/lib.rs
    - crates/eaml-parser/src/type_expr.rs

key-decisions:
  - "Agent 'model' field uses at(KwModel) check since lexer tokenizes 'model' as keyword, not contextual ident"
  - "KwNull handled as valid primitive type name in type_expr parser for 'null' type in schemas"

patterns-established:
  - "Declaration parsers follow expect-or-synchronize pattern: on any expect failure, emit diagnostic, synchronize, return DeclId::Error"
  - "Prompt/agent body fields use at_contextual() checks for field label dispatch"
  - "Tool body uses left-factored dispatch: KwPythonBridge vs description vs empty vs native"

requirements-completed: [PAR-01, PAR-05, PAR-06, PAR-07, PAR-08, PAR-09]

# Metrics
duration: 11min
completed: 2026-03-16
---

# Phase 2 Plan 3: Declaration Parsers Summary

**All 7 EAML declaration types parsed with requires clauses, python bridge tool bodies, agent error policies, and declaration-level error recovery**

## Performance

- **Duration:** 11 min
- **Started:** 2026-03-16T12:18:58Z
- **Completed:** 2026-03-16T12:30:09Z
- **Tasks:** 2
- **Files modified:** 5 (+ 9 snapshot files)

## Accomplishments
- All 7 declaration types (model, schema, prompt, tool, agent, import, let) parse into correct AST nodes
- parse_program() dispatches on keyword with error limit checking and recovery
- All 4 populated example files (minimal, sentiment, types, bad_model) parse with zero errors
- Post-MVP reserved syntax (pipeline, enum, extends, @annotations) emits specific error codes
- 33 declaration tests passing, all workspace tests green

## Task Commits

Each task was committed atomically:

1. **Task 1: Import, model, schema, let parsers and parse_program** - `48100b8` (feat)
2. **Task 2: Prompt, tool, agent parsers with requires clauses** - `c514081` (feat)

## Files Created/Modified
- `crates/eaml-parser/src/decl.rs` - All declaration parsers, parse_program entry point, param list, requires clause, error policy
- `crates/eaml-parser/src/parser.rs` - Removed parse_program stub, added source() accessor
- `crates/eaml-parser/src/lib.rs` - Added `pub mod decl`
- `crates/eaml-parser/src/type_expr.rs` - Added KwNull handling as primitive type
- `crates/eaml-parser/tests/declarations.rs` - 33 tests covering all declaration types and example files

## Decisions Made
- Agent "model" field parsed via `at(TokenKind::KwModel)` since the lexer tokenizes "model" as a keyword -- `at_contextual("model")` would never match
- `null` type handled in type_expr by matching `KwNull` and interning "null" as a Named type expression

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Agent field "model" not recognized as contextual keyword**
- **Found during:** Task 2 (agent declaration parser)
- **Issue:** The lexer tokenizes "model" as KwModel keyword, so at_contextual("model") never matches inside agent body fields
- **Fix:** Changed agent body parsing to check `self.at(TokenKind::KwModel)` instead of `self.at_contextual("model")`
- **Files modified:** crates/eaml-parser/src/decl.rs
- **Verification:** decl_agent_all_fields and decl_agent_retry_policy tests pass
- **Committed in:** c514081

**2. [Rule 1 - Bug] KwNull not recognized as valid type name**
- **Found during:** Task 2 (example file integration tests)
- **Issue:** types.eaml has `empty: null` field -- `null` is KwNull token, not Ident, so parse_base_type rejected it
- **Fix:** Added KwNull match arm in parse_base_type that interns "null" and returns Named type
- **Files modified:** crates/eaml-parser/src/type_expr.rs
- **Verification:** decl_example_types test passes, types.eaml parses with 0 errors
- **Committed in:** c514081

---

**Total deviations:** 2 auto-fixed (2 bugs)
**Impact on plan:** Both fixes necessary for correctness. No scope creep.

## Issues Encountered
None beyond the auto-fixed deviations.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Parser can now parse any valid EAML source file into a complete AST
- Ready for Plan 02-04 (integration tests and final parser verification)
- Semantic analysis (Phase 3) can consume the AST with all declaration types populated

---
*Phase: 02-parser*
*Completed: 2026-03-16*
