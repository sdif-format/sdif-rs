//! AST nodes for the SDIF v1 parser.
//!
//! Spans are tracked on `ParseError` (for diagnostics) but not on individual
//! AST nodes — mirroring the Python reference implementation which has no
//! span fields on its AST types.

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
}

// ---------------------------------------------------------------------------
// ObjectBlock  (key:\n  …)
// ---------------------------------------------------------------------------

/// A named block that contains nested statements.
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectBlock {
    pub key: String,
    pub statements: Vec<Statement>,
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
    /// Indices (0-based) of columns whose cell values must be quoted in
    /// canonical form. Populated by Task 7; always empty after initial parse.
    pub quoted_columns: Vec<usize>,
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
}

// ---------------------------------------------------------------------------
// Narrative  (key> text…)
// ---------------------------------------------------------------------------

/// A narrative (prose) statement associated with a key.
#[derive(Debug, Clone, PartialEq)]
pub struct Narrative {
    pub key: String,
    pub text: String,
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
