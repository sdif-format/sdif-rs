//! JSON ↔ SDIF conversion helpers.
//!
//! Mirrors `sdif.json.converter` from the Python reference implementation.
//!
//! # `document_to_json`
//! Converts a parsed [`Document`] into a [`serde_json::Value`] object.
//!
//! # `json_to_sdif`
//! Encodes a [`serde_json::Value`] mapping as SDIF text with an `@sdif 1.0`
//! header.
//!
//! ## Relation / rule key divergence
//! Python uses `rel` / `rules` as keys; the task specification uses
//! `_relations` / `_rules`. We follow the task specification here because this
//! is a new Rust API and `rel`/`rules` may conflict with user field names.

use serde_json::{Map, Value};

use crate::ast::{Document, Field, ObjectBlock, Relation, Statement, Table};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const ARRAY_ITEM_KEY: &str = "__item";
const ARRAY_VALUE_KEY: &str = "__value";

// Regex-like constants used to decide quoting (no regex dep — use char checks).
// A scalar "must be quoted" in SDIF field context if it looks like a keyword
// or a number, or if it contains special chars.

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Convert a parsed SDIF [`Document`] into a JSON object value.
///
/// Scalar fields are coerced: `"null"` → `null`, `"true"` / `"false"` →
/// booleans, integer / decimal strings → numbers, everything else → string.
/// Fields with `quoted == true` are always returned as strings (no coercion).
///
/// Table cells are always returned as strings (no coercion).
pub fn document_to_json(doc: &Document) -> Value {
    statements_to_json_value(&doc.statements)
}

/// Encode a JSON mapping as SDIF text.
///
/// The output always starts with `@sdif 1.0` followed by SDIF statements
/// representing the JSON value.  Arrays of uniform scalar objects are encoded
/// as SDIF tables; all-scalar arrays use the `[a,b,c]` inline literal syntax;
/// mixed / nested arrays use `__item` object blocks.
///
/// # Panics
/// Panics if `data` is not a `Value::Object`.
pub fn json_to_sdif(data: &Value) -> String {
    let map = data.as_object().expect("json_to_sdif: data must be a JSON object");
    let mut lines: Vec<String> = vec!["@sdif 1.0".to_owned()];
    emit_mapping(map, &mut lines, 0);
    let mut result = lines.join("\n");
    // Strip trailing whitespace/newlines then add exactly one trailing newline.
    while result.ends_with('\n') || result.ends_with(' ') {
        result.pop();
    }
    result.push('\n');
    result
}

// ---------------------------------------------------------------------------
// document_to_json helpers
// ---------------------------------------------------------------------------

fn statements_to_json_value(statements: &[Statement]) -> Value {
    // If all statements are `__item` ObjectBlocks → emit as JSON array.
    if !statements.is_empty() && statements.iter().all(is_array_item_statement) {
        let items: Vec<Value> = statements
            .iter()
            .map(|s| match s {
                Statement::ObjectBlock(o) => array_item_to_json_value(o),
                _ => unreachable!(),
            })
            .collect();
        return Value::Array(items);
    }

    let mut map: Map<String, Value> = Map::new();
    let mut relations: Vec<Value> = Vec::new();
    let mut rules: Vec<Value> = Vec::new();

    for statement in statements {
        match statement {
            Statement::Field(f) => {
                map.insert(f.key.clone(), parse_field_value(f));
            }
            Statement::ObjectBlock(o) => {
                map.insert(o.key.clone(), statements_to_json_value(&o.statements));
            }
            Statement::Table(t) => {
                map.insert(t.name.clone(), table_to_json(t));
            }
            Statement::Relation(r) => {
                relations.push(relation_to_json(r));
            }
            Statement::Rule(r) => {
                rules.push(Value::String(r.source.clone()));
            }
            Statement::Narrative(n) => {
                map.insert(n.key.clone(), Value::String(n.text.clone()));
            }
        }
    }

    if !relations.is_empty() {
        map.insert("_relations".to_owned(), Value::Array(relations));
    }
    if !rules.is_empty() {
        map.insert("_rules".to_owned(), Value::Array(rules));
    }

    Value::Object(map)
}

fn parse_field_value(field: &Field) -> Value {
    if field.quoted {
        return Value::String(field.value.clone());
    }
    coerce_scalar(&field.value)
}

/// Coerce an unquoted scalar string into the most specific JSON value.
fn coerce_scalar(raw: &str) -> Value {
    match raw {
        "null" => Value::Null,
        "true" => Value::Bool(true),
        "false" => Value::Bool(false),
        _ => {
            // Try integer first.
            if let Some(n) = parse_integer(raw) {
                return Value::Number(n.into());
            }
            // Try decimal.
            if let Some(f) = parse_decimal(raw) {
                if let Some(n) = serde_json::Number::from_f64(f) {
                    return Value::Number(n);
                }
            }
            Value::String(raw.to_owned())
        }
    }
}

fn parse_integer(s: &str) -> Option<i64> {
    // Pattern: [+-]?(0|[1-9][0-9]*)
    let stripped = s.strip_prefix(['+', '-']).unwrap_or(s);
    if stripped == "0" || (stripped.starts_with(|c: char| c.is_ascii_digit() && c != '0')
        && stripped.chars().all(|c| c.is_ascii_digit()))
    {
        s.parse::<i64>().ok()
    } else {
        None
    }
}

fn parse_decimal(s: &str) -> Option<f64> {
    // Pattern: [+-]?(0|[1-9][0-9]*)\.[0-9]+
    let stripped = s.strip_prefix(['+', '-']).unwrap_or(s);
    let dot_pos = stripped.find('.')?;
    let int_part = &stripped[..dot_pos];
    let frac_part = &stripped[dot_pos + 1..];
    if frac_part.is_empty() || !frac_part.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let valid_int = int_part == "0"
        || (int_part.starts_with(|c: char| c.is_ascii_digit() && c != '0')
            && int_part.chars().all(|c| c.is_ascii_digit()));
    if valid_int {
        s.parse::<f64>().ok()
    } else {
        None
    }
}

fn table_to_json(table: &Table) -> Value {
    let rows: Vec<Value> = table
        .rows
        .iter()
        .map(|row| {
            let mut obj = Map::new();
            for (col_idx, (col, cell)) in table.columns.iter().zip(row.iter()).enumerate() {
                // Strip trailing '$' from column name (quoted-column marker).
                let col_name = if col.ends_with('$') {
                    col[..col.len() - 1].to_owned()
                } else {
                    col.clone()
                };
                // Table cells are always returned as strings (no coercion).
                // Quoted columns and '$'-suffixed columns: strip surrounding quotes
                // if present, then return as string.
                let value = if table.quoted_columns.contains(&col_idx) || col.ends_with('$') {
                    unquote_if_quoted(cell)
                } else {
                    Value::String(cell.clone())
                };
                obj.insert(col_name, value);
            }
            Value::Object(obj)
        })
        .collect();
    Value::Array(rows)
}

fn unquote_if_quoted(s: &str) -> Value {
    if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
        // Unescape: \" → ", \\ → \, \n → newline, \t → tab
        let inner = &s[1..s.len() - 1];
        Value::String(unescape_string(inner))
    } else {
        Value::String(s.to_owned())
    }
}

fn unescape_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('"') => result.push('"'),
                Some('\\') => result.push('\\'),
                Some('n') => result.push('\n'),
                Some('t') => result.push('\t'),
                Some(other) => {
                    result.push('\\');
                    result.push(other);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }
    result
}

fn relation_to_json(rel: &Relation) -> Value {
    let object_val = if rel.object_quoted {
        Value::String(rel.object.clone())
    } else {
        coerce_scalar(&rel.object)
    };
    let mut map = Map::new();
    map.insert("subject".to_owned(), Value::String(rel.subject.clone()));
    map.insert("predicate".to_owned(), Value::String(rel.predicate.clone()));
    map.insert("object".to_owned(), object_val);
    Value::Object(map)
}

fn is_array_item_statement(s: &Statement) -> bool {
    matches!(s, Statement::ObjectBlock(o) if o.key == ARRAY_ITEM_KEY)
}

fn array_item_to_json_value(block: &ObjectBlock) -> Value {
    // If the block has exactly one statement that is a `__value` field/block,
    // unwrap it.
    if block.statements.len() == 1 {
        match &block.statements[0] {
            Statement::Field(f) if f.key == ARRAY_VALUE_KEY => {
                return parse_field_value(f);
            }
            Statement::ObjectBlock(o) if o.key == ARRAY_VALUE_KEY => {
                return statements_to_json_value(&o.statements);
            }
            Statement::Narrative(n) if n.key == ARRAY_VALUE_KEY => {
                return Value::String(n.text.clone());
            }
            _ => {}
        }
    }
    statements_to_json_value(&block.statements)
}

// ---------------------------------------------------------------------------
// json_to_sdif helpers
// ---------------------------------------------------------------------------

fn emit_mapping(map: &Map<String, Value>, lines: &mut Vec<String>, indent: usize) {
    let prefix = " ".repeat(indent);
    for (key, value) in map {
        match value {
            // Relations list: key == "_relations" and each item has subject/predicate/object
            Value::Array(arr)
                if key == "_relations" && arr.iter().all(is_relation_object) =>
            {
                lines.push(format!("{prefix}{key}:"));
                for item in arr {
                    let obj = item.as_object().unwrap();
                    let subject = format_scalar_str(obj.get("subject").unwrap());
                    let predicate = format_scalar_str(obj.get("predicate").unwrap());
                    let object = format_scalar_str(obj.get("object").unwrap());
                    lines.push(format!(
                        "{indent2}{subject} {predicate} {object}",
                        indent2 = " ".repeat(indent + 2)
                    ));
                }
            }
            // Rules list: key == "_rules" and all items are strings
            Value::Array(arr) if key == "_rules" && arr.iter().all(|v| v.is_string()) => {
                lines.push(format!("{prefix}{key}:"));
                for item in arr {
                    let s = item.as_str().unwrap();
                    lines.push(format!("{}{}", " ".repeat(indent + 2), s));
                }
            }
            // Array: try table, else inline or __item blocks
            Value::Array(arr) => {
                if can_emit_as_table(arr) {
                    emit_table(key, arr, lines, indent);
                } else {
                    emit_list(key, arr, lines, indent);
                }
            }
            // Nested object
            Value::Object(obj) => {
                lines.push(format!("{prefix}{key}:"));
                emit_mapping(obj, lines, indent + 2);
            }
            // Multi-line string at top level
            Value::String(s) if s.contains('\n') && indent == 0 => {
                lines.push(format!("{prefix}{key} \"\"\""));
                for line in s.split('\n') {
                    lines.push(line.to_owned());
                }
                lines.push(format!("{prefix}\"\"\""));
            }
            // Scalar field
            _ => {
                lines.push(format!("{prefix}{key} {}", format_scalar_str(value)));
            }
        }
    }
}

fn can_emit_as_table(arr: &[Value]) -> bool {
    if arr.is_empty() {
        return false;
    }
    // All items must be objects with the same set of keys and all scalar values.
    let first = match arr[0].as_object() {
        Some(m) => m,
        None => return false,
    };
    let columns: Vec<&str> = first.keys().map(String::as_str).collect();
    if columns.is_empty() {
        return false;
    }
    let expected_keys: std::collections::BTreeSet<&str> = columns.iter().copied().collect();
    arr.iter().all(|v| match v.as_object() {
        Some(obj) => {
            let keys: std::collections::BTreeSet<&str> =
                obj.keys().map(String::as_str).collect();
            keys == expected_keys && obj.values().all(is_json_scalar)
        }
        None => false,
    })
}

fn emit_table(name: &str, rows: &[Value], lines: &mut Vec<String>, indent: usize) {
    let prefix = " ".repeat(indent);
    let first = rows[0].as_object().unwrap();
    let columns: Vec<&str> = first.keys().map(String::as_str).collect();
    lines.push(format!("{prefix}{}[{}]:", name, columns.join(",")));
    for row in rows {
        let obj = row.as_object().unwrap();
        let cells: Vec<String> = columns
            .iter()
            .map(|col| format_table_cell(obj.get(*col).unwrap()))
            .collect();
        lines.push(format!("{}{}", " ".repeat(indent + 2), cells.join("\t")));
    }
}

fn emit_list(key: &str, arr: &[Value], lines: &mut Vec<String>, indent: usize) {
    let prefix = " ".repeat(indent);
    if arr.is_empty() {
        lines.push(format!("{prefix}{key} []"));
        return;
    }
    // All scalars → inline [a,b,c]
    if arr.iter().all(is_json_scalar) {
        let items: Vec<String> = arr.iter().map(format_list_item).collect();
        lines.push(format!("{prefix}{key} [{}]", items.join(",")));
        return;
    }
    // Otherwise: __item blocks
    lines.push(format!("{prefix}{key}:"));
    for item in arr {
        emit_array_item(item, lines, indent + 2);
    }
}

fn emit_array_item(value: &Value, lines: &mut Vec<String>, indent: usize) {
    let prefix = " ".repeat(indent);
    lines.push(format!("{prefix}{ARRAY_ITEM_KEY}:"));
    match value {
        Value::Object(obj) => {
            emit_mapping(obj, lines, indent + 2);
        }
        Value::Array(arr) => {
            lines.push(format!("{}{ARRAY_VALUE_KEY}:", " ".repeat(indent + 2)));
            for item in arr {
                emit_array_item(item, lines, indent + 4);
            }
        }
        _ => {
            lines.push(format!(
                "{}{ARRAY_VALUE_KEY} {}",
                " ".repeat(indent + 2),
                format_scalar_str(value)
            ));
        }
    }
}

fn is_relation_object(v: &Value) -> bool {
    match v.as_object() {
        Some(obj) => {
            obj.contains_key("subject")
                && obj.contains_key("predicate")
                && obj.contains_key("object")
        }
        None => false,
    }
}

fn is_json_scalar(v: &Value) -> bool {
    matches!(v, Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_))
}

fn format_scalar_str(value: &Value) -> String {
    match value {
        Value::Null => "null".to_owned(),
        Value::Bool(true) => "true".to_owned(),
        Value::Bool(false) => "false".to_owned(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => {
            if must_quote_string(s) {
                quote_string(s)
            } else {
                s.clone()
            }
        }
        Value::Array(_) | Value::Object(_) => {
            // Shouldn't reach here from scalar contexts, but be safe.
            quote_string(&value.to_string())
        }
    }
}

fn format_list_item(value: &Value) -> String {
    if let Value::String(s) = value {
        if s.contains(',') {
            return quote_string(s);
        }
    }
    format_scalar_str(value)
}

fn format_table_cell(value: &Value) -> String {
    match value {
        Value::Null => "null".to_owned(),
        Value::Bool(true) => "true".to_owned(),
        Value::Bool(false) => "false".to_owned(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => {
            if must_quote_table_cell(s) {
                quote_string(s)
            } else {
                s.clone()
            }
        }
        Value::Array(_) | Value::Object(_) => {
            quote_string(&value.to_string())
        }
    }
}

fn must_quote_string(s: &str) -> bool {
    if s.is_empty() {
        return true;
    }
    if matches!(s, "null" | "true" | "false") {
        return true;
    }
    if parse_integer(s).is_some() || parse_decimal(s).is_some() {
        return true;
    }
    // Inline list literal [...]
    if s.starts_with('[') && s.ends_with(']') && s.len() >= 2 {
        return true;
    }
    if s.contains('"') || s.contains('\\') || s.contains('\n') || s.contains('\t') {
        return true;
    }
    if s.contains(' ') || s.contains('#') {
        return true;
    }
    false
}

fn must_quote_table_cell(s: &str) -> bool {
    if s.is_empty() {
        return true;
    }
    if matches!(s, "null" | "true" | "false") {
        return true;
    }
    if parse_integer(s).is_some() || parse_decimal(s).is_some() {
        return true;
    }
    if s.starts_with('[') && s.ends_with(']') && s.len() >= 2 {
        return true;
    }
    if s.contains('"') || s.contains('\\') || s.contains('\n') || s.contains('\t') {
        return true;
    }
    false
}

fn quote_string(s: &str) -> String {
    let escaped = s
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\t', "\\t");
    format!("\"{escaped}\"")
}
