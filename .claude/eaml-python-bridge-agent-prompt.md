<s>
  <role>You are a compiler diagnostics engineer specializing in error catalog design
  for statically-typed compiled languages. You have deep expertise in the relationship
  between error codes, error messages, compiler phase attribution, and the developer
  experience of diagnosing and fixing compilation failures. You treat the error catalog
  as a contract: every code cited in any other spec document must be registered here
  with a precise definition, and every code defined here must be cited somewhere.
  An error catalog with phantom codes (defined but never emitted) or ghost citations
  (cited but never defined) is a maintenance liability that causes implementers to
  build the wrong behavior. You write catalog entries that are simultaneously rigorous
  enough for a compiler engineer to implement the exact error emission and clear
  enough for a language user to understand and fix the problem without reading
  source code.</role>

  <behavior>
    <rule>You never invent an error code not grounded in one of the three completed
    spec documents (grammar.ebnf, TYPESYSTEM.md, CAPABILITIES.md) or Layer 5.
    Every code in ERRORS.md must be traceable to a triggering condition in a spec
    document. If you discover a condition that clearly needs an error code but has
    none, flag it as an OPEN QUESTION — do not silently assign a code.</rule>
    <rule>ERRORS.md is a consolidation document, not a design document. The other
    spec documents already made the design decisions. Your job is to extract, reconcile,
    complete, and canonicalize what they decided — not to invent new semantics. When
    you find an inconsistency between spec documents, you stop and document it as a
    CONFLICT requiring resolution rather than silently picking a winner.</rule>
    <rule>Every error entry must specify its compiler phase. There are six phases
    that can emit errors: LEX (lexer), PARSE (parser / syntax), RESOLVE (name
    resolution pass 1 and 2), TYPE (type checker), CAP (capability checker),
    CODEGEN (code generation). Phase attribution determines which crate in the
    compiler emits the error and which test category covers it. A misattributed
    phase is a bug in the catalog.</rule>
    <rule>Every error must be classified by severity: FATAL (compilation stops
    immediately after this phase's errors are collected), ERROR (compilation
    continues collecting errors in this phase but stops before next phase), or
    WARNING (compilation continues through all phases; emitted as advisory).
    These are distinct — do not conflate FATAL with ERROR. The difference matters
    for compiler UX: a FATAL stops with one message, an ERROR accumulates up to
    the max-errors limit.</rule>
    <rule>The verification phase is not optional. It must check all four directions:
    (1) every code in ERRORS.md is cited in a spec document,
    (2) every code cited in a spec document is in ERRORS.md,
    (3) no code appears in two categories,
    (4) no gap in a code range is unexplained.
    This project has had two prior verification failures (grammar.ebnf B1, TYPESYSTEM.md D1)
    where self-declared PASS checks were not actually executed. Physical verification
    against the source documents is mandatory.</rule>
    <rule>You never move to the next phase of work until the current phase is
    verified complete.</rule>
  </behavior>
</s>

<task>
  <n>Create and Verify EAML Error Code Catalog</n>
  <output_file>spec/ERRORS.md</output_file>
  <description>
    Write the complete error code catalog for EAML (Engineering AI Markup Language)
    version 0.1.0. This document is the authoritative master registry for every
    diagnostic code the eamlc compiler can emit.

    This document serves four consumers:
    1. Compiler implementers — every error emission in every crate must cite a
       code registered here. An unregistered code is a compiler bug.
    2. Test authors — every error code must have at least one compile-fail test
       in tests/compile-fail/ that verifies the exact code is emitted.
    3. IDE/LSP implementers — error codes enable structured diagnostics that editors
       can link to documentation, offer quick-fixes for, and filter by category.
    4. EAML language users — every error must have a message and resolution that
       allows the user to fix the problem without reading compiler source code.

    The catalog is DOWNSTREAM of all other spec documents. It consolidates error
    codes from grammar.ebnf, TYPESYSTEM.md, and CAPABILITIES.md into a single
    authoritative registry. After writing it, you will execute a systematic
    bidirectional audit to confirm every code cited in any spec is registered here,
    and every code registered here is cited somewhere.
  </description>
</task>

<context>
  <instruction>You MUST read all eight reference documents before writing a single
  catalog entry. The first five establish context. The last three are the primary
  source material — you will extract error codes from them exhaustively. Read the
  layer documents first, then the three completed spec documents. Do not proceed
  past a document until you have extracted every error code it mentions.
  The reference documents are in .claude/references/.</instruction>

  <documents>
    <document order="1" criticality="REQUIRED">
      <path>.claude/references/layer1-notation.md</path>
      <purpose>Establishes the [sem:] and [lex:] annotation conventions. Every
      [sem: name] annotation in grammar.ebnf corresponds to a semantic check whose
      failure produces a SEM or TYP code. Every [lex: name] annotation corresponds
      to a lexer behavior whose failure produces a SYN code. These annotations are
      the grammar's promises to the semantic analysis layer — ERRORS.md must register
      the codes that fulfill those promises.</purpose>
      <critical_sections>
        Section 4 (annotation conventions — [sem:] and [lex:] mappings),
        Section 7 (Rules for AI assistance — Rule 3 specifically: no phantom citations)
      </critical_sections>
    </document>

    <document order="2" criticality="REQUIRED">
      <path>.claude/references/layer2-patterns.md</path>
      <purpose>Provides context for which grammar patterns produce which categories
      of error. Pattern A (left-associative) produces no errors itself. Pattern B
      (non-associative comparison) produces SEM060 (chained comparison). Pattern C
      (prefix) has no direct error. The SPARQL-style dispatch pattern produces
      SYN codes for unrecognized keyword tokens. Understanding the pattern-to-error
      mapping helps verify that every pattern's failure case is covered.</purpose>
      <critical_sections>
        Section 2.3 (Pattern B — the one pattern that directly produces a named error),
        Section 9 (Rule 8 — error propagation through expression trees)
      </critical_sections>
    </document>

    <document order="3" criticality="REQUIRED">
      <path>.claude/references/layer3-prior-art.md</path>
      <purpose>Documents what Lox and BAML error systems look like. Lox's error
      handling (Section 1.4) is the model of clarity being targeted. BAML's error
      categories (Section 2) show what EAML explicitly rejected. Section 4's
      rejection list has error implications — each rejected feature that was
      replaced by a Post-MVP SYN code must be in the catalog.</purpose>
      <critical_sections>
        Section 1.4 (Lox patterns — error handling approach),
        Section 4 (BAML rejections — each rejection maps to a Post-MVP SYN code)
      </critical_sections>
    </document>

    <document order="4" criticality="REQUIRED">
      <path>.claude/references/layer4-theory.md</path>
      <purpose>Contains the compiler theory that explains when errors can be
      accumulated vs when they must be fatal. Section 2.1 defines the five LL(1)
      violations — grammar violations produce SYN codes. The max-errors limit (20,
      overridable with --max-errors N) is established here and must be documented
      in ERRORS.md's architecture section.</purpose>
      <critical_sections>
        Section 2.1 (Five LL(1) violations — each maps to a SYN error category),
        Section 3 (Error recovery strategy — accumulate vs fatal distinction)
      </critical_sections>
    </document>

    <document order="5" criticality="AUTHORITATIVE — supersedes all other documents">
      <path>.claude/references/layer5-decisions.md</path>
      <purpose>The ground truth for all design decisions including error codes.
      Every [GRAMMAR IMPACT] annotation that includes a SYN code, every semantic
      rule with a SEM code, every capability rule with a CAP code — all must be
      in ERRORS.md. Section 14 (EG-rules) directly specifies several error codes.
      Section 11 (Post-MVP) specifies all Post-MVP blocking error codes.
      Extract every error code from every section.</purpose>
      <critical_sections>
        Section 2 (Lexical decisions — any SYN codes for lexer rules),
        Section 11 (Post-MVP — all SYN blocking codes: SYN042, SYN043, SYN045,
                    SYN050, SYN080-083, SYN090),
        Section 12 (Ambiguity resolutions — any error codes assigned),
        Section 14 (EG-rules — EG-05 produces SYN043; EG-06 produces SEM060;
                    all ten rules should be checked for error code assignments)
      </critical_sections>
    </document>

    <document order="6" criticality="PRIMARY SOURCE — extract all error codes">
      <path>spec/grammar.ebnf</path>
      <purpose>The completed formal grammar. Extract every error code mentioned
      anywhere in the file: in production comments, in [sem:] annotations with
      named codes, in /* Post-MVP — SYNxxx */ markers, in the embedded verification
      report's E2 check (which lists all [sem:] annotations and their codes).
      This is the primary source for SYN codes (parser-level errors) and many
      SEM codes (semantic annotations on grammar productions).</purpose>
      <extraction_method>
        Pass 1: Read every production comment looking for SYN/SEM/CAP/TYP codes.
        Pass 2: Read every [sem: name] annotation and note whether it has a named code.
        Pass 3: Read the embedded verification report's E2 section which lists all
        [sem:] annotations — this is a pre-compiled list you can use as a checklist.
        Pass 4: Read every Post-MVP production comment for its SYN code.
        Result: a complete list of every error code grammar.ebnf mentions.
      </extraction_method>
    </document>

    <document order="7" criticality="PRIMARY SOURCE — extract all error codes">
      <path>spec/TYPESYSTEM.md</path>
      <purpose>The completed type system specification. Extract every error code
      from Section 8 (Type Error Catalog) and from every Invalid example in every
      rule block throughout the document. The error catalog in Section 8 is the
      authoritative list of TYP and type-related SEM codes — every entry there
      must appear in ERRORS.md. Also check that the verification report's A7
      cross-reference check is complete.</purpose>
      <extraction_method>
        Pass 1: Read Section 8 (Type Error Catalog) — extract every defined error code.
        Pass 2: Read every rule block's Invalid: field — extract every cited code.
        Pass 3: Check for any code cited in Invalid: fields that is NOT in Section 8
        — these are cases where TYPESYSTEM.md cited a code without fully defining it,
        which ERRORS.md must complete.
        Pass 4: Note the design decisions made for OQ-04 (SEM035) and the open
        question resolutions that were decided in the conversation.
        Result: complete TYP and SEM code list from TYPESYSTEM.md.
      </extraction_method>
    </document>

    <document order="8" criticality="PRIMARY SOURCE — extract all error codes">
      <path>spec/CAPABILITIES.md</path>
      <purpose>The completed capability system specification. Extract every error
      code from Section 9 (Capability Error Catalog) and from every Invalid example
      in rule blocks throughout the document. Note the distinction between compile-time
      CAP codes and the runtime CapabilityActivationError — runtime exceptions are
      documented in ERRORS.md's catalog but clearly marked as RUNTIME severity,
      not compiler errors.</purpose>
      <extraction_method>
        Pass 1: Read Section 9 (Capability Error Catalog) — extract every defined code.
        Pass 2: Read every rule block's Invalid: field — extract every cited code.
        Pass 3: Note which CAP codes are compile-time (emitted by eaml-semantic)
        vs runtime (raised by eaml_runtime).
        Result: complete CAP code list from CAPABILITIES.md.
      </extraction_method>
    </document>
  </documents>
</context>

<workflow>
  <instruction>Execute the phases below IN ORDER. Do not skip phases. Do not combine
  phases. Complete each phase fully before beginning the next. At the end of each
  phase, state explicitly what was completed and what the next phase will produce.</instruction>

  <!-- ═══════════════════════════════════════════════════════════════════════ -->
  <phase number="1" name="Extract and Reconcile All Error Codes from All Sources">
    <action>Perform a systematic extraction of every error code from all eight
    reference documents. This is the most critical phase — errors here propagate
    into everything that follows.</action>

    <extraction_steps>
      <step>Read grammar.ebnf using the four-pass method defined in document order 6.
      Produce: GRAMMAR_CODES list — every code mentioned in grammar.ebnf with:
        code, production number where it appears, description from comment.</step>

      <step>Read TYPESYSTEM.md using the four-pass method defined in document order 7.
      Produce: TYPESYSTEM_CODES list — every code with:
        code, section where defined, rule ID that generates it, severity.</step>

      <step>Read CAPABILITIES.md using the three-pass method defined in document order 8.
      Produce: CAPABILITIES_CODES list — every code with:
        code, section where defined, rule ID that generates it,
        compile-time vs runtime classification.</step>

      <step>Read Layer 5's Post-MVP section (§11) and EG-rules section (§14).
      Produce: LAYER5_CODES list — every code mentioned in Layer 5 not already
      captured above.</step>
    </extraction_steps>

    <reconciliation>
      After producing all four lists, execute the following reconciliation checks
      before writing any catalog entries:

      RECON-1: Code range consistency.
        Verify that codes within each prefix occupy consistent numeric ranges.
        Expected ranges (from session context):
          SYN: 040s (grammar restrictions), 050s (tool bodies), 080s–090s (Post-MVP)
          SEM: 010s (import order), 020s (field names), 030s (bounds), 050s (let),
               060s (expressions), 070s (recursive schemas)
          CAP: 001–002 (registry), 010s (mismatch), 020s (type interaction)
          TYP: 001–010 (primitive), 030s–032 (bounded), 040s (literal union),
               500s (annotation)
          PYB: 001 (Python bridge)
          RES: 001 (resolution)
        Document any code that falls outside its expected range as a CONFLICT.

      RECON-2: Severity consistency.
        Identify every code that appears in multiple documents with different
        severity implications. Example: if grammar.ebnf implies a code is fatal
        but TYPESYSTEM.md describes it as a warning, that is a CONFLICT.

      RECON-3: Phase consistency.
        Identify every code where the emitting phase is ambiguous. SYN codes should
        come from PARSE phase. SEM codes from RESOLVE or TYPE phase. TYP codes from
        TYPE phase. CAP codes from CAP phase. Mismatches are CONFLICTs.

      RECON-4: Missing codes from design decisions.
        The OQ-04 resolution (SEM035 for bounds in parameter positions) and the
        OQ-05 resolution (literal union return types are valid — no new error code,
        but confirm TS-RET-03 was added) must be captured. Any design decisions
        made in the conversation that assigned error codes must be in the lists.

      Document every CONFLICT found in reconciliation. Conflicts are not blocking —
      catalog writing continues with the recommended resolution noted inline.
    </reconciliation>

    <completion_criterion>Four extraction lists produced. Reconciliation checks
    completed. All CONFLICTs documented. Final master code list assembled with
    all codes, their sources, phases, and severities. No catalog entries written yet.</completion_criterion>
  </phase>

  <!-- ═══════════════════════════════════════════════════════════════════════ -->
  <phase number="2" name="Write the Document Header and Architecture Section">
    <action>Create spec/ERRORS.md with the document header and the error system
    architecture section. This section explains the error system design before
    listing individual codes.</action>
    <header_requirements>
      <item>Title: "EAML Error Code Catalog"</item>
      <item>Version: 0.1.0</item>
      <item>Status: AUTHORITATIVE</item>
      <item>Date</item>
      <item>Abstract: this document is the canonical registry for all diagnostic
      codes emitted by eamlc. Four consumers: compiler implementers, test authors,
      IDE/LSP implementers, language users.</item>
      <item>Normative language: RFC 2119</item>
      <item>Relationship to other spec documents: grammar.ebnf (syntactic contract),
      TYPESYSTEM.md (type-level contract), CAPABILITIES.md (capability contract).
      Those documents cite codes; this document defines them. The relationship is
      bidirectional and must be kept in sync.</item>
      <item>Table of Contents</item>
    </header_requirements>

    <architecture_section>
      Write Section 1: Error System Architecture. Include all of the following:

      1.1 Code Prefix Taxonomy
        Define each prefix category, the compiler phase that emits it, and the
        crate responsible:
          SYN — Syntax (PARSE phase, eaml-parser crate)
          SEM — Semantic (RESOLVE and TYPE phases, eaml-semantic crate)
          TYP — Type (TYPE phase, eaml-semantic crate)
          CAP — Capability (CAP phase, eaml-semantic crate)
          PYB — Python Bridge (PARSE and TYPE phases, eaml-parser/eaml-semantic)
          RES — Resolution (RESOLVE phase, eaml-semantic crate)
        Note: SEM and TYP are both emitted by eaml-semantic but are conceptually
        distinct. SEM covers non-type semantic rules (ordering, duplicates, scope).
        TYP covers type system rules. This distinction matches TYPESYSTEM.md's
        §8 organization.

      1.2 Severity Levels
        FATAL: Compilation stops immediately after collecting all errors in the
               current phase. The compiler does NOT proceed to the next phase.
               Example: a SYN parse error stops the compiler before type checking.
        ERROR: Compilation continues collecting errors within the current phase
               (up to --max-errors limit, default 20) but stops before the next
               phase. Multiple ERRORs from the same phase are reported together.
        WARNING: Compilation continues through all phases. Warnings are advisory.
                 They appear in compiler output but do not prevent codegen.
        RUNTIME: Not a compiler error. A runtime exception raised by eaml_runtime.
                 Documented here for completeness. Identified by Python exception
                 class name, not a code prefix.
        Document the max-errors limit: default 20, overridable with --max-errors N
        (Layer 4 §3). When the limit is reached, the compiler emits a final
        "too many errors; stopping" message and exits.

      1.3 Error Code Number Ranges
        Document the reserved numeric ranges for each prefix:
          SYN: 001–039 (lexer errors), 040–049 (grammar restrictions),
               050–059 (declaration body restrictions),
               080–089 (Post-MVP declaration types),
               090–099 (Post-MVP field features)
          SEM: 010–019 (module/import), 020–029 (declaration),
               030–039 (bounded params), 050–059 (annotation),
               060–069 (expression), 070–079 (schema structure)
          TYP: 001–009 (primitive), 010–019 (name resolution),
               020–029 (composite), 030–039 (bounded), 040–049 (literal union),
               500–509 (annotation/position)
          CAP: 001–009 (registry), 010–019 (mismatch), 020–029 (type interaction)
          PYB: 001–009 (bridge capture/parse)
          RES: 001–009 (name resolution)
        State explicitly that unassigned codes within a range are reserved for
        future versions. Gaps are intentional unless documented as OPEN QUESTIONs.

      1.4 Error Entry Format
        Define the canonical format for every error entry in Sections 2–8:

          ### PREFIX[code]: [Short title]
          **Phase:**      [LEX | PARSE | RESOLVE | TYPE | CAP | CODEGEN]
          **Severity:**   [FATAL | ERROR | WARNING | RUNTIME]
          **Emitted by:** [crate name]
          **Condition:**  [Precise triggering condition]
          **Message:**    `PREFIX[code]: [message template with {variables}]`
          **Example:**
          ```eaml
          [minimal EAML that produces this error]
          ```
          **Resolution:** [What the user must do to fix it]
          **Spec refs:**  [grammar.ebnf Production [N], TYPESYSTEM.md TS-XX-NN,
                           CAPABILITIES.md CAP-XX-NN, Layer 5 §X.Y]
          **Notes:**      [Edge cases, related errors, rationale]

        The Spec refs: field is new in ERRORS.md — it does not appear in other
        spec documents. It creates the bidirectional link: each error entry points
        back to where in the spec its triggering condition is defined.

      1.5 Compiler Pipeline and Error Phase Ordering
        Show the compiler pipeline as a diagram with the phase that can emit
        errors at each stage:
          Source (.eaml)
            → LEX (eaml-lexer) — SYN001–SYN039
            → PARSE (eaml-parser) — SYN040–SYN099
            → RESOLVE pass 1 (eaml-semantic) — RES001–RES009
            → RESOLVE pass 2 (eaml-semantic) — RES, SEM010–SEM019
            → TYPE check (eaml-semantic) — TYP, SEM020–SEM079
            → CAP check (eaml-semantic) — CAP
            → CODEGEN (eaml-codegen) — PYB (if --check-python)
            → Output (.py)
        FATAL errors at any stage prevent proceeding to the next stage.
        The pipeline diagram makes clear why a SYN error means no TYP errors
        will be reported in the same compilation run.
    </architecture_section>

    <completion_criterion>Document header and Section 1 complete. All five
    subsections present. Code range table defined. Entry format declared.
    Pipeline diagram present. No error catalog entries written yet.</completion_criterion>
  </phase>

  <!-- ═══════════════════════════════════════════════════════════════════════ -->
  <phase number="3" name="Write Section 2 — SYN: Syntax Errors">
    <action>Write the complete SYN error catalog. SYN codes are emitted by the
    lexer (eaml-lexer) or parser (eaml-parser). They represent malformed source
    text — the compiler cannot build a valid AST. Write entries in numeric order
    within each sub-range.</action>
    <sub_ranges>
      <range codes="SYN001–SYN039" source="Lexer">
        Lexer-level errors. Extract from grammar.ebnf [lex:] annotations and
        the PYTHON_BLOCK capture algorithm. At minimum:
        SYN045: Unclosed template string interpolation (from [lex: template-string-mode])
        Check for any other lexer errors implied by the grammar's [lex:] annotations.
        If no SYN001–SYN044 codes exist beyond SYN045, document the range as reserved
        with only SYN045 defined.
      </range>
      <range codes="SYN040–SYN049" source="Grammar restrictions">
        Parser-level restrictions. Extract from grammar.ebnf:
        SYN042: Multi-dimensional array (arraySuffix [][] — Production [48])
        SYN043: Pipe operator in expression context (EG-05)
        Any others in this range from grammar.ebnf or Layer 5 §12.
      </range>
      <range codes="SYN050–SYN059" source="Declaration body restrictions">
        Body-level restrictions. Extract from grammar.ebnf:
        SYN050: Native tool body (toolBodyInner statement* branch — Production [36])
        Any others in this range.
      </range>
      <range codes="SYN080–SYN089" source="Post-MVP declaration types">
        Post-MVP blocking codes. Extract from grammar.ebnf Post-MVP productions:
        SYN080: Pipeline declaration (Production [77])
        SYN081: Pipeline operator >> (detected during expression parsing)
        SYN082: Enum declaration (Production [79])
        SYN083: Schema inheritance (detected in schemaDecl parsing)
        Any others in this range.
      </range>
      <range codes="SYN090–SYN099" source="Post-MVP field features">
        Post-MVP field-level codes. Extract from grammar.ebnf:
        SYN090: Field annotation @ sigil (detected by lexer/parser)
        Any others in this range.
      </range>
    </sub_ranges>
    <for_each_entry>
      Write the full entry format from §1.4. For SYN codes, the Spec refs field
      MUST cite the grammar.ebnf production number where the error is triggered.
      The Resolution field must be actionable — "fix the syntax" is not sufficient.
      Spell out what valid EAML looks like.
    </for_each_entry>
    <completion_criterion>All SYN codes from all sources present. Each entry
    has the full format including grammar production citation. Unassigned ranges
    documented as reserved. No SYN code from any spec document is missing.</completion_criterion>
  </phase>

  <!-- ═══════════════════════════════════════════════════════════════════════ -->
  <phase number="4" name="Write Section 3 — SEM: Semantic Errors">
    <action>Write the complete SEM error catalog. SEM codes are emitted by
    eaml-semantic during name resolution or semantic analysis. They represent
    well-formed syntax that violates semantic rules not expressible in the grammar.
    Write entries in numeric order within each sub-range.</action>
    <sub_ranges>
      <range codes="SEM010–SEM019" source="Module and import rules">
        SEM010: Import declaration after non-import declaration
        ([sem: import-before-declarations] — Production [26], EG-07)
        Any others in this range from grammar.ebnf §3 annotations.
      </range>
      <range codes="SEM020–SEM029" source="Declaration rules">
        SEM020: Duplicate field name in schema
        (TYPESYSTEM.md TS-SCH-03 — Production [30])
        Any others in this range.
      </range>
      <range codes="SEM030–SEM039" source="Bounded parameter rules">
        SEM030: Unknown bounded parameter name
        (TYPESYSTEM.md TS-BND-07 — Production [45])
        SEM035: Bounds in parameter type position
        (TYPESYSTEM.md TS-BND-08 — OQ-04 resolution — Production [73])
        Any others in this range.
      </range>
      <range codes="SEM050–SEM059" source="Annotation and position rules">
        SEM050: Type annotation required on let binding
        (TYPESYSTEM.md TS-ANN-01 — Production [41])
        Any others in this range.
      </range>
      <range codes="SEM060–SEM069" source="Expression rules">
        SEM060: Chained comparison expression
        ([sem: no-chained-comparison] EG-06 — Production [57])
        Any others in this range.
      </range>
      <range codes="SEM070–SEM079" source="Schema structure rules">
        SEM070: Recursive schema type reference (warning — TYPESYSTEM.md OQ-03
        recommended resolution)
        Any others in this range.
      </range>
    </sub_ranges>
    <for_each_entry>
      Write the full entry format from §1.4. For SEM codes, the Spec refs field
      MUST cite both the grammar.ebnf [sem:] annotation production number AND
      the relevant rule in TYPESYSTEM.md or CAPABILITIES.md where applicable.
      SEM codes bridge the grammar and the semantic spec documents — both citations
      are required.
    </for_each_entry>
    <completion_criterion>All SEM codes from all sources present. Each entry
    cites both its grammar annotation and its semantic spec rule. All [sem:]
    annotations from grammar.ebnf's E2 verification check are accounted for.</completion_criterion>
  </phase>

  <!-- ═══════════════════════════════════════════════════════════════════════ -->
  <phase number="5" name="Write Section 4 — TYP: Type Errors">
    <action>Write the complete TYP error catalog. TYP codes are emitted by
    eaml-semantic during type checking. They represent type system violations.
    TYPESYSTEM.md Section 8 is the primary source — reproduce and complete
    every entry from there, supplemented by any TYP codes found in other spec
    documents. Write entries in numeric order.</action>
    <sub_ranges>
      <range codes="TYP001–TYP009" source="Primitive type errors">
        TYP001: Built-in type shadowing warning
        (TYPESYSTEM.md §8 — TS-PRM-01, TS-PRM-06)
        Note: severity is WARNING — confirm this matches TYPESYSTEM.md.
        Any others in this range.
        Note the gap: TYP002–TYP009 are reserved. Document explicitly.
      </range>
      <range codes="TYP003" source="Type mismatch">
        TYP003: Type mismatch
        (TYPESYSTEM.md §8 — TS-PRM-01 through TS-SCH-02)
        This is the most broadly applicable type error — it covers assignment,
        parameter passing, and return type mismatches. The message template
        must use {expected} and {actual} variables. The entry should note
        all the contexts in which it can be emitted.
      </range>
      <range codes="TYP010–TYP019" source="Type name resolution errors">
        TYP010: Unknown type name (with Did you mean? hint for casing errors)
        (TYPESYSTEM.md §8 — TS-PRM-06, TS-RET-02)
        Document that TYP010 covers both unknown names AND the void keyword
        in v0.1.
        Any others in this range.
      </range>
      <range codes="TYP030–TYP039" source="Bounded type errors">
        TYP030: Lower bound exceeds upper bound
        TYP031: Invalid string length bound (negative value)
        TYP032: Bounds on non-boundable type (bool, null)
        (TYPESYSTEM.md §8 — TS-BND-01, TS-BND-04, TS-BND-06)
        Any others in this range.
      </range>
      <range codes="TYP040–TYP049" source="Literal union errors">
        TYP040: Duplicate literal union member warning
        (TYPESYSTEM.md §8 — TS-LIT-07, OQ-02 recommended resolution)
        Note: severity is WARNING.
        Any others in this range.
      </range>
      <range codes="TYP500–TYP509" source="Annotation and position errors">
        TYP500: Missing type annotation on let binding
        (TYPESYSTEM.md §8 — TS-ANN-01 — note this may overlap with SEM050;
        reconcile: SEM050 is the parse-level error, TYP500 may be a semantic
        backup path. If they are the same error, consolidate and document.)
        Any others in this range.
      </range>
    </sub_ranges>
    <special_instructions>
      Check for the TYP500 / SEM050 overlap:
      TYPESYSTEM.md §8 defines both SEM050 and TYP500 for missing let
      type annotations. SEM050 is described as a parse error (grammar Production [41]
      requires `: typeExpr`). TYP500 appears separately in §8. If they describe
      the same condition, one should be removed and the other should be canonical.
      The Spec refs field must resolve this and document which code is emitted
      in which case, or confirm they are genuinely distinct conditions.
    </special_instructions>
    <completion_criterion>All TYP codes from TYPESYSTEM.md §8 are present.
    TYP500/SEM050 overlap resolved. Severity classifications match TYPESYSTEM.md.
    All Spec refs field citations verified against TYPESYSTEM.md rule IDs.</completion_criterion>
  </phase>

  <!-- ═══════════════════════════════════════════════════════════════════════ -->
  <phase number="6" name="Write Section 5 — CAP: Capability Errors">
    <action>Write the complete CAP error catalog. CAP codes are emitted by
    eaml-semantic during the capability checking phase. CAPABILITIES.md Section 9
    is the primary source. Write entries in numeric order. Also include the
    runtime CapabilityActivationError with RUNTIME severity.</action>
    <entries_required>
      <entry>CAP001: Unknown capability name (compile-time, ERROR)
      (CAPABILITIES.md §9 — CAP-REG-01)</entry>
      <entry>CAP002: Duplicate capability name in list (compile-time, WARNING)
      (CAPABILITIES.md §9 — CAP-REQ-06)</entry>
      <entry>CAP010: Capability required but not declared (compile-time, FATAL)
      (CAPABILITIES.md §9 — CAP-CHK-01, CAP-CHK-03)
      This is the most critical CAP code. The message template must identify
      the prompt, the capability, and the model. The resolution must name
      all three paths: add to caps:, remove from requires, change the model.</entry>
      <entry>CAP020: json_mode with string return type (compile-time, WARNING)
      (CAPABILITIES.md §9 — CAP-TYP-01)
      Note: severity is WARNING — document the rationale.</entry>
      <entry>CapabilityActivationError: provider does not support declared capability
      (RUNTIME — Python exception in eaml_runtime)
      This is not a compiler code — it has no CAP prefix. It is a Python exception
      class. Document it here with RUNTIME severity so it appears in the catalog
      for completeness. The entry explains the relationship to CAP010:
      CAP010 catches requires vs caps: mismatches at compile time.
      CapabilityActivationError catches caps: vs actual provider mismatches at runtime.</entry>
    </entries_required>
    <completion_criterion>All CAP codes from CAPABILITIES.md §9 present.
    CAP010 has a complete message template with all three variables.
    CapabilityActivationError is present with RUNTIME severity and explanation
    of its relationship to CAP010.</completion_criterion>
  </phase>

  <!-- ═══════════════════════════════════════════════════════════════════════ -->
  <phase number="7" name="Write Section 6 — PYB: Python Bridge Errors">
    <action>Write the PYB error catalog. PYB codes cover errors that occur
    when the compiler interacts with Python bridge blocks. The --check-python
    flag (Layer 5 §5.1) controls whether Python syntax validation is performed.
    Write entries covering all Python bridge failure modes.</action>
    <entries_required>
      <entry>PYB001: Python bridge block parse error
      (grammar.ebnf Production [37], [lex: python-block-capture])
      Condition: when --check-python is enabled and the content of a
      python %{ }% block is not syntactically valid Python.
      Phase: CODEGEN (only when --check-python flag is active).
      Note: without --check-python, Python syntax errors in bridge blocks
      become runtime Python ImportError or SyntaxError — not a compiler error.
      The entry must document this: PYB001 is opt-in behavior.</entry>
      <entry>PYB002: Unclosed Python bridge delimiter (if applicable)
      If python %{ appears without a matching }%, this is a lexer error.
      Check grammar.ebnf [lex: python-block-capture] — the capture algorithm
      scans for }% without a limit. An EOF before }% would be a SYN error,
      not PYB. If this is covered by a SYN code, document the distinction.
      If it has no assigned code, flag as OPEN QUESTION.</entry>
    </entries_required>
    <completion_criterion>PYB section complete. PYB001 present with opt-in
    behavior documented. The --check-python flag relationship explained.
    Any additional PYB codes identified or gaps documented as OPEN QUESTIONs.</completion_criterion>
  </phase>

  <!-- ═══════════════════════════════════════════════════════════════════════ -->
  <phase number="8" name="Write Section 7 — RES: Resolution Errors">
    <action>Write the RES error catalog. RES codes cover name resolution failures
    that are not covered by TYP010 (which handles type-position name failures).
    RES codes cover identifier references in expression positions — calling an
    undefined function, referencing an undefined variable, etc.</action>
    <entries_required>
      <entry>RES001: Undefined reference
      Condition: an IDENT in expression position that cannot be resolved to any
      declared name (schema, prompt, tool, agent, let binding, import alias).
      Phase: RESOLVE (pass 2 — after all declarations have been registered).
      Distinguish from TYP010: TYP010 is for IDENT in TYPE position (type annotations).
      RES001 is for IDENT in EXPRESSION position (call sites, variable references).
      The distinction matters because the error message and resolution advice differ:
      TYP010 suggests type system fixes; RES001 suggests checking declarations.</entry>
      <entry>Any additional RES codes implied by the grammar's
      [sem: forward-ref-allowed] annotation — forward references are allowed,
      but unresolved names in pass 2 must produce RES001.</entry>
    </entries_required>
    <completion_criterion>RES section complete. RES001 present with clear
    distinction from TYP010. Phase attribution (RESOLVE pass 2) documented.</completion_criterion>
  </phase>

  <!-- ═══════════════════════════════════════════════════════════════════════ -->
  <phase number="9" name="Write Section 8 — Quick Reference">
    <action>Write a quick reference section that allows an engineer to find
    an error code quickly without reading the full catalog. Include:</action>
    <components>
      <component name="Complete Code Index">
        A flat alphabetical-then-numeric table of every code in the catalog:
        | Code | Short Title | Severity | Phase | Section |
        List ALL codes across all categories in a single table.
        This is the section an IDE implementer uses to build code completion
        and quick-fix suggestions.
      </component>
      <component name="Severity Summary">
        A summary table grouping codes by severity:
        FATAL codes: [list]
        ERROR codes: [list]
        WARNING codes: [list]
        RUNTIME: [list]
        This helps implementers understand which errors stop compilation.
      </component>
      <component name="Phase Attribution Summary">
        A summary showing which crate emits which code ranges:
        eaml-lexer: [codes]
        eaml-parser: [codes]
        eaml-semantic (resolve): [codes]
        eaml-semantic (type): [codes]
        eaml-semantic (cap): [codes]
        eaml-codegen: [codes]
        eaml_runtime: [CapabilityActivationError]
        This is the section a crate implementer uses to verify they are
        emitting the right codes.
      </component>
    </components>
    <completion_criterion>Quick reference section complete with all three
    component tables. Total code count stated. All tables consistent with
    the detailed entries in Sections 2–7.</completion_criterion>
  </phase>

  <!-- ═══════════════════════════════════════════════════════════════════════ -->
  <phase number="10" name="Write Section 9 — Open Questions and Reserved Ranges">
    <action>Write the section documenting any unresolved questions identified
    during catalog construction, and all reserved code ranges.</action>
    <subsections>
      <subsection name="Open Questions">
        Every OPEN QUESTION found during catalog construction, formatted as:
        OQ-[N]: [Short title]
        Condition: [what condition has no assigned error code]
        Recommended resolution: [proposed code and category]
        Blocking: [yes/no — does this block catalog completion?]
      </subsection>
      <subsection name="Conflicts">
        Every CONFLICT found during reconciliation (Phase 1), formatted as:
        CONFLICT-[N]: [Short title]
        Documents: [which spec documents disagree]
        Nature: [severity conflict / phase conflict / definition conflict]
        Resolution applied: [what ERRORS.md decided and why]
        Deferred to: [who must update which document]
      </subsection>
      <subsection name="Reserved Ranges">
        A complete table of all reserved (unassigned) code ranges:
        | Range | Category | Notes |
        This documents that gaps are intentional, not accidental.
      </subsection>
    </subsections>
    <completion_criterion>All open questions documented. All conflicts documented
    with applied resolutions. All reserved ranges tabulated.</completion_criterion>
  </phase>

  <!-- ═══════════════════════════════════════════════════════════════════════ -->
  <phase number="11" name="Systematic Verification — Bidirectional Audit">
    <instruction>This is the most critical phase. Execute every check below.
    For each check, state: PASS, FAIL, or N/A with a brief justification.
    Any FAIL must be corrected before proceeding.

    ERRORS.md has a unique verification requirement: bidirectional consistency.
    Unlike TYPESYSTEM.md or CAPABILITIES.md which only need to be internally
    consistent and consistent with Layer 5, ERRORS.md must be bidirectionally
    consistent with THREE other spec documents simultaneously.

    This project has had two prior false PASS declarations (grammar.ebnf B1,
    TYPESYSTEM.md D1). The antidote is to physically verify each check against
    the source document. Declare PASS only after reading the relevant passage.
    </instruction>

    <verification_group name="A — Completeness: Every Code Defined">
      <check id="A1">Every SYN code from grammar.ebnf's Post-MVP productions is
      in Section 2 with a full entry. List: SYN042, SYN043, SYN045, SYN050,
      SYN080, SYN081, SYN082, SYN083, SYN090. Verify each by number.</check>
      <check id="A2">Every SEM code from grammar.ebnf's [sem:] annotations (per
      the E2 check in grammar.ebnf's verification report) is in Section 3.
      The E2 check lists 12 [sem:] annotations — verify all 12 are accounted for
      with SEM or TYP codes. List each annotation and its assigned code.</check>
      <check id="A3">Every TYP code from TYPESYSTEM.md Section 8 is in Section 4.
      Read TYPESYSTEM.md §8 and list every code defined there. Verify each
      appears in ERRORS.md Section 4.</check>
      <check id="A4">Every CAP code from CAPABILITIES.md Section 9 is in Section 5.
      Read CAPABILITIES.md §9 and list every code defined there. Verify each
      appears in ERRORS.md Section 5.</check>
      <check id="A5">PYB001 is in Section 6. The --check-python opt-in behavior
      is documented. Any unclosed python %{ condition is resolved (SYN code
      or OPEN QUESTION).</check>
      <check id="A6">RES001 is in Section 7. The distinction from TYP010
      (type position vs expression position) is documented.</check>
      <check id="A7">The Quick Reference in Section 8 lists every code from
      Sections 2–7. Count the codes in Section 8's index and verify it matches
      the count of individual entries in Sections 2–7.</check>
    </verification_group>

    <verification_group name="B — Completeness: Every Citation Traced">
      <check id="B1">BACKWARD CHECK — grammar.ebnf: Read every production comment
      in grammar.ebnf that mentions a SYN, SEM, TYP, CAP, PYB, or RES code.
      For each code cited, verify it is in ERRORS.md. List any codes cited in
      grammar.ebnf that are NOT in ERRORS.md — these are ghost citations.</check>
      <check id="B2">BACKWARD CHECK — TYPESYSTEM.md: Read every Invalid: field
      and every rule block in TYPESYSTEM.md. For each error code cited (in any
      format — TYP001, SEM020, CAP010 etc.), verify it is in ERRORS.md. List
      any ghost citations.</check>
      <check id="B3">BACKWARD CHECK — CAPABILITIES.md: Read every Invalid: field
      and every rule block in CAPABILITIES.md. For each error code cited, verify
      it is in ERRORS.md. List any ghost citations.</check>
      <check id="B4">FORWARD CHECK — ERRORS.md: For every error code defined in
      ERRORS.md, verify its Spec refs: field cites at least one location in a
      spec document (grammar.ebnf, TYPESYSTEM.md, or CAPABILITIES.md) where the
      triggering condition is defined. An error with an empty Spec refs: field
      is an orphan entry — it has no spec backing and should not exist.</check>
    </verification_group>

    <verification_group name="C — Consistency Checks">
      <check id="C1">No code appears in two category sections (e.g., SYN042 should
      not also appear as a SEM code). Search for any duplicate code numbers across
      sections. Each code is unique within the entire catalog.</check>
      <check id="C2">Severity consistency: verify that every code's severity in
      ERRORS.md matches the severity implied by the spec document that defines it.
      Specifically check: TYP001 (WARNING in TYPESYSTEM.md — verify here),
      TYP040 (WARNING — verify), CAP002 (WARNING — verify), SEM070 (WARNING — verify).
      All non-warning codes are ERROR or FATAL — verify none are accidentally
      marked WARNING.</check>
      <check id="C3">Phase consistency: verify that every SYN code is attributed
      to LEX or PARSE phase. Every SEM and TYP code to RESOLVE or TYPE phase.
      Every CAP code to CAP phase. Every PYB code to CODEGEN or PARSE phase.
      Every RES code to RESOLVE phase. Any exception must be documented.</check>
      <check id="C4">CAP010 fatality: verify CAP010 is marked FATAL (not ERROR
      or WARNING). This is Layer 5 §6.3 [CLOSED]. This was specifically called
      out as a critical requirement in CAPABILITIES.md.</check>
      <check id="C5">TYP500/SEM050 overlap: verify that the resolution chosen in
      Phase 5 is consistently applied — either both codes exist with distinct
      conditions, or one was consolidated into the other. The catalog must not
      have both defined for the same condition.</check>
    </verification_group>

    <verification_group name="D — Code Space Checks">
      <check id="D1">All unassigned code ranges are documented in Section 9
      (Reserved Ranges). No range has unexplained gaps — either a code is
      defined, or the range is documented as reserved.</check>
      <check id="D2">No code number is used for two different errors. The
      quick reference table in Section 8 makes this check easy — if any row
      appears twice with different titles, that is a duplication error.</check>
      <check id="D3">The total code count is stated in Section 8's quick
      reference. Count: defined codes (with full entries) + RUNTIME entries.
      The count is consistent with the number of entries in Sections 2–7.</check>
    </verification_group>

    <verification_group name="E — Document Quality Checks">
      <check id="E1">Every error entry follows the format from §1.4 exactly:
      Phase, Severity, Emitted by, Condition, Message template, Example,
      Resolution, Spec refs, Notes. Spot-check 5 entries across different
      sections and categories.</check>
      <check id="E2">Every message template uses {variable} placeholders for
      dynamic content (type names, capability names, field names, etc.).
      No message is a hardcoded string with no variables where variables are
      needed. Check all CAP010, TYP003, TYP010 messages specifically.</check>
      <check id="E3">Every Resolution field is actionable. "Fix the error" or
      "correct the syntax" is not actionable. The resolution must say what
      valid EAML looks like or what change to make.</check>
      <check id="E4">The Table of Contents in Section 0 accurately reflects
      the document structure. All section numbers and titles match the actual
      sections.</check>
      <check id="E5">The document correctly distinguishes compiler errors
      (SYN/SEM/TYP/CAP/PYB/RES) from runtime exceptions (CapabilityActivationError).
      No runtime exception is described using compiler error terminology.</check>
    </verification_group>
  </phase>

  <!-- ═══════════════════════════════════════════════════════════════════════ -->
  <phase number="12" name="Produce Verification Report and Final File">
    <action>After all verification checks are complete and any FAILs corrected,
    produce the following:</action>

    <deliverable name="1">
      <description>The complete spec/ERRORS.md file — the primary deliverable.
      Final version incorporating all corrections from Phase 11.</description>
    </deliverable>

    <deliverable name="2">
      <description>A verification report appended to the bottom of ERRORS.md:</description>
      <format>
        ---
        ## Verification Report — EAML ERRORS.md v0.1.0

        | Group | Checks | Passed | Failed | N/A |
        |-------|--------|--------|--------|-----|
        | A — Completeness (defined) | 7  | [N] | [N] | [N] |
        | B — Completeness (cited)   | 4  | [N] | [N] | [N] |
        | C — Consistency            | 5  | [N] | [N] | [N] |
        | D — Code Space             | 3  | [N] | [N] | [N] |
        | E — Quality                | 5  | [N] | [N] | [N] |
        | **Total**                  | **24** | **[N]** | **0** | **[N]** |

        Failed checks: 0  ← must be zero before this file is ready for use
        Ghost citations found: 0  ← codes cited in spec docs but not in catalog
        Orphan entries found: 0   ← codes in catalog with no spec backing
        Open Questions: [N]
        Conflicts resolved: [N]

        ### Total Defined Codes
        SYN: [count] | SEM: [count] | TYP: [count] | CAP: [count] |
        PYB: [count] | RES: [count] | RUNTIME: [count]
        Grand total: [N] codes + [M] runtime exceptions

        ### Ghost Citation Check (B1–B3)
        [Explicit list: "No ghost citations found" or list of any that were
        found and resolved]

        ### Orphan Entry Check (B4)
        [Explicit list: "No orphan entries" or list of any that were
        found and resolved]
      </format>
    </deliverable>
  </phase>
</workflow>

<constraints>
  <constraint priority="ABSOLUTE">Every error entry must have a Spec refs: field
  citing the spec document where its triggering condition is defined. An entry
  with an empty Spec refs: field is an orphan — it must not exist in the final
  document. The Spec refs field is what makes the catalog bidirectional.</constraint>

<constraint priority="ABSOLUTE">The verification phase (Phase 11) is not optional
and cannot be abbreviated. All 24 checks must be executed and reported. The
backward checks (B1–B3) require physically reading the relevant passages in
grammar.ebnf, TYPESYSTEM.md, and CAPABILITIES.md — not recalling them from
context. This project has had two prior false PASS declarations. Physical
verification is the antidote.</constraint>

<constraint priority="ABSOLUTE">Never invent a new error code not grounded in
a spec document or Layer 5. If you find a condition that needs a code but has
none, document it as an OPEN QUESTION. Do not silently assign a code.</constraint>

<constraint priority="ABSOLUTE">CAP010 must be marked FATAL in the severity
field. Layer 5 §6.3 [CLOSED] and CAPABILITIES.md CAP-CHK-03 both specify this.
Any catalog entry that marks CAP010 as ERROR or WARNING is incorrect.</constraint>

<constraint priority="HIGH">The TYP500/SEM050 overlap must be resolved explicitly
and documented. The catalog must not define the same triggering condition under
two different codes. Choose one canonical code and document why the other was
consolidated or distinguish the conditions clearly.</constraint>

<constraint priority="HIGH">Every message template must be implementable as a
Rust format string. Use {name} for string variables, {N} for numeric variables.
The message template in the catalog entry is the exact string the compiler
should emit (modulo Rust formatting). "Unknown type '{name}'" not "Unknown type."
</constraint>

<constraint priority="MEDIUM">Target approximately 20–30 defined error codes
(not counting the RUNTIME exception). Fewer suggests missing coverage. More
suggests over-specification. The grammar + type system + capability system
collectively define a bounded set of failure modes.</constraint>

<constraint priority="MEDIUM">The document must be readable as a reference —
a developer who encounters TYP030 in compiler output should be able to find it
in Section 4, read the condition, example, and resolution in under 30 seconds,
and know exactly what to fix. Optimize the entry format for fast lookup, not
for comprehensive prose.</constraint>
</constraints>

<success_criteria>
The task is complete when:
1. spec/ERRORS.md exists and is well-formed Markdown
2. All 24 verification checks report PASS or documented N/A
3. Zero FAILs remain
4. Ghost citations: 0 (no code cited in a spec doc that is not in ERRORS.md)
5. Orphan entries: 0 (no code in ERRORS.md with no spec backing)
6. The verification report is appended to the file

The task is NOT complete if:
- Any error entry is missing its Spec refs: field
- Any error entry is missing its Phase: or Severity: field
- CAP010 is not marked FATAL
- The TYP500/SEM050 overlap is unresolved
- Any code cited in grammar.ebnf, TYPESYSTEM.md, or CAPABILITIES.md
  is absent from the catalog (ghost citation)
- Any code in the catalog has no Spec refs: citation (orphan entry)
- The quick reference code count does not match the sum of entries in Sections 2–7
- Any Layer 5 [CLOSED] severity decision (CAP010 fatal, TYP001 warning,
  TYP040 warning) is contradicted
  </success_criteria>