//! SDIF command-line tool.

use std::process;

fn main() {
    process::exit(run());
}

fn run() -> i32 {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        print_usage();
        return 1;
    }

    match args[1].as_str() {
        "parse" => cmd_parse(&args[2..]),
        "canonical" => cmd_canonical(&args[2..]),
        "hash" => cmd_hash(&args[2..]),
        "validate" => cmd_validate(&args[2..]),
        "to-json" => cmd_to_json(&args[2..]),
        "from-json" => cmd_from_json(&args[2..]),
        "ai-view" => cmd_ai_view(&args[2..]),
        command => {
            eprintln!("Unknown command: {command}");
            print_usage();
            1
        }
    }
}

fn print_usage() {
    eprintln!("Usage: sdif <command> [args]");
    eprintln!("Commands: parse, canonical, hash, validate, to-json, from-json, ai-view");
}

fn require_path<'a>(args: &'a [String], usage: &str) -> Result<&'a str, i32> {
    args.first().map(String::as_str).ok_or_else(|| {
        eprintln!("Usage: {usage}");
        1
    })
}

fn read_doc_from_file(path: &str) -> Result<sdif::Document, String> {
    let text = std::fs::read_to_string(path).map_err(|err| format!("cannot read {path}: {err}"))?;
    sdif::parse_text(&text).map_err(|err| err.to_string())
}

fn cmd_parse(args: &[String]) -> i32 {
    let path = match require_path(args, "sdif parse <file.sdif>") {
        Ok(path) => path,
        Err(code) => return code,
    };
    match read_doc_from_file(path) {
        Ok(_) => {
            println!("ok");
            0
        }
        Err(err) => {
            eprintln!("{err}");
            1
        }
    }
}

fn cmd_canonical(args: &[String]) -> i32 {
    let path = match require_path(args, "sdif canonical <file.sdif>") {
        Ok(path) => path,
        Err(code) => return code,
    };
    match read_doc_from_file(path) {
        Ok(doc) => {
            print!("{}", sdif::canonicalize(&doc));
            0
        }
        Err(err) => {
            eprintln!("{err}");
            1
        }
    }
}

fn cmd_hash(args: &[String]) -> i32 {
    let path = match require_path(args, "sdif hash <file.sdif>") {
        Ok(path) => path,
        Err(code) => return code,
    };
    match read_doc_from_file(path) {
        Ok(doc) => {
            println!("{}", sdif::sdif_hash(&doc));
            0
        }
        Err(err) => {
            eprintln!("{err}");
            1
        }
    }
}

fn cmd_validate(args: &[String]) -> i32 {
    let path = match require_path(args, "sdif validate <file.sdif> [schema.sdif]") {
        Ok(path) => path,
        Err(code) => return code,
    };
    let doc = match read_doc_from_file(path) {
        Ok(doc) => doc,
        Err(err) => {
            eprintln!("{err}");
            return 1;
        }
    };
    let schema = match args.get(1) {
        Some(schema_path) => match read_doc_from_file(schema_path) {
            Ok(schema_doc) => match sdif::Schema::from_document(&schema_doc) {
                Ok(schema) => schema,
                Err(err) => {
                    eprintln!("{err}");
                    return 1;
                }
            },
            Err(err) => {
                eprintln!("{err}");
                return 1;
            }
        },
        None => sdif::Schema::default(),
    };

    let diagnostics = sdif::validate(&doc, &schema);
    if diagnostics.is_empty() {
        println!("ok");
        0
    } else {
        for diagnostic in diagnostics {
            eprintln!(
                "{} [{}] {}: {}",
                diagnostic.code, diagnostic.severity, diagnostic.path, diagnostic.message
            );
            if let Some(hint) = diagnostic.hint {
                eprintln!("hint: {hint}");
            }
        }
        1
    }
}

fn cmd_to_json(args: &[String]) -> i32 {
    let path = match require_path(args, "sdif to-json <file.sdif>") {
        Ok(path) => path,
        Err(code) => return code,
    };
    match read_doc_from_file(path) {
        Ok(doc) => match serde_json::to_string_pretty(&sdif::document_to_json(&doc)) {
            Ok(json) => {
                println!("{json}");
                0
            }
            Err(err) => {
                eprintln!("{err}");
                1
            }
        },
        Err(err) => {
            eprintln!("{err}");
            1
        }
    }
}

fn cmd_from_json(args: &[String]) -> i32 {
    let path = match require_path(args, "sdif from-json <file.json>") {
        Ok(path) => path,
        Err(code) => return code,
    };
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(err) => {
            eprintln!("cannot read {path}: {err}");
            return 1;
        }
    };
    match serde_json::from_str::<serde_json::Value>(&text) {
        Ok(value) => {
            print!("{}", sdif::json_to_sdif(&value));
            0
        }
        Err(err) => {
            eprintln!("{err}");
            1
        }
    }
}

fn cmd_ai_view(args: &[String]) -> i32 {
    let path = match require_path(args, "sdif ai-view <file.sdif>") {
        Ok(path) => path,
        Err(code) => return code,
    };
    match read_doc_from_file(path) {
        Ok(doc) => {
            print!("{}", sdif::ai_view(&doc));
            0
        }
        Err(err) => {
            eprintln!("{err}");
            1
        }
    }
}
