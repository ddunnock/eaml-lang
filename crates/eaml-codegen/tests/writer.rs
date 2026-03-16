use eaml_codegen::writer::CodeWriter;

#[test]
fn new_writer_is_empty() {
    let w = CodeWriter::new();
    assert_eq!(w.finish(), "");
}

#[test]
fn writeln_at_base_level() {
    let mut w = CodeWriter::new();
    w.writeln("class Foo:");
    assert_eq!(w.finish(), "class Foo:\n");
}

#[test]
fn writeln_after_indent() {
    let mut w = CodeWriter::new();
    w.indent();
    w.writeln("pass");
    assert_eq!(w.finish(), "    pass\n");
}

#[test]
fn writeln_after_indent_and_dedent() {
    let mut w = CodeWriter::new();
    w.indent();
    w.writeln("pass");
    w.dedent();
    w.writeln("x = 1");
    assert_eq!(w.finish(), "    pass\nx = 1\n");
}

#[test]
fn blank_line_has_no_indentation() {
    let mut w = CodeWriter::new();
    w.indent();
    w.writeln("a = 1");
    w.blank_line();
    w.writeln("b = 2");
    assert_eq!(w.finish(), "    a = 1\n\n    b = 2\n");
}

#[test]
fn write_prepends_indent_only_at_line_start() {
    let mut w = CodeWriter::new();
    w.indent();
    w.write("x");
    w.write(" = 1");
    assert_eq!(w.finish(), "    x = 1");
}

#[test]
fn two_levels_of_indent() {
    let mut w = CodeWriter::new();
    w.indent();
    w.indent();
    w.writeln("deeply_nested()");
    assert_eq!(w.finish(), "        deeply_nested()\n");
}

#[test]
fn dedent_below_zero_saturates() {
    let mut w = CodeWriter::new();
    w.dedent();
    w.writeln("still at base");
    assert_eq!(w.finish(), "still at base\n");
}
