//! Schema definition for SDIF document validation.

/// Schema-driven validation rules for an SDIF document.
#[derive(Debug, Default, Clone)]
pub struct Schema {
    /// Field keys that must be present in the document.
    pub required_fields: Vec<String>,
    /// Field type constraints: key → expected type name (e.g., "string", "int").
    pub field_types: std::collections::HashMap<String, String>,
    /// Enumeration constraints: key → allowed values.
    pub enumerations: std::collections::HashMap<String, Vec<String>>,
    /// Table names allowed in the document. Empty = allow any.
    pub allowed_tables: Vec<String>,
    /// Per-table required columns: table_name → required column names.
    pub required_columns: std::collections::HashMap<String, Vec<String>>,
    /// Allowed relation predicates. Empty = allow any.
    pub allowed_predicates: Vec<String>,
    /// Allowed rule function names. Empty = allow any.
    pub allowed_rule_functions: Vec<String>,
}
