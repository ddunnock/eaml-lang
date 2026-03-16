use eaml_codegen::names::{to_snake_case, to_upper_snake_case};

#[test]
fn pascal_case_to_snake() {
    assert_eq!(to_snake_case("AnalyzeSentiment"), "analyze_sentiment");
}

#[test]
fn camel_case_to_snake() {
    assert_eq!(to_snake_case("fetchPage"), "fetch_page");
}

#[test]
fn all_caps_sequence_to_snake() {
    assert_eq!(to_snake_case("HTTPClient"), "http_client");
}

#[test]
fn single_word_lowercase() {
    assert_eq!(to_snake_case("run"), "run");
}

#[test]
fn camel_get_data() {
    assert_eq!(to_snake_case("getData"), "get_data");
}

#[test]
fn upper_snake_single_word() {
    assert_eq!(to_upper_snake_case("Sonnet"), "SONNET");
}

#[test]
fn upper_snake_pascal() {
    assert_eq!(to_upper_snake_case("ClaudeHaiku"), "CLAUDE_HAIKU");
}

#[test]
fn upper_snake_trailing_digits() {
    assert_eq!(to_upper_snake_case("GPT4"), "GPT4");
}
