use sdif::parse_text;
use sdif::schema::Schema;
use sdif::validation::{validate, Severity};

#[test]
fn test_schema_empty_accepts_any_doc() {
    let doc = parse_text("@sdif 1.0\nname \"Alice\"\n").unwrap();
    assert!(validate(&doc, &Schema::default()).is_empty());
}

#[test]
fn test_schema_required_field_missing_produces_diagnostic() {
    let doc = parse_text("@sdif 1.0\nage \"30\"\n").unwrap();
    let mut schema = Schema::default();
    schema.required_fields.push("name".to_string());
    let diagnostics = validate(&doc, &schema);
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].severity, Severity::Error);
    assert!(diagnostics[0].message.contains("name"));
}

#[test]
fn test_schema_required_field_present_passes() {
    let doc = parse_text("@sdif 1.0\nname \"Alice\"\n").unwrap();
    let mut schema = Schema::default();
    schema.required_fields.push("name".to_string());
    assert!(validate(&doc, &schema).is_empty());
}

#[test]
fn test_schema_allowed_tables_rejects_unknown() {
    let doc = parse_text("@sdif 1.0\nitems[id,name]:\n  1\tfoo\n").unwrap();
    let mut schema = Schema::default();
    schema.allowed_tables.push("users".to_string());
    let diagnostics = validate(&doc, &schema);
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].severity, Severity::Error);
    assert!(diagnostics[0].message.contains("items"));
}

#[test]
fn test_schema_allowed_tables_empty_allows_any() {
    let doc = parse_text("@sdif 1.0\nitems[id,name]:\n  1\tfoo\n").unwrap();
    let schema = Schema::default(); // empty allowed_tables = allow any
    assert!(validate(&doc, &schema).is_empty());
}

#[test]
fn test_schema_required_columns_missing_produces_diagnostic() {
    let doc = parse_text("@sdif 1.0\nusers[id,name]:\n  1\tAlice\n").unwrap();
    let mut schema = Schema::default();
    schema
        .required_columns
        .insert("users".to_string(), vec!["id".to_string(), "email".to_string()]);
    let diagnostics = validate(&doc, &schema);
    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].message.contains("email"));
}

#[test]
fn test_schema_allowed_predicates_rejects_unknown() {
    let doc = parse_text("@sdif 1.0\nrel:\n  alice knows bob\n").unwrap();
    let mut schema = Schema::default();
    schema.allowed_predicates.push("likes".to_string());
    let diagnostics = validate(&doc, &schema);
    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].message.contains("knows"));
}
