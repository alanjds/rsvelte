//! Prints one component's generated JS, so a corpus divergence can be reproduced
//! from a single source file without standing the whole corpus harness back up.
//!
//! Usage:
//!   cargo run -p `rsvelte_devtools` --bin `compile_one` -- <file.svelte> [--server] [--dev]
//!     [--runes-false | --runes-true] [--css]
//!
//! `--css` prints `{ css, warn, err, errStart }` as JSON instead. The JS text
//! alone cannot show a CSS defect: a pruned rule differs in the stylesheet AND
//! in the warning set, and an over-rejection has no output at all — so the three
//! have to be read together.

use std::fmt::Write as _;

use rsvelte_core::{CompileOptions, CompileResult, GenerateMode, compile};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(path) = args.iter().find(|a| !a.starts_with("--")) else {
        eprintln!("usage: compile_one <file.svelte> [--server] [--dev] [--css]");
        std::process::exit(1);
    };
    let dev = args.iter().any(|a| a == "--dev");
    let css_mode = args.iter().any(|a| a == "--css");
    let runes = if args.iter().any(|a| a == "--runes-false") {
        Some(false)
    } else if args.iter().any(|a| a == "--runes-true") {
        Some(true)
    } else {
        None
    };
    let generate = if args.iter().any(|a| a == "--server") {
        GenerateMode::Server
    } else {
        GenerateMode::Client
    };

    let source = match std::fs::read_to_string(path) {
        Ok(source) => source,
        Err(err) => {
            eprintln!("{path}: {err}");
            std::process::exit(1);
        }
    };

    match compile(
        &source,
        CompileOptions {
            generate,
            dev,
            runes,
            filename: Some(path.clone()),
            ..Default::default()
        },
    ) {
        Ok(result) => {
            if css_mode {
                print!("{}", css_report(&result));
            } else {
                print!("{}", result.js.code);
            }
        }
        Err(err) => {
            if !css_mode {
                eprintln!("{err:?}");
                std::process::exit(2);
            }
            let diagnostic = err.diagnostic();
            print!(
                "{{\"css\":null,\"warn\":[],\"err\":{},\"errStart\":{}}}",
                json_string(diagnostic.code.as_deref().unwrap_or("?")),
                diagnostic.span.map_or(-1i64, |(start, _)| i64::from(start))
            );
        }
    }
}

/// `{ css, warn, err, errStart }` for one compile, warnings rendered as
/// `code@line:column` so the harness can compare them to the official
/// compiler's own warning shape.
fn css_report(result: &CompileResult) -> String {
    let mut warnings: Vec<String> = result
        .warnings
        .iter()
        .map(|warning| {
            let (line, column) = warning
                .start
                .as_ref()
                .map_or((0, 0), |start| (start.line, start.column));
            format!("{}@{}:{}", warning.code, line, column)
        })
        .collect();
    warnings.sort();
    let warnings: Vec<String> = warnings
        .iter()
        .map(|warning| json_string(warning))
        .collect();
    format!(
        "{{\"css\":{},\"warn\":[{}],\"err\":null,\"errStart\":-1}}",
        result
            .css
            .as_ref()
            .map_or_else(|| "null".to_string(), |css| json_string(&css.code)),
        warnings.join(",")
    )
}

fn json_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
