//! AST nodes for the SDIF v1 parser.
//!
//! Source spans are intentionally kept out of AST nodes; diagnostics carry
//! source locations through `ParseError`.

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
    /// canonical form. Populated by the parser when cells in a column are
    /// written with double-quotes in the source.
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

// ---------------------------------------------------------------------------
// Accessor iterators — mirror Python sdif-py .fields, .tables, etc. properties
// ---------------------------------------------------------------------------

impl Document {
    pub fn fields(&self) -> impl Iterator<Item = &Field> {
        self.statements.iter().filter_map(|s| match s {
            Statement::Field(f) => Some(f),
            _ => None,
        })
    }
    pub fn objects(&self) -> impl Iterator<Item = &ObjectBlock> {
        self.statements.iter().filter_map(|s| match s {
            Statement::ObjectBlock(o) => Some(o),
            _ => None,
        })
    }
    pub fn tables(&self) -> impl Iterator<Item = &Table> {
        self.statements.iter().filter_map(|s| match s {
            Statement::Table(t) => Some(t),
            _ => None,
        })
    }
    pub fn relations(&self) -> impl Iterator<Item = &Relation> {
        self.statements.iter().filter_map(|s| match s {
            Statement::Relation(r) => Some(r),
            _ => None,
        })
    }
    pub fn rules(&self) -> impl Iterator<Item = &Rule> {
        self.statements.iter().filter_map(|s| match s {
            Statement::Rule(r) => Some(r),
            _ => None,
        })
    }
    pub fn narratives(&self) -> impl Iterator<Item = &Narrative> {
        self.statements.iter().filter_map(|s| match s {
            Statement::Narrative(n) => Some(n),
            _ => None,
        })
    }
}

impl ObjectBlock {
    pub fn fields(&self) -> impl Iterator<Item = &Field> {
        self.statements.iter().filter_map(|s| match s {
            Statement::Field(f) => Some(f),
            _ => None,
        })
    }
    pub fn objects(&self) -> impl Iterator<Item = &ObjectBlock> {
        self.statements.iter().filter_map(|s| match s {
            Statement::ObjectBlock(o) => Some(o),
            _ => None,
        })
    }
    pub fn tables(&self) -> impl Iterator<Item = &Table> {
        self.statements.iter().filter_map(|s| match s {
            Statement::Table(t) => Some(t),
            _ => None,
        })
    }
    pub fn relations(&self) -> impl Iterator<Item = &Relation> {
        self.statements.iter().filter_map(|s| match s {
            Statement::Relation(r) => Some(r),
            _ => None,
        })
    }
    pub fn rules(&self) -> impl Iterator<Item = &Rule> {
        self.statements.iter().filter_map(|s| match s {
            Statement::Rule(r) => Some(r),
            _ => None,
        })
    }
    pub fn narratives(&self) -> impl Iterator<Item = &Narrative> {
        self.statements.iter().filter_map(|s| match s {
            Statement::Narrative(n) => Some(n),
            _ => None,
        })
    }
}
