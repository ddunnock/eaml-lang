//! Symbol table for EAML semantic analysis.
//!
//! Stores all top-level declarations and primitive types, supporting
//! lookup, duplicate detection, and iteration for suggestions.

use eaml_errors::Span;
use eaml_lexer::Interner;
use eaml_parser::ast::*;
use std::collections::{HashMap, HashSet};

use lasso::Spur;

/// The kind of a symbol in the symbol table.
#[derive(Debug, Clone)]
pub enum SymbolKind {
    Model(ModelDeclId),
    Schema(SchemaDeclId),
    Prompt(PromptDeclId),
    Tool(ToolDeclId),
    Agent(AgentDeclId),
    Import(ImportDeclId),
    Let(LetDeclId),
}

/// Information about a declared symbol.
#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub kind: SymbolKind,
    pub span: Span,
    pub name_spur: Spur,
}

/// Primitive type names in EAML.
const PRIMITIVE_NAMES: &[&str] = &["string", "int", "float", "bool", "null"];

/// The symbol table holding all top-level declarations and primitive types.
pub struct SymbolTable {
    declarations: HashMap<Spur, SymbolInfo>,
    primitives: HashSet<Spur>,
}

impl SymbolTable {
    /// Creates a new symbol table with primitive types pre-populated.
    ///
    /// Only primitives that are already interned (i.e., referenced in the source)
    /// are added. This uses `Interner::get` to avoid mutating the interner.
    pub fn new(interner: &Interner) -> Self {
        let mut primitives = HashSet::new();
        for name in PRIMITIVE_NAMES {
            if let Some(spur) = interner.get(name) {
                primitives.insert(spur);
            }
        }
        Self {
            declarations: HashMap::new(),
            primitives,
        }
    }

    /// Inserts a symbol. Returns `Err` with the existing info if a duplicate.
    pub fn insert(&mut self, name: Spur, info: SymbolInfo) -> Result<(), &SymbolInfo> {
        if self.declarations.contains_key(&name) {
            return Err(&self.declarations[&name]);
        }
        self.declarations.insert(name, info);
        Ok(())
    }

    /// Looks up a symbol by its interned name.
    pub fn get(&self, name: Spur) -> Option<&SymbolInfo> {
        self.declarations.get(&name)
    }

    /// Returns true if the name is a primitive type.
    pub fn is_primitive(&self, name: Spur) -> bool {
        self.primitives.contains(&name)
    }

    /// Returns true if the name matches a primitive type name string.
    ///
    /// This is used when the interner might not have the primitive interned yet.
    pub fn is_primitive_name(name: &str) -> bool {
        PRIMITIVE_NAMES.contains(&name)
    }

    /// Returns true if the name is a known type (primitive or schema declaration).
    pub fn is_known_type(&self, name: Spur) -> bool {
        if self.is_primitive(name) {
            return true;
        }
        matches!(
            self.declarations.get(&name),
            Some(SymbolInfo {
                kind: SymbolKind::Schema(_),
                ..
            })
        )
    }

    /// Iterates over all declarations for suggestion lookups.
    pub fn iter(&self) -> impl Iterator<Item = (&Spur, &SymbolInfo)> {
        self.declarations.iter()
    }
}
