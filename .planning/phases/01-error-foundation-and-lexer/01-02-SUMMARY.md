---
phase: 01-error-foundation-and-lexer
plan: 02
subsystem: lexer
tags: [logos, lasso, lexer, tokenizer, string-interning, template-strings]

# Dependency graph
requires:
  - phase: 01-error-foundation-and-lexer/01
    provides: ErrorCode enum, Diagnostic struct, Span type, DiagnosticCollector
provides:
  - Token/TokenKind types with all 27 keywords, operators, literals
  - Logos-based RawToken DFA for fast keyword/operator/literal matching
  - Lasso Interner wrapper for identifier string interning
  - Lexer wrapper with Normal, TemplateString, Interpolation, PythonBridge modes
  - Public lex() function producing LexOutput (tokens + diagnostics + interner)
  - Error recovery for unexpected chars (SYN001), unterminated strings (SYN002), invalid escapes (SYN004)
affects: [01-error-foundation-and-lexer/03, 02-parser]

# Tech tracking
tech-stack:
  added: [logos 0.14, lasso 0.7]
  patterns: [logos-wrapper-for-mode-switching, base-offset-span-calculation, crlf-normalization]

key-files:
  created:
    - crates/eaml-lexer/src/token.rs
    - crates/eaml-lexer/src/logos_lexer.rs
    - crates/eaml-lexer/src/intern.rs
    - crates/eaml-lexer/src/lexer.rs
    - crates/eaml-lexer/tests/keywords.rs
    - crates/eaml-lexer/tests/literals.rs
    - crates/eaml-lexer/tests/operators.rs
    - crates/eaml-lexer/tests/comments.rs
    - crates/eaml-lexer/tests/task1_raw_tokens.rs
  modified:
    - crates/eaml-lexer/src/lib.rs

key-decisions:
  - "All double-quoted strings tokenized as template strings at lexer level (TmplStart/TmplText/TmplEnd) to avoid context-sensitivity"
  - "Logos lexer spans computed with fixed base offset per scan_normal/scan_interpolation call to avoid mid-iteration offset corruption"
  - "PythonBridge mode implemented as functional stub scanning for }% at line-start, ready for Plan 03 refinement"

patterns-established:
  - "logos-wrapper pattern: create logos::Lexer from source[base..], compute abs spans as base+span.start, break out on mode switch"
  - "template string scanning: hand-scan char-by-char for text/escapes/interpolation, emit multi-token sequence"
  - "error recovery: emit diagnostic + skip bad byte + continue lexing"

requirements-completed: [LEX-01, LEX-02, LEX-03, LEX-04, LEX-07, LEX-08, LEX-09]

# Metrics
duration: 8min
completed: 2026-03-15
---

# Phase 1 Plan 2: Core Lexer Summary

**Logos-based lexer with 4-mode wrapper tokenizing all 27 EAML keywords, operators, numeric/string literals with escape handling, interned identifiers via lasso, and SYN error recovery**

## Performance

- **Duration:** 8 min
- **Started:** 2026-03-15T22:04:06Z
- **Completed:** 2026-03-15T22:11:43Z
- **Tasks:** 2
- **Files modified:** 11

## Accomplishments
- Complete TokenKind enum with all 27 keywords (active + reserved), operators, literals, template string tokens, python bridge tokens, and Eof
- Logos-derived RawToken DFA with skip patterns for whitespace, line comments, doc comments, and multi-line block comments
- Lexer wrapper with Normal mode (logos-driven), TemplateString mode (hand-scanned with escape handling), Interpolation mode (brace-depth tracking), and PythonBridge mode (stub)
- Error recovery producing SYN001/SYN002/SYN004 diagnostics while continuing to lex
- 89 total tests (28 unit + 37 integration + 24 eaml-errors) all passing with clippy clean

## Task Commits

1. **Task 1: Token types, logos DFA, and interner** - `a7358b7` (feat)
2. **Task 2: Lexer wrapper with Normal mode tokenization and error recovery** - `1468b76` (feat)

## Files Created/Modified
- `crates/eaml-lexer/src/token.rs` - Token struct and TokenKind enum with all variant groups
- `crates/eaml-lexer/src/logos_lexer.rs` - Logos-derived RawToken DFA with 27 keywords, all operators, skip patterns
- `crates/eaml-lexer/src/intern.rs` - Lasso Rodeo wrapper for identifier interning
- `crates/eaml-lexer/src/lexer.rs` - Lexer wrapper with 4 modes, error recovery, CRLF normalization
- `crates/eaml-lexer/src/lib.rs` - Public API exports: lex(), Token, TokenKind, Span, Interner, LexOutput
- `crates/eaml-lexer/tests/keywords.rs` - 7 tests for all keyword groups
- `crates/eaml-lexer/tests/operators.rs` - 5 tests for single-char and multi-char operators
- `crates/eaml-lexer/tests/literals.rs` - 15 tests for integers, floats, strings, escapes, interning
- `crates/eaml-lexer/tests/comments.rs` - 10 tests for comment skipping, span accuracy, error recovery
- `crates/eaml-lexer/tests/task1_raw_tokens.rs` - 11 tests for raw token types and interner

## Decisions Made
- All double-quoted strings are tokenized as template strings at the lexer level (TmplStart, TmplText, TmplEnd) to avoid parser-feedback context sensitivity -- per RESEARCH.md recommendation
- Logos spans computed with fixed base offset per scan call to prevent mid-iteration offset corruption when self.pos changes
- PythonBridge mode implemented as a functional stub that scans for `}%` at line-start, ready for Plan 03 to refine

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed span calculation using base offset instead of self.pos**
- **Found during:** Task 2 (Lexer wrapper implementation)
- **Issue:** Using `self.pos + span.start` where `self.pos` was modified during the logos iteration loop caused out-of-bounds panics when tokens appeared after skipped content (comments, whitespace)
- **Fix:** Captured `let base = self.pos` before creating the logos lexer and used `base + span.start` for all offset calculations within that scan pass
- **Files modified:** `crates/eaml-lexer/src/lexer.rs`
- **Verification:** All comment and error recovery tests pass with correct span offsets
- **Committed in:** 1468b76

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Essential fix for correctness. No scope creep.

## Issues Encountered
None beyond the span calculation bug documented above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Core lexer infrastructure complete, ready for Plan 03 (template string interpolation and python bridge mode completion)
- All 4 LexerMode variants exist with entry points; Plan 03 extends TemplateString and PythonBridge
- Token types include all template and python bridge variants needed by Plan 03

---
*Phase: 01-error-foundation-and-lexer*
*Completed: 2026-03-15*
