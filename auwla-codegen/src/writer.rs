/// A low-level buffer for emitting indented source code.
///
/// Used by `JsEmitter` to manage both the main output and extensions output
/// through a uniform API, eliminating duplicated `write`/`write_ext` pairs.
pub(crate) struct CodeWriter {
    buffer: String,
    indent: usize,
}

impl CodeWriter {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            indent: 0,
        }
    }

    /// Append raw text without indentation.
    pub fn write(&mut self, s: &str) {
        self.buffer.push_str(s);
    }

    /// Write the current indentation prefix.
    pub fn write_indent(&mut self) {
        for _ in 0..self.indent {
            self.buffer.push_str("  ");
        }
    }

    /// Write an indented line (indent + text + newline).
    pub fn writeln(&mut self, s: &str) {
        self.write_indent();
        self.buffer.push_str(s);
        self.buffer.push('\n');
    }

    /// Increase indentation by one level.
    pub fn indent(&mut self) {
        self.indent += 1;
    }

    /// Decrease indentation by one level.
    pub fn dedent(&mut self) {
        self.indent -= 1;
    }

    /// Get the current indentation level.
    pub fn indent_level(&self) -> usize {
        self.indent
    }

    /// Set the indentation level directly.
    pub fn set_indent(&mut self, level: usize) {
        self.indent = level;
    }

    /// Get a reference to the accumulated buffer.
    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        &self.buffer
    }

    /// Get the current byte length of the buffer.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Consume the writer and return the accumulated string.
    pub fn into_string(self) -> String {
        self.buffer
    }

    /// Capture output: temporarily redirect writes into a fresh buffer,
    /// execute the closure, then restore the original buffer and return
    /// the captured string.
    pub fn capture<F>(&mut self, f: F) -> String
    where
        F: FnOnce(&mut Self),
    {
        let old = std::mem::take(&mut self.buffer);
        f(self);
        let captured = std::mem::take(&mut self.buffer);
        self.buffer = old;
        captured
    }

    /// Truncate the buffer to `len` bytes and push `replacement`.
    #[allow(dead_code)]
    pub fn replace_tail(&mut self, len: usize, replacement: &str) {
        self.buffer.truncate(len);
        self.buffer.push_str(replacement);
    }
}
