//! SDIF format parser — Rust implementation.
//!
//! Public re-exports cover all stable types needed by downstream crates
//! (e.g. `sdif-lsp`). The `parser` module is a placeholder and will be filled
//! in Task 3.

mod span;
mod ast;
mod error;
mod policy;
mod lexer;
pub mod parser;

pub use span::Span;
pub use ast::{
    Document, Directive, Field, Table, Relation, Narrative, Rule,
    ObjectBlock, Statement, RuleExpression, RuleArg, RuleCall,
};
pub use error::{ParseError, PolicyError};
pub use policy::{Policy, RESERVED_TERMS};
pub use lexer::{Token, TokenKind, lex_lines};
