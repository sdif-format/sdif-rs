use sdif::parse_text;
use sdif::validation::{validate, Schema, SchemaError};

#[test]
fn validate_empty_doc_against_empty_schema_ok() {
    let doc = parse_text("@sdif 1.0\n").unwrap();
    let schema = Schema::default();
    let diags = validate(&doc, &schema);
    assert!(diags.is_empty());
}

#[test]
fn schema_from_document_requires_kind_schema() {
    let doc = parse_text("@sdif 1.0\nkind Agent\n").unwrap();
    let result = Schema::from_document(&doc);
    assert!(matches!(result, Err(SchemaError(_))));
}

#[test]
fn schema_from_document_ok_for_schema_doc() {
    let doc = parse_text("@sdif 1.0\nkind Schema\n").unwrap();
    let schema = Schema::from_document(&doc).unwrap();
    assert!(schema.required_fields.is_empty());
}

#[test]
fn schema_from_document_loads_required_fields() {
    let schema_doc =
        parse_text("@sdif 1.0\nkind Schema\nrequired[field]:\n  id\n  name\n").unwrap();
    let schema = Schema::from_document(&schema_doc).unwrap();
    assert!(schema.required_fields.contains("id"));
    assert!(schema.required_fields.contains("name"));
}

#[test]
fn validate_reports_missing_required_field_with_rich_diagnostic() {
    let schema_doc = parse_text("@sdif 1.0\nkind Schema\nrequired[field]:\n  id\n").unwrap();
    let schema = Schema::from_document(&schema_doc).unwrap();
    let doc = parse_text("@sdif 1.0\nkind Agent\n").unwrap();
    let diagnostics = validate(&doc, &schema);
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, "SDIF_MISSING_FIELD");
    assert_eq!(diagnostics[0].severity, "error");
    assert_eq!(diagnostics[0].path, "$.id");
    assert!(diagnostics[0].message.contains("id"));
}

#[test]
fn schema_from_document_loads_field_types() {
    let schema_doc =
        parse_text("@sdif 1.0\nkind Schema\nfield_types[field,type]:\n  age	Integer\n").unwrap();
    let schema = Schema::from_document(&schema_doc).unwrap();
    assert_eq!(
        schema.field_types.get("age").map(String::as_str),
        Some("Integer")
    );
}

#[test]
fn validate_reports_field_type_mismatch() {
    let schema_doc =
        parse_text("@sdif 1.0\nkind Schema\nfield_types[field,type]:\n  age	Integer\n").unwrap();
    let schema = Schema::from_document(&schema_doc).unwrap();
    let doc = parse_text("@sdif 1.0\nage nope\n").unwrap();
    let diagnostics = validate(&doc, &schema);
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, "SDIF_TYPE_MISMATCH");
    assert_eq!(diagnostics[0].path, "$.age");
}

#[test]
fn schema_from_document_loads_required_table_columns() {
    let schema_doc = parse_text(
        "@sdif 1.0\nkind Schema\nrequired_columns[table,column]:\n  users	id\n  users	email\n",
    )
    .unwrap();
    let schema = Schema::from_document(&schema_doc).unwrap();
    let users = schema.required_table_columns.get("users").unwrap();
    assert!(users.contains("id"));
    assert!(users.contains("email"));
}

#[test]
fn validate_reports_missing_required_table_column() {
    let schema_doc =
        parse_text("@sdif 1.0\nkind Schema\nrequired_columns[table,column]:\n  users	email\n")
            .unwrap();
    let schema = Schema::from_document(&schema_doc).unwrap();
    let doc = parse_text("@sdif 1.0\nusers[id,name]:\n  1	Alice\n").unwrap();
    let diagnostics = validate(&doc, &schema);
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, "SDIF_MISSING_COLUMN");
    assert_eq!(diagnostics[0].path, "$.users");
}
