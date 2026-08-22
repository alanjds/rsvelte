//! scratch: print CSS output + warnings + errors for one component.
use rsvelte_core::{CompileOptions, GenerateMode, compile, compiler::CssMode};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let path = args.iter().find(|a| !a.starts_with("--")).unwrap();
    let source = std::fs::read_to_string(path).unwrap();
    match compile(
        &source,
        CompileOptions {
            generate: GenerateMode::Client,
            dev: false,
            css: CssMode::External,
            filename: Some(path.clone()),
            ..Default::default()
        },
    ) {
        Ok(result) => {
            println!("CSS:\n{}", result.css.map(|c| c.code).unwrap_or_default());
            for w in &result.warnings {
                let (l, c) = w
                    .start
                    .as_ref()
                    .map(|p| (p.line, p.column))
                    .unwrap_or((0, 0));
                println!(
                    "WARN {}@{}:{} {}",
                    w.code,
                    l,
                    c,
                    w.message.lines().next().unwrap_or("")
                );
            }
        }
        Err(rsvelte_core::compiler::CompileError::Parse(
            rsvelte_core::error::ParseError::SvelteError { code, span, .. },
        )) => println!("ERROR {code}@{}", span.0),
        Err(other) => {
            let d = format!("{other:?}");
            let code = d
                .split("code: \"")
                .nth(1)
                .and_then(|r| r.split('"').next())
                .unwrap_or("?");
            let start = d
                .split("start: Some(")
                .nth(1)
                .and_then(|r| r.split(')').next())
                .unwrap_or("?");
            println!("ERROR {code}@{start}");
        }
    }
}
