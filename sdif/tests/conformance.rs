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
    let Statement::Table(t) = &doc.statements[0] else { panic!("expected Table") };
    assert_eq!(t.quoted_columns, Vec::<usize>::new());
}
