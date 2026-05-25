//! Line-oriented lexer for the SDIF v1 parser.
//!
//! Ported from the Python reference implementation in
//! `sdif/src/sdif/parser/lexer.py`. Each non-blank line produces exactly one
//! `Token`. Blank lines are skipped. Line endings (`\r\n` and `\r`) are
//! normalised to `\n` before processing.

// ---------------------------------------------------------------------------
// TokenKind
// ---------------------------------------------------------------------------

/// The syntactic category of a lexed line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    /// A line starting with `#`.
    Comment,
    /// A line starting with `@`.
    Directive,
    /// A key–value or block-opener line (catch-all for non-structural lines).
    Field,
    /// A block opener ending with `:` but without `[…]` brackets.
    Block,
    /// A table header line: ends with `:` and contains `[…]`.
    TableHeader,
    /// A table data row: contains at least one literal HTAB character.
    TableRow,
}

// ---------------------------------------------------------------------------
// Token
// ---------------------------------------------------------------------------

/// A single lexed line token.
#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    /// The line body after stripping leading spaces (the indent).
    pub value: String,
    /// 1-based line number in the source document.
    pub line: u32,
    /// 1-based column of the first non-space character.
    pub col: u32,
    /// Number of leading space characters (used to detect nesting level).
    pub indent: u32,
}

// ---------------------------------------------------------------------------
// lex_lines
// ---------------------------------------------------------------------------

/// Tokenise `text` into a flat list of tokens, one per non-blank line.
pub fn lex_lines(text: &str) -> Vec<Token> {
    // Normalise line endings exactly as the Python reference does.
    let normalised = text.replace("\r\n", "\n").replace('\r', "\n");

    let mut tokens = Vec::new();

    for (zero_idx, raw) in normalised.split('\n').enumerate() {
        let line_no = (zero_idx + 1) as u32;

        if raw.trim().is_empty() {
            continue;
        }

        // Count leading spaces to determine indent level.
        let indent = raw.len() - raw.trim_start_matches(' ').len();
        let body = &raw[indent..];

        let kind = classify(body);
        tokens.push(Token {
            kind,
            value: body.to_owned(),
            line: line_no,
            // Column is 1-based; the body starts at column indent+1.
            col: indent as u32 + 1,
            indent: indent as u32,
        });
    }

    tokens
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Determine the `TokenKind` for a non-empty, indent-stripped line body.
fn classify(body: &str) -> TokenKind {
    if body.starts_with('#') {
        return TokenKind::Comment;
    }
    if body.starts_with('@') {
        return TokenKind::Directive;
    }
    // A literal HTAB anywhere in the body means it's a table data row.
    if body.contains('\t') {
        return TokenKind::TableRow;
    }
    if looks_like_table_header(body) {
        return TokenKind::TableHeader;
    }
    if body.ends_with(':') {
        return TokenKind::Block;
    }
    TokenKind::Field
}

/// A table header ends with `:` and has `[` before `]`.
fn looks_like_table_header(body: &str) -> bool {
    if !body.ends_with(':') {
        return false;
    }
    match (body.find('['), body.find(']')) {
        (Some(open), Some(close)) => open < close,
        _ => false,
    }
}
