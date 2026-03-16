//! Snapshot tests for tool declaration emission.

mod test_helpers;

use eaml_codegen::emitters::emit_tool;
use eaml_codegen::types::ImportTracker;
use eaml_codegen::writer::CodeWriter;
use eaml_parser::ast::DeclId;

/// Helper: emits all tools from source, returns the combined output.
fn emit_tools(source: &str) -> String {
    let (parse_output, analysis) = test_helpers::parse_and_analyze(source);
    let ast = &parse_output.ast;
    let interner = &parse_output.interner;
    let type_annotations = &analysis.type_annotations;

    let mut writer = CodeWriter::new();
    let mut imports = ImportTracker::new();

    for decl in &parse_output.program.declarations {
        if let DeclId::Tool(id) = decl {
            emit_tool(
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
fn test_tool_with_bridge_body() {
    let source = r#"
schema PageInfo {
  title: string
  length: int
}

tool fetchPage(url: string, timeout: int) -> PageInfo {
  python %{
    import httpx
    response = httpx.get(url, timeout=timeout)
    return {"title": response.headers.get("title", url), "length": len(response.text)}
  }%
}
"#;
    let output = emit_tools(source);
    insta::assert_snapshot!(output, @r#"
    def fetch_page(url: str, timeout: int) -> dict:
        import httpx
        response = httpx.get(url, timeout=timeout)
        return {"title": response.headers.get("title", url), "length": len(response.text)}

    def _eaml_call_fetch_page(url: str, timeout: int) -> PageInfo:
        result = fetch_page(url, timeout)
        return PageInfo.model_validate(result)

    _tool_fetch_page = ToolMetadata(
        name="fetchPage",
        description="fetchPage",
        parameters=[
            {"name": "url", "type": "string"},
            {"name": "timeout", "type": "int"},
        ],
        return_type="PageInfo",
        function=_eaml_call_fetch_page,
    )

    "#);
}

#[test]
fn test_tool_with_description() {
    let source = r#"
schema Result {
  value: string
}

tool analyzeText(text: string) -> Result {
  description: "Analyze the given text"
  python %{
    return {"value": text.upper()}
  }%
}
"#;
    let output = emit_tools(source);
    insta::assert_snapshot!(output, @r#"
    def analyze_text(text: str) -> dict:
        """Analyze the given text"""
        return {"value": text.upper()}

    def _eaml_call_analyze_text(text: str) -> Result:
        result = analyze_text(text)
        return Result.model_validate(result)

    _tool_analyze_text = ToolMetadata(
        name="analyzeText",
        description="Analyze the given text",
        parameters=[
            {"name": "text", "type": "string"},
        ],
        return_type="Result",
        function=_eaml_call_analyze_text,
    )

    "#);
}

#[test]
fn test_tool_primitive_return_type() {
    let source = r#"
tool greet(name: string) -> string {
  python %{
    return f"Hello, {name}!"
  }%
}
"#;
    let output = emit_tools(source);
    insta::assert_snapshot!(output, @r#"
    def greet(name: str) -> dict:
        return f"Hello, {name}!"

    def _eaml_call_greet(name: str) -> str:
        result = greet(name)
        if not isinstance(result, str):
            raise TypeError(f"Tool 'greet' expected 'str', got '{type(result).__name__}'")
        return result

    _tool_greet = ToolMetadata(
        name="greet",
        description="greet",
        parameters=[
            {"name": "name", "type": "string"},
        ],
        return_type="string",
        function=_eaml_call_greet,
    )

    "#);
}
