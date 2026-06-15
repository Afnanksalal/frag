//! Diagnostics and source locations.
//!
//! Frag uses byte spans internally. The CLI converts those spans into
//! line/column snippets for user-facing error messages.

use std::fmt;

/// Byte range in the original source text.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Span {
    /// Inclusive start byte offset.
    pub start: usize,
    /// Exclusive end byte offset.
    pub end: usize,
}

impl Span {
    /// Create a new byte span.
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Return a span that covers both spans.
    pub fn join(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

/// Compiler error with an optional source span.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    /// Human-readable error message.
    pub message: String,
    /// Optional source location for the error.
    pub span: Option<Span>,
}

impl Diagnostic {
    /// Create a diagnostic without a source span.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            span: None,
        }
    }

    /// Create a diagnostic at a source span.
    pub fn at(span: Span, message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            span: Some(span),
        }
    }

    /// Render the diagnostic with a source snippet when a span is available.
    pub fn with_source(&self, source: &str) -> String {
        let Some(span) = self.span else {
            return self.message.clone();
        };

        let (line_no, line_start) = line_start_for_offset(source, span.start);
        let line_end = source[line_start..]
            .find('\n')
            .map(|offset| line_start + offset)
            .unwrap_or(source.len());
        let line = &source[line_start..line_end];
        let column = span.start.saturating_sub(line_start) + 1;
        let marker_len = span.end.saturating_sub(span.start).max(1);

        format!(
            "{}\n --> line {}, column {}\n{}\n{}{}",
            self.message,
            line_no,
            column,
            line,
            " ".repeat(column.saturating_sub(1)),
            "^".repeat(marker_len.min(line.len().saturating_sub(column - 1)).max(1))
        )
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for Diagnostic {}

/// Result type used throughout the compiler.
pub type Result<T> = std::result::Result<T, Diagnostic>;

fn line_start_for_offset(source: &str, offset: usize) -> (usize, usize) {
    let mut line_no = 1;
    let mut line_start = 0;

    for (idx, ch) in source.char_indices() {
        if idx >= offset {
            break;
        }
        if ch == '\n' {
            line_no += 1;
            line_start = idx + 1;
        }
    }

    (line_no, line_start)
}
