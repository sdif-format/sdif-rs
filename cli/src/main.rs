//! SDIF command-line tool.

use std::process;

type CliResult<T = ()> = Result<T, String>;

const USAGE: &str = "\
Usage: sdif <command> [args]

Commands:
  parse <file.sdif>
  canonical <file.sdif>
  hash <file.sdif>
  validate <file.sdif> [schema.sdif]
  validate <file.sdif> --schema <schema.sdif>
  to-json <file.sdif>
  from-json <file.json>
  ai-view <file.sdif>
";

fn main() {
    process::exit(match run() {
        Ok(()) => 0,
        Err(message) => {
            eprintln!("{message}");
            1
        }
    });
}

fn run() -> CliResult {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let Some((command, command_args)) = args.split_first() else {
        return Err(USAGE.to_string());
    };

    match command.as_str() {
        "parse" => cmd_parse(command_args),
        "canonical" => cmd_canonical(command_args),
        "hash" => cmd_hash(command_args),
        "validate" => cmd_validate(command_args),
        "to-json" => cmd_to_json(command_args),
        "from-json" => cmd_from_json(command_args),
        "ai-view" => cmd_ai_view(command_args),
        unknown => Err(format!("Unknown command: {unknown}\n\n{USAGE}")),
    }
}

fn require_single_path<'a>(args: &'a [String], usage: &str) -> CliResult<&'a str> {
    match args {
        [path] => Ok(path),
        _ => Err(format!("Usage: {usage}")),
    }
}

fn read_text_from_file(path: &str) -> CliResult<String> {
    std::fs::read_to_string(path).map_err(|error| format!("cannot read {path}: {error}"))
}

fn read_doc_from_file(path: &str) -> CliResult<sdif::Document> {
    let text = read_text_from_file(path)?;
    sdif::parse_text(&text).map_err(|error| error.to_string())
}

fn cmd_parse(args: &[String]) -> CliResult {
    let path = require_single_path(args, "sdif parse <file.sdif>")?;
    let doc = read_doc_from_file(path)?;

    println!(
        "ok: {} fields, {} tables, {} relations",
        doc.fields().count(),
        doc.tables().count(),
        doc.relations().count()
    );

    Ok(())
}

fn cmd_canonical(args: &[String]) -> CliResult {
    let path = require_single_path(args, "sdif canonical <file.sdif>")?;
    let doc = read_doc_from_file(path)?;

    print!("{}", sdif::canonicalize(&doc));

    Ok(())
}

fn cmd_hash(args: &[String]) -> CliResult {
    let path = require_single_path(args, "sdif hash <file.sdif>")?;
    let doc = read_doc_from_file(path)?;

    println!("{}", sdif::sdif_hash(&doc));

    Ok(())
}

fn cmd_validate(args: &[String]) -> CliResult {
    let (doc_path, schema_path) = parse_validate_args(args)?;

    let doc = read_doc_from_file(doc_path)?;
    let schema = match schema_path {
        Some(path) => {
            let schema_doc = read_doc_from_file(path)?;
            sdif::Schema::from_document(&schema_doc).map_err(|error| error.to_string())?
        }
        None => sdif::Schema::default(),
    };

    let diagnostics = sdif::validate(&doc, &schema);

    if diagnostics.is_empty() {
        println!("ok");
        return Ok(());
    }

    for diagnostic in diagnostics {
        eprintln!(
            "{} [{}] {}: {}",
            diagnostic.code, diagnostic.severity, diagnostic.path, diagnostic.message
        );

        if let Some(hint) = diagnostic.hint {
            eprintln!("hint: {hint}");
        }
    }

    Err("validation failed".to_string())
}

fn parse_validate_args(args: &[String]) -> CliResult<(&str, Option<&str>)> {
    match args {
        [doc_path] => Ok((doc_path, None)),
        [doc_path, schema_path] if schema_path != "--schema" => Ok((doc_path, Some(schema_path))),
        [doc_path, flag, schema_path] if flag == "--schema" => Ok((doc_path, Some(schema_path))),
        _ => Err("Usage: sdif validate <file.sdif> [schema.sdif]\n       sdif validate <file.sdif> --schema <schema.sdif>".to_string()),
    }
}

fn cmd_to_json(args: &[String]) -> CliResult {
    let path = require_single_path(args, "sdif to-json <file.sdif>")?;
    let doc = read_doc_from_file(path)?;

    let json = serde_json::to_string_pretty(&sdif::document_to_json(&doc))
        .map_err(|error| error.to_string())?;

    println!("{json}");

    Ok(())
}

fn cmd_from_json(args: &[String]) -> CliResult {
    let path = require_single_path(args, "sdif from-json <file.json>")?;

    let text = read_text_from_file(path)?;
    let value = serde_json::from_str::<serde_json::Value>(&text)
        .map_err(|error| error.to_string())?;

    print!("{}", sdif::json_to_sdif(&value));

    Ok(())
}

fn cmd_ai_view(args: &[String]) -> CliResult {
    let path = require_single_path(args, "sdif ai-view <file.sdif>")?;
    let doc = read_doc_from_file(path)?;

    print!("{}", sdif::ai_view(&doc));

    Ok(())
}