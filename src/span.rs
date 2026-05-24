/// Source location spanning a range of characters in an SDIF document.
///
/// Lines and columns are 1-based. `end_col` is exclusive (one past the last
/// character). Both `start_line` and `end_line` refer to the same logical line
/// for single-line tokens; multi-line nodes use different values.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start_line: u32,
    pub start_col: u32,
    pub end_line: u32,
    pub end_col: u32,
}

impl Span {
    /// Create a span from explicit start and end positions.
    pub fn new(start_line: u32, start_col: u32, end_line: u32, end_col: u32) -> Self {
        Self { start_line, start_col, end_line, end_col }
    }

    /// Convenience constructor for a span that sits on a single line.
    pub fn single_line(line: u32, start_col: u32, end_col: u32) -> Self {
        Self { start_line: line, start_col, end_line: line, end_col }
    }
}
