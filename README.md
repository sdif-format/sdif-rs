<p align="center">
  <img src="https://raw.githubusercontent.com/sdif-format/.github/main/profile/assets/sdif-logo-t.png" alt="SDIF Rust" width="520">
</p>

<p align="center">
  <strong>SDIF Rust Parser Library</strong>
</p>

<p align="center">
  Pure Rust implementation of the Semantic Data Interchange Format (SDIF) parser,<br>
  featuring a span-annotated Abstract Syntax Tree (AST) suitable for editor tooling.
</p>

<p align="center">
  <a href="#what-is-sdif">What is SDIF?</a>
  ·
  <a href="#public-api">Public API</a>
  ·
  <a href="#format-at-a-glance">Format at a glance</a>
  ·
  <a href="#token-efficiency">Token efficiency</a>
  ·
  <a href="#ecosystem">Ecosystem</a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/language-Rust-2563eb?style=flat-square" alt="Rust">
  <img src="https://img.shields.io/badge/license-Apache%202.0-374151?style=flat-square" alt="Apache 2.0">
  <img src="https://img.shields.io/badge/spec-v1.0.0-0f766e?style=flat-square" alt="Spec v1.0.0">
</p>

<br>

<div align="center">

<table>
  <tr>
    <td align="center" width="25%">
      <strong>Compact</strong>
      <br><br>
      Less repeated structure.<br>
      Fewer wasted tokens.
    </td>
    <td align="center" width="25%">
      <strong>Semantic</strong>
      <br><br>
      Tables, relations,<br>
      metadata and intent.
    </td>
    <td align="center" width="25%">
      <strong>Canonical</strong>
      <br><br>
      Stable output for hashing,<br>
      signing and comparison.
    </td>
    <td align="center" width="25%">
      <strong>Auditable</strong>
      <br><br>
      Designed to be read,<br>
      reviewed and trusted.
    </td>
  </tr>
</table>

</div>

<br>

---

## What is SDIF?

**SDIF — Semantic Data Interchange Format** is a compact, canonicalizable and AI-friendly data format for structured information that needs to move cleanly between humans, tools, agents and deterministic workflows.

It is designed for cases where data should be:

- small enough to be efficient in AI context windows;
- structured enough for machines to parse and validate;
- readable enough for humans to review;
- deterministic enough for hashing, signing and reproducible workflows;
- semantic enough to express tables, relations, metadata and intent.

SDIF also includes an AI projection surface, `.sdif.ai`, designed for token-dense agent exchange while remaining reversible back into canonical SDIF when the projection contract is respected.

<br>

```sdif
@sdif 1.0

kind Plan
id release.v1
title "Release readiness plan"

items[id,status,owner,evidence]:
  R1 done build "reports/build.md"
  R2 open qa "reports/tests.md"
  R3 done security "reports/audit.md"

rel:
  release.v1 validated_by R1
  release.v1 blocked_by R2
  release.v1 governed_by R3
```

<br>

<p align="center">
  <strong>
    Structured information closer to a document,<br>
    while still behaving like a contract.
  </strong>
</p>

<br>

---

## Format at a glance

JSON repeats field names across every record:

```json
[
  { "id": "R1", "status": "done",    "owner": "build",    "evidence": "reports/build.md"  },
  { "id": "R2", "status": "open",    "owner": "qa",       "evidence": "reports/tests.md"  },
  { "id": "R3", "status": "done",    "owner": "security", "evidence": "reports/audit.md"  }
]
```

SDIF declares the shape once and uses literal tabs between cells. Editors must preserve tabs — this is a deliberate tradeoff for compactness:

```sdif
@sdif 1.0

kind Plan
id   release.v1
title "Release readiness plan"

items[id,status,owner,evidence]:
  R1	done	build	reports/build.md
  R2	open	qa	reports/tests.md
  R3	done	security	reports/audit.md

rel:
  release.v1  validated_by  R1
  release.v1  blocked_by    R2
  release.v1  governed_by   R3
```

Semantic relationships are first-class, not embedded strings.

<br>

---

## Token efficiency

The benchmark derives every compared format from the same canonical JSON source in `examples/golden/`. Results below are from the most recent run across 21 documents and 3 tokenizers.

<div align="center">

| Format | Consensus avg rank | Median ratio vs JSON Compact |
| --- | ---: | ---: |
| **SDIF AI** | **1.10** | **56.8%** |
| SDIF | 2.60 | 59.5% |
| CSV Bundle | 2.70 | 61.2% |
| YAML | 5.35 | 95.3% |
| JSON Compact | 5.65 | 100.0% |
| JSON Pretty | 7.00 | 137.3% |
| XML | 8.00 | 171.7% |

</div>

<br>

SDIF AI wins 57 of 63 tokenizer/document pairs. SDIF canonical wins 2.

The benchmark repository contains the exact corpus model, generated artifacts and methodology needed to reproduce these numbers.

These results are corpus-dependent. Not every data shape benefits equally from tabular projection. Claude and Llama tokenizers require separate opt-in before claiming results for those models.

For full methodology, corpus model and per-document breakdowns, see [`sdif-benchmarks`](https://github.com/sdif-format/sdif-benchmarks).

<br>

---

## What SDIF is not

SDIF does not try to replace JSON, YAML, CSV, Markdown, XML, Parquet or Protocol Buffers. Those formats are useful and battle-tested.

<table>
  <tr>
    <td width="25%" valign="top">
      <strong>JSON</strong>
      <br><br>
      Universal and reliable, but noisy when repeated records dominate.
    </td>
    <td width="25%" valign="top">
      <strong>YAML</strong>
      <br><br>
      Readable, but too permissive for deterministic workflows.
    </td>
    <td width="25%" valign="top">
      <strong>CSV</strong>
      <br><br>
      Compact, but loses structure, relations and meaning quickly.
    </td>
    <td width="25%" valign="top">
      <strong>Markdown</strong>
      <br><br>
      Great for humans, not enough when data must be parsed and verified.
    </td>
  </tr>
</table>

<br>

SDIF focuses on a narrower problem:

<p align="center">
  <strong>
    compact, semantic, canonicalizable structured data<br>
    that can move cleanly between humans, machines and AI systems.
  </strong>
</p>

<br>

---

## Public API

`sdif-rs` provides a zero-copy parser and a span-annotated AST in Rust.

The library exposes the following interface:

- `parse_text(text: &str) -> Result<Document, ParseError>` — Parse a string slice into a span-annotated AST document.
- `parse_text_with_policy(text: &str, policy: &Policy) -> Result<Document, ParseError>` — Parse using custom size limits and policies.
- `Policy` — Controls runtime resource boundaries (e.g., document size limits, matching the default 1 MB limit).
- `ParseError` — Standard error representation containing error codes, span coordinates, message, and diagnostic hints.
- `Span` — Zero-copy location tracking (`start_line`, `start_col`, `end_line`, `end_col`, 1-indexed, end exclusive).
- `Document` / AST nodes — Tree structures representing fields, directives, tables, relations, narratives, rules, and object blocks.

<br>

---

## Development

Build and run cargo tests:

```bash
cargo build                  # compile library
cargo test                   # unit tests + conformance suite
cargo clippy -- -D warnings  # run compiler linter
cargo fmt --check            # run formatting check
```

<br>

---

## Conformance

This crate runs automated conformance tests reading fixtures from the sibling [`sdif-spec`](../sdif-spec) repository. The Python `sdif-py` implementation remains the normative reference; `sdif-rs` must match its syntax rules and error codes exactly.

<br>

---

## Ecosystem

This GitHub organization hosts the official SDIF ecosystem: the core format, reference tooling, benchmarks, examples, libraries, and editor extensions.

<div align="center">

<table>
  <tr>
    <td width="33%" valign="top">
      <p><sub>PYTHON CLIENT & CLI</sub></p>
      <h3>sdif-py</h3>
      <p>
        Specification, parser, canonicalizer, and CLI.<br>
        The normative reference implementation.
      </p>
      <p><a href="https://github.com/sdif-format/sdif-py"><strong>Explore sdif-py →</strong></a></p>
    </td>
    <td width="33%" valign="top">
      <p><sub>SPECIFICATION (SSOT)</sub></p>
      <h3>sdif-spec</h3>
      <p>
        Official format specification, canonicalization rules,<br>
        and portable conformance test suite.
      </p>
      <p><a href="https://github.com/sdif-format/sdif-spec"><strong>View specification →</strong></a></p>
    </td>
    <td width="33%" valign="top">
      <p><sub>BENCHMARKS</sub></p>
      <h3>sdif-benchmarks</h3>
      <p>
        Reproducible benchmark datasets and reports comparing SDIF with JSON, YAML, XML, and CSV.
      </p>
      <p><a href="https://github.com/sdif-format/sdif-benchmarks"><strong>View benchmarks →</strong></a></p>
    </td>
  </tr>
  <tr>
    <td width="33%" valign="top">
      <p><sub>RUST IMPLEMENTATION</sub></p>
      <h3>sdif-rs</h3>
      <p>
        Pure Rust parser implementation with a span-annotated AST designed for editor tooling.
      </p>
      <p><a href="https://github.com/sdif-format/sdif-rs"><strong>Explore sdif-rs →</strong></a></p>
    </td>
    <td width="33%" valign="top">
      <p><sub>LANGUAGE SERVER (LSP)</sub></p>
      <h3>sdif-lsp</h3>
      <p>
        LSP language server binary (tower-lsp) providing real-time diagnostics and IDE features.
      </p>
      <p><a href="https://github.com/sdif-format/sdif-lsp"><strong>View sdif-lsp →</strong></a></p>
    </td>
    <td width="33%" valign="top">
      <p><sub>EDITOR INTEGRATION</sub></p>
      <h3>vscode-sdif</h3>
      <p>
        VS Code extension client providing syntax highlighting, diagnostics, and LSP configuration.
      </p>
      <p><a href="https://github.com/sdif-format/vscode-sdif"><strong>Open extension →</strong></a></p>
    </td>
  </tr>
  <tr>
    <td width="33%" valign="top">
      <p><sub>GRAMMAR FOUNDATION</sub></p>
      <h3>tree-sitter-sdif</h3>
      <p>
        Tree-sitter grammar foundation for syntax highlighting and incremental parsing.
      </p>
      <p><a href="https://github.com/sdif-format/tree-sitter-sdif"><strong>Open grammar →</strong></a></p>
    </td>
    <td width="33%" valign="top">
      <p><sub>DOCUMENTATION</sub></p>
      <h3>sdif-format.github.io</h3>
      <p>
        Official documentation website containing specification guides, tutorials, and examples.
      </p>
      <p><a href="https://github.com/sdif-format/sdif-format.github.io"><strong>Read docs →</strong></a></p>
    </td>
    <td width="33%" valign="top">
      <p><sub>ORGANIZATION META</sub></p>
      <h3>.github</h3>
      <p>
        Organization profile, assets, and shared community configuration files.
      </p>
      <p><a href="https://github.com/sdif-format/.github"><strong>View profile →</strong></a></p>
    </td>
  </tr>
</table>

</div>

<br>

<details>
  <summary><strong>Repository map</strong></summary>

<br>

| Repository | Purpose |
| --------------------------------------------------------------------- | ---------------------------------------------------------------- |
| [`sdif-py`](https://github.com/sdif-format/sdif-py)                   | Core Python parser, validator, canonicalizer, and CLI |
| [`sdif-spec`](https://github.com/sdif-format/sdif-spec)               | Official format specification and conformance test suite (SSOT) |
| [`sdif-benchmarks`](https://github.com/sdif-format/sdif-benchmarks)   | Benchmark datasets, reports, and comparison tooling |
| [`sdif-rs`](https://github.com/sdif-format/sdif-rs)                   | Rust parser crate with span-annotated AST |
| [`sdif-lsp`](https://github.com/sdif-format/sdif-lsp)                 | LSP language server binary |
| [`tree-sitter-sdif`](https://github.com/sdif-format/tree-sitter-sdif) | Tree-sitter grammar foundation for syntax highlighting |
| [`vscode-sdif`](https://github.com/sdif-format/vscode-sdif)           | VS Code extension client for SDIF |
| [`sdif-format.github.io`](https://github.com/sdif-format/sdif-format.github.io) | Public documentation website (Docusaurus) |
| [`.github`](https://github.com/sdif-format/.github)                   | Organization profile, assets, and shared GitHub community files |

</details>

<br>

## License

Licensed under the Apache License, Version 2.0 (Apache-2.0).
