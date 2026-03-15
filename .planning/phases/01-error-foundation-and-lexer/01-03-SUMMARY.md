---
phase: 01-error-foundation-and-lexer
plan: 03
subsystem: lexer
tags: [template-strings, interpolation, python-bridge, error-recovery, insta-snapshots]

# Dependency graph
requires:
  - phase: 01-error-foundation-and-lexer/02
    provides: Lexer wrapper with 4 modes, TokenKind enum, logos DFA, error recovery skeleton
provides:
  - Complete template string interpolation with brace-depth tracking and escape handling
  - Python bridge opaque content capture with line-start }% detection
  - Adjacent SYN001 diagnostic collapsing into single spanning diagnostic
  - 25 snapshot tests covering template strings, python bridge, and error recovery
affects: [02-parser]

# Tech tracking
tech-stack:
  added: []
  patterns: [error-collapsing-post-processing, inline-snapshot-tests-with-insta]

key-files:
  created:
    - crates/eaml-lexer/tests/template.rs
    - crates/eaml-lexer/tests/python_bridge.rs
    - crates/eaml-lexer/tests/errors.rs
  modified:
    - crates/eaml-lexer/src/lexer.rs

key-decisions:
  - "Adjacent SYN001 diagnostics collapsed in post-processing pass after tokenization completes"
  - "Python bridge content span includes trailing newline before }% but excludes the }% delimiter itself"

patterns-established:
  - "Inline snapshot tests with format_tokens() and format_diagnostics() helpers for consistent snapshot output"
  - "format_tokens_with_content() helper extracts PythonBlock source content for readable snapshot verification"

requirements-completed: [LEX-05, LEX-06]

# Metrics
duration: 6min
completed: 2026-03-15
---

# Phase 1 Plan 3: Template String and Python Bridge Summary

**Template string interpolation with brace-depth tracking, python bridge opaque capture with line-start }% detection, and adjacent error collapsing -- 25 new snapshot tests**

## Performance

- **Duration:** 6 min
- **Started:** 2026-03-15T22:16:16Z
- **Completed:** 2026-03-15T22:22:37Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- 11 template string snapshot tests verifying interpolation, nested braces, escaped braces, escape sequences, error recovery (SYN045/SYN002)
- 7 python bridge snapshot tests verifying opaque capture, f-string edge case, whitespace close, unterminated (SYN046), and post-bridge lexing continuation
- 7 error recovery tests verifying SYN001 collapsing, multi-error-type sources, and full EAML snippets with mixed errors
- Adjacent SYN001 diagnostic collapsing implemented as post-processing pass in `into_output()`

## Task Commits

1. **Task 1: Template string interpolation tests** - `10ba757` (test)
2. **Task 2: Python bridge tests, error tests, error collapsing** - `1198567` (feat)

## Files Created/Modified
- `crates/eaml-lexer/tests/template.rs` - 11 template string snapshot tests with format helpers
- `crates/eaml-lexer/tests/python_bridge.rs` - 7 python bridge snapshot tests with content extraction
- `crates/eaml-lexer/tests/errors.rs` - 7 error recovery and diagnostic collapsing tests
- `crates/eaml-lexer/src/lexer.rs` - Added `collapse_adjacent_errors` post-processing in `into_output()`

## Decisions Made
- Adjacent SYN001 collapsing done as a post-processing pass rather than inline during tokenization -- simpler, avoids complicating the hot path
- Python bridge content span includes the trailing newline before `}%` as part of the block content, matching user expectations for indentation-preserving capture

## Deviations from Plan

None - plan executed exactly as written. The template string and python bridge scanning were already implemented in Plan 02 as functional stubs; this plan added comprehensive tests and the error collapsing feature.

## Issues Encountered
None.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Phase 1 (Error Foundation and Lexer) is now complete
- All 3 plans executed: error infrastructure, core lexer, template/bridge completion
- 114 total tests across eaml-errors (24) and eaml-lexer (90), all passing, clippy clean
- Parser (Phase 2) can consume the token stream: TmplStart/TmplText/TmplInterpStart/TmplInterpEnd/TmplEnd for template strings, KwPythonBridge/PythonBlock for python bridge blocks

---
*Phase: 01-error-foundation-and-lexer*
*Completed: 2026-03-15*
