use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn fixture_path(name: &str, extension: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!(
        "sdif_cli_{}_{}_{}.{}",
        name,
        std::process::id(),
        std::thread::current().name().unwrap_or("test"),
        extension
    ));
    path
}

fn write_fixture(name: &str, extension: &str, contents: &str) -> PathBuf {
    let path = fixture_path(name, extension);
    fs::write(&path, contents).unwrap();
    path
}

#[test]
fn canonical_command_outputs_canonical_sdif() {
    let src = write_fixture("canonical", "sdif", "@sdif 1.0\nkind Agent\nid bob\n");
    let output = Command::new(env!("CARGO_BIN_EXE_sdif"))
        .arg("canonical")
        .arg(&src)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.starts_with("@sdif 1.0\n"));
    assert!(stdout.contains("kind Agent\n"));
    assert!(stdout.contains("id bob\n"));
}

#[test]
fn to_json_command_outputs_json() {
    let src = write_fixture("json", "sdif", "@sdif 1.0\nkind Agent\nid bob\n");
    let output = Command::new(env!("CARGO_BIN_EXE_sdif"))
        .arg("to-json")
        .arg(&src)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["kind"], "Agent");
    assert_eq!(json["id"], "bob");
}

#[test]
fn from_json_command_outputs_parseable_sdif() {
    let src = write_fixture("from_json", "json", r#"{"kind":"Agent","id":"bob"}"#);
    let output = Command::new(env!("CARGO_BIN_EXE_sdif"))
        .arg("from-json")
        .arg(&src)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8(output.stdout).unwrap();
    sdif::parse_text(&text).unwrap();
    assert!(text.contains("kind Agent\n"));
}

#[test]
fn validate_command_accepts_schema_file() {
    let schema = write_fixture(
        "schema",
        "sdif",
        "@sdif 1.0\nkind Schema\nrequired[field]:\n  id\n",
    );
    let doc = write_fixture("doc", "sdif", "@sdif 1.0\nid bob\n");
    let output = Command::new(env!("CARGO_BIN_EXE_sdif"))
        .arg("validate")
        .arg(&doc)
        .arg(&schema)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "ok\n");
}
