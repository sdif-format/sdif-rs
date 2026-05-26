//! Schema-driven validation of SDIF documents.

use crate::ast::{Document, RuleArg, Statement};
pub use crate::schema::{RelationPolicy, Schema, SchemaError, TablePolicy};

/// A validation diagnostic (not a parse error).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: String,
    pub severity: String,
    pub message: String,
    pub path: String,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub hint: Option<String>,
}

/// Validate a parsed Document against a Schema. Returns diagnostics (empty = valid).
pub fn validate(doc: &Document, schema: &Schema) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    check_required_fields(doc, schema, &mut diagnostics);
    check_field_types(doc, schema, &mut diagnostics);
    check_table_columns(doc, schema, &mut diagnostics);
    check_table_column_types(doc, schema, &mut diagnostics);
    check_relation_policies(doc, schema, &mut diagnostics);
    check_legacy_allowed_tables(doc, schema, &mut diagnostics);
    check_legacy_allowed_predicates(doc, schema, &mut diagnostics);
    check_legacy_allowed_rule_functions(doc, schema, &mut diagnostics);
    diagnostics
}

fn error(code: &str, message: String, path: String, hint: Option<String>) -> Diagnostic {
    Diagnostic {
        code: code.to_string(),
        severity: "error".to_string(),
        message,
        path,
        line: None,
        column: None,
        hint,
    }
}

fn check_required_fields(doc: &Document, schema: &Schema, out: &mut Vec<Diagnostic>) {
    for required in &schema.required_fields {
        if doc.fields().all(|field| &field.key != required) {
            out.push(error(
                "SDIF_MISSING_FIELD",
                format!("required field `{required}` is missing"),
                format!("$.{required}"),
                None,
            ));
        }
    }
}

fn check_field_types(doc: &Document, schema: &Schema, out: &mut Vec<Diagnostic>) {
    for field in doc.fields() {
        if let Some(expected_type) = schema.field_types.get(&field.key) {
            if !value_matches_type(&field.value, expected_type) {
                out.push(error(
                    "SDIF_TYPE_MISMATCH",
                    format!(
                        "field `{}` expected type `{expected_type}`, got `{}`",
                        field.key, field.value
                    ),
                    format!("$.{}", field.key),
                    Some(format!("Use a value matching `{expected_type}`.")),
                ));
            }
        }
    }
}

fn check_table_columns(doc: &Document, schema: &Schema, out: &mut Vec<Diagnostic>) {
    for table in doc.tables() {
        if let Some(required_columns) = schema.required_table_columns.get(&table.name) {
            for column in required_columns {
                if !table.columns.contains(column) {
                    out.push(error(
                        "SDIF_MISSING_COLUMN",
                        format!("table `{}` missing required column `{column}`", table.name),
                        format!("$.{}", table.name),
                        None,
                    ));
                }
            }
        }
        if let Some(required_columns) = schema.required_columns.get(&table.name) {
            for column in required_columns {
                if !table.columns.contains(column) {
                    out.push(error(
                        "SDIF_MISSING_COLUMN",
                        format!("table `{}` missing required column `{column}`", table.name),
                        format!("$.{}", table.name),
                        None,
                    ));
                }
            }
        }
    }
}

fn check_table_column_types(doc: &Document, schema: &Schema, out: &mut Vec<Diagnostic>) {
    for table in doc.tables() {
        if let Some(column_types) = schema.table_column_types.get(&table.name) {
            for (column, expected_type) in column_types {
                if let Some(column_index) = table.columns.iter().position(|c| c == column) {
                    for cell in table.rows.iter().filter_map(|row| row.get(column_index)) {
                        if !value_matches_type(cell, expected_type) {
                            out.push(error(
                                "SDIF_TYPE_MISMATCH",
                                format!(
                                    "table `{}` column `{column}` expected `{expected_type}`, got `{cell}`",
                                    table.name
                                ),
                                format!("$.{}.{column}", table.name),
                                Some(format!("Use `{expected_type}` values in this column.")),
                            ));
                        }
                    }
                }
            }
        }
    }
}

fn check_relation_policies(doc: &Document, schema: &Schema, out: &mut Vec<Diagnostic>) {
    for policy in schema
        .relation_policies
        .values()
        .filter(|policy| policy.required)
    {
        if doc
            .relations()
            .all(|relation| relation.predicate != policy.predicate)
        {
            out.push(error(
                "SDIF_MISSING_RELATION",
                format!("required relation `{}` not found", policy.predicate),
                "$.rel".to_string(),
                None,
            ));
        }
    }
}

fn check_legacy_allowed_tables(doc: &Document, schema: &Schema, out: &mut Vec<Diagnostic>) {
    if schema.allowed_tables.is_empty() {
        return;
    }
    for table in doc.tables() {
        if !schema.allowed_tables.contains(&table.name) {
            out.push(error(
                "SDIF_TABLE_NOT_ALLOWED",
                format!("table '{}' is not in the allowed tables list", table.name),
                format!("$.{}", table.name),
                None,
            ));
        }
    }
}

fn check_legacy_allowed_predicates(doc: &Document, schema: &Schema, out: &mut Vec<Diagnostic>) {
    if schema.allowed_predicates.is_empty() {
        return;
    }
    for relation in doc.relations() {
        if !schema.allowed_predicates.contains(&relation.predicate) {
            out.push(error(
                "SDIF_PREDICATE_NOT_ALLOWED",
                format!("relation predicate '{}' is not allowed", relation.predicate),
                "$.rel".to_string(),
                None,
            ));
        }
    }
}

fn check_legacy_allowed_rule_functions(doc: &Document, schema: &Schema, out: &mut Vec<Diagnostic>) {
    if schema.allowed_rule_functions.is_empty() {
        return;
    }
    for stmt in &doc.statements {
        if let Statement::Rule(rule) = stmt {
            if let Some(expr) = &rule.expression {
                if !schema.allowed_rule_functions.contains(&expr.function) {
                    out.push(error(
                        "SDIF_RULE_FUNCTION_NOT_ALLOWED",
                        format!("rule function '{}' is not allowed", expr.function),
                        "$.rule".to_string(),
                        None,
                    ));
                }
                for arg in &expr.args {
                    check_nested_call(arg, schema, out);
                }
            }
        }
    }
}

fn check_nested_call(arg: &RuleArg, schema: &Schema, out: &mut Vec<Diagnostic>) {
    if let RuleArg::Call(call) = arg {
        if !schema.allowed_rule_functions.contains(&call.name) {
            out.push(error(
                "SDIF_RULE_FUNCTION_NOT_ALLOWED",
                format!("rule function '{}' is not allowed", call.name),
                "$.rule".to_string(),
                None,
            ));
        }
        for nested in &call.args {
            check_nested_call(nested, schema, out);
        }
    }
}

fn value_matches_type(value: &str, type_name: &str) -> bool {
    match type_name {
        "String" | "string" => true,
        "Integer" | "int" | "integer" => value.parse::<i64>().is_ok(),
        "Decimal" | "Float" | "float" | "number" => value.parse::<f64>().is_ok(),
        "Boolean" | "bool" | "boolean" => matches!(value, "true" | "false"),
        "Identifier" | "identifier" => {
            !value.is_empty()
                && value
                    .chars()
                    .all(|c| c.is_alphanumeric() || "_./:-".contains(c))
        }
        "Date" | "date" => {
            value.len() == 10
                && value.chars().nth(4) == Some('-')
                && value.chars().nth(7) == Some('-')
        }
        _ => true,
    }
}
