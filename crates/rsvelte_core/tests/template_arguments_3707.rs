//! Regression tests for #3707: template expressions use a typed expression
//! walker and therefore never reached the script-side `Identifier` visitor's
//! `invalid_arguments_usage` check.

use rsvelte_core::{CompileOptions, GenerateMode, compile};

fn compile_error(
    source: &str,
    generate: GenerateMode,
) -> Option<(Option<String>, Option<(u32, u32)>)> {
    match compile(
        source,
        CompileOptions {
            filename: Some("main.svelte".to_string()),
            generate,
            ..Default::default()
        },
    ) {
        Ok(_) => None,
        Err(error) => {
            let diagnostic = error.diagnostic();
            Some((diagnostic.code, diagnostic.span))
        }
    }
}

#[track_caller]
fn assert_invalid_arguments(source: &str) {
    let start = source
        .find("arguments")
        .expect("test case contains arguments") as u32;
    let expected = Some((
        Some("invalid_arguments_usage".to_string()),
        Some((start, start + "arguments".len() as u32)),
    ));
    for generate in [GenerateMode::Client, GenerateMode::Server] {
        assert_eq!(
            compile_error(source, generate),
            expected,
            "for {source:?} ({generate:?})"
        );
    }
}

#[track_caller]
fn assert_compiles(source: &str) {
    for generate in [GenerateMode::Client, GenerateMode::Server] {
        assert_eq!(
            compile_error(source, generate),
            None,
            "for {source:?} ({generate:?})"
        );
    }
}

#[test]
fn template_references_to_arguments_are_rejected() {
    assert_invalid_arguments("{arguments}");
    assert_invalid_arguments("<p title={arguments.length}></p>");
    assert_invalid_arguments("{#if String(arguments)}ok{/if}");
    assert_invalid_arguments("{#each [arguments[0]] as item}{item}{/each}");
    assert_invalid_arguments("{#if true}{@const value = arguments}<p>{value}</p>{/if}");
    assert_invalid_arguments("{@html arguments}");
}

#[test]
fn a_top_level_arrow_does_not_introduce_arguments() {
    assert_invalid_arguments("{(() => arguments)()}");
}

#[test]
fn an_ordinary_function_introduces_arguments_for_nested_arrows() {
    assert_compiles("{(function () { return arguments; })()}");
    assert_compiles("{(function () { return () => arguments; })()}");
    assert_compiles("{(function (value = arguments) { return value; })()}");
    assert_compiles("{ok}");
}
