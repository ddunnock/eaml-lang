//! Snapshot tests for full example .eaml file generation.

mod test_helpers;

#[test]
fn test_generate_minimal() {
    let source = include_str!("../../../examples/01-minimal/minimal.eaml");
    let output = test_helpers::generate_from_source_with_name(source, "minimal.eaml");
    insta::assert_snapshot!(output);
}

#[test]
fn test_generate_sentiment() {
    let source = include_str!("../../../examples/02-sentiment/sentiment.eaml");
    let output = test_helpers::generate_from_source_with_name(source, "sentiment.eaml");
    insta::assert_snapshot!(output);
}

#[test]
fn test_generate_all_type_variants() {
    let source = include_str!("../../../examples/07-all-type-variants/types.eaml");
    let output = test_helpers::generate_from_source_with_name(source, "types.eaml");
    insta::assert_snapshot!(output);
}
