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
        Self {
            start_line,
            start_col,
            end_line,
            end_col,
        }
    }

    /// Convenience constructor for a span that sits on a single line.
    pub fn single_line(line: u32, start_col: u32, end_col: u32) -> Self {
        Self {
            start_line: line,
            start_col,
            end_line: line,
            end_col,
        }
    }

    /// Returns true if (line, col) falls within this span (1-based, end_col exclusive).
    pub fn contains(&self, line: u32, col: u32) -> bool {
        if line < self.start_line || line > self.end_line {
            return false;
        }
        if line == self.start_line && col < self.start_col {
            return false;
        }
        if line == self.end_line && col >= self.end_col {
            return false;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::Span;

    #[test]
    fn test_span_contains_point_inside() {
        let s = Span::single_line(3, 5, 10);
        assert!(s.contains(3, 7));
    }

    #[test]
    fn test_span_contains_point_at_start() {
        let s = Span::single_line(3, 5, 10);
        assert!(s.contains(3, 5));
    }

    #[test]
    fn test_span_contains_point_at_end_exclusive() {
        let s = Span::single_line(3, 5, 10);
        assert!(!s.contains(3, 10));
    }

    #[test]
    fn test_span_contains_wrong_line() {
        let s = Span::single_line(3, 5, 10);
        assert!(!s.contains(2, 7));
    }

    #[test]
    fn test_span_contains_multiline() {
        let s = Span::new(2, 1, 4, 5);
        assert!(s.contains(3, 1));
        assert!(s.contains(2, 3));
        assert!(s.contains(4, 1));
        assert!(!s.contains(4, 5));
        assert!(!s.contains(5, 1));
    }
}
