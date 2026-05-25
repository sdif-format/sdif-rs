//! Security and resource policy for the SDIF parser.
//!
//! `Policy` mirrors the Python `sdif.core.policy.Policy` dataclass exactly,
//! including all default values. Create an instance with `Policy::default()`.

use std::collections::HashSet;
use std::path::PathBuf;

/// Identifiers that may not be used as user-defined alias names or targets.
pub const RESERVED_TERMS: &[&str] = &["include", "alias"];

/// Resource limits and permission flags that govern parser behaviour.
///
/// All boolean flags default to `false` (most restrictive). Numeric limits
/// match the Python reference implementation's defaults.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Policy {
    /// Allow `@include` directives.
    pub allow_includes: bool,
    /// Allowlist of filesystem paths from which `@include` directives may load
    /// files. An empty set means **no path is permitted** — callers must
    /// populate this set to enable any file inclusion (Python semantics).
    /// Mirrors `allowed_include_paths: frozenset[Path]` in the Python Policy.
    pub allowed_include_paths: HashSet<PathBuf>,
    /// Allow `@include` directives that reference remote URLs.
    pub allow_remote_includes: bool,
    /// Allow `@schema` directives that reference remote URLs.
    pub allow_remote_schemas: bool,
    /// Maximum document size in bytes before parsing is rejected (1 MB).
    pub max_document_size: usize,
    /// Maximum nesting depth of object blocks.
    pub max_nesting_depth: usize,
    /// Maximum length of any single string value in bytes (64 KB).
    pub max_string_length: usize,
    /// Maximum number of rows in any single table.
    pub max_table_row_count: usize,
    /// Maximum depth of nested `@include` chains.
    pub max_include_depth: usize,
    /// Maximum cumulative size of all included content after expansion (2 MB).
    pub max_expanded_bytes: usize,
    /// Maximum number of alias expansions during parsing.
    pub max_alias_expansion: usize,
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            allow_includes: false,
            allowed_include_paths: HashSet::new(),
            allow_remote_includes: false,
            allow_remote_schemas: false,
            max_document_size: 1_000_000,
            max_nesting_depth: 16,
            max_string_length: 65536,
            max_table_row_count: 10000,
            max_include_depth: 5,
            max_expanded_bytes: 2_000_000,
            max_alias_expansion: 500,
        }
    }
}
