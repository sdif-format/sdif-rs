//! AST nodes for the SDIF v1 parser.
//!
//! Every node carries a `span` recording its source location. Field and Table
//! additionally expose sub-spans for their constituent parts so that tooling
//! (e.g. an LSP) can highlight individual components.

use crate::span::Span;

// ---------------------------------------------------------------------------
// Top-level document
// ---------------------------------------------------------------------------

/// A parsed SDIF document.
#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub directives: Vec<Directive>,
    pub statements: Vec<Statement>,
}

// ---------------------------------------------------------------------------
// Directive  (@sdif 1.0, @include …, etc.)
// ---------------------------------------------------------------------------

/// A line starting with `@`, e.g. `@sdif 1.0`.
#[derive(Debug, Clone, PartialEq)]
pub struct Directive {
    pub name: String,
    pub args: Vec<String>,
    pub span: Span,
}

// ---------------------------------------------------------------------------
// Field  (key: value or key: "value")
// ---------------------------------------------------------------------------

/// A key–value field statement.
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub key: String,
    pub value: String,
    /// True when the value was written in double-quotes.
    pub quoted: bool,
    /// Span of the entire `key: value` line.
    pub span: Span,
    /// Span of just the key token.
    pub key_span: Span,
    /// Span of just the value token (excluding surrounding quotes if any).
    pub value_span: Span,
}

// ---------------------------------------------------------------------------
// ObjectBlock  (key:\n  …)
// ---------------------------------------------------------------------------

/// A named block that contains nested statements.
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectBlock {
    pub key: String,
    pub statements: Vec<Statement>,
    pub span: Span,
}

// ---------------------------------------------------------------------------
// Table  (name[col1\tcol2…]: …)
// ---------------------------------------------------------------------------

/// A tabular data block. Columns are separated by literal HTAB characters.
#[derive(Debug, Clone, PartialEq)]
pub struct Table {
    pub name: String,
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    /// Span of the complete table (header + all rows).
    pub span: Span,
    /// Span of the header line only.
    pub header_span: Span,
}

// ---------------------------------------------------------------------------
// Relation  (subject predicate object)
// ---------------------------------------------------------------------------

/// A triple-style relation statement.
#[derive(Debug, Clone, PartialEq)]
pub struct Relation {
    pub subject: String,
    pub predicate: String,
    pub object: String,
    /// True when the object value was written in double-quotes.
    pub object_quoted: bool,
    pub span: Span,
}

// ---------------------------------------------------------------------------
// Narrative  (key> text…)
// ---------------------------------------------------------------------------

/// A narrative (prose) statement associated with a key.
#[derive(Debug, Clone, PartialEq)]
pub struct Narrative {
    pub key: String,
    pub text: String,
    pub span: Span,
}

// ---------------------------------------------------------------------------
// Rule  (rule …)
// ---------------------------------------------------------------------------

/// A rule statement, stored verbatim as `source` plus an optional parsed
/// expression tree.
#[derive(Debug, Clone, PartialEq)]
pub struct Rule {
    pub source: String,
    pub expression: Option<RuleExpression>,
    pub span: Span,
}

// ---------------------------------------------------------------------------
// Rule expression sub-types
// ---------------------------------------------------------------------------

/// An argument value in a rule call.
#[derive(Debug, Clone, PartialEq)]
pub enum RuleArg {
    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
    Ident(String),
    Call(RuleCall),
}

/// A function call appearing inside a rule expression.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleCall {
    pub name: String,
    pub args: Vec<RuleArg>,
}

/// A top-level rule expression: `action function(args…)`.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleExpression {
    pub action: String,
    pub function: String,
    pub args: Vec<RuleArg>,
}

// ---------------------------------------------------------------------------
// Statement — sum type over all node variants
// ---------------------------------------------------------------------------

/// Any statement that can appear at document or object-block scope.
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Field(Field),
    ObjectBlock(ObjectBlock),
    Table(Table),
    Relation(Relation),
    Rule(Rule),
    Narrative(Narrative),
}
