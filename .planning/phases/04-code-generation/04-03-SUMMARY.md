---
phase: 04-code-generation
plan: 03
subsystem: codegen
tags: [prompt, tool, agent, emitter, python, f-string, bridge, ToolMetadata, execute_prompt]

# Dependency graph
requires:
  - phase: 04-01
    provides: CodeWriter, ImportTracker, emit_type_annotation, names utilities
  - phase: 04-02
    provides: emit_schema, emit_model, emit_let, emit_expr_value, extract_template_text
provides:
  - emit_prompt() producing async def with message lists and execute_prompt() calls
  - emit_tool() producing bridge function + _eaml_call wrapper + ToolMetadata per PYB-GEN-01
  - emit_agent() producing classes extending eaml_runtime.Agent
  - emit_template_as_python_string() for f-string/plain string selection
  - dedent_bridge_code() for bridge block content normalization
  - eaml_type_name() for EAML type name emission in ToolMetadata
affects: [04-04]

# Tech tracking
tech-stack:
  added: []
  patterns: [prompt message list construction, tool three-part emission (bridge/wrapper/metadata), agent class generation]

key-files:
  created:
    - crates/eaml-codegen/tests/prompts.rs
    - crates/eaml-codegen/tests/tools.rs
    - crates/eaml-codegen/tests/agents.rs
  modified:
    - crates/eaml-codegen/src/emitters.rs

key-decisions:
  - "Template strings with no interpolation emit plain strings; with interpolation emit f-strings with brace escaping"
  - "Tool bridge functions return dict per PYB-GEN-01; wrapper validates with model_validate (schema) or isinstance (primitive)"
  - "Agent fields with no entries omit pass; agent tool references use snake_case Python names"

patterns-established:
  - "Prompt emitter: async def with keyword-only model param, message list, execute_prompt call with optional kwargs"
  - "Tool emitter: three-part output (bridge def, _eaml_call wrapper, _tool metadata) per PYB-GEN-01 through PYB-GEN-05"
  - "Agent emitter: class extending eaml_runtime.Agent with model/tools/system_prompt/max_turns/on_error attributes"

requirements-completed: [GEN-05, GEN-06, GEN-08, GEN-09]

# Metrics
duration: 5min
completed: 2026-03-16
---

# Phase 4 Plan 3: Prompt, Tool & Agent Emitters Summary

**Async prompt functions with f-string templates, tool bridge/wrapper/metadata triplets per PYB-GEN spec, and agent orchestration classes**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-16T20:05:42Z
- **Completed:** 2026-03-16T20:10:42Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- Prompt declarations produce async def functions with message list construction, execute_prompt() call, and optional kwargs (temperature, max_tokens, max_retries)
- Template strings correctly convert to f-strings with brace escaping for literal { and }; plain strings when no interpolation present
- Tool declarations emit three-part structure per PYB-GEN-01: bridge function (returns dict), _eaml_call wrapper (validates return), ToolMetadata registration
- Bridge code properly dedented from EAML indentation level; descriptions become triple-quote docstrings
- Agent declarations produce classes with model config reference, snake_case tool names, system prompt, max_turns, on_error policies
- 10 snapshot tests covering all prompt/tool/agent variants

## Task Commits

Each task was committed atomically:

1. **Task 1: Prompt emitter with template-to-f-string conversion** - `27ba9b3` (feat)
2. **Task 2: Tool and Agent emitters** - `2bd93f4` (feat)

## Files Created/Modified
- `crates/eaml-codegen/src/emitters.rs` - Added emit_prompt, emit_tool, emit_agent, emit_template_as_python_string, dedent_bridge_code, eaml_type_name
- `crates/eaml-codegen/tests/prompts.rs` - 4 snapshot tests for prompt emission
- `crates/eaml-codegen/tests/tools.rs` - 3 snapshot tests for tool bridge/wrapper/metadata
- `crates/eaml-codegen/tests/agents.rs` - 3 snapshot tests for agent class generation

## Decisions Made
- Template strings with no interpolation emit plain strings (no f-prefix); interpolation triggers f-string with {{ }} escaping for literal braces
- Tool bridge functions always return `dict` per PYB-GEN-01; wrapper validates with `model_validate` for schema returns, `isinstance` for primitives
- Agent tool references use snake_case Python function names per CONTEXT.md locked decision

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- All individual emitters complete: emit_schema, emit_model, emit_let, emit_prompt, emit_tool, emit_agent
- Ready for plan 04-04 (full pipeline wiring: generate() function, file-level orchestration, import emission)
- emit_template_as_python_string available for reuse in any template context

---
*Phase: 04-code-generation*
*Completed: 2026-03-16*
