use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const AGENT_SDIF: &str = "@sdif 1.0\nkind Agent\nid bob\n";
const AGENT_JSON: &str = r#"{"kind":"Agent","id":"bob"}"#;
const SCHEMA_SDIF: &str = "@sdif 1.0\nkind Schema\nrequired[field]:\n  id\n";

struct FixtureFile {
    path: PathBuf,
}

impl FixtureFile {
    fn write(name: &str, extension: &str, contents: &str) -> Self {
        let path = fixture_path(name, extension);

        fs::write(&path, contents).unwrap_or_else(|error| {
            panic!("failed to write fixture {}: {error}", path.display());
        });

        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for FixtureFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn fixture_path(name: &str, extension: &str) -> PathBuf {
    let current_thread = std::thread::current();
    let thread_name = current_thread.name().unwrap_or("unnamed_test");

    let file_name = format!(
        "sdif_cli_{}_{}_{}.{}",
        sanitize_path_component(name),
        std::process::id(),
        sanitize_path_component(thread_name),
        sanitize_path_component(extension),
    );

    std::env::temp_dir().join(file_name)
}

fn sanitize_path_component(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' || character == '_' {
                character
            } else {
                '_'
            }
        })
        .collect()
}

fn sdif_command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_sdif"))
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn stdout_text(output: &Output) -> &str {
    std::str::from_utf8(&output.stdout).expect("stdout must be valid UTF-8")
}

#[test]
fn parse_command_reports_statement_counts() {
    let src = FixtureFile::write("parse", "sdif", AGENT_SDIF);

    let output = sdif_command()
        .arg("parse")
        .arg(src.path())
        .output()
        .expect("failed to run sdif parse command");

    assert_success(&output);

    assert_eq!(
        stdout_text(&output),
        "ok: 2 fields, 0 tables, 0 relations\n"
    );
}

#[test]
fn canonical_command_outputs_canonical_sdif() {
    let src = FixtureFile::write("canonical", "sdif", AGENT_SDIF);

    let output = sdif_command()
        .arg("canonical")
        .arg(src.path())
        .output()
        .expect("failed to run sdif canonical command");

    assert_success(&output);

    let stdout = stdout_text(&output);

    assert!(stdout.starts_with("@sdif 1.0\n"));
    assert!(stdout.contains("kind Agent\n"));
    assert!(stdout.contains("id bob\n"));
}

#[test]
fn to_json_command_outputs_json() {
    let src = FixtureFile::write("json", "sdif", AGENT_SDIF);

    let output = sdif_command()
        .arg("to-json")
        .arg(src.path())
        .output()
        .expect("failed to run sdif to-json command");

    assert_success(&output);

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout must be valid JSON");

    assert_eq!(json["kind"], "Agent");
    assert_eq!(json["id"], "bob");
}

#[test]
fn from_json_command_outputs_parseable_sdif() {
    let src = FixtureFile::write("from_json", "json", AGENT_JSON);

    let output = sdif_command()
        .arg("from-json")
        .arg(src.path())
        .output()
        .expect("failed to run sdif from-json command");

    assert_success(&output);

    let text = stdout_text(&output);

    sdif::parse_text(text).expect("generated SDIF must be parseable");

    assert!(text.contains("kind Agent\n"));
}

#[test]
fn validate_command_accepts_schema_file() {
    let doc = FixtureFile::write("doc", "sdif", "@sdif 1.0\nid bob\n");
    let schema = FixtureFile::write("schema", "sdif", SCHEMA_SDIF);

    let output = sdif_command()
        .arg("validate")
        .arg(doc.path())
        .arg("--schema")
        .arg(schema.path())
        .output()
        .expect("failed to run sdif validate command");

    assert_success(&output);

    assert_eq!(stdout_text(&output), "ok\n");
}