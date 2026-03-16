//! Snapshot tests for prompt declaration emission.

mod test_helpers;

use eaml_codegen::emitters::emit_prompt;
use eaml_codegen::types::ImportTracker;
use eaml_codegen::writer::CodeWriter;
use eaml_parser::ast::DeclId;

/// Helper: emits all prompts from source, returns the combined output.
fn emit_prompts(source: &str) -> String {
    let (parse_output, analysis) = test_helpers::parse_and_analyze(source);
    let ast = &parse_output.ast;
    let interner = &parse_output.interner;
    let type_annotations = &analysis.type_annotations;

    let mut writer = CodeWriter::new();
    let mut imports = ImportTracker::new();

    for decl in &parse_output.program.declarations {
        if let DeclId::Prompt(id) = decl {
            emit_prompt(
                &ast[*id],
                ast,
                interner,
                type_annotations,
                source,
                &mut writer,
                &mut imports,
            );
            writer.blank_line();
        }
    }

    writer.finish()
}

#[test]
fn test_minimal_prompt_user_only() {
    let source = r#"
schema Response {
  answer: string
}

prompt AskQuestion(question: string) -> Response {
  user: "Please answer: {question}"
}
"#;
    let output = emit_prompts(source);
    insta::assert_snapshot!(output, @r#"
    async def ask_question(question: str, *, model: dict) -> Response:
        messages = [
            {"role": "user", "content": f"Please answer: {question}"},
        ]
        return await eaml_runtime.execute_prompt(
            model=model,
            messages=messages,
            return_type=Response,
        )

    "#);
}

#[test]
fn test_sentiment_prompt_full() {
    let source = r#"
model Sonnet = Model(
  id: "anthropic/claude-3-5-sonnet-20241022",
  provider: "anthropic",
  caps: [json_mode, streaming]
)

schema SentimentResult {
  sentiment: "positive" | "neutral" | "negative"
  confidence: float<0.0, 1.0>
  explanation: string
}

prompt AnalyzeSentiment(text: string)
  requires json_mode
  -> SentimentResult
{
  system: "You are a sentiment analysis expert. Classify the sentiment of the given text and provide a confidence score between 0 and 1. Be concise in your explanation."
  user: "Analyze the sentiment of the following text:

{text}"
  temperature: 0.2
  max_tokens: 256
}
"#;
    let output = emit_prompts(source);
    insta::assert_snapshot!(output, @r#"
    async def analyze_sentiment(text: str, *, model: dict) -> SentimentResult:
        messages = [
            {"role": "system", "content": "You are a sentiment analysis expert. Classify the sentiment of the given text and provide a confidence score between 0 and 1. Be concise in your explanation."},
            {"role": "user", "content": f"Analyze the sentiment of the following text:\n\n{text}"},
        ]
        return await eaml_runtime.execute_prompt(
            model=model,
            messages=messages,
            return_type=SentimentResult,
            temperature=0.2,
            max_tokens=256,
        )

    "#);
}

#[test]
fn test_prompt_plain_text_no_interpolation() {
    let source = r#"
schema Greeting {
  message: string
}

prompt SayHello() -> Greeting {
  user: "Say hello to the world"
}
"#;
    let output = emit_prompts(source);
    insta::assert_snapshot!(output, @r#"
    async def say_hello(*, model: dict) -> Greeting:
        messages = [
            {"role": "user", "content": "Say hello to the world"},
        ]
        return await eaml_runtime.execute_prompt(
            model=model,
            messages=messages,
            return_type=Greeting,
        )

    "#);
}

#[test]
fn test_prompt_with_max_retries() {
    let source = r#"
schema Result {
  value: string
}

prompt Retry(input: string) -> Result {
  user: "Process: {input}"
  max_retries: 3
}
"#;
    let output = emit_prompts(source);
    insta::assert_snapshot!(output, @r#"
    async def retry(input: str, *, model: dict) -> Result:
        messages = [
            {"role": "user", "content": f"Process: {input}"},
        ]
        return await eaml_runtime.execute_prompt(
            model=model,
            messages=messages,
            return_type=Result,
            max_retries=3,
        )

    "#);
}
