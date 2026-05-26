//! Schema definition for SDIF document validation.

use crate::Document;
use std::collections::{HashMap, HashSet};

/// Error returned when an SDIF document cannot be interpreted as a schema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaError(pub String);

impl std::fmt::Display for SchemaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SchemaError: {}", self.0)
    }
}

impl std::error::Error for SchemaError {}

/// Validation policy for one table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TablePolicy {
    pub name: String,
    pub ordered: bool,
    pub primary_key: Option<String>,
}

/// Validation policy for one relation predicate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationPolicy {
    pub predicate: String,
    pub subject_type: String,
    pub object_type: String,
    pub required: bool,
}

/// Schema-driven validation rules for an SDIF document.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Schema {
    /// Field keys that must be present in the document.
    pub required_fields: HashSet<String>,
    /// Field type constraints: key -> expected type name.
    pub field_types: HashMap<String, String>,
    /// Per-table required columns: table_name -> required column names.
    pub required_table_columns: HashMap<String, HashSet<String>>,
    /// Per-table column type constraints.
    pub table_column_types: HashMap<String, HashMap<String, String>>,
    /// Per-table structural policies.
    pub table_policies: HashMap<String, TablePolicy>,
    /// Per-predicate relation policies.
    pub relation_policies: HashMap<String, RelationPolicy>,

    /// Legacy direct construction support: empty = allow any table.
    pub allowed_tables: Vec<String>,
    /// Legacy direct construction support: table_name -> required columns.
    pub required_columns: HashMap<String, Vec<String>>,
    /// Legacy direct construction support: empty = allow any predicate.
    pub allowed_predicates: Vec<String>,
    /// Legacy direct construction support: empty = allow any rule function.
    pub allowed_rule_functions: Vec<String>,
}

impl Schema {
    /// Build a Schema from a parsed SDIF document whose `kind` field is `Schema`.
    pub fn from_document(doc: &Document) -> Result<Schema, SchemaError> {
        let kind = doc.fields().find(|f| f.key == "kind");
        match kind {
            Some(f) if f.value == "Schema" => {}
            Some(f) => {
                return Err(SchemaError(format!(
                    "expected schema document with `kind Schema`, got `{}`",
                    f.value
                )))
            }
            None => {
                return Err(SchemaError(
                    "expected schema document with `kind Schema`, got `<missing>`".to_string(),
                ))
            }
        }

        Ok(Schema {
            required_fields: load_required_fields(doc),
            field_types: load_field_types(doc),
            required_table_columns: load_required_columns(doc),
            table_column_types: load_column_types(doc),
            table_policies: load_table_policies(doc),
            relation_policies: load_relation_policies(doc),
            ..Schema::default()
        })
    }
}

fn load_required_fields(doc: &Document) -> HashSet<String> {
    doc.tables()
        .filter(|t| t.name == "required" && t.columns.first().is_some_and(|c| c == "field"))
        .flat_map(|t| t.rows.iter().filter_map(|r| r.first().cloned()))
        .collect()
}

fn load_field_types(doc: &Document) -> HashMap<String, String> {
    doc.tables()
        .filter(|t| t.name == "field_types")
        .flat_map(|t| {
            let field_index = t.columns.iter().position(|c| c == "field");
            let type_index = t.columns.iter().position(|c| c == "type");
            t.rows
                .iter()
                .filter_map(move |row| {
                    Some((
                        row.get(field_index?)?.clone(),
                        row.get(type_index?)?.clone(),
                    ))
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn load_required_columns(doc: &Document) -> HashMap<String, HashSet<String>> {
    let mut map: HashMap<String, HashSet<String>> = HashMap::new();
    for table in doc.tables().filter(|t| t.name == "required_columns") {
        let table_index = table.columns.iter().position(|c| c == "table");
        let column_index = table.columns.iter().position(|c| c == "column");
        if let (Some(table_index), Some(column_index)) = (table_index, column_index) {
            for row in &table.rows {
                if let (Some(table_name), Some(column)) =
                    (row.get(table_index), row.get(column_index))
                {
                    map.entry(table_name.clone())
                        .or_default()
                        .insert(column.clone());
                }
            }
        }
    }
    map
}

fn load_column_types(doc: &Document) -> HashMap<String, HashMap<String, String>> {
    let mut map: HashMap<String, HashMap<String, String>> = HashMap::new();
    for table in doc.tables().filter(|t| t.name == "column_types") {
        let table_index = table.columns.iter().position(|c| c == "table");
        let column_index = table.columns.iter().position(|c| c == "column");
        let type_index = table.columns.iter().position(|c| c == "type");
        if let (Some(table_index), Some(column_index), Some(type_index)) =
            (table_index, column_index, type_index)
        {
            for row in &table.rows {
                if let (Some(table_name), Some(column), Some(type_name)) = (
                    row.get(table_index),
                    row.get(column_index),
                    row.get(type_index),
                ) {
                    map.entry(table_name.clone())
                        .or_default()
                        .insert(column.clone(), type_name.clone());
                }
            }
        }
    }
    map
}

fn load_table_policies(doc: &Document) -> HashMap<String, TablePolicy> {
    let mut map = HashMap::new();
    for table in doc.tables().filter(|t| t.name == "table_policies") {
        let name_index = table.columns.iter().position(|c| c == "name");
        let ordered_index = table.columns.iter().position(|c| c == "ordered");
        let primary_key_index = table.columns.iter().position(|c| c == "primary_key");
        if let Some(name_index) = name_index {
            for row in &table.rows {
                if let Some(name) = row.get(name_index) {
                    let ordered = ordered_index
                        .and_then(|i| row.get(i))
                        .map(|value| value != "false")
                        .unwrap_or(true);
                    let primary_key = primary_key_index
                        .and_then(|i| row.get(i))
                        .filter(|value| !value.is_empty())
                        .cloned();
                    map.insert(
                        name.clone(),
                        TablePolicy {
                            name: name.clone(),
                            ordered,
                            primary_key,
                        },
                    );
                }
            }
        }
    }
    map
}

fn load_relation_policies(doc: &Document) -> HashMap<String, RelationPolicy> {
    let mut map = HashMap::new();
    for table in doc.tables().filter(|t| t.name == "relation_policies") {
        let predicate_index = table.columns.iter().position(|c| c == "predicate");
        if let Some(predicate_index) = predicate_index {
            for row in &table.rows {
                if let Some(predicate) = row.get(predicate_index) {
                    let get = |column: &str| {
                        table
                            .columns
                            .iter()
                            .position(|c| c == column)
                            .and_then(|i| row.get(i))
                            .cloned()
                    };
                    map.insert(
                        predicate.clone(),
                        RelationPolicy {
                            predicate: predicate.clone(),
                            subject_type: get("subject_type")
                                .unwrap_or_else(|| "Identifier".to_string()),
                            object_type: get("object_type")
                                .unwrap_or_else(|| "Identifier".to_string()),
                            required: get("required").is_some_and(|value| value == "true"),
                        },
                    );
                }
            }
        }
    }
    map
}
