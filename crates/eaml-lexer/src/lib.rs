//! EAML lexer -- tokenizes EAML source into a token stream.
//!
//! Public API: [`lex()`] function, [`Token`], [`TokenKind`], [`Span`], [`Interner`].

pub mod intern;
pub mod lexer;
pub(crate) mod logos_lexer;
pub mod token;

pub use intern::Interner;
pub use lexer::{lex, LexOutput};
pub use token::{Span, Token, TokenKind};
