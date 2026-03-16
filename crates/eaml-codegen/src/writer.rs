//! Python code writer with indentation management.
//!
//! Produces 4-space-indented Python output. Handles indent/dedent,
//! line-start tracking, and blank line insertion.

/// A buffered Python code writer with automatic indentation.
pub struct CodeWriter {
    buf: String,
    indent: usize,
    at_line_start: bool,
}

impl CodeWriter {
    /// Creates a new empty writer at indent level 0.
    pub fn new() -> Self {
        Self {
            buf: String::new(),
            indent: 0,
            at_line_start: true,
        }
    }

    /// Increases the indentation level by one (4 spaces).
    pub fn indent(&mut self) {
        self.indent += 1;
    }

    /// Decreases the indentation level by one, saturating at zero.
    pub fn dedent(&mut self) {
        self.indent = self.indent.saturating_sub(1);
    }

    /// Writes text, prepending indentation if at the start of a line.
    pub fn write(&mut self, s: &str) {
        if self.at_line_start && !s.is_empty() {
            for _ in 0..self.indent {
                self.buf.push_str("    ");
            }
            self.at_line_start = false;
        }
        self.buf.push_str(s);
    }

    /// Writes text followed by a newline, prepending indentation if needed.
    pub fn writeln(&mut self, s: &str) {
        self.write(s);
        self.buf.push('\n');
        self.at_line_start = true;
    }

    /// Writes a blank line with no indentation.
    pub fn blank_line(&mut self) {
        self.buf.push('\n');
        self.at_line_start = true;
    }

    /// Consumes the writer and returns the accumulated output.
    pub fn finish(self) -> String {
        self.buf
    }
}

impl Default for CodeWriter {
    fn default() -> Self {
        Self::new()
    }
}
