//! Type checking pass for EAML semantic analysis.
//!
//! Validates bounded types, literal unions, composite type modifiers,
//! schema field types, prompt/tool structure, and template variable scoping.

use std::collections::{HashMap, HashSet};

use lasso::Spur;

use eaml_errors::{Diagnostic, DiagnosticCollector, ErrorCode, Severity, Span};
use eaml_lexer::Interner;
use eaml_parser::ast::*;

use crate::scope::Scope;
use crate::symbol_table::{SymbolKind, SymbolTable};

/// Known EAML provider strings.
const KNOWN_PROVIDERS: &[&str] = &["anthropic", "openai", "ollama"];

/// Resolved type annotation for a type expression.
#[derive(Debug, Clone)]
pub enum ResolvedType {
    Primitive(String),
    Schema(SchemaDeclId),
    Array(Box<ResolvedType>),
    Optional(Box<ResolvedType>),
    LiteralUnion(Vec<String>),
    Error,
}

/// Type annotations accumulated during type checking.
pub struct TypeAnnotations {
    pub type_exprs: HashMap<TypeExprId, ResolvedType>,
}

/// Runs the type checking pass over the AST.
///
/// This validates bounded types (TYP030/031/032), literal unions (TYP040),
/// schema fields (SEM020), prompts (SEM025), tools (SEM040), chained
/// comparisons (SEM060), type shadowing (TYP001), and unknown providers (PYB010).
///
/// All composite type modifier orderings (T[], T[]?, T?[], T?[]?) are legal
/// per spec/TYPESYSTEM.md TS-COMP-01 through TS-COMP-04. The type checker
/// simply recurses through Array and Optional nodes without rejecting any
/// ordering. Nested arrays (T[][]) are rejected by the parser, not here.
pub fn check(
    program: &Program,
    ast: &Ast,
    interner: &Interner,
    symbols: &SymbolTable,
    source: &str,
    diags: &mut DiagnosticCollector,
) -> TypeAnnotations {
    let mut annotations = TypeAnnotations {
        type_exprs: HashMap::new(),
    };

    for decl_id in &program.declarations {
        match decl_id {
            DeclId::Model(id) => {
                check_model(&ast[*id], source, diags);
            }
            DeclId::Schema(id) => {
                check_schema(
                    &ast[*id],
                    ast,
                    interner,
                    symbols,
                    source,
                    diags,
                    &mut annotations,
                );
            }
            DeclId::Prompt(id) => {
                check_prompt(
                    &ast[*id],
                    ast,
                    interner,
                    symbols,
                    source,
                    diags,
                    &mut annotations,
                );
            }
            DeclId::Tool(id) => {
                check_tool(
                    &ast[*id],
                    ast,
                    interner,
                    symbols,
                    source,
                    diags,
                    &mut annotations,
                );
            }
            DeclId::Let(id) => {
                let decl = &ast[*id];
                // Check type expression
                check_type_expr(
                    decl.type_expr,
                    ast,
                    interner,
                    symbols,
                    source,
                    diags,
                    &mut annotations,
                );
                // Check value expression (e.g., for chained comparisons)
                let scope = Scope::new();
                check_expr(decl.value, ast, interner, symbols, diags, &scope);
            }
            _ => {}
        }
    }

    annotations
}

/// Builds a scope with parameters and top-level let bindings.
fn build_param_scope(params: &[Param], symbols: &SymbolTable) -> Scope {
    let mut scope = Scope::new();
    for param in params {
        scope.insert(param.name, param.span.clone());
    }
    for (spur, info) in symbols.iter() {
        if matches!(info.kind, SymbolKind::Let(_)) {
            scope.insert(*spur, info.span.clone());
        }
    }
    scope
}

// =============================================================================
// Type expression validation
// =============================================================================

fn check_type_expr(
    id: TypeExprId,
    ast: &Ast,
    interner: &Interner,
    symbols: &SymbolTable,
    source: &str,
    diags: &mut DiagnosticCollector,
    annotations: &mut TypeAnnotations,
) -> ResolvedType {
    let resolved = match &ast[id] {
        TypeExpr::Named(spur, _span) => {
            let name = interner.resolve(spur);
            if symbols.is_primitive(*spur) {
                ResolvedType::Primitive(name.to_string())
            } else if let Some(info) = symbols.get(*spur) {
                if let SymbolKind::Schema(schema_id) = info.kind {
                    ResolvedType::Schema(schema_id)
                } else {
                    // Referenced a non-type symbol -- resolver already emitted error
                    ResolvedType::Error
                }
            } else if SymbolTable::is_primitive_name(name) {
                // Primitive name known statically but not interned in symbol table
                ResolvedType::Primitive(name.to_string())
            } else {
                // Unknown type -- resolver already emitted RES001
                ResolvedType::Error
            }
        }
        TypeExpr::Bounded { base, params, span } => {
            check_bounded_type(*base, params, span, interner, source, diags)
        }
        TypeExpr::Array(inner, _) => {
            // All composite modifier orderings are legal per spec -- just recurse
            let inner_resolved =
                check_type_expr(*inner, ast, interner, symbols, source, diags, annotations);
            ResolvedType::Array(Box::new(inner_resolved))
        }
        TypeExpr::Optional(inner, _) => {
            // All composite modifier orderings are legal per spec -- just recurse
            let inner_resolved =
                check_type_expr(*inner, ast, interner, symbols, source, diags, annotations);
            ResolvedType::Optional(Box::new(inner_resolved))
        }
        TypeExpr::LiteralUnion { members, .. } => check_literal_union(members, source, diags),
        TypeExpr::Grouped(inner, _) => {
            check_type_expr(*inner, ast, interner, symbols, source, diags, annotations)
        }
        TypeExpr::Error(_) => ResolvedType::Error,
    };

    annotations.type_exprs.insert(id, resolved.clone());
    resolved
}

fn check_bounded_type(
    base: Spur,
    params: &[BoundParam],
    span: &Span,
    interner: &Interner,
    source: &str,
    diags: &mut DiagnosticCollector,
) -> ResolvedType {
    let base_name = interner.resolve(&base);

    // Only string, int, float accept bounded parameters
    match base_name {
        "string" | "int" | "float" => {}
        _ => {
            diags.emit(Diagnostic::new(
                ErrorCode::Typ032,
                format!("type '{base_name}' does not accept bounded parameters"),
                span.clone(),
                Severity::Error,
                "not boundable".to_string(),
            ));
            return ResolvedType::Error;
        }
    }

    // Determine valid param names based on base type
    let valid_names: &[&str] = if base_name == "string" {
        &["minLen", "maxLen", "min", "max"]
    } else {
        &["min", "max"]
    };

    // Check for named vs positional params and normalize
    let all_named = params.iter().all(|p| p.name.is_some());
    let all_positional = params.iter().all(|p| p.name.is_none());

    let mut min_val: Option<f64> = None;
    let mut max_val: Option<f64> = None;

    if all_positional && params.len() >= 2 {
        // Positional: first=min, second=max
        if let Ok(v) = source[params[0].value_span.clone()].parse::<f64>() {
            min_val = Some(v);
        }
        if let Ok(v) = source[params[1].value_span.clone()].parse::<f64>() {
            max_val = Some(v);
        }
    } else if all_named || !all_positional {
        // Named params
        for param in params {
            if let Some(name_spur) = param.name {
                let param_name = interner.resolve(&name_spur);
                if !valid_names.contains(&param_name) {
                    diags.emit(Diagnostic::new(
                        ErrorCode::Sem030,
                        format!("unknown bounded parameter name '{param_name}'"),
                        param.span.clone(),
                        Severity::Error,
                        "unknown parameter".to_string(),
                    ));
                    continue;
                }
                if let Ok(v) = source[param.value_span.clone()].parse::<f64>() {
                    match param_name {
                        "min" | "minLen" => min_val = Some(v),
                        "max" | "maxLen" => max_val = Some(v),
                        _ => {}
                    }
                }
            }
        }
    } else if all_positional && params.len() == 1 {
        // Single positional param -- treat as max
        if let Ok(v) = source[params[0].value_span.clone()].parse::<f64>() {
            max_val = Some(v);
        }
    }

    // Validate bounds
    if let (Some(min), Some(max)) = (min_val, max_val) {
        if min > max {
            diags.emit(Diagnostic::new(
                ErrorCode::Typ030,
                format!("lower bound ({min}) exceeds upper bound ({max})"),
                span.clone(),
                Severity::Error,
                "invalid bounds".to_string(),
            ));
        }
    }

    // String-specific: check for negative bounds
    if base_name == "string" {
        for val in [min_val, max_val].iter().flatten() {
            if *val < 0.0 {
                diags.emit(Diagnostic::new(
                    ErrorCode::Typ031,
                    "invalid string length bound: must be non-negative".to_string(),
                    span.clone(),
                    Severity::Error,
                    "negative bound".to_string(),
                ));
                break;
            }
        }
    }

    ResolvedType::Primitive(base_name.to_string())
}

fn check_literal_union(
    members: &[Span],
    source: &str,
    diags: &mut DiagnosticCollector,
) -> ResolvedType {
    let mut seen = HashSet::new();
    let mut values = Vec::new();

    for member_span in members {
        let raw = &source[member_span.clone()];
        // Strip surrounding quotes
        let value = raw.trim_matches('"').to_string();
        if !seen.insert(value.clone()) {
            diags.emit(Diagnostic::new(
                ErrorCode::Typ040,
                format!("duplicate literal union member '{value}'"),
                member_span.clone(),
                Severity::Warning,
                "duplicate".to_string(),
            ));
        }
        values.push(value);
    }

    ResolvedType::LiteralUnion(values)
}

// =============================================================================
// Schema validation
// =============================================================================

fn check_schema(
    schema: &SchemaDecl,
    ast: &Ast,
    interner: &Interner,
    symbols: &SymbolTable,
    source: &str,
    diags: &mut DiagnosticCollector,
    annotations: &mut TypeAnnotations,
) {
    let schema_name = interner.resolve(&schema.name);

    // TYP001: schema name shadows built-in type
    if SymbolTable::is_primitive_name(schema_name) {
        diags.emit(Diagnostic::new(
            ErrorCode::Typ001,
            format!("schema '{schema_name}' shadows built-in type"),
            schema.span.clone(),
            Severity::Warning,
            "shadows built-in".to_string(),
        ));
    }

    // SEM020: duplicate field names within schema
    let mut field_names: HashMap<Spur, Span> = HashMap::new();
    for field in &schema.fields {
        if let Some(first_span) = field_names.get(&field.name) {
            let field_name = interner.resolve(&field.name);
            diags.emit(
                Diagnostic::new(
                    ErrorCode::Sem020,
                    format!("duplicate field name '{field_name}' in schema '{schema_name}'"),
                    field.span.clone(),
                    Severity::Error,
                    "duplicate field".to_string(),
                )
                .with_secondary(first_span.clone(), "first defined here"),
            );
        } else {
            field_names.insert(field.name, field.span.clone());
        }

        // Validate the field's type expression
        check_type_expr(
            field.type_expr,
            ast,
            interner,
            symbols,
            source,
            diags,
            annotations,
        );
    }
}

// =============================================================================
// Prompt validation
// =============================================================================

fn check_prompt(
    prompt: &PromptDecl,
    ast: &Ast,
    interner: &Interner,
    symbols: &SymbolTable,
    source: &str,
    diags: &mut DiagnosticCollector,
    annotations: &mut TypeAnnotations,
) {
    // Validate return type
    check_type_expr(
        prompt.return_type,
        ast,
        interner,
        symbols,
        source,
        diags,
        annotations,
    );

    // Validate parameter types
    for param in &prompt.params {
        check_type_expr(
            param.type_expr,
            ast,
            interner,
            symbols,
            source,
            diags,
            annotations,
        );
    }

    // SEM025: prompt body must have at least one user: field
    let has_user = prompt
        .body
        .fields
        .iter()
        .any(|f| matches!(f, PromptField::User(_)));
    if !has_user {
        diags.emit(Diagnostic::new(
            ErrorCode::Sem025,
            "prompt body missing required 'user:' field".to_string(),
            prompt.body.span.clone(),
            Severity::Error,
            "missing user field".to_string(),
        ));
    }

    // Build scope and validate template interpolation variables in prompt body
    let scope = build_param_scope(&prompt.params, symbols);
    for field in &prompt.body.fields {
        match field {
            PromptField::User(ts) | PromptField::System(ts) => {
                check_template_string(ts, &scope, ast, interner, symbols, diags);
            }
            _ => {}
        }
    }
}

// =============================================================================
// Tool validation
// =============================================================================

fn check_tool(
    tool: &ToolDecl,
    ast: &Ast,
    interner: &Interner,
    symbols: &SymbolTable,
    source: &str,
    diags: &mut DiagnosticCollector,
    annotations: &mut TypeAnnotations,
) {
    // Validate return type
    check_type_expr(
        tool.return_type,
        ast,
        interner,
        symbols,
        source,
        diags,
        annotations,
    );

    // Validate parameter types
    for param in &tool.params {
        check_type_expr(
            param.type_expr,
            ast,
            interner,
            symbols,
            source,
            diags,
            annotations,
        );
    }

    // SEM040: tool body must have implementation
    if matches!(tool.body, ToolBody::Empty(_)) {
        diags.emit(Diagnostic::new(
            ErrorCode::Sem040,
            "tool body has no implementation".to_string(),
            tool.span.clone(),
            Severity::Error,
            "empty tool body".to_string(),
        ));
    }

    // For native bodies, check expressions (including chained comparisons)
    if let ToolBody::Native { stmts, .. } = &tool.body {
        let scope = build_param_scope(&tool.params, symbols);
        for stmt_id in stmts {
            check_expr(*stmt_id, ast, interner, symbols, diags, &scope);
        }
    }

    // Python bridge bodies are OPAQUE -- do NOT validate
}

// =============================================================================
// Model validation
// =============================================================================

fn check_model(model: &ModelDecl, source: &str, diags: &mut DiagnosticCollector) {
    // PYB010: warn for unknown provider string
    // Extract provider from template string -- check the first text part
    let provider_text = extract_template_text(&model.provider, source);
    if let Some(provider) = provider_text {
        if !KNOWN_PROVIDERS.contains(&provider.as_str()) {
            diags.emit(Diagnostic::new(
                ErrorCode::Pyb010,
                format!("unknown provider '{provider}'"),
                model.provider.span.clone(),
                Severity::Warning,
                "unknown provider".to_string(),
            ));
        }
    }
}

/// Extract the full text content of a template string (ignoring interpolations).
fn extract_template_text(ts: &TemplateString, source: &str) -> Option<String> {
    let mut text = String::new();
    for part in &ts.parts {
        if let TemplatePart::Text(span) = part {
            text.push_str(&source[span.clone()]);
        }
    }
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

// =============================================================================
// Expression validation
// =============================================================================

#[allow(clippy::only_used_in_recursion)]
fn check_expr(
    expr_id: ExprId,
    ast: &Ast,
    interner: &Interner,
    symbols: &SymbolTable,
    diags: &mut DiagnosticCollector,
    scope: &Scope,
) {
    match &ast[expr_id] {
        Expr::BinaryOp {
            left,
            op,
            right,
            span,
        } => {
            if is_comparison(*op) {
                // Check for chained comparisons: if left or right is also a comparison
                if is_comparison_expr(*left, ast) || is_comparison_expr(*right, ast) {
                    diags.emit(
                        Diagnostic::new(
                            ErrorCode::Sem060,
                            "chained comparison detected".to_string(),
                            span.clone(),
                            Severity::Error,
                            "chained comparison".to_string(),
                        )
                        .with_hint("use explicit grouping: (a == b) && (b == c)".to_string()),
                    );
                }
            }
            check_expr(*left, ast, interner, symbols, diags, scope);
            check_expr(*right, ast, interner, symbols, diags, scope);
        }
        Expr::UnaryOp { operand, .. } => {
            check_expr(*operand, ast, interner, symbols, diags, scope);
        }
        Expr::FieldAccess { object, .. } => {
            check_expr(*object, ast, interner, symbols, diags, scope);
        }
        Expr::FnCall { callee, args, .. } => {
            check_expr(*callee, ast, interner, symbols, diags, scope);
            for arg in args {
                check_expr(arg.value, ast, interner, symbols, diags, scope);
            }
        }
        Expr::Index { object, index, .. } => {
            check_expr(*object, ast, interner, symbols, diags, scope);
            check_expr(*index, ast, interner, symbols, diags, scope);
        }
        Expr::Paren { inner, .. } => {
            check_expr(*inner, ast, interner, symbols, diags, scope);
        }
        Expr::If {
            condition,
            then_block,
            else_block,
            ..
        } => {
            check_expr(*condition, ast, interner, symbols, diags, scope);
            for e in then_block {
                check_expr(*e, ast, interner, symbols, diags, scope);
            }
            if let Some(elses) = else_block {
                for e in elses {
                    check_expr(*e, ast, interner, symbols, diags, scope);
                }
            }
        }
        Expr::Return { value, .. } => {
            if let Some(v) = value {
                check_expr(*v, ast, interner, symbols, diags, scope);
            }
        }
        Expr::Await { operand, .. } => {
            check_expr(*operand, ast, interner, symbols, diags, scope);
        }
        Expr::Error(_)
        | Expr::IntLit(_)
        | Expr::FloatLit(_)
        | Expr::BoolLit(_, _)
        | Expr::NullLit(_)
        | Expr::Ident(_, _)
        | Expr::StringLit(_)
        | Expr::TemplateStr(_)
        | Expr::Let { .. } => {}
    }
}

fn is_comparison(op: BinOp) -> bool {
    matches!(
        op,
        BinOp::Eq | BinOp::NotEq | BinOp::Lt | BinOp::Gt | BinOp::LtEq | BinOp::GtEq
    )
}

fn is_comparison_expr(expr_id: ExprId, ast: &Ast) -> bool {
    matches!(
        &ast[expr_id],
        Expr::BinaryOp { op, .. } if is_comparison(*op)
    )
}

// =============================================================================
// Template string validation
// =============================================================================

fn check_template_string(
    ts: &TemplateString,
    scope: &Scope,
    ast: &Ast,
    interner: &Interner,
    symbols: &SymbolTable,
    diags: &mut DiagnosticCollector,
) {
    for part in &ts.parts {
        if let TemplatePart::Interpolation(expr_id, _span) = part {
            check_template_expr(*expr_id, scope, ast, interner, symbols, diags);
        }
    }
}

fn check_template_expr(
    expr_id: ExprId,
    scope: &Scope,
    ast: &Ast,
    interner: &Interner,
    symbols: &SymbolTable,
    diags: &mut DiagnosticCollector,
) {
    match &ast[expr_id] {
        Expr::Ident(spur, span) => {
            // Check if the variable is in scope (params + lets) or is a top-level declaration
            if !scope.contains(*spur) && symbols.get(*spur).is_none() {
                let name = interner.resolve(spur);
                diags.emit(
                    Diagnostic::new(
                        ErrorCode::Res001,
                        format!("undefined variable '{name}' in template interpolation"),
                        span.clone(),
                        Severity::Error,
                        "not in scope".to_string(),
                    )
                    .with_hint("template interpolation can reference parameters and let bindings"),
                );
            }
        }
        Expr::FieldAccess { object, .. } => {
            // Validate the root object is in scope; don't validate field names (runtime)
            check_template_expr(*object, scope, ast, interner, symbols, diags);
        }
        Expr::BinaryOp { left, right, .. } => {
            check_template_expr(*left, scope, ast, interner, symbols, diags);
            check_template_expr(*right, scope, ast, interner, symbols, diags);
        }
        Expr::FnCall { callee, args, .. } => {
            check_template_expr(*callee, scope, ast, interner, symbols, diags);
            for arg in args {
                check_template_expr(arg.value, scope, ast, interner, symbols, diags);
            }
        }
        Expr::Paren { inner, .. } => {
            check_template_expr(*inner, scope, ast, interner, symbols, diags);
        }
        _ => {}
    }
}
