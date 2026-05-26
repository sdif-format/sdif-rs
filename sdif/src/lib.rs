//! SDIF format parser — Rust implementation.
//!
//! Public re-exports cover all stable types needed by downstream crates
//! (e.g. `sdif-lsp`). The `parser` module is a placeholder and will be filled
//! in Task 3.

mod ast;
pub mod canonical;
mod error;
pub mod json;
mod lexer;
pub mod parser;
mod policy;
pub mod schema;
mod span;
pub mod validation;

pub use ast::{
    Directive, Document, Field, Narrative, ObjectBlock, Relation, Rule, RuleArg, RuleCall,
    RuleExpression, Statement, Table,
};
pub use error::{ParseError, PolicyError};
pub use lexer::{lex_lines, Token, TokenKind};
pub use canonical::{canonicalize, sdif_hash};
pub use json::{document_to_json, json_to_sdif};
pub use parser::{parse_file, parse_text, parse_text_with_policy};
pub use policy::{Policy, RESERVED_TERMS};
pub use span::Span;
