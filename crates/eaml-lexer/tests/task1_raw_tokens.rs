//! Tests for the Interner public API (Task 1).

use eaml_lexer::intern::Interner;
use eaml_lexer::token::TokenKind;

// === Interner tests ===

#[test]
fn interner_returns_same_key_for_same_string() {
    let mut interner = Interner::new();
    let k1 = interner.intern("foo");
    let k2 = interner.intern("foo");
    assert_eq!(k1, k2);
}

#[test]
fn interner_returns_different_keys_for_different_strings() {
    let mut interner = Interner::new();
    let k1 = interner.intern("foo");
    let k2 = interner.intern("bar");
    assert_ne!(k1, k2);
}

#[test]
fn interner_resolves_to_original_string() {
    let mut interner = Interner::new();
    let key = interner.intern("hello");
    assert_eq!(interner.resolve(key), "hello");
}

#[test]
fn token_kind_ident_round_trips_through_interner() {
    let mut interner = Interner::new();
    let key = interner.intern("myVar");
    let kind = TokenKind::Ident(key);
    match kind {
        TokenKind::Ident(k) => assert_eq!(interner.resolve(k), "myVar"),
        _ => panic!("expected Ident"),
    }
}
