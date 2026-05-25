//! SDIF format parser — Rust implementation.
//!
//! Public re-exports cover all stable types needed by downstream crates
//! (e.g. `sdif-lsp`). The `parser` module is a placeholder and will be filled
//! in Task 3.

mod ast;
mod error;
mod lexer;
pub mod parser;
mod policy;
mod span;

pub use ast::{
    Directive, Document, Field, Narrative, ObjectBlock, Relation, Rule, RuleArg, RuleCall,
    RuleExpression, Statement, Table,
};
pub use error::{ParseError, PolicyError};
pub use lexer::{lex_lines, Token, TokenKind};
pub use policy::{Policy, RESERVED_TERMS};
pub use span::Span;
