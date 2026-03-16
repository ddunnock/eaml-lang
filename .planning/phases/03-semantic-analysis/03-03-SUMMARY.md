---
phase: 03-semantic-analysis
plan: 03
subsystem: compiler
tags: [rust, capability-checking, semantic-analysis, diagnostics, testing]

# Dependency graph
requires:
  - phase: 03-semantic-analysis/plan-01
    provides: "Symbol table, resolver, cycle detection"
  - phase: 03-semantic-analysis/plan-02
    provides: "Type checker, TypeAnnotations, ResolvedType"
provides:
  - "Capability subset checking (cap_checker module)"
  - "CAP010/CAP001/CAP002/CAP020 diagnostic emission"
  - "Integration tests confirming all SEM/TYP/CAP/RES/PYB codes are emittable"
  - "Example file regression tests"
affects: [04-codegen, eaml-cli]

# Tech tracking
tech-stack:
  added: []
  patterns: ["cap_checker as final pass in analyze() pipeline", "HashSet-based capability subset checking"]

key-files:
  created:
    - "crates/eaml-semantic/src/cap_checker.rs"
    - "crates/eaml-semantic/tests/capabilities.rs"
    - "crates/eaml-semantic/tests/integration.rs"
  modified:
    - "crates/eaml-semantic/src/lib.rs"

key-decisions:
  - "Capability subset checking uses HashSet<Spur> for O(1) membership tests"
  - "No agents present: check all prompts against all models; agents present: check only against agent-referenced models"
  - "IntoSpan helper trait on TypeExprId for clean span extraction in CAP020 check"

patterns-established:
  - "Three-step capability validation: declarations, subset, json_mode+string"
  - "Integration test per error code as SEM-11 compliance pattern"

requirements-completed: [SEM-08, SEM-09, SEM-11]

# Metrics
duration: 8min
completed: 2026-03-16
---

# Phase 3 Plan 3: Capability Checking and Integration Tests Summary

**Capability subset checker validates prompt requires against model caps with FATAL CAP010, plus SEM-11 integration tests confirming all 20 semantic error codes are emittable**

## Performance

- **Duration:** 8 min
- **Started:** 2026-03-16T14:40:13Z
- **Completed:** 2026-03-16T14:48:23Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- Capability checker module with CAP010 (FATAL), CAP001/CAP002 (warnings), CAP020 (warning)
- Subset checking validates prompt requires against model caps with full diff message
- 24 integration tests confirming every semantic error code (RES, SEM, TYP, CAP, PYB) is triggerable
- Example file regression tests for 01-minimal, 02-sentiment, 06-capability-error, 07-types

## Task Commits

Each task was committed atomically:

1. **Task 1: Capability checker module with subset checking** - `5f22275` (feat)
2. **Task 2: Integration tests confirming all error codes are emittable** - `fa1b0ab` (test)

## Files Created/Modified
- `crates/eaml-semantic/src/cap_checker.rs` - Capability validation pass with KNOWN_CAPABILITIES registry, subset checking, json_mode+string detection
- `crates/eaml-semantic/src/lib.rs` - Added cap_checker module and integrated into analyze() pipeline
- `crates/eaml-semantic/tests/capabilities.rs` - 14 tests for CAP010/CAP001/CAP002/CAP020 and has_fatal flag
- `crates/eaml-semantic/tests/integration.rs` - 24 tests for SEM-11 error code coverage across all categories

## Decisions Made
- Capability subset checking uses HashSet<Spur> for O(1) membership tests on interned capability names
- When no agents exist, all prompts with requires clauses are checked against all models in the file
- When agents exist, prompts are checked only against agent-referenced models
- IntoSpan helper trait added to TypeExprId for ergonomic span extraction in CAP020 diagnostics
- Parser requires clause syntax is `requires [cap1, cap2]` (bracketed) or `requires cap` (bare single) -- comma-separated bare list not supported

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed requires clause syntax in test sources**
- **Found during:** Task 1 (RED phase)
- **Issue:** Tests used `-> R requires json_mode` but parser expects `requires json_mode -> R` (requires before arrow)
- **Fix:** Reordered all test source strings to match parser grammar
- **Files modified:** crates/eaml-semantic/tests/capabilities.rs
- **Verification:** All 14 tests pass
- **Committed in:** 5f22275 (part of Task 1 commit)

**2. [Rule 1 - Bug] Fixed multi-cap requires syntax in tests**
- **Found during:** Task 1 (GREEN phase)
- **Issue:** Tests used `requires json_mode, streaming` but parser only supports bracketed multi-cap `requires [json_mode, streaming]`
- **Fix:** Changed to bracketed syntax in affected tests
- **Files modified:** crates/eaml-semantic/tests/capabilities.rs
- **Verification:** All 14 tests pass
- **Committed in:** 5f22275 (part of Task 1 commit)

---

**Total deviations:** 2 auto-fixed (2 bugs in test source syntax)
**Impact on plan:** Both fixes were necessary to match parser grammar. No scope creep.

## Issues Encountered
None beyond the syntax deviations documented above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- All three semantic analysis plans complete (resolution, type checking, capability checking)
- Full semantic analysis pipeline: resolve -> type_check -> cap_check
- All SEM/TYP/CAP/RES/PYB error codes confirmed emittable (SEM-11 satisfied)
- has_fatal flag correctly prevents codegen on CAP010
- Ready for Phase 4: Code Generation

---
*Phase: 03-semantic-analysis*
*Completed: 2026-03-16*
