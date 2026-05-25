//! Canonical SDIF serializer — mirrors `sdif.canonical.canonicalize()` from the
//! Python reference implementation byte-for-byte.
//!
//! The canonical form is a deterministic, normalized representation of an SDIF
//! document. `sdif_hash()` returns the SHA-256 hex digest of that form encoded
//! as UTF-8.

use sha2::{Digest, Sha256};

use crate::ast::{Directive, Document, Narrative, Statement};

// ---------------------------------------------------------------------------
// Directive sort ranks — mirrors _DIRECTIVE_ORDER in Python
// ---------------------------------------------------------------------------

fn directive_rank(name: &str) -> u32 {
    match name {
        "sdif" | "sdif.ai" => 0,
        "alias" => 1,
        "profile" => 2,
        "vocab" => 3,
        "base" => 4,
        "namespace" => 5,
        "include" => 6,
        _ => 100,
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Produce the canonical text form of `doc`.
///
/// The output is byte-for-byte identical to the Python
/// `sdif.canonical.canonicalize(doc)` function (no `Schema` argument; schema
/// support belongs to Task 13).
pub fn canonicalize(doc: &Document) -> String {
    let mut lines: Vec<String> = Vec::new();

    // Sort directives: (rank, name, args) — Vec<String> Ord is lex-by-element.
    let mut sorted_directives: Vec<&Directive> = doc.directives.iter().collect();
    sorted_directives.sort_by(|a, b| {
        let ra = directive_rank(&a.name);
        let rb = directive_rank(&b.name);
        ra.cmp(&rb)
            .then_with(|| a.name.cmp(&b.name))
            .then_with(|| a.args.cmp(&b.args))
    });

    for directive in sorted_directives {
        lines.push(emit_directive(directive));
    }

    for statement in canonical_statement_order(&doc.statements) {
        emit_statement(statement, &mut lines, 0);
    }

    // Final output: rstrip all trailing whitespace then exactly one newline.
    let joined = lines.join("\n");
    format!("{}\n", joined.trim_end())
}

/// Return the SHA-256 hex digest of the canonical UTF-8 form of `doc`.
pub fn sdif_hash(doc: &Document) -> String {
    let canonical = canonicalize(doc);
    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    format!("{:x}", hasher.finalize())
}

// ---------------------------------------------------------------------------
// Directive emit
// ---------------------------------------------------------------------------

fn emit_directive(d: &Directive) -> String {
    if d.name == "alias" {
        // alias[k=kind,st=status]  — no @, no space
        return format!("alias[{}]", d.args.join(","));
    }
    let args = d.args.join(" ");
    if args.is_empty() {
        format!("@{}", d.name)
    } else {
        format!("@{} {}", d.name, args)
    }
}

// ---------------------------------------------------------------------------
// Statement ordering — mirrors _canonical_statement_order
// ---------------------------------------------------------------------------

/// Priority for well-known field keys.
fn field_priority(key: &str) -> (u32, &str) {
    let rank = match key {
        "kind" => 0,
        "id" => 1,
        "schema" => 2,
        "authority" => 3,
        "lifecycle" => 4,
        _ => 100,
    };
    (rank, key)
}

/// Return statements in canonical order.
///
/// fields (sorted by priority/key) → others (ObjectBlock, Table, Narrative,
/// in original relative order) → relations (sorted) → rules (sorted).
fn canonical_statement_order(statements: &[Statement]) -> Vec<&Statement> {
    let mut fields: Vec<&Statement> = statements
        .iter()
        .filter(|s| matches!(s, Statement::Field(_)))
        .collect();

    let others: Vec<&Statement> = statements
        .iter()
        .filter(|s| {
            matches!(
                s,
                Statement::ObjectBlock(_) | Statement::Table(_) | Statement::Narrative(_)
            )
        })
        .collect();

    let mut relations: Vec<&Statement> = statements
        .iter()
        .filter(|s| matches!(s, Statement::Relation(_)))
        .collect();

    let mut rules: Vec<&Statement> = statements
        .iter()
        .filter(|s| matches!(s, Statement::Rule(_)))
        .collect();

    fields.sort_by(|a, b| {
        let ka = if let Statement::Field(f) = a { field_priority(&f.key) } else { unreachable!() };
        let kb = if let Statement::Field(f) = b { field_priority(&f.key) } else { unreachable!() };
        ka.cmp(&kb)
    });

    relations.sort_by(|a, b| {
        let ra = if let Statement::Relation(r) = a { r } else { unreachable!() };
        let rb = if let Statement::Relation(r) = b { r } else { unreachable!() };
        (&ra.subject, &ra.predicate, &ra.object).cmp(&(&rb.subject, &rb.predicate, &rb.object))
    });

    rules.sort_by(|a, b| {
        let sa = if let Statement::Rule(r) = a { &r.source } else { unreachable!() };
        let sb = if let Statement::Rule(r) = b { &r.source } else { unreachable!() };
        sa.cmp(sb)
    });

    let mut result = fields;
    result.extend(others);
    result.extend(relations);
    result.extend(rules);
    result
}

// ---------------------------------------------------------------------------
// Statement emit
// ---------------------------------------------------------------------------

fn emit_statement(statement: &Statement, lines: &mut Vec<String>, indent: usize) {
    let prefix = " ".repeat(indent);
    match statement {
        Statement::Field(f) => {
            lines.push(format!("{}{} {}", prefix, f.key, quote_if_needed(&f.value, f.quoted)));
        }
        Statement::ObjectBlock(obj) => {
            lines.push(format!("{}{}:", prefix, obj.key));
            for child in canonical_statement_order(&obj.statements) {
                emit_statement(child, lines, indent + 2);
            }
        }
        Statement::Table(table) => {
            lines.push(format!("{}{}[{}]:", prefix, table.name, table.columns.join(",")));
            let row_prefix = " ".repeat(indent + 2);
            for row in &table.rows {
                let cells: Vec<String> = row
                    .iter()
                    .enumerate()
                    .map(|(idx, cell)| {
                        if table.quoted_columns.contains(&idx) {
                            quote_if_needed(cell, true)
                        } else {
                            cell.clone()
                        }
                    })
                    .collect();
                lines.push(format!("{}{}", row_prefix, cells.join("\t")));
            }
        }
        Statement::Relation(rel) => {
            let header = format!("{}rel:", prefix);
            if !inside_current_block(lines, &header) {
                lines.push(header);
            }
            let obj_str = quote_if_needed(&rel.object, rel.object_quoted);
            lines.push(format!(
                "{}{} {} {}",
                " ".repeat(indent + 2),
                rel.subject,
                rel.predicate,
                obj_str
            ));
        }
        Statement::Rule(rule) => {
            let header = format!("{}rules:", prefix);
            if !inside_current_block(lines, &header) {
                lines.push(header);
            }
            lines.push(format!("{}{}", " ".repeat(indent + 2), rule.source));
        }
        Statement::Narrative(narr) => {
            emit_narrative(narr, lines, &prefix);
        }
    }
}

fn emit_narrative(narr: &Narrative, lines: &mut Vec<String>, prefix: &str) {
    lines.push(format!("{}{} \"\"\"", prefix, narr.key));
    for line in narr.text.split('\n') {
        lines.push(format!("{}{}", prefix, line));
    }
    lines.push(format!("{}\"\"\"", prefix));
}

// ---------------------------------------------------------------------------
// _inside_current_block — mirrors Python's function exactly
// ---------------------------------------------------------------------------

/// Walk backward through emitted lines to check if we are still inside the
/// block started by `header`. Stops as soon as a non-indented (non-`"  "`-
/// prefixed) line is seen that doesn't match the header.
fn inside_current_block(lines: &[String], header: &str) -> bool {
    for line in lines.iter().rev() {
        if line == header {
            return true;
        }
        if !line.starts_with("  ") {
            return false;
        }
    }
    false
}

// ---------------------------------------------------------------------------
// _quote_if_needed — mirrors Python's function exactly
// ---------------------------------------------------------------------------

fn quote_if_needed(value: &str, force: bool) -> String {
    if force {
        let escaped = value
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n");
        return format!("\"{}\"", escaped);
    }
    // Bare keywords
    if matches!(value, "null" | "true" | "false") {
        return value.to_string();
    }
    // Array form
    if value.len() >= 2 && value.starts_with('[') && value.ends_with(']') {
        return value.to_string();
    }
    // Safe chars: alphanumeric or in "_-./:[] ,"
    let safe = value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || "_-./:[] ,".contains(ch));
    if safe && !value.is_empty() && !value.contains(' ') && !value.contains(',') {
        return value.to_string();
    }
    let escaped = value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n");
    format!("\"{}\"", escaped)
}
