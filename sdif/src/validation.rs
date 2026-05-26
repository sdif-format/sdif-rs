//! Schema-driven validation of SDIF documents.

use crate::ast::{Document, Statement};
use crate::schema::Schema;

/// Severity level for a validation diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

/// A validation diagnostic (not a parse error).
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
}

/// Validate a parsed Document against a Schema. Returns diagnostics (empty = valid).
pub fn validate(doc: &Document, schema: &Schema) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    check_required_fields(doc, schema, &mut diagnostics);
    check_allowed_tables(doc, schema, &mut diagnostics);
    check_required_columns(doc, schema, &mut diagnostics);
    check_allowed_predicates(doc, schema, &mut diagnostics);
    check_allowed_rule_functions(doc, schema, &mut diagnostics);
    diagnostics
}

fn check_required_fields(doc: &Document, schema: &Schema, out: &mut Vec<Diagnostic>) {
    let present: std::collections::HashSet<&str> = doc
        .statements
        .iter()
        .filter_map(|s| match s {
            Statement::Field(f) => Some(f.key.as_str()),
            _ => None,
        })
        .collect();
    for required in &schema.required_fields {
        if !present.contains(required.as_str()) {
            out.push(Diagnostic {
                severity: Severity::Error,
                message: format!("required field '{}' is missing", required),
            });
        }
    }
}

fn check_allowed_tables(doc: &Document, schema: &Schema, out: &mut Vec<Diagnostic>) {
    if schema.allowed_tables.is_empty() {
        return;
    }
    for stmt in &doc.statements {
        if let Statement::Table(t) = stmt {
            if !schema.allowed_tables.contains(&t.name) {
                out.push(Diagnostic {
                    severity: Severity::Error,
                    message: format!("table '{}' is not in the allowed tables list", t.name),
                });
            }
        }
    }
}

fn check_required_columns(doc: &Document, schema: &Schema, out: &mut Vec<Diagnostic>) {
    for stmt in &doc.statements {
        if let Statement::Table(t) = stmt {
            if let Some(required) = schema.required_columns.get(&t.name) {
                for col in required {
                    if !t.columns.contains(col) {
                        out.push(Diagnostic {
                            severity: Severity::Error,
                            message: format!(
                                "table '{}' is missing required column '{}'",
                                t.name, col
                            ),
                        });
                    }
                }
            }
        }
    }
}

fn check_allowed_predicates(doc: &Document, schema: &Schema, out: &mut Vec<Diagnostic>) {
    if schema.allowed_predicates.is_empty() {
        return;
    }
    for stmt in &doc.statements {
        if let Statement::Relation(r) = stmt {
            if !schema.allowed_predicates.contains(&r.predicate) {
                out.push(Diagnostic {
                    severity: Severity::Error,
                    message: format!("relation predicate '{}' is not allowed", r.predicate),
                });
            }
        }
    }
}

fn check_allowed_rule_functions(doc: &Document, schema: &Schema, out: &mut Vec<Diagnostic>) {
    if schema.allowed_rule_functions.is_empty() {
        return;
    }
    for stmt in &doc.statements {
        if let Statement::Rule(r) = stmt {
            if let Some(expr) = &r.expression {
                if !schema.allowed_rule_functions.contains(&expr.function) {
                    out.push(Diagnostic {
                        severity: Severity::Error,
                        message: format!("rule function '{}' is not allowed", expr.function),
                    });
                }
            }
        }
    }
}
