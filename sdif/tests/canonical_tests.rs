//! Canonical serializer conformance tests.
//!
//! For each valid conformance case that ships a `canonical.sdif` fixture,
//! we verify that `canonicalize()` produces byte-for-byte identical output
//! and that `sdif_hash()` matches `canonical.sha256`.

use sdif::{canonicalize, parser::parse_text, sdif_hash};

const SDIF_ROOT: &str = "../../sdif-spec";

fn case_dir(name: &str) -> std::path::PathBuf {
    std::path::Path::new(SDIF_ROOT)
        .join("conformance/cases")
        .join(name)
}

fn read_case_file(case: &str, filename: &str) -> String {
    let path = case_dir(case).join(filename);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read {:?}: {}", path, e))
}

// ---------------------------------------------------------------------------
// core-agent
// ---------------------------------------------------------------------------

#[test]
fn canonical_core_agent() {
    let doc = parse_text(&read_case_file("core-agent", "source.sdif")).unwrap();
    let expected = read_case_file("core-agent", "canonical.sdif");
    let got = canonicalize(&doc);
    assert_eq!(got, expected, "canonical mismatch for core-agent");
}

#[test]
fn hash_core_agent() {
    let doc = parse_text(&read_case_file("core-agent", "source.sdif")).unwrap();
    let expected = read_case_file("core-agent", "canonical.sha256")
        .trim()
        .to_string();
    assert_eq!(sdif_hash(&doc), expected, "hash mismatch for core-agent");
}

// ---------------------------------------------------------------------------
// version-valid
// ---------------------------------------------------------------------------

#[test]
fn canonical_version_valid() {
    let doc = parse_text(&read_case_file("version-valid", "source.sdif")).unwrap();
    let expected = read_case_file("version-valid", "canonical.sdif");
    let got = canonicalize(&doc);
    assert_eq!(got, expected, "canonical mismatch for version-valid");
}

#[test]
fn hash_version_valid() {
    let doc = parse_text(&read_case_file("version-valid", "source.sdif")).unwrap();
    let expected = read_case_file("version-valid", "canonical.sha256")
        .trim()
        .to_string();
    assert_eq!(sdif_hash(&doc), expected, "hash mismatch for version-valid");
}

// ---------------------------------------------------------------------------
// ai-alias-header
// ---------------------------------------------------------------------------

#[test]
fn canonical_ai_alias_header() {
    let doc = parse_text(&read_case_file("ai-alias-header", "source.sdif")).unwrap();
    let expected = read_case_file("ai-alias-header", "canonical.sdif");
    let got = canonicalize(&doc);
    assert_eq!(got, expected, "canonical mismatch for ai-alias-header");
}

#[test]
fn hash_ai_alias_header() {
    let doc = parse_text(&read_case_file("ai-alias-header", "source.sdif")).unwrap();
    let expected = read_case_file("ai-alias-header", "canonical.sha256")
        .trim()
        .to_string();
    assert_eq!(
        sdif_hash(&doc),
        expected,
        "hash mismatch for ai-alias-header"
    );
}

// ---------------------------------------------------------------------------
// ai-compact-table
// ---------------------------------------------------------------------------

#[test]
fn canonical_ai_compact_table() {
    let doc = parse_text(&read_case_file("ai-compact-table", "source.sdif")).unwrap();
    let expected = read_case_file("ai-compact-table", "canonical.sdif");
    let got = canonicalize(&doc);
    assert_eq!(got, expected, "canonical mismatch for ai-compact-table");
}

#[test]
fn hash_ai_compact_table() {
    let doc = parse_text(&read_case_file("ai-compact-table", "source.sdif")).unwrap();
    let expected = read_case_file("ai-compact-table", "canonical.sha256")
        .trim()
        .to_string();
    assert_eq!(
        sdif_hash(&doc),
        expected,
        "hash mismatch for ai-compact-table"
    );
}

// ---------------------------------------------------------------------------
// ai-relation-quoted-object
// ---------------------------------------------------------------------------

#[test]
fn canonical_ai_relation_quoted_object() {
    let doc =
        parse_text(&read_case_file("ai-relation-quoted-object", "source.sdif")).unwrap();
    let expected = read_case_file("ai-relation-quoted-object", "canonical.sdif");
    let got = canonicalize(&doc);
    assert_eq!(
        got,
        expected,
        "canonical mismatch for ai-relation-quoted-object"
    );
}

#[test]
fn hash_ai_relation_quoted_object() {
    let doc =
        parse_text(&read_case_file("ai-relation-quoted-object", "source.sdif")).unwrap();
    let expected = read_case_file("ai-relation-quoted-object", "canonical.sha256")
        .trim()
        .to_string();
    assert_eq!(
        sdif_hash(&doc),
        expected,
        "hash mismatch for ai-relation-quoted-object"
    );
}

// ---------------------------------------------------------------------------
// ai-roundtrip-safe
// ---------------------------------------------------------------------------

#[test]
fn canonical_ai_roundtrip_safe() {
    let doc = parse_text(&read_case_file("ai-roundtrip-safe", "source.sdif")).unwrap();
    let expected = read_case_file("ai-roundtrip-safe", "canonical.sdif");
    let got = canonicalize(&doc);
    assert_eq!(got, expected, "canonical mismatch for ai-roundtrip-safe");
}

#[test]
fn hash_ai_roundtrip_safe() {
    let doc = parse_text(&read_case_file("ai-roundtrip-safe", "source.sdif")).unwrap();
    let expected = read_case_file("ai-roundtrip-safe", "canonical.sha256")
        .trim()
        .to_string();
    assert_eq!(
        sdif_hash(&doc),
        expected,
        "hash mismatch for ai-roundtrip-safe"
    );
}

// ---------------------------------------------------------------------------
// nested-narrative-canonical
// ---------------------------------------------------------------------------

#[test]
fn canonical_nested_narrative_canonical() {
    let doc =
        parse_text(&read_case_file("nested-narrative-canonical", "source.sdif")).unwrap();
    let expected = read_case_file("nested-narrative-canonical", "canonical.sdif");
    let got = canonicalize(&doc);
    assert_eq!(
        got,
        expected,
        "canonical mismatch for nested-narrative-canonical"
    );
}

#[test]
fn hash_nested_narrative_canonical() {
    let doc =
        parse_text(&read_case_file("nested-narrative-canonical", "source.sdif")).unwrap();
    let expected = read_case_file("nested-narrative-canonical", "canonical.sha256")
        .trim()
        .to_string();
    assert_eq!(
        sdif_hash(&doc),
        expected,
        "hash mismatch for nested-narrative-canonical"
    );
}
