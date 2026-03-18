//! Capability checking pass for EAML semantic analysis.
//!
//! Validates model capability declarations and prompt requires clauses:
//! - CAP001: unknown capability name (warning)
//! - CAP002: duplicate capability in same declaration (warning)
//! - CAP010: prompt requires capabilities the model does not declare (FATAL)
//! - CAP020: json_mode with string return type (warning)

use std::collections::HashSet;

use lasso::Spur;

use eaml_errors::{Diagnostic, DiagnosticCollector, ErrorCode, Severity, Span};
use eaml_lexer::Interner;
use eaml_parser::ast::*;

use crate::type_checker::{ResolvedType, TypeAnnotations};

/// Known capabilities for EAML v0.1.
const KNOWN_CAPABILITIES: &[&str] = &[
    "json_mode",
    "streaming",
    "function_calling",
    "vision",
    "code_interpreter",
];

/// Runs the capability checking pass.
///
/// This validates:
/// 1. Individual capability declarations (unknown caps, duplicates)
/// 2. Capability subset checking (prompt requires vs model caps)
/// 3. json_mode + string return type warning
pub fn check(
    program: &Program,
    ast: &Ast,
    interner: &Interner,
    type_annotations: &TypeAnnotations,
    diags: &mut DiagnosticCollector,
) {
    // Step 1: Validate individual capability declarations
    validate_cap_declarations(program, ast, interner, diags);

    // Step 2: Capability subset checking
    check_capability_subsets(program, ast, interner, diags);

    // Step 3: json_mode + string return type check
    check_json_mode_string_return(program, ast, interner, type_annotations, diags);
}

// =============================================================================
// Step 1: Validate individual cap declarations
// =============================================================================

fn validate_cap_declarations(
    program: &Program,
    ast: &Ast,
    interner: &Interner,
    diags: &mut DiagnosticCollector,
) {
    for decl_id in &program.declarations {
        match decl_id {
            DeclId::Model(id) => {
                let model = &ast[*id];
                validate_cap_list(&model.caps, interner, diags);
            }
            DeclId::Prompt(id) => {
                let prompt = &ast[*id];
                if let Some(ref requires) = prompt.requires {
                    validate_cap_list(&requires.caps, interner, diags);
                }
            }
            _ => {}
        }
    }
}

/// Validates a list of capabilities for unknown names and duplicates.
fn validate_cap_list(caps: &[(Spur, Span)], interner: &Interner, diags: &mut DiagnosticCollector) {
    let mut seen: HashSet<Spur> = HashSet::new();

    for (spur, span) in caps {
        let name = interner.resolve(*spur);

        // CAP001: unknown capability
        if !KNOWN_CAPABILITIES.contains(&name) {
            diags.emit(Diagnostic::new(
                ErrorCode::Cap001,
                format!("unknown capability '{name}'"),
                span.clone(),
                Severity::Warning,
                "unknown capability".to_string(),
            ));
        }

        // CAP002: duplicate capability
        if !seen.insert(*spur) {
            diags.emit(Diagnostic::new(
                ErrorCode::Cap002,
                format!("duplicate capability '{name}'"),
                span.clone(),
                Severity::Warning,
                "duplicate".to_string(),
            ));
        }
    }
}

// =============================================================================
// Step 2: Capability subset checking
// =============================================================================

fn check_capability_subsets(
    program: &Program,
    ast: &Ast,
    interner: &Interner,
    diags: &mut DiagnosticCollector,
) {
    // Collect all models and their capability sets
    let models: Vec<(&ModelDecl, HashSet<Spur>)> = ast
        .models
        .iter()
        .map(|m| {
            let cap_set: HashSet<Spur> = m.caps.iter().map(|(s, _)| *s).collect();
            (m, cap_set)
        })
        .collect();

    if models.is_empty() {
        return;
    }

    // Collect model spurs referenced by agents
    let mut agent_model_spurs: Vec<Spur> = Vec::new();
    for decl_id in &program.declarations {
        if let DeclId::Agent(id) = decl_id {
            let agent = &ast[*id];
            for field in &agent.fields {
                if let AgentField::Model(model_spur, _) = field {
                    agent_model_spurs.push(*model_spur);
                }
            }
        }
    }

    // For each prompt with requires, check against models
    for decl_id in &program.declarations {
        if let DeclId::Prompt(id) = decl_id {
            let prompt = &ast[*id];
            if let Some(ref requires) = prompt.requires {
                let required_caps: HashSet<Spur> = requires.caps.iter().map(|(s, _)| *s).collect();

                if required_caps.is_empty() {
                    continue;
                }

                // Check against agent-referenced models, or all models if no agents
                let effective_models: Vec<&(&ModelDecl, HashSet<Spur>)> =
                    if agent_model_spurs.is_empty() {
                        models.iter().collect()
                    } else {
                        models
                            .iter()
                            .filter(|(m, _)| agent_model_spurs.contains(&m.name))
                            .collect()
                    };

                for (model, model_caps) in effective_models {
                    let missing: Vec<Spur> = required_caps
                        .iter()
                        .filter(|cap| !model_caps.contains(cap))
                        .copied()
                        .collect();

                    if !missing.is_empty() {
                        let model_name = interner.resolve(model.name);
                        let required_str: Vec<&str> = requires
                            .caps
                            .iter()
                            .map(|(s, _)| interner.resolve(*s))
                            .collect();
                        let provided_str: Vec<&str> = model
                            .caps
                            .iter()
                            .map(|(s, _)| interner.resolve(*s))
                            .collect();
                        let missing_str: Vec<&str> =
                            missing.iter().map(|s| interner.resolve(*s)).collect();

                        diags.emit(
                            Diagnostic::new(
                                ErrorCode::Cap010,
                                format!(
                                    "model '{}' is missing required capabilities. Required: [{}]. Provided: [{}]. Missing: [{}]",
                                    model_name,
                                    required_str.join(", "),
                                    provided_str.join(", "),
                                    missing_str.join(", "),
                                ),
                                requires.span.clone(),
                                Severity::Fatal,
                                "required here".to_string(),
                            )
                            .with_secondary(model.span.clone(), "model declared here"),
                        );
                    }
                }
            }
        }
    }
}

// =============================================================================
// Step 3: json_mode + string return type
// =============================================================================

fn check_json_mode_string_return(
    program: &Program,
    ast: &Ast,
    interner: &Interner,
    type_annotations: &TypeAnnotations,
    diags: &mut DiagnosticCollector,
) {
    for decl_id in &program.declarations {
        if let DeclId::Prompt(id) = decl_id {
            let prompt = &ast[*id];
            if let Some(ref requires) = prompt.requires {
                // Check if json_mode is in the requires list
                let has_json_mode = requires.caps.iter().any(|(spur, _)| {
                    let name = interner.resolve(*spur);
                    name == "json_mode"
                });

                if has_json_mode {
                    // Check if return type is string
                    if let Some(resolved) = type_annotations.type_exprs.get(&prompt.return_type) {
                        if matches!(resolved, ResolvedType::Primitive(name) if name == "string") {
                            diags.emit(Diagnostic::new(
                                ErrorCode::Cap020,
                                "json_mode is typically used with structured return types, not string".to_string(),
                                ast[prompt.return_type].span().clone(),
                                Severity::Warning,
                                "string return type".to_string(),
                            ));
                        }
                    }
                }
            }
        }
    }
}
