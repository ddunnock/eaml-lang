//! EAML code generator -- emits Python code from an analyzed AST.
//!
//! Public API: [`generate()`] function producing Python source.

pub mod emitters;
pub mod names;
pub mod types;
pub mod writer;

use std::collections::{HashMap, HashSet, VecDeque};

use eaml_lexer::Interner;
use eaml_parser::ast::*;
use eaml_semantic::type_checker::{ResolvedType, TypeAnnotations};

/// Generates Python source code from a parsed and analyzed EAML program.
///
/// This is the main entry point for code generation. It takes the parse
/// output (AST + interner), the analysis output (symbol table + type
/// annotations), the original source text, and the filename.
///
/// Returns the complete Python module as a string.
pub fn generate(
    parse_output: &eaml_parser::ParseOutput,
    analysis: &eaml_semantic::AnalysisOutput,
    source: &str,
    filename: &str,
) -> String {
    let ast = &parse_output.ast;
    let program = &parse_output.program;
    let interner = &parse_output.interner;
    let type_annotations = &analysis.type_annotations;

    // Two-pass generation per RESEARCH.md Pattern 3:
    // Pass 1: Classify declarations into groups, collect imports
    // Pass 2: Emit code in dependency order

    let mut imports = types::ImportTracker::new();
    let mut body_writer = writer::CodeWriter::new();

    // Classify declarations into ordered groups
    let mut import_decls: Vec<ImportDeclId> = Vec::new();
    let mut let_decls: Vec<LetDeclId> = Vec::new();
    let mut schema_decls: Vec<SchemaDeclId> = Vec::new();
    let mut model_decls: Vec<ModelDeclId> = Vec::new();
    let mut prompt_decls: Vec<PromptDeclId> = Vec::new();
    let mut tool_decls: Vec<ToolDeclId> = Vec::new();
    let mut agent_decls: Vec<AgentDeclId> = Vec::new();

    for decl_id in &program.declarations {
        match decl_id {
            DeclId::Import(id) => import_decls.push(*id),
            DeclId::Let(id) => let_decls.push(*id),
            DeclId::Schema(id) => schema_decls.push(*id),
            DeclId::Model(id) => model_decls.push(*id),
            DeclId::Prompt(id) => prompt_decls.push(*id),
            DeclId::Tool(id) => tool_decls.push(*id),
            DeclId::Agent(id) => agent_decls.push(*id),
            DeclId::Error(_) => {} // Skip error recovery nodes silently
        }
    }

    // Topological sort schemas to handle forward references (RESEARCH.md Pitfall 5).
    schema_decls = toposort_schemas(&schema_decls, ast, interner, type_annotations);

    // --- Emit Python import declarations (from `import python "..."`) ---
    for id in &import_decls {
        let decl = &ast[*id];
        if let ImportDecl::Python { module, .. } = decl {
            let module_text = extract_template_text(module, source);
            body_writer.writeln(&format!("import {module_text}"));
        }
    }
    if !import_decls.is_empty() {
        let has_python_import = import_decls
            .iter()
            .any(|id| matches!(&ast[*id], ImportDecl::Python { .. }));
        if has_python_import {
            body_writer.blank_line();
        }
    }

    // --- Emit let bindings ---
    if !let_decls.is_empty() {
        body_writer.writeln("# --- Let Bindings ---");
        for id in &let_decls {
            emitters::emit_let(
                &ast[*id],
                ast,
                interner,
                type_annotations,
                source,
                &mut body_writer,
                &mut imports,
            );
        }
        body_writer.blank_line();
    }

    // --- Emit schemas ---
    if !schema_decls.is_empty() {
        body_writer.writeln("# --- Schemas ---");
        body_writer.blank_line();
        for (i, id) in schema_decls.iter().enumerate() {
            emitters::emit_schema(
                &ast[*id],
                ast,
                interner,
                type_annotations,
                source,
                &mut body_writer,
                &mut imports,
            );
            if i < schema_decls.len() - 1 {
                body_writer.blank_line();
            }
        }
        body_writer.blank_line();
    }

    // --- Emit models ---
    if !model_decls.is_empty() {
        body_writer.writeln("# --- Models ---");
        body_writer.blank_line();
        for (i, id) in model_decls.iter().enumerate() {
            emitters::emit_model(&ast[*id], interner, source, &mut body_writer);
            if i < model_decls.len() - 1 {
                body_writer.blank_line();
            }
        }
        body_writer.blank_line();
    }

    // --- Emit prompts and tools ---
    if !prompt_decls.is_empty() || !tool_decls.is_empty() {
        body_writer.writeln("# --- Prompts and Tools ---");
        body_writer.blank_line();
        for id in &prompt_decls {
            emitters::emit_prompt(
                &ast[*id],
                ast,
                interner,
                type_annotations,
                source,
                &mut body_writer,
                &mut imports,
            );
            body_writer.blank_line();
        }
        for id in &tool_decls {
            emitters::emit_tool(
                &ast[*id],
                ast,
                interner,
                type_annotations,
                source,
                &mut body_writer,
                &mut imports,
            );
            body_writer.blank_line();
        }
    }

    // --- Emit agents ---
    if !agent_decls.is_empty() {
        body_writer.writeln("# --- Agents ---");
        body_writer.blank_line();
        for (i, id) in agent_decls.iter().enumerate() {
            emitters::emit_agent(
                &ast[*id],
                ast,
                interner,
                source,
                &mut body_writer,
                &mut imports,
            );
            if i < agent_decls.len() - 1 {
                body_writer.blank_line();
            }
        }
        body_writer.blank_line();
    }

    // --- Assemble final output ---
    let mut output_writer = writer::CodeWriter::new();

    // Header comment
    output_writer.writeln(&format!("# Generated by eamlc from {filename}"));
    output_writer.blank_line();

    // Imports (deduplicated via ImportTracker)
    imports.emit_imports(&mut output_writer);

    // Body
    let body = body_writer.finish();
    if !body.is_empty() {
        output_writer.write(&body);
    }

    output_writer.finish()
}

/// Extracts plain text from a template string (ignoring interpolations).
pub(crate) fn extract_template_text(ts: &TemplateString, source: &str) -> String {
    let mut text = String::new();
    for part in &ts.parts {
        if let TemplatePart::Text(span) = part {
            text.push_str(&source[span.clone()]);
        }
    }
    text
}

/// Topologically sorts schema declarations so dependencies come before dependents.
///
/// Uses Kahn's algorithm (BFS topological sort). If a cycle is detected
/// (recursive schemas, allowed with SEM070 warning), falls back to source order.
fn toposort_schemas(
    schemas: &[SchemaDeclId],
    ast: &Ast,
    interner: &Interner,
    type_annotations: &TypeAnnotations,
) -> Vec<SchemaDeclId> {
    if schemas.len() <= 1 {
        return schemas.to_vec();
    }

    // Build a map from schema name -> index in the input vec
    let mut name_to_idx: HashMap<&str, usize> = HashMap::new();
    for (idx, id) in schemas.iter().enumerate() {
        let name = interner.resolve(&ast[*id].name);
        name_to_idx.insert(name, idx);
    }

    // Build adjacency list: edges[i] = set of indices that schema i depends on
    // (i.e., schemas whose types appear in i's fields)
    let n = schemas.len();
    let mut dependents: Vec<Vec<usize>> = vec![Vec::new(); n]; // dep -> list of schemas that depend on it
    let mut in_degree: Vec<usize> = vec![0; n];

    for (idx, id) in schemas.iter().enumerate() {
        let schema = &ast[*id];
        let mut seen_deps: HashSet<usize> = HashSet::new();
        for field in &schema.fields {
            if let Some(resolved) = type_annotations.type_exprs.get(&field.type_expr) {
                collect_schema_deps(
                    resolved,
                    ast,
                    interner,
                    &name_to_idx,
                    idx,
                    &mut dependents,
                    &mut in_degree,
                    &mut seen_deps,
                );
            }
        }
    }

    // Kahn's algorithm
    let mut queue: VecDeque<usize> = VecDeque::new();
    for (i, &deg) in in_degree.iter().enumerate() {
        if deg == 0 {
            queue.push_back(i);
        }
    }

    let mut result: Vec<SchemaDeclId> = Vec::with_capacity(n);
    while let Some(idx) = queue.pop_front() {
        result.push(schemas[idx]);
        for &dep_idx in &dependents[idx] {
            in_degree[dep_idx] -= 1;
            if in_degree[dep_idx] == 0 {
                queue.push_back(dep_idx);
            }
        }
    }

    if result.len() < n {
        // Cycle detected -- fall back to source order
        schemas.to_vec()
    } else {
        result
    }
}

/// Recursively collects schema dependencies from a resolved type.
#[allow(clippy::too_many_arguments)]
fn collect_schema_deps(
    resolved: &ResolvedType,
    ast: &Ast,
    interner: &Interner,
    name_to_idx: &HashMap<&str, usize>,
    current_idx: usize,
    dependents: &mut [Vec<usize>],
    in_degree: &mut [usize],
    seen_deps: &mut HashSet<usize>,
) {
    match resolved {
        ResolvedType::Schema(id) => {
            let dep_name = interner.resolve(&ast[*id].name);
            if let Some(&dep_idx) = name_to_idx.get(dep_name) {
                if dep_idx != current_idx && seen_deps.insert(dep_idx) {
                    // dep_idx must come before current_idx (only add edge once)
                    dependents[dep_idx].push(current_idx);
                    in_degree[current_idx] += 1;
                }
            }
        }
        ResolvedType::Array(inner) | ResolvedType::Optional(inner) => {
            collect_schema_deps(
                inner,
                ast,
                interner,
                name_to_idx,
                current_idx,
                dependents,
                in_degree,
                seen_deps,
            );
        }
        _ => {}
    }
}
