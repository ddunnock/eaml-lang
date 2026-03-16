//! Local scope tracking for let bindings and parameters within prompt/tool bodies.

use eaml_errors::Span;
use lasso::Spur;
use std::collections::HashMap;

/// A local scope for tracking parameters and let bindings.
pub struct Scope {
    locals: HashMap<Spur, Span>,
}

impl Scope {
    /// Creates a new empty scope.
    pub fn new() -> Self {
        Self {
            locals: HashMap::new(),
        }
    }

    /// Inserts a local binding.
    pub fn insert(&mut self, name: Spur, span: Span) {
        self.locals.insert(name, span);
    }

    /// Gets the span of a local binding.
    pub fn get(&self, name: Spur) -> Option<&Span> {
        self.locals.get(&name)
    }

    /// Returns true if the scope contains the given name.
    pub fn contains(&self, name: Spur) -> bool {
        self.locals.contains_key(&name)
    }
}

impl Default for Scope {
    fn default() -> Self {
        Self::new()
    }
}
