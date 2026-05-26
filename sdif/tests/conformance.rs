//! Conformance tests for the SDIF parser.
//!
//! Reads fixtures from `../sdif-spec/conformance/` and verifies that the parser
//! accepts all valid cases and rejects all invalid cases with the expected
//! error code.

use sdif::parser::parse_text;
use std::path::Path;

const SDIF_ROOT: &str = "../../sdif-spec";

fn fixture_path(relative: &str) -> std::path::PathBuf {
    Path::new(SDIF_ROOT).join(relative)
}

fn read_fixture(relative: &str) -> String {
    let path = fixture_path(relative);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read fixture {:?}: {}", path, e))
}

// ---------------------------------------------------------------------------
// Valid cases — must parse successfully
// ---------------------------------------------------------------------------

#[test]
fn valid_core_agent() {
    let text = read_fixture("conformance/cases/core-agent/source.sdif");
    let result = parse_text(&text);
    assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
}

#[test]
fn valid_ai_compact_table() {
    let text = read_fixture("conformance/cases/ai-compact-table/source.sdif");
    let result = parse_text(&text);
    assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
}

#[test]
fn valid_ai_alias_header() {
    let text = read_fixture("conformance/cases/ai-alias-header/source.sdif");
    let result = parse_text(&text);
    assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
}

#[test]
fn valid_version_valid() {
    let text = read_fixture("conformance/cases/version-valid/source.sdif");
    let result = parse_text(&text);
    assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
}

#[test]
fn valid_nested_narrative_canonical() {
    let text = read_fixture("conformance/cases/nested-narrative-canonical/source.sdif");
    let result = parse_text(&text);
    assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
}

#[test]
fn valid_ai_roundtrip_safe() {
    let text = read_fixture("conformance/cases/ai-roundtrip-safe/source.sdif");
    let result = parse_text(&text);
    assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
}

#[test]
fn valid_ai_relation_quoted_object() {
    let text = read_fixture("conformance/cases/ai-relation-quoted-object/source.sdif");
    let result = parse_text(&text);
    assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
}

// ---------------------------------------------------------------------------
// Invalid cases — must fail with specific error code
// ---------------------------------------------------------------------------

#[test]
fn invalid_alias_collision() {
    let text = read_fixture("conformance/invalid/alias_collision.sdif");
    let result = parse_text(&text);
    assert!(result.is_err(), "Expected Err, got Ok");
    let err = result.unwrap_err();
    assert_eq!(
        err.code, "SDIF_POLICY_ALIAS_COLLISION",
        "Wrong error code. Got: {:?}",
        err
    );
}

#[test]
fn invalid_alias_reserved() {
    let text = read_fixture("conformance/invalid/alias_reserved.sdif");
    let result = parse_text(&text);
    assert!(result.is_err(), "Expected Err, got Ok");
    let err = result.unwrap_err();
    assert_eq!(
        err.code, "SDIF_POLICY_ALIAS_RESERVED",
        "Wrong error code. Got: {:?}",
        err
    );
}

#[test]
fn invalid_directive_unknown() {
    let text = read_fixture("conformance/invalid/directive_unknown.sdif");
    let result = parse_text(&text);
    assert!(result.is_err(), "Expected Err, got Ok");
    let err = result.unwrap_err();
    assert_eq!(
        err.code, "SDIF_DIRECTIVE_UNKNOWN",
        "Wrong error code. Got: {:?}",
        err
    );
}

#[test]
fn invalid_narrative_bad_close() {
    let text = read_fixture("conformance/invalid/nested_narrative_bad_close.sdif");
    let result = parse_text(&text);
    assert!(result.is_err(), "Expected Err, got Ok");
    let err = result.unwrap_err();
    assert_eq!(
        err.code, "SDIF_NARRATIVE_CLOSE_ALIGN",
        "Wrong error code. Got: {:?}",
        err
    );
}

#[test]
fn invalid_policy_nesting_depth() {
    let text = read_fixture("conformance/invalid/policy_nesting_depth.sdif");
    let result = parse_text(&text);
    assert!(result.is_err(), "Expected Err, got Ok");
    let err = result.unwrap_err();
    assert_eq!(
        err.code, "SDIF_POLICY_NESTING_DEPTH",
        "Wrong error code. Got: {:?}",
        err
    );
}

#[test]
fn invalid_source_grouped_relation() {
    let text = read_fixture("conformance/invalid/source_grouped_relation.sdif");
    let result = parse_text(&text);
    assert!(result.is_err(), "Expected Err, got Ok");
    let err = result.unwrap_err();
    assert_eq!(
        err.code, "SDIF_AI_REL_SUBJECT",
        "Wrong error code. Got: {:?}",
        err
    );
}

#[test]
fn invalid_table_row_comment() {
    let text = read_fixture("conformance/invalid/table_row_comment.sdif");
    let result = parse_text(&text);
    assert!(result.is_err(), "Expected Err, got Ok");
    let err = result.unwrap_err();
    assert_eq!(
        err.code, "SDIF_TABLE_ROW_COMMENT",
        "Wrong error code. Got: {:?}",
        err
    );
}

#[test]
fn invalid_table_too_few_cells() {
    let text = read_fixture("conformance/invalid/table_too_few_cells.sdif");
    let result = parse_text(&text);
    assert!(result.is_err(), "Expected Err, got Ok");
    let err = result.unwrap_err();
    assert_eq!(
        err.code, "SDIF_TABLE_ARITY",
        "Wrong error code. Got: {:?}",
        err
    );
}

#[test]
fn invalid_table_too_many_cells() {
    let text = read_fixture("conformance/invalid/table_too_many_cells.sdif");
    let result = parse_text(&text);
    assert!(result.is_err(), "Expected Err, got Ok");
    let err = result.unwrap_err();
    assert_eq!(
        err.code, "SDIF_TABLE_ARITY",
        "Wrong error code. Got: {:?}",
        err
    );
}

#[test]
fn invalid_version_bad_token() {
    let text = read_fixture("conformance/invalid/version_bad_token.sdif");
    let result = parse_text(&text);
    assert!(result.is_err(), "Expected Err, got Ok");
    let err = result.unwrap_err();
    assert_eq!(
        err.code, "SDIF_VERSION_UNSUPPORTED",
        "Wrong error code. Got: {:?}",
        err
    );
}

#[test]
fn invalid_version_missing() {
    let text = read_fixture("conformance/invalid/version_missing.sdif");
    let result = parse_text(&text);
    assert!(result.is_err(), "Expected Err, got Ok");
    let err = result.unwrap_err();
    assert_eq!(
        err.code, "SDIF_VERSION_MISSING",
        "Wrong error code. Got: {:?}",
        err
    );
}

#[test]
fn invalid_version_unsupported() {
    let text = read_fixture("conformance/invalid/version_unsupported.sdif");
    let result = parse_text(&text);
    assert!(result.is_err(), "Expected Err, got Ok");
    let err = result.unwrap_err();
    assert_eq!(
        err.code, "SDIF_VERSION_UNSUPPORTED",
        "Wrong error code. Got: {:?}",
        err
    );
}

#[test]
fn invalid_scalar_unclosed_quote() {
    let text = read_fixture("conformance/invalid/scalar_unclosed_quote.sdif");
    let result = parse_text(&text);
    assert!(result.is_err(), "Expected Err, got Ok");
    let err = result.unwrap_err();
    assert_eq!(
        err.code, "SDIF_STRING_UNCLOSED",
        "Wrong error code. Got: {:?}",
        err
    );
}

#[test]
fn invalid_scalar_trailing_after_quote() {
    let text = read_fixture("conformance/invalid/scalar_trailing_after_quote.sdif");
    let result = parse_text(&text);
    assert!(result.is_err(), "Expected Err, got Ok");
    let err = result.unwrap_err();
    assert_eq!(
        err.code, "SDIF_STRING_TRAILING",
        "Wrong error code. Got: {:?}",
        err
    );
}

#[test]
fn invalid_directive_empty() {
    let text = read_fixture("conformance/invalid/directive_empty.sdif");
    let result = parse_text(&text);
    assert!(result.is_err(), "Expected Err, got Ok");
    let err = result.unwrap_err();
    assert_eq!(
        err.code, "SDIF_DIRECTIVE",
        "Wrong error code. Got: {:?}",
        err
    );
}

#[test]
fn invalid_field_no_value() {
    let text = read_fixture("conformance/invalid/field_no_value.sdif");
    let result = parse_text(&text);
    assert!(result.is_err(), "Expected Err, got Ok");
    let err = result.unwrap_err();
    assert_eq!(err.code, "SDIF_FIELD", "Wrong error code. Got: {:?}", err);
}

#[test]
fn invalid_indent_tab() {
    let text = read_fixture("conformance/invalid/indent_tab.sdif");
    let result = parse_text(&text);
    assert!(result.is_err(), "Expected Err, got Ok");
    let err = result.unwrap_err();
    assert_eq!(
        err.code, "SDIF_INDENT_TAB",
        "Wrong error code. Got: {:?}",
        err
    );
}

#[test]
fn invalid_indent_unexpected() {
    let text = read_fixture("conformance/invalid/indent_unexpected.sdif");
    let result = parse_text(&text);
    assert!(result.is_err(), "Expected Err, got Ok");
    let err = result.unwrap_err();
    assert_eq!(err.code, "SDIF_INDENT", "Wrong error code. Got: {:?}", err);
}

#[test]
fn invalid_narrative_unclosed() {
    let text = read_fixture("conformance/invalid/narrative_unclosed.sdif");
    let result = parse_text(&text);
    assert!(result.is_err(), "Expected Err, got Ok");
    let err = result.unwrap_err();
    assert_eq!(
        err.code, "SDIF_NARRATIVE_UNCLOSED",
        "Wrong error code. Got: {:?}",
        err
    );
}

#[test]
fn invalid_object_directive() {
    let text = read_fixture("conformance/invalid/object_directive.sdif");
    let result = parse_text(&text);
    assert!(result.is_err(), "Expected Err, got Ok");
    let err = result.unwrap_err();
    assert_eq!(
        err.code, "SDIF_OBJECT_DIRECTIVE",
        "Wrong error code. Got: {:?}",
        err
    );
}

#[test]
fn invalid_relation_arity() {
    let text = read_fixture("conformance/invalid/relation_arity.sdif");
    let result = parse_text(&text);
    assert!(result.is_err(), "Expected Err, got Ok");
    let err = result.unwrap_err();
    assert_eq!(
        err.code, "SDIF_REL_ARITY",
        "Wrong error code. Got: {:?}",
        err
    );
}

#[test]
fn invalid_relation_quote() {
    let text = read_fixture("conformance/invalid/relation_quote.sdif");
    let result = parse_text(&text);
    assert!(result.is_err(), "Expected Err, got Ok");
    let err = result.unwrap_err();
    assert_eq!(
        err.code, "SDIF_REL_QUOTE",
        "Wrong error code. Got: {:?}",
        err
    );
}

#[test]
fn invalid_rule_expr() {
    let text = read_fixture("conformance/invalid/rule_expr.sdif");
    let result = parse_text(&text);
    assert!(result.is_err(), "Expected Err, got Ok");
    let err = result.unwrap_err();
    assert_eq!(
        err.code, "SDIF_RULE_EXPR",
        "Wrong error code. Got: {:?}",
        err
    );
}

#[test]
fn invalid_table_header_empty() {
    let text = read_fixture("conformance/invalid/table_header_empty.sdif");
    let result = parse_text(&text);
    assert!(result.is_err(), "Expected Err, got Ok");
    let err = result.unwrap_err();
    assert_eq!(
        err.code, "SDIF_TABLE_HEADER",
        "Wrong error code. Got: {:?}",
        err
    );
}

#[test]
fn invalid_version_conflict() {
    let text = read_fixture("conformance/invalid/version_conflict.sdif");
    let result = parse_text(&text);
    assert!(result.is_err(), "Expected Err, got Ok");
    let err = result.unwrap_err();
    assert_eq!(
        err.code, "SDIF_VERSION_CONFLICT",
        "Wrong error code. Got: {:?}",
        err
    );
}

#[test]
fn invalid_version_syntax() {
    let text = read_fixture("conformance/invalid/version_syntax.sdif");
    let result = parse_text(&text);
    assert!(result.is_err(), "Expected Err, got Ok");
    let err = result.unwrap_err();
    assert_eq!(
        err.code, "SDIF_VERSION_SYNTAX",
        "Wrong error code. Got: {:?}",
        err
    );
}

// ---------------------------------------------------------------------------
// Table.quoted_columns field
// ---------------------------------------------------------------------------

#[test]
fn table_quoted_columns_is_empty_for_plain_headers() {
    use sdif::Statement;
    let src = "@sdif 1.0\npeople[name,age]:\n  alice\t30\n";
    let doc = sdif::parser::parse_text(src).unwrap();
    let Statement::Table(t) = &doc.statements[0] else {
        panic!("expected Table")
    };
    assert_eq!(t.quoted_columns, Vec::<usize>::new());
}

#[test]
fn table_tracks_quoted_columns() {
    use sdif::Statement;
    let src = "@sdif 1.0\npeople[id,name]:\n  1\t\"Alice\"\n  2\t\"Bob\"\n";
    let doc = sdif::parser::parse_text(src).unwrap();
    let Statement::Table(t) = &doc.statements[0] else {
        panic!("expected Table")
    };
    // column index 1 ("name") is always quoted
    assert!(t.quoted_columns.contains(&1), "column 1 should be quoted");
    assert!(
        !t.quoted_columns.contains(&0),
        "column 0 should not be quoted"
    );
}

#[test]
fn table_quoted_columns_empty_when_no_quotes() {
    use sdif::Statement;
    let src = "@sdif 1.0\npeople[id,name]:\n  1\talice\n  2\tbob\n";
    let doc = sdif::parser::parse_text(src).unwrap();
    let Statement::Table(t) = &doc.statements[0] else {
        panic!("expected Table")
    };
    assert!(t.quoted_columns.is_empty());
}

// ---------------------------------------------------------------------------
// Accessor iterator methods
// ---------------------------------------------------------------------------

#[test]
fn document_fields_accessor_returns_fields() {
    let src = "@sdif 1.0\nkind Agent\nid foo\n";
    let doc = sdif::parser::parse_text(src).unwrap();
    let keys: Vec<&str> = doc.fields().map(|f| f.key.as_str()).collect();
    assert_eq!(keys, ["kind", "id"]);
}

#[test]
fn document_tables_accessor_returns_tables() {
    let src = "@sdif 1.0\nitems[id,val]:\n  a\t1\n";
    let doc = sdif::parser::parse_text(src).unwrap();
    let names: Vec<&str> = doc.tables().map(|t| t.name.as_str()).collect();
    assert_eq!(names, ["items"]);
}

#[test]
fn object_block_fields_accessor() {
    use sdif::Statement;
    let src = "@sdif 1.0\ncontact:\n  name Alice\n  email a@b.com\n";
    let doc = sdif::parser::parse_text(src).unwrap();
    let Statement::ObjectBlock(obj) = &doc.statements[0] else {
        panic!("expected ObjectBlock")
    };
    let keys: Vec<&str> = obj.fields().map(|f| f.key.as_str()).collect();
    assert_eq!(keys, ["name", "email"]);
}

#[test]
fn policy_allowed_include_paths_defaults_empty() {
    use sdif::Policy;
    let p = Policy::default();
    assert!(p.allowed_include_paths.is_empty());
}

// ---------------------------------------------------------------------------
// parse_file() — include resolution tests
// ---------------------------------------------------------------------------

use std::fs;
use std::path::PathBuf;

fn temp_sdif(name: &str, content: &str) -> PathBuf {
    let path = std::env::temp_dir().join(name);
    fs::write(&path, content).unwrap();
    path
}

#[test]
fn parse_file_basic() {
    let path = temp_sdif("pf_basic.sdif", "@sdif 1.0\nkind Test\n");
    let doc = sdif::parser::parse_file(&path, &sdif::Policy::default()).unwrap();
    assert_eq!(
        doc.fields().find(|f| f.key == "kind").unwrap().value,
        "Test"
    );
}

#[test]
fn parse_file_include_disabled_by_default() {
    let inc = temp_sdif("pf_inc.sdif", "@sdif 1.0\nkind Inner\n");
    let main_content = format!("@sdif 1.0\n@include \"{}\"\n", inc.display());
    let main = temp_sdif("pf_main.sdif", &main_content);
    let err = sdif::parser::parse_file(&main, &sdif::Policy::default()).unwrap_err();
    assert_eq!(err.code, "SDIF_POLICY_INCLUDE");
}

#[test]
fn parse_file_cycle_detected() {
    use std::collections::HashSet;
    let dir = std::env::temp_dir();
    let a = dir.join("pf_cycle_a.sdif");
    let b = dir.join("pf_cycle_b.sdif");
    fs::write(&a, format!("@sdif 1.0\n@include \"{}\"\n", b.display())).unwrap();
    fs::write(&b, format!("@sdif 1.0\n@include \"{}\"\n", a.display())).unwrap();
    let policy = sdif::Policy {
        allow_includes: true,
        allowed_include_paths: HashSet::from([dir.clone()]),
        ..sdif::Policy::default()
    };
    let err = sdif::parser::parse_file(&a, &policy).unwrap_err();
    assert_eq!(err.code, "SDIF_POLICY_INCLUDE_CYCLE");
}

// ---------------------------------------------------------------------------
// JSON module
// ---------------------------------------------------------------------------

#[test]
fn to_json_scalar_fields() {
    use sdif::document_to_json;
    let src = "@sdif 1.0\nkind Agent\nid foo\n";
    let doc = sdif::parser::parse_text(src).unwrap();
    let json = document_to_json(&doc);
    assert_eq!(json["kind"], "Agent");
    assert_eq!(json["id"], "foo");
}

#[test]
fn to_json_table_rows() {
    use sdif::document_to_json;
    // IMPORTANT: table rows use HTAB separators
    let src = "@sdif 1.0\npeople[id,age]:\n  alice\t30\n  bob\t25\n";
    let doc = sdif::parser::parse_text(src).unwrap();
    let json = document_to_json(&doc);
    assert!(json["people"].is_array());
    assert_eq!(json["people"][0]["id"], "alice");
    assert_eq!(json["people"][0]["age"], "30");
}

#[test]
fn from_json_produces_parseable_sdif() {
    use sdif::json_to_sdif;
    let data = serde_json::json!({"kind": "Test", "id": "x1"});
    let sdif_text = json_to_sdif(&data);
    let doc = sdif::parser::parse_text(&sdif_text).unwrap();
    assert_eq!(doc.fields().find(|f| f.key == "id").unwrap().value, "x1");
}

#[test]
fn ai_view_returns_sdif_ai_header() {
    let doc = sdif::parser::parse_text("@sdif 1.0\nkind Agent\n").unwrap();
    let view = sdif::ai_view(&doc);
    assert!(view.contains("@sdif.ai"), "ai_view must emit @sdif.ai header");
    sdif::parser::parse_text(&view).expect("ai_view output must be valid SDIF");
}

#[test]
fn ai_view_conformance_ai_alias_header_parses() {
    let src = std::fs::read_to_string(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../sdif-spec/conformance/cases/ai-alias-header/source.sdif"
        )
    )
    .unwrap();
    let doc = sdif::parser::parse_text(&src).unwrap();
    let view = sdif::ai_view(&doc);
    sdif::parser::parse_text(&view).expect("ai_view output must be valid SDIF");
}
