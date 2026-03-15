//! String interning for the EAML lexer.
//!
//! Wraps [`lasso::Rodeo`] to provide identifier interning with
//! compact [`Spur`] keys.

use lasso::{Rodeo, Spur};

/// String interner backed by lasso's [`Rodeo`].
///
/// Identifiers are interned during lexing so that repeated occurrences
/// of the same name share a single [`Spur`] key.
pub struct Interner {
    rodeo: Rodeo,
}

impl Interner {
    /// Creates a new empty interner.
    pub fn new() -> Self {
        Self {
            rodeo: Rodeo::default(),
        }
    }

    /// Interns a string, returning its [`Spur`] key.
    ///
    /// If the string was previously interned, returns the same key.
    pub fn intern(&mut self, s: &str) -> Spur {
        self.rodeo.get_or_intern(s)
    }

    /// Resolves a [`Spur`] key back to the original string.
    pub fn resolve(&self, key: &Spur) -> &str {
        self.rodeo.resolve(key)
    }
}

impl Default for Interner {
    fn default() -> Self {
        Self::new()
    }
}
