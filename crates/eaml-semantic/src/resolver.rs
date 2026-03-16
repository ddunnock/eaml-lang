//! Two-pass name resolution for EAML semantic analysis.
//!
//! Pass 1: Register all top-level declarations in the symbol table.
//! Pass 2: Resolve all references against the symbol table.
//! Pass 3: Detect cyclic schema references via DFS.

use std::collections::{HashMap, HashSet};

use lasso::Spur;

use eaml_errors::{Diagnostic, DiagnosticCollector, ErrorCode, Severity, Span};
use eaml_lexer::Interner;
use eaml_parser::ast::*;

use crate::symbol_table::{SymbolInfo, SymbolKind, SymbolTable};

/// Performs name resolution on the AST, returning a populated symbol table.
pub fn resolve(
    program: &Program,
    ast: &Ast,
    interner: &Interner,
    _source: &str,
    diags: &mut DiagnosticCollector,
) -> SymbolTable {
    let mut symbols = SymbolTable::new(interner);

    // Pass 1: Register all top-level declarations
    pass1_register(program, ast, interner, &mut symbols, diags);

    // Pass 2: Resolve all references
    pass2_resolve(program, ast, interner, &symbols, diags);

    // Pass 3: Detect cyclic schema references
    pass3_cycle_detection(ast, interner, &symbols, diags);

    symbols
}

// =============================================================================
// Pass 1: Register declarations
// =============================================================================

fn pass1_register(
    program: &Program,
    ast: &Ast,
    interner: &Interner,
    symbols: &mut SymbolTable,
    diags: &mut DiagnosticCollector,
) {
    let mut seen_non_import = false;

    for decl_id in &program.declarations {
        match decl_id {
            DeclId::Model(id) => {
                seen_non_import = true;
                let decl = &ast[*id];
                register_symbol(
                    symbols,
                    decl.name,
                    SymbolInfo {
                        kind: SymbolKind::Model(*id),
                        span: decl.span.clone(),
                        name_spur: decl.name,
                    },
                    interner,
                    diags,
                );
            }
            DeclId::Schema(id) => {
                seen_non_import = true;
                let decl = &ast[*id];
                register_symbol(
                    symbols,
                    decl.name,
                    SymbolInfo {
                        kind: SymbolKind::Schema(*id),
                        span: decl.span.clone(),
                        name_spur: decl.name,
                    },
                    interner,
                    diags,
                );
            }
            DeclId::Prompt(id) => {
                seen_non_import = true;
                let decl = &ast[*id];
                register_symbol(
                    symbols,
                    decl.name,
                    SymbolInfo {
                        kind: SymbolKind::Prompt(*id),
                        span: decl.span.clone(),
                        name_spur: decl.name,
                    },
                    interner,
                    diags,
                );
            }
            DeclId::Tool(id) => {
                seen_non_import = true;
                let decl = &ast[*id];
                register_symbol(
                    symbols,
                    decl.name,
                    SymbolInfo {
                        kind: SymbolKind::Tool(*id),
                        span: decl.span.clone(),
                        name_spur: decl.name,
                    },
                    interner,
                    diags,
                );
            }
            DeclId::Agent(id) => {
                seen_non_import = true;
                let decl = &ast[*id];
                register_symbol(
                    symbols,
                    decl.name,
                    SymbolInfo {
                        kind: SymbolKind::Agent(*id),
                        span: decl.span.clone(),
                        name_spur: decl.name,
                    },
                    interner,
                    diags,
                );
            }
            DeclId::Import(id) => {
                let decl = &ast[*id];
                let is_python = matches!(decl, ImportDecl::Python { .. });

                // Check ordering: Python imports must appear before declarations
                if is_python && seen_non_import {
                    let span = import_span(decl);
                    diags.emit(Diagnostic::new(
                        ErrorCode::Sem010,
                        "python import must appear before all declarations".to_string(),
                        span,
                        Severity::Error,
                        "import after declaration".to_string(),
                    ));
                }

                if !is_python {
                    seen_non_import = true;
                }

                // Register the import name if it has an alias
                if let Some((name_spur, span)) = import_name(decl) {
                    register_symbol(
                        symbols,
                        name_spur,
                        SymbolInfo {
                            kind: SymbolKind::Import(*id),
                            span,
                            name_spur,
                        },
                        interner,
                        diags,
                    );
                }
            }
            DeclId::Let(id) => {
                seen_non_import = true;
                let decl = &ast[*id];
                register_symbol(
                    symbols,
                    decl.name,
                    SymbolInfo {
                        kind: SymbolKind::Let(*id),
                        span: decl.span.clone(),
                        name_spur: decl.name,
                    },
                    interner,
                    diags,
                );
            }
            DeclId::Error(_) => {
                // Skip error recovery nodes silently
            }
        }
    }
}

fn register_symbol(
    symbols: &mut SymbolTable,
    name: Spur,
    info: SymbolInfo,
    interner: &Interner,
    diags: &mut DiagnosticCollector,
) {
    let span = info.span.clone();
    if let Err(existing) = symbols.insert(name, info) {
        let name_str = interner.resolve(&name);
        diags.emit(
            Diagnostic::new(
                ErrorCode::Res010,
                format!("duplicate definition of '{name_str}'"),
                span,
                Severity::Error,
                "duplicate definition".to_string(),
            )
            .with_secondary(existing.span.clone(), "first defined here"),
        );
    }
}

fn import_span(decl: &ImportDecl) -> Span {
    match decl {
        ImportDecl::Eaml { span, .. } | ImportDecl::Python { span, .. } => span.clone(),
    }
}

fn import_name(decl: &ImportDecl) -> Option<(Spur, Span)> {
    match decl {
        ImportDecl::Eaml { alias, span, .. } | ImportDecl::Python { alias, span, .. } => {
            alias.map(|a| (a, span.clone()))
        }
    }
}

// =============================================================================
// Pass 2: Resolve references
// =============================================================================

fn pass2_resolve(
    program: &Program,
    ast: &Ast,
    interner: &Interner,
    symbols: &SymbolTable,
    diags: &mut DiagnosticCollector,
) {
    // Track which let bindings are visible (sequential scoping)
    let mut visible_lets: HashSet<Spur> = HashSet::new();

    for decl_id in &program.declarations {
        match decl_id {
            DeclId::Schema(id) => {
                let decl = &ast[*id];
                for field in &decl.fields {
                    resolve_type_expr(field.type_expr, ast, interner, symbols, diags);
                }
            }
            DeclId::Prompt(id) => {
                let decl = &ast[*id];
                resolve_type_expr(decl.return_type, ast, interner, symbols, diags);
                for param in &decl.params {
                    resolve_type_expr(param.type_expr, ast, interner, symbols, diags);
                }
            }
            DeclId::Tool(id) => {
                let decl = &ast[*id];
                resolve_type_expr(decl.return_type, ast, interner, symbols, diags);
                for param in &decl.params {
                    resolve_type_expr(param.type_expr, ast, interner, symbols, diags);
                }
                // Python bridge bodies are opaque -- not validated
            }
            DeclId::Agent(id) => {
                let decl = &ast[*id];
                for field in &decl.fields {
                    match field {
                        AgentField::Model(spur, span) => {
                            resolve_agent_model(*spur, span, interner, symbols, diags);
                        }
                        AgentField::Tools(tools, _) => {
                            for (spur, span) in tools {
                                resolve_agent_tool(*spur, span, interner, symbols, diags);
                            }
                        }
                        _ => {}
                    }
                }
            }
            DeclId::Let(id) => {
                let decl = &ast[*id];
                // Let bindings are sequential: resolve the value expression
                // with only preceding lets visible.
                // For now, resolve the type expression.
                resolve_type_expr(decl.type_expr, ast, interner, symbols, diags);
                // Mark this let as visible for subsequent lets.
                visible_lets.insert(decl.name);
            }
            _ => {}
        }
    }

    // Suppress unused variable warning
    let _ = visible_lets;
}

fn resolve_type_expr(
    id: TypeExprId,
    ast: &Ast,
    interner: &Interner,
    symbols: &SymbolTable,
    diags: &mut DiagnosticCollector,
) {
    match &ast[id] {
        TypeExpr::Named(spur, span) => {
            let name = interner.resolve(spur);
            if !SymbolTable::is_primitive_name(name) && !symbols.is_known_type(*spur) {
                emit_unresolved(name, span.clone(), interner, symbols, diags);
            }
        }
        TypeExpr::Bounded { base, .. } => {
            // Base must be a primitive -- checked at type-checking time
            // For name resolution, just verify the base name is known
            let name = interner.resolve(base);
            if !SymbolTable::is_primitive_name(name) {
                // Bounded types should only have primitive bases
                // but we don't emit an error here -- that's for the type checker
            }
        }
        TypeExpr::Array(inner, _) => {
            resolve_type_expr(*inner, ast, interner, symbols, diags);
        }
        TypeExpr::Optional(inner, _) => {
            resolve_type_expr(*inner, ast, interner, symbols, diags);
        }
        TypeExpr::Grouped(inner, _) => {
            resolve_type_expr(*inner, ast, interner, symbols, diags);
        }
        TypeExpr::LiteralUnion { .. } => {
            // No name resolution needed for literal unions
        }
        TypeExpr::Error(_) => {
            // Skip error recovery nodes
        }
    }
}

fn resolve_agent_model(
    spur: Spur,
    span: &Span,
    interner: &Interner,
    symbols: &SymbolTable,
    diags: &mut DiagnosticCollector,
) {
    match symbols.get(spur) {
        Some(info) => {
            if !matches!(info.kind, SymbolKind::Model(_)) {
                let name = interner.resolve(&spur);
                diags.emit(
                    Diagnostic::new(
                        ErrorCode::Res001,
                        format!("'{name}' is not a model"),
                        span.clone(),
                        Severity::Error,
                        "expected a model".to_string(),
                    )
                    .with_secondary(info.span.clone(), "defined here as non-model"),
                );
            }
        }
        None => {
            let name = interner.resolve(&spur);
            emit_unresolved(name, span.clone(), interner, symbols, diags);
        }
    }
}

fn resolve_agent_tool(
    spur: Spur,
    span: &Span,
    interner: &Interner,
    symbols: &SymbolTable,
    diags: &mut DiagnosticCollector,
) {
    match symbols.get(spur) {
        Some(info) => {
            if !matches!(info.kind, SymbolKind::Tool(_)) {
                let name = interner.resolve(&spur);
                diags.emit(
                    Diagnostic::new(
                        ErrorCode::Res001,
                        format!("'{name}' is not a tool"),
                        span.clone(),
                        Severity::Error,
                        "expected a tool".to_string(),
                    )
                    .with_secondary(info.span.clone(), "defined here as non-tool"),
                );
            }
        }
        None => {
            let name = interner.resolve(&spur);
            emit_unresolved(name, span.clone(), interner, symbols, diags);
        }
    }
}

fn emit_unresolved(
    name: &str,
    span: Span,
    interner: &Interner,
    symbols: &SymbolTable,
    diags: &mut DiagnosticCollector,
) {
    let mut diag = Diagnostic::new(
        ErrorCode::Res001,
        format!("undefined reference to '{name}'"),
        span,
        Severity::Error,
        "not found".to_string(),
    );

    if let Some(suggestion) = suggest_similar(name, symbols, interner) {
        diag = diag.with_hint(format!("did you mean '{suggestion}'?"));
    }

    diags.emit(diag);
}

fn suggest_similar(name: &str, symbols: &SymbolTable, interner: &Interner) -> Option<String> {
    let threshold = 3;
    symbols
        .iter()
        .map(|(spur, _)| {
            let resolved = interner.resolve(spur);
            (strsim::levenshtein(name, resolved), resolved.to_string())
        })
        .filter(|(dist, _)| *dist > 0 && *dist <= threshold)
        .min_by_key(|(dist, _)| *dist)
        .map(|(_, s)| s)
}

// =============================================================================
// Pass 3: Cycle detection
// =============================================================================

#[derive(Clone, Copy, PartialEq)]
enum Color {
    White,
    Gray,
    Black,
}

fn pass3_cycle_detection(
    ast: &Ast,
    interner: &Interner,
    symbols: &SymbolTable,
    diags: &mut DiagnosticCollector,
) {
    // Build adjacency graph: schema Spur -> set of referenced schema Spurs
    let mut graph: HashMap<Spur, Vec<Spur>> = HashMap::new();

    for schema in &ast.schemas {
        let mut refs = Vec::new();
        for field in &schema.fields {
            collect_schema_refs(field.type_expr, ast, symbols, &mut refs);
        }
        graph.insert(schema.name, refs);
    }

    // DFS cycle detection
    let mut color: HashMap<Spur, Color> = HashMap::new();
    for schema in &ast.schemas {
        color.insert(schema.name, Color::White);
    }

    let mut reported: HashSet<Spur> = HashSet::new();

    for schema in &ast.schemas {
        if color[&schema.name] == Color::White {
            let mut path = Vec::new();
            dfs(
                schema.name,
                &graph,
                &mut color,
                &mut path,
                interner,
                symbols,
                diags,
                &mut reported,
            );
        }
    }
}

fn collect_schema_refs(
    type_id: TypeExprId,
    ast: &Ast,
    symbols: &SymbolTable,
    refs: &mut Vec<Spur>,
) {
    match &ast[type_id] {
        TypeExpr::Named(spur, _) => {
            if let Some(info) = symbols.get(*spur) {
                if matches!(info.kind, SymbolKind::Schema(_)) {
                    refs.push(*spur);
                }
            }
        }
        TypeExpr::Array(inner, _) | TypeExpr::Optional(inner, _) | TypeExpr::Grouped(inner, _) => {
            collect_schema_refs(*inner, ast, symbols, refs);
        }
        _ => {}
    }
}

#[allow(clippy::too_many_arguments)]
fn dfs(
    node: Spur,
    graph: &HashMap<Spur, Vec<Spur>>,
    color: &mut HashMap<Spur, Color>,
    path: &mut Vec<Spur>,
    interner: &Interner,
    symbols: &SymbolTable,
    diags: &mut DiagnosticCollector,
    reported: &mut HashSet<Spur>,
) {
    color.insert(node, Color::Gray);
    path.push(node);

    if let Some(neighbors) = graph.get(&node) {
        for &neighbor in neighbors {
            match color.get(&neighbor) {
                Some(Color::Gray) => {
                    // Found a cycle -- emit SEM070 for the node that starts the cycle
                    if !reported.contains(&neighbor) {
                        reported.insert(neighbor);
                        let name = interner.resolve(&neighbor);
                        let cycle_start = path.iter().position(|s| *s == neighbor).unwrap();
                        let cycle_path: Vec<String> = path[cycle_start..]
                            .iter()
                            .map(|s| interner.resolve(s).to_string())
                            .collect();
                        let cycle_str = format!("{} -> {}", cycle_path.join(" -> "), cycle_path[0]);

                        let span = symbols
                            .get(neighbor)
                            .map(|i| i.span.clone())
                            .unwrap_or(0..0);

                        diags.emit(
                            Diagnostic::new(
                                ErrorCode::Sem070,
                                format!("schema '{name}' contains a recursive type reference. This is allowed but may cause issues with deeply nested data."),
                                span,
                                Severity::Warning,
                                "recursive reference".to_string(),
                            )
                            .with_hint(format!("cycle: {cycle_str}")),
                        );
                    }
                }
                Some(Color::White) | None => {
                    // If neighbor is a schema in the graph, visit it
                    if color.contains_key(&neighbor) {
                        dfs(
                            neighbor, graph, color, path, interner, symbols, diags, reported,
                        );
                    }
                }
                Some(Color::Black) => {
                    // Already fully visited, no cycle through this path
                }
            }
        }
    }

    path.pop();
    color.insert(node, Color::Black);
}
