//! Scratch probe (untracked): compiles one file as a component or a module.

use rsvelte_core::{CompileOptions, GenerateMode, ModuleCompileOptions, compile, compile_module};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(path) = args.iter().find(|a| !a.starts_with("--")) else {
        eprintln!("usage: runed_probe <file> [--server] [--dev]");
        std::process::exit(1);
    };
    let dev = args.iter().any(|a| a == "--dev");
    let generate = if args.iter().any(|a| a == "--server") {
        GenerateMode::Server
    } else {
        GenerateMode::Client
    };
    let source = std::fs::read_to_string(path).expect("read");

    if path.ends_with(".svelte") {
        match compile(
            &source,
            CompileOptions {
                generate,
                dev,
                filename: Some(path.clone()),
                ..Default::default()
            },
        ) {
            Ok(result) => {
                print!("{}", result.js.code);
                for w in &result.warnings {
                    eprintln!(
                        "WARN {} @ {:?}",
                        w.code,
                        w.start.as_ref().map(|s| (s.line, s.column))
                    );
                }
            }
            Err(err) => {
                eprintln!("{err:?}");
                std::process::exit(2);
            }
        }
    } else {
        match compile_module(
            &source,
            ModuleCompileOptions {
                generate,
                dev,
                filename: Some(path.clone()),
                ..Default::default()
            },
        ) {
            Ok(result) => print!("{}", result.js.code),
            Err(err) => {
                eprintln!("{err:?}");
                std::process::exit(2);
            }
        }
    }
}
