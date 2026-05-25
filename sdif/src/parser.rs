//! Normative parser for SDIF v1 source and AI projection syntax.
//!
//! Ported from the Python reference implementation in
//! `sdif/src/sdif/parser/parser.py`. Span tracking is added to every AST
//! node; all error codes are identical to the Python implementation.

use crate::ast::{
    Directive, Document, Field, Narrative, ObjectBlock, Relation, Rule, RuleArg, RuleCall,
    RuleExpression, Statement, Table,
};
use crate::error::{ParseError, PolicyError};
use crate::policy::{Policy, RESERVED_TERMS};
use crate::span::Span;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Parse an SDIF document from text using the default `Policy`.
pub fn parse_text(text: &str) -> Result<Document, ParseError> {
    parse_text_with_policy(text, &Policy::default())
}

/// Parse an SDIF document from text using a caller-supplied `Policy`.
pub fn parse_text_with_policy(text: &str, policy: &Policy) -> Result<Document, ParseError> {
    let mut p = Parser::new(text, policy)?;
    p.parse_document()
}

// ---------------------------------------------------------------------------
// Internal parser state
// ---------------------------------------------------------------------------

struct Parser<'a> {
    policy: &'a Policy,
    lines: Vec<String>,
    index: usize,
    is_ai_profile: bool,
    format_directive_name: Option<String>,
    /// Maps alias name → canonical name (e.g. "e" → "rel").
    alias_to_canonical: std::collections::HashMap<String, String>,
    current_nesting_depth: usize,
}

impl<'a> Parser<'a> {
    fn new(text: &str, policy: &'a Policy) -> Result<Self, ParseError> {
        // Policy: document size (UTF-8 bytes; Rust `str::len()` is bytes).
        if text.len() > policy.max_document_size {
            return Err(ParseError::from_policy(PolicyError::new(
                "SDIF_POLICY_DOCUMENT_SIZE",
                format!(
                    "Document size exceeds maximum limit of {} bytes",
                    policy.max_document_size
                ),
            )));
        }

        // Normalise line endings exactly as the Python reference does.
        let normalised = text.replace("\r\n", "\n").replace('\r', "\n");
        let mut lines: Vec<String> = normalised.split('\n').map(str::to_owned).collect();
        // Python pops the trailing empty string produced by a final newline.
        if lines.last().map(String::is_empty).unwrap_or(false) {
            lines.pop();
        }

        Ok(Self {
            policy,
            lines,
            index: 0,
            is_ai_profile: false,
            format_directive_name: None,
            alias_to_canonical: std::collections::HashMap::new(),
            current_nesting_depth: 0,
        })
    }

    // -----------------------------------------------------------------------
    // Policy helpers
    // -----------------------------------------------------------------------

    fn check_string_length(&self, value: &str, field_desc: &str) -> Result<(), ParseError> {
        if value.len() > self.policy.max_string_length {
            return Err(ParseError::from_policy(PolicyError::new(
                "SDIF_POLICY_STRING_LENGTH",
                format!(
                    "{field_desc} length {} exceeds maximum limit of {}",
                    value.len(),
                    self.policy.max_string_length
                ),
            )));
        }
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Top-level document
    // -----------------------------------------------------------------------

    fn parse_document(&mut self) -> Result<Document, ParseError> {
        let mut directives: Vec<Directive> = Vec::new();
        let mut statements: Vec<Statement> = Vec::new();

        while self.index < self.lines.len() {
            match self.parse_next(0)? {
                ParsedItem::Nothing => {}
                ParsedItem::Dir(d) => directives.push(d),
                ParsedItem::Stmt(s) => statements.push(s),
                ParsedItem::Stmts(ss) => statements.extend(ss),
            }
        }

        if self.format_directive_name.is_none() {
            return Err(ParseError::new(
                "SDIF_VERSION_MISSING",
                "document must declare @sdif 1.0 or @sdif.ai 1.0",
                Span::single_line(1, 1, 1),
            ));
        }

        Ok(Document {
            directives,
            statements,
        })
    }

    // -----------------------------------------------------------------------
    // Dispatch — mirrors Python's `_parse_next`
    // -----------------------------------------------------------------------

    fn parse_next(&mut self, indent: u32) -> Result<ParsedItem, ParseError> {
        let line_no = (self.index + 1) as u32;
        let raw = self.lines[self.index].clone();

        // Skip blank lines and comment-only lines.
        if raw.trim().is_empty() || raw.trim_start().starts_with('#') {
            self.index += 1;
            return Ok(ParsedItem::Nothing);
        }

        let actual_indent = compute_indent(&raw, line_no)?;

        if actual_indent < indent {
            // Caller will handle backing out; return nothing.
            return Ok(ParsedItem::Nothing);
        }
        if actual_indent > indent {
            return Err(ParseError::new(
                "SDIF_INDENT",
                "unexpected indentation",
                Span::single_line(line_no, actual_indent + 1, actual_indent + 1),
            ));
        }

        let body = &raw[indent as usize..];

        // --- directive ---
        if body.starts_with('@') {
            self.index += 1;
            let d = self.parse_directive(body, line_no, indent)?;
            return Ok(ParsedItem::Dir(d));
        }

        // --- alias[a=b,...] ---
        let stripped_body = strip_inline_comment(body);
        if let Some(entries) = parse_alias_header(&stripped_body) {
            self.index += 1;
            let dir = self.apply_aliases(&entries, line_no)?;
            return Ok(ParsedItem::Dir(dir));
        }

        // --- narrative: key """ ---
        if let Some(key) = parse_narrative_opener(body) {
            let stmt = self.parse_narrative(&key, indent, line_no)?;
            return Ok(ParsedItem::Stmt(Statement::Narrative(stmt)));
        }

        // --- table header ---
        if let Some((name, cols_raw)) = parse_table_header(&stripped_body) {
            if self.is_relation_subject_header(&name) {
                if !self.is_ai_profile {
                    return Err(ParseError::new(
                        "SDIF_AI_REL_SUBJECT",
                        "rel[subject]: syntax is only valid in .sdif.ai documents",
                        Span::single_line(line_no, indent + 1, indent + 1 + body.len() as u32),
                    ));
                }
                let rels = self.parse_relations_for_subject(&cols_raw, indent, line_no)?;
                return Ok(ParsedItem::Stmts(
                    rels.into_iter().map(Statement::Relation).collect(),
                ));
            }
            let tbl = self.parse_table(&name, &cols_raw, indent, line_no)?;
            return Ok(ParsedItem::Stmt(Statement::Table(tbl)));
        }

        // --- block opener ---
        if let Some(key) = parse_block_key(&stripped_body) {
            if key == "rel" || self.alias_to_canonical.get(&key).map(|s| s.as_str()) == Some("rel")
            {
                let rels = self.parse_relations(indent, line_no)?;
                return Ok(ParsedItem::Stmts(
                    rels.into_iter().map(Statement::Relation).collect(),
                ));
            }
            if key == "rules" {
                let rules = self.parse_rules(indent, line_no)?;
                return Ok(ParsedItem::Stmts(
                    rules.into_iter().map(Statement::Rule).collect(),
                ));
            }
            let obj = self.parse_object(&key, indent, line_no)?;
            return Ok(ParsedItem::Stmt(Statement::ObjectBlock(obj)));
        }

        // --- field ---
        self.index += 1;
        let field = parse_field(body, line_no, indent, self.policy)?;
        Ok(ParsedItem::Stmt(Statement::Field(field)))
    }

    // -----------------------------------------------------------------------
    // Directive
    // -----------------------------------------------------------------------

    fn parse_directive(
        &mut self,
        body: &str,
        line_no: u32,
        indent: u32,
    ) -> Result<Directive, ParseError> {
        let clean = strip_inline_comment(body);
        let parts: Vec<&str> = clean[1..].split_whitespace().collect();
        if parts.is_empty() {
            return Err(ParseError::new(
                "SDIF_DIRECTIVE",
                "empty directive",
                Span::single_line(line_no, indent + 1, indent + 1 + body.len() as u32),
            ));
        }
        let name = parts[0];
        let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();

        const KNOWN: &[&str] = &[
            "sdif",
            "sdif.ai",
            "profile",
            "vocab",
            "base",
            "namespace",
            "include",
        ];
        if !KNOWN.contains(&name) {
            return Err(ParseError::new(
                "SDIF_DIRECTIVE_UNKNOWN",
                format!("unknown directive @{name}"),
                Span::single_line(line_no, indent + 1, indent + 1 + body.len() as u32),
            )
            .with_hint("v1 documents only allow known core directives"));
        }

        if name == "sdif" || name == "sdif.ai" {
            if args.len() != 1 {
                return Err(ParseError::new(
                    "SDIF_VERSION_SYNTAX",
                    format!("@{name} requires exactly one version argument"),
                    Span::single_line(line_no, indent + 1, indent + 1 + body.len() as u32),
                ));
            }
            if args[0] != "1.0" {
                return Err(ParseError::new(
                    "SDIF_VERSION_UNSUPPORTED",
                    format!("unsupported @{name} version `{}`", args[0]),
                    Span::single_line(line_no, indent + 1, indent + 1 + body.len() as u32),
                )
                .with_hint("this implementation supports format version 1.0"));
            }
            if self.format_directive_name.is_some() {
                return Err(ParseError::new(
                    "SDIF_VERSION_CONFLICT",
                    "document must declare exactly one format version directive",
                    Span::single_line(line_no, indent + 1, indent + 1 + body.len() as u32),
                ));
            }
            self.format_directive_name = Some(name.to_string());
        }
        if name == "sdif.ai" {
            self.is_ai_profile = true;
        }

        Ok(Directive {
            name: name.to_string(),
            args,
        })
    }

    // -----------------------------------------------------------------------
    // Alias
    // -----------------------------------------------------------------------

    fn apply_aliases(&mut self, entries: &[String], line_no: u32) -> Result<Directive, ParseError> {
        for entry in entries {
            let Some(eq) = entry.find('=') else {
                return Err(ParseError::new(
                    "SDIF_ALIAS_SYNTAX",
                    "invalid alias entry syntax",
                    Span::single_line(line_no, 1, 1),
                ));
            };
            let alias_name = entry[..eq].trim();
            let canonical_name = entry[eq + 1..].trim();

            // Policy: reserved terms.
            if RESERVED_TERMS.contains(&alias_name) || RESERVED_TERMS.contains(&canonical_name) {
                return Err(ParseError::from_policy(PolicyError::new(
                    "SDIF_POLICY_ALIAS_RESERVED",
                    format!("Alias entry '{entry}' uses or targets a reserved term"),
                )));
            }
            // Policy: collision.
            if let Some(existing) = self.alias_to_canonical.get(alias_name) {
                if existing != canonical_name {
                    return Err(ParseError::from_policy(PolicyError::new(
                        "SDIF_POLICY_ALIAS_COLLISION",
                        format!(
                            "Alias collision: '{alias_name}' is mapped to both '{existing}' and '{canonical_name}'"
                        ),
                    )));
                }
            }
            self.alias_to_canonical
                .insert(alias_name.to_string(), canonical_name.to_string());
        }
        Ok(Directive {
            name: "alias".to_string(),
            args: entries.to_vec(),
        })
    }

    // -----------------------------------------------------------------------
    // Object block
    // -----------------------------------------------------------------------

    fn parse_object(
        &mut self,
        key: &str,
        indent: u32,
        line_no: u32,
    ) -> Result<ObjectBlock, ParseError> {
        self.current_nesting_depth += 1;
        if self.current_nesting_depth > self.policy.max_nesting_depth {
            let depth = self.current_nesting_depth;
            self.current_nesting_depth -= 1;
            return Err(ParseError::from_policy(PolicyError::new(
                "SDIF_POLICY_NESTING_DEPTH",
                format!(
                    "Nesting depth {depth} exceeds maximum limit of {}",
                    self.policy.max_nesting_depth
                ),
            )));
        }

        self.index += 1;
        let child_indent = indent + 2;
        let mut statements: Vec<Statement> = Vec::new();

        loop {
            if self.index >= self.lines.len() {
                break;
            }
            let raw = &self.lines[self.index];
            if raw.trim().is_empty() {
                self.index += 1;
                continue;
            }
            let actual = compute_indent(raw, (self.index + 1) as u32)?;
            if actual < child_indent {
                break;
            }
            match self.parse_next(child_indent)? {
                ParsedItem::Nothing => {}
                ParsedItem::Dir(_) => {
                    self.current_nesting_depth -= 1;
                    return Err(ParseError::new(
                        "SDIF_OBJECT_DIRECTIVE",
                        "directive not allowed inside object",
                        Span::single_line(line_no, indent + 1, indent + 1),
                    ));
                }
                ParsedItem::Stmt(s) => statements.push(s),
                ParsedItem::Stmts(ss) => statements.extend(ss),
            }
        }

        self.current_nesting_depth -= 1;
        Ok(ObjectBlock {
            key: key.to_string(),
            statements,
        })
    }

    // -----------------------------------------------------------------------
    // Table
    // -----------------------------------------------------------------------

    fn parse_table(
        &mut self,
        name: &str,
        cols_raw: &str,
        indent: u32,
        line_no: u32,
    ) -> Result<Table, ParseError> {
        let columns: Vec<String> = cols_raw
            .split(',')
            .map(|c| c.trim().to_string())
            .filter(|c| !c.is_empty())
            .collect();

        if columns.is_empty() {
            return Err(ParseError::new(
                "SDIF_TABLE_HEADER",
                "table must declare columns",
                Span::single_line(line_no, indent + 1, indent + 1),
            ));
        }

        self.index += 1;
        let child_indent = indent + 2;
        let mut rows: Vec<Vec<String>> = Vec::new();

        loop {
            if self.index >= self.lines.len() {
                break;
            }
            let row_no = (self.index + 1) as u32;
            let raw = self.lines[self.index].clone();
            if raw.trim().is_empty() {
                self.index += 1;
                continue;
            }
            let actual = compute_indent(&raw, row_no)?;
            let row_text: &str = if actual == child_indent {
                &raw[child_indent as usize..]
            } else if actual == indent && raw[indent as usize..].contains('\t') {
                &raw[indent as usize..]
            } else if actual < child_indent {
                break;
            } else {
                return Err(ParseError::new(
                    "SDIF_INDENT",
                    "invalid table row indentation",
                    Span::single_line(row_no, actual + 1, actual + 1),
                ));
            };

            // Inline comments are prohibited in table rows.
            let stripped = strip_inline_comment(row_text);
            if stripped != row_text.trim_end() {
                return Err(ParseError::new(
                    "SDIF_TABLE_ROW_COMMENT",
                    "inline comments inside table rows are prohibited in strict mode",
                    Span::single_line(row_no, indent + 1, indent + 1 + raw.len() as u32),
                ));
            }

            let cells: Vec<String> = row_text.split('\t').map(str::to_string).collect();
            if cells.len() != columns.len() {
                let msg = format!(
                    "table row has {} cells but header declares {} columns",
                    cells.len(),
                    columns.len()
                );
                return Err(ParseError::new(
                    "SDIF_TABLE_ARITY",
                    msg,
                    Span::single_line(row_no, child_indent + 1, child_indent + 1),
                )
                .with_hint("check HTAB separators and missing cells"));
            }

            for cell in &cells {
                self.check_string_length(cell, "Table cell")?;
            }

            rows.push(cells);

            if rows.len() > self.policy.max_table_row_count {
                return Err(ParseError::from_policy(PolicyError::new(
                    "SDIF_POLICY_TABLE_ROW_COUNT",
                    format!(
                        "Table row count {} exceeds maximum limit of {}",
                        rows.len(),
                        self.policy.max_table_row_count
                    ),
                )));
            }

            self.index += 1;
        }

        Ok(Table {
            name: name.to_string(),
            columns,
            rows,
        })
    }

    // -----------------------------------------------------------------------
    // Relations
    // -----------------------------------------------------------------------

    fn parse_relations(&mut self, indent: u32, _line_no: u32) -> Result<Vec<Relation>, ParseError> {
        self.index += 1;
        let child_indent = indent + 2;
        let mut relations: Vec<Relation> = Vec::new();

        loop {
            if self.index >= self.lines.len() {
                break;
            }
            let row_no = (self.index + 1) as u32;
            let raw = self.lines[self.index].clone();
            if raw.trim().is_empty() {
                self.index += 1;
                continue;
            }
            let actual = compute_indent(&raw, row_no)?;
            if actual < child_indent {
                break;
            }
            if actual != child_indent {
                return Err(ParseError::new(
                    "SDIF_INDENT",
                    "invalid relation row indentation",
                    Span::single_line(row_no, actual + 1, actual + 1),
                ));
            }
            let row_text = &raw[child_indent as usize..];
            let parts =
                split_quoted_whitespace(&strip_inline_comment(row_text), row_no, "SDIF_REL_QUOTE")?;
            if parts.len() != 3 {
                return Err(ParseError::new(
                    "SDIF_REL_ARITY",
                    "relation row must have exactly three parts",
                    Span::single_line(row_no, child_indent + 1, child_indent + 1),
                ));
            }
            let unquoted_obj = unquote(&parts[2]);
            self.check_string_length(&unquoted_obj, "Relation object")?;
            let object_quoted = is_quoted(&parts[2]);
            relations.push(Relation {
                subject: parts[0].clone(),
                predicate: parts[1].clone(),
                object: unquoted_obj,
                object_quoted,
            });
            self.index += 1;
        }
        Ok(relations)
    }

    fn parse_relations_for_subject(
        &mut self,
        subject: &str,
        indent: u32,
        _line_no: u32,
    ) -> Result<Vec<Relation>, ParseError> {
        self.index += 1;
        let child_indent = indent + 2;
        let mut relations: Vec<Relation> = Vec::new();

        loop {
            if self.index >= self.lines.len() {
                break;
            }
            let row_no = (self.index + 1) as u32;
            let raw = self.lines[self.index].clone();
            if raw.trim().is_empty() {
                self.index += 1;
                continue;
            }
            let actual = compute_indent(&raw, row_no)?;
            if actual < child_indent {
                break;
            }
            if actual != child_indent {
                return Err(ParseError::new(
                    "SDIF_INDENT",
                    "invalid relation row indentation",
                    Span::single_line(row_no, actual + 1, actual + 1),
                ));
            }
            let row_text = &raw[child_indent as usize..];
            let parts =
                split_quoted_whitespace(&strip_inline_comment(row_text), row_no, "SDIF_REL_QUOTE")?;
            if parts.len() != 2 {
                return Err(ParseError::new(
                    "SDIF_REL_ARITY",
                    "subject-grouped relation row must have exactly two parts",
                    Span::single_line(row_no, child_indent + 1, child_indent + 1),
                ));
            }
            let unquoted_obj = unquote(&parts[1]);
            self.check_string_length(&unquoted_obj, "Relation object")?;
            let object_quoted = is_quoted(&parts[1]);
            relations.push(Relation {
                subject: subject.to_string(),
                predicate: parts[0].clone(),
                object: unquoted_obj,
                object_quoted,
            });
            self.index += 1;
        }
        Ok(relations)
    }

    // -----------------------------------------------------------------------
    // Rules
    // -----------------------------------------------------------------------

    fn parse_rules(&mut self, indent: u32, _line_no: u32) -> Result<Vec<Rule>, ParseError> {
        self.index += 1;
        let child_indent = indent + 2;
        let mut rules: Vec<Rule> = Vec::new();

        loop {
            if self.index >= self.lines.len() {
                break;
            }
            let row_no = (self.index + 1) as u32;
            let raw = self.lines[self.index].clone();
            if raw.trim().is_empty() {
                self.index += 1;
                continue;
            }
            let actual = compute_indent(&raw, row_no)?;
            if actual < child_indent {
                break;
            }
            if actual != child_indent {
                return Err(ParseError::new(
                    "SDIF_INDENT",
                    "invalid rule row indentation",
                    Span::single_line(row_no, actual + 1, actual + 1),
                ));
            }
            let source = strip_inline_comment(&raw[child_indent as usize..])
                .trim()
                .to_string();
            let expression = tokenize_and_parse_rule(&source, row_no)?;
            rules.push(Rule {
                source,
                expression: Some(expression),
            });
            self.index += 1;
        }
        Ok(rules)
    }

    // -----------------------------------------------------------------------
    // Narrative
    // -----------------------------------------------------------------------

    fn parse_narrative(
        &mut self,
        key: &str,
        indent: u32,
        line_no: u32,
    ) -> Result<Narrative, ParseError> {
        self.index += 1;
        let mut content: Vec<String> = Vec::new();
        let prefix = " ".repeat(indent as usize);

        loop {
            if self.index >= self.lines.len() {
                return Err(ParseError::new(
                    "SDIF_NARRATIVE_UNCLOSED",
                    "unterminated narrative block",
                    Span::single_line(line_no, indent + 1, indent + 1),
                )
                .with_hint(
                    "make sure to close the narrative block with triple quotes aligned to the opening indentation",
                ));
            }
            let raw = self.lines[self.index].clone();
            if raw.trim() == r#"""""# {
                // The close must be exactly prefix + `"""`.
                let expected = format!("{}{}", prefix, r#"""""#);
                if raw != expected {
                    let col = raw.len() - raw.trim_start().len() + 1;
                    return Err(ParseError::new(
                        "SDIF_NARRATIVE_CLOSE_ALIGN",
                        "unterminated narrative block or mismatched alignment at close",
                        Span::single_line((self.index + 1) as u32, col as u32, col as u32),
                    )
                    .with_hint(
                        "closing triple quotes must match the indentation of the opening block",
                    ));
                }
                self.index += 1;
                let content_str = content.join("\n");
                self.check_string_length(&content_str, "Narrative content")?;
                return Ok(Narrative {
                    key: key.to_string(),
                    text: content_str,
                });
            }
            let line_content = if raw.starts_with(&prefix) {
                raw[prefix.len()..].to_string()
            } else {
                raw.clone()
            };
            content.push(line_content);
            self.index += 1;
        }
    }

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn is_relation_subject_header(&self, name: &str) -> bool {
        name == "rel" || self.alias_to_canonical.get(name).map(|s| s.as_str()) == Some("rel")
    }
}

// ---------------------------------------------------------------------------
// Enum returned by parse_next
// ---------------------------------------------------------------------------

enum ParsedItem {
    Nothing,
    Dir(Directive),
    Stmt(Statement),
    Stmts(Vec<Statement>),
}

// ---------------------------------------------------------------------------
// Field parsing (free function — no parser state needed)
// ---------------------------------------------------------------------------

fn parse_field(
    body: &str,
    line_no: u32,
    indent: u32,
    policy: &Policy,
) -> Result<Field, ParseError> {
    let clean = strip_inline_comment(body);
    // A field must contain at least one whitespace separator between key and value.
    if !clean.contains(' ') && !clean.contains('\t') {
        return Err(ParseError::new(
            "SDIF_FIELD",
            "field requires a key and value",
            Span::single_line(line_no, indent + 1, indent + 1 + body.len() as u32),
        ));
    }

    // Split on first whitespace.
    let sep = clean
        .char_indices()
        .find(|(_, c)| c.is_ascii_whitespace())
        .map(|(i, _)| i)
        .unwrap();
    let key = &clean[..sep];
    let raw_value = clean[sep..].trim().to_string();

    ensure_scalar_quote_closed(&raw_value, line_no)?;
    let unquoted = unquote(&raw_value);

    if unquoted.len() > policy.max_string_length {
        return Err(ParseError::from_policy(PolicyError::new(
            "SDIF_POLICY_STRING_LENGTH",
            format!(
                "Field value length {} exceeds maximum limit of {}",
                unquoted.len(),
                policy.max_string_length
            ),
        )));
    }

    let quoted = is_quoted(&raw_value);

    Ok(Field {
        key: key.to_string(),
        value: unquoted,
        quoted,
    })
}

// ---------------------------------------------------------------------------
// Free helper functions (mirror Python module-level functions)
// ---------------------------------------------------------------------------

/// Count leading spaces; error on tab used for indentation.
fn compute_indent(raw: &str, line_no: u32) -> Result<u32, ParseError> {
    let mut count: u32 = 0;
    for ch in raw.chars() {
        match ch {
            ' ' => count += 1,
            '\t' => {
                return Err(ParseError::new(
                    "SDIF_INDENT_TAB",
                    "tabs must not be used for indentation",
                    Span::single_line(line_no, count + 1, count + 1),
                ));
            }
            _ => break,
        }
    }
    Ok(count)
}

/// Strip a trailing inline comment (not inside quotes). Returns the trimmed body.
///
/// `#` at index 0 or preceded by whitespace (and outside a quoted region) starts a comment.
fn strip_inline_comment(body: &str) -> String {
    let mut in_quote = false;
    let mut escaped = false;
    let bytes = body.as_bytes();
    for (idx, &byte) in bytes.iter().enumerate() {
        let ch = byte as char;
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' && in_quote {
            escaped = true;
            continue;
        }
        if ch == '"' {
            in_quote = !in_quote;
            continue;
        }
        if ch == '#' && !in_quote && (idx == 0 || bytes[idx - 1].is_ascii_whitespace()) {
            return body[..idx].trim_end().to_string();
        }
    }
    body.trim_end().to_string()
}

/// Return true when `value` is a double-quoted string.
fn is_quoted(value: &str) -> bool {
    value.len() >= 2 && value.starts_with('"') && value.ends_with('"')
}

/// Validate that a quoted scalar value is properly closed.
fn ensure_scalar_quote_closed(value: &str, line_no: u32) -> Result<(), ParseError> {
    if !value.starts_with('"') {
        return Ok(());
    }
    let chars: Vec<char> = value.chars().collect();
    let mut escaped = false;
    for (i, &ch) in chars[1..].iter().enumerate() {
        let idx = i + 2; // 1-based inside the quoted string (i=0 → idx=2)
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == '"' {
            if idx != chars.len() {
                return Err(ParseError::new(
                    "SDIF_STRING_TRAILING",
                    "quoted scalar field has trailing content after closing quote",
                    Span::single_line(line_no, idx as u32 + 1, idx as u32 + 1),
                ));
            }
            return Ok(());
        }
    }
    Err(ParseError::new(
        "SDIF_STRING_UNCLOSED",
        "unterminated quoted scalar field",
        Span::single_line(line_no, chars.len() as u32 + 1, chars.len() as u32 + 1),
    ))
}

/// Unescape a possibly-quoted value.
///
/// Only a subset of escapes are handled (`\n`, `\t`, `\r`, `\"`, `\\`).
/// More exotic Python `unicode_escape` sequences (`\xNN`, `\uNNNN`) are
/// passed through verbatim — see limitations in the report.
fn unquote(value: &str) -> String {
    if !is_quoted(value) {
        return value.to_string();
    }
    let inner = &value[1..value.len() - 1];
    let mut result = String::with_capacity(inner.len());
    let mut chars = inner.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('t') => result.push('\t'),
                Some('r') => result.push('\r'),
                Some('"') => result.push('"'),
                Some('\\') => result.push('\\'),
                Some(c) => {
                    result.push('\\');
                    result.push(c);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(ch);
        }
    }
    result
}

/// Split `source` on unquoted whitespace, respecting `"…"` strings.
fn split_quoted_whitespace(
    source: &str,
    line_no: u32,
    error_code: &str,
) -> Result<Vec<String>, ParseError> {
    let mut parts: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;
    let mut escaped = false;

    for ch in source.chars() {
        if escaped {
            current.push(ch);
            escaped = false;
            continue;
        }
        if ch == '\\' && in_quote {
            current.push(ch);
            escaped = true;
            continue;
        }
        if ch == '"' {
            current.push(ch);
            in_quote = !in_quote;
            continue;
        }
        if ch.is_whitespace() && !in_quote {
            if !current.is_empty() {
                parts.push(current.clone());
                current.clear();
            }
            continue;
        }
        current.push(ch);
    }

    if escaped || in_quote {
        return Err(ParseError::new(
            error_code,
            "unterminated quoted value",
            Span::single_line(line_no, source.len() as u32 + 1, source.len() as u32 + 1),
        ));
    }

    if !current.is_empty() {
        parts.push(current);
    }
    Ok(parts)
}

// ---------------------------------------------------------------------------
// Header pattern parsers
// ---------------------------------------------------------------------------

/// Match `alias[a=b,c=d]` (no trailing `:`). Returns entries if matched.
fn parse_alias_header(body: &str) -> Option<Vec<String>> {
    let body = body.trim();
    // Must start with "alias[" and end with "]"
    let after_alias = body.strip_prefix("alias[")?;
    let entries_str = after_alias.strip_suffix(']')?;
    // Validate each entry looks like identifier=identifier.
    let entries: Vec<String> = entries_str.split(',').map(|s| s.to_string()).collect();
    for entry in &entries {
        let eq = entry.find('=')?;
        let left = &entry[..eq];
        let right = &entry[eq + 1..];
        if !is_valid_ident(left) || !is_valid_ident(right) {
            return None;
        }
    }
    Some(entries)
}

/// Match a narrative opener: `key """`.
fn parse_narrative_opener(body: &str) -> Option<String> {
    // Pattern: `key  """` (key followed by whitespace and `"""`)
    let body = body.trim_end();
    if !body.ends_with(r#"""""#) {
        return None;
    }
    let without_suffix = body[..body.len() - 3].trim_end();
    if without_suffix.is_empty() {
        return None;
    }
    // The remaining part must be a single identifier (no spaces).
    if without_suffix.contains(' ') || without_suffix.contains('\t') {
        return None;
    }
    if is_valid_ident(without_suffix) {
        Some(without_suffix.to_string())
    } else {
        None
    }
}

/// Match a table header: `name[cols]:`. Returns `(name, cols_raw)`.
fn parse_table_header(body: &str) -> Option<(String, String)> {
    let body = body.trim();
    if !body.ends_with(':') {
        return None;
    }
    let without_colon = &body[..body.len() - 1];
    let bracket_open = without_colon.find('[')?;
    let bracket_close = without_colon[bracket_open..].find(']')? + bracket_open;
    if bracket_open >= bracket_close {
        return None;
    }
    let name = &without_colon[..bracket_open];
    let cols_raw = &without_colon[bracket_open + 1..bracket_close];
    if is_valid_ident(name) {
        Some((name.to_string(), cols_raw.to_string()))
    } else {
        None
    }
}

/// Match a block opener: `key:`. Returns the key.
fn parse_block_key(body: &str) -> Option<String> {
    let body = body.trim();
    if !body.ends_with(':') {
        return None;
    }
    let key = &body[..body.len() - 1];
    // Must be a single identifier (no brackets — those are table headers).
    if key.contains('[') || key.contains(']') {
        return None;
    }
    if is_valid_ident(key) {
        Some(key.to_string())
    } else {
        None
    }
}

/// True when `s` matches `[A-Za-z_][A-Za-z0-9_-]*`.
fn is_valid_ident(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

// ---------------------------------------------------------------------------
// Rule expression parsing (mirrors Python's _tokenize_rule + _parse_rule_*)
// ---------------------------------------------------------------------------

fn tokenize_and_parse_rule(source: &str, row_no: u32) -> Result<RuleExpression, ParseError> {
    let tokens = tokenize_rule(source);
    match parse_rule_tokens(&tokens) {
        Ok(expr) => Ok(expr),
        Err(e) => Err(ParseError::new(
            "SDIF_RULE_EXPR",
            format!("invalid rule expression: {e}"),
            Span::single_line(row_no, 1, source.len() as u32 + 1),
        )),
    }
}

fn tokenize_rule(source: &str) -> Vec<String> {
    let mut tokens: Vec<String> = Vec::new();
    let chars: Vec<char> = source.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        if ch.is_whitespace() {
            i += 1;
            continue;
        }
        if ch == '(' || ch == ')' || ch == ',' {
            tokens.push(ch.to_string());
            i += 1;
            continue;
        }
        if ch == '"' {
            let start = i;
            i += 1;
            let mut escaped = false;
            while i < chars.len() {
                if escaped {
                    escaped = false;
                } else if chars[i] == '\\' {
                    escaped = true;
                } else if chars[i] == '"' {
                    i += 1;
                    break;
                }
                i += 1;
            }
            let s: String = chars[start..i].iter().collect();
            tokens.push(s);
            continue;
        }
        // Identifier / number token.
        let start = i;
        while i < chars.len() && (chars[i].is_alphanumeric() || "_-./:[]".contains(chars[i])) {
            i += 1;
        }
        if i > start {
            tokens.push(chars[start..i].iter().collect());
        } else {
            tokens.push(ch.to_string());
            i += 1;
        }
    }
    tokens
}

/// Internal node type used during rule expression parsing.
enum RuleNode {
    Lit(RuleArg),
    Call { name: String, args: Vec<RuleArg> },
}

fn parse_rule_tokens(tokens: &[String]) -> Result<RuleExpression, String> {
    let (node, pos) = parse_rule_node(tokens, 0)?;
    if pos < tokens.len() {
        return Err("Extra tokens at end of rule expression".to_string());
    }
    // Convert the top-level node to a RuleExpression.
    match node {
        RuleNode::Call { name, args } => to_rule_expression(&name, &args),
        RuleNode::Lit(_) => {
            Err("Rule expression must start with a parenthesized action call".to_string())
        }
    }
}

fn parse_rule_node(tokens: &[String], pos: usize) -> Result<(RuleNode, usize), String> {
    if pos >= tokens.len() {
        return Err("Unexpected end of expression".to_string());
    }
    let token = &tokens[pos];

    // Prefix call: `(name arg arg …)`.
    if token == "(" {
        let pos = pos + 1;
        if pos >= tokens.len() {
            return Err("Unterminated parenthesis".to_string());
        }
        let name = &tokens[pos];
        if !is_rule_ident(name) {
            return Err(format!("Expected identifier for call name, got `{name}`"));
        }
        let pos = pos + 1;
        let (args, pos) = collect_rule_args(tokens, pos, false)?;
        if pos >= tokens.len() || tokens[pos] != ")" {
            return Err("Expected `)` to close call".to_string());
        }
        return Ok((
            RuleNode::Call {
                name: name.clone(),
                args,
            },
            pos + 1,
        ));
    }

    // Compact call: `name(arg, arg, …)`.
    if pos + 1 < tokens.len() && tokens[pos + 1] == "(" {
        let name = token.clone();
        let pos = pos + 2;
        let (args, pos) = collect_rule_args(tokens, pos, true)?;
        if pos >= tokens.len() || tokens[pos] != ")" {
            return Err("Expected `)` to close compact call".to_string());
        }
        return Ok((RuleNode::Call { name, args }, pos + 1));
    }

    // Literal or identifier.
    let arg = token_to_arg(token);
    Ok((RuleNode::Lit(arg), pos + 1))
}

fn collect_rule_args(
    tokens: &[String],
    mut pos: usize,
    skip_commas: bool,
) -> Result<(Vec<RuleArg>, usize), String> {
    let mut args: Vec<RuleArg> = Vec::new();
    while pos < tokens.len() && tokens[pos] != ")" {
        if skip_commas && tokens[pos] == "," {
            pos += 1;
            continue;
        }
        let (node, new_pos) = parse_rule_node(tokens, pos)?;
        pos = new_pos;
        let arg = match node {
            RuleNode::Lit(a) => a,
            RuleNode::Call {
                name,
                args: sub_args,
            } => RuleArg::Call(RuleCall {
                name,
                args: sub_args,
            }),
        };
        args.push(arg);
    }
    Ok((args, pos))
}

fn token_to_arg(token: &str) -> RuleArg {
    if token.starts_with('"') && token.ends_with('"') {
        return RuleArg::Str(unquote(token));
    }
    if token == "null" {
        return RuleArg::Null;
    }
    if token == "true" {
        return RuleArg::Bool(true);
    }
    if token == "false" {
        return RuleArg::Bool(false);
    }
    if let Ok(n) = token.parse::<i64>() {
        return RuleArg::Int(n);
    }
    if let Ok(f) = token.parse::<f64>() {
        return RuleArg::Float(f);
    }
    RuleArg::Ident(token.to_string())
}

fn is_rule_ident(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_alphanumeric() || "_-./:[]".contains(c))
}

fn to_rule_expression(action: &str, args: &[RuleArg]) -> Result<RuleExpression, String> {
    if action != "deny" && action != "warn" {
        return Err(format!(
            "Invalid rule action: `{action}`. Must be `deny` or `warn`."
        ));
    }
    if args.is_empty() {
        return Err("Rule expression must have at least one function or argument".to_string());
    }
    match &args[0] {
        RuleArg::Call(call) => Ok(RuleExpression {
            action: action.to_string(),
            function: call.name.clone(),
            args: call.args.clone(),
        }),
        RuleArg::Ident(name) => Ok(RuleExpression {
            action: action.to_string(),
            function: name.clone(),
            args: args[1..].to_vec(),
        }),
        first => Err(format!(
            "Invalid rule function or first argument: `{first:?}`"
        )),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_document() {
        let doc = parse_text("@sdif 1.0\nkey value\n").unwrap();
        assert!(doc.directives.iter().any(|d| d.name == "sdif"));
        assert_eq!(doc.statements.len(), 1);
    }

    #[test]
    fn test_version_missing() {
        let err = parse_text("key value\n").unwrap_err();
        assert_eq!(err.code, "SDIF_VERSION_MISSING");
    }

    #[test]
    fn test_version_conflict() {
        let err = parse_text("@sdif 1.0\n@sdif 1.0\nkey value\n").unwrap_err();
        assert_eq!(err.code, "SDIF_VERSION_CONFLICT");
    }

    #[test]
    fn test_quoted_field() {
        let doc = parse_text("@sdif 1.0\nkey \"hello world\"\n").unwrap();
        if let Statement::Field(f) = &doc.statements[0] {
            assert!(f.quoted);
            assert_eq!(f.value, "hello world");
        } else {
            panic!("Expected a Field statement");
        }
    }

    #[test]
    fn test_unclosed_quote() {
        let err = parse_text("@sdif 1.0\nkey \"unclosed\n").unwrap_err();
        assert_eq!(err.code, "SDIF_STRING_UNCLOSED");
    }

    #[test]
    fn test_table() {
        let src = "@sdif 1.0\ndata[a,b]:\n  x\ty\n";
        let doc = parse_text(src).unwrap();
        if let Statement::Table(t) = &doc.statements[0] {
            assert_eq!(t.name, "data");
            assert_eq!(t.columns, vec!["a", "b"]);
            assert_eq!(t.rows[0], vec!["x", "y"]);
        } else {
            panic!("Expected a Table statement");
        }
    }

    #[test]
    fn test_relation_block() {
        let src = "@sdif 1.0\nrel:\n  a b c\n";
        let doc = parse_text(src).unwrap();
        if let Statement::Relation(r) = &doc.statements[0] {
            assert_eq!(r.subject, "a");
            assert_eq!(r.predicate, "b");
            assert_eq!(r.object, "c");
        } else {
            panic!("Expected a Relation statement");
        }
    }

    #[test]
    fn test_indent_tab_error() {
        let err = parse_text("@sdif 1.0\n\tkey value\n").unwrap_err();
        assert_eq!(err.code, "SDIF_INDENT_TAB");
    }

    #[test]
    fn test_policy_document_size() {
        let policy = Policy {
            max_document_size: 10,
            ..Policy::default()
        };
        let err = parse_text_with_policy("@sdif 1.0\nkey value\n", &policy).unwrap_err();
        assert_eq!(err.code, "SDIF_POLICY_DOCUMENT_SIZE");
    }
}
