//! Restricted identifiers in template expressions and binding patterns.
//!
//! The template expression walker used to bypass the script-side `Identifier`
//! visitor, while several template binding hosts parsed patterns through
//! different paths. These tables pin the official diagnostic, message and byte
//! range at the two boundaries (#3707 and #3728).

use rsvelte_core::{CompileOptions, GenerateMode, compile};

fn diagnostic(src: &str, generate: GenerateMode) -> Option<(String, String, (u32, u32))> {
    compile(
        src,
        CompileOptions {
            filename: Some("Comp.svelte".into()),
            generate,
            ..Default::default()
        },
    )
    .err()
    .map(|err| {
        let d = err.diagnostic();
        (
            d.code.unwrap_or_default(),
            d.message.lines().next().unwrap_or_default().to_string(),
            d.span.unwrap_or((u32::MAX, u32::MAX)),
        )
    })
}

fn assert_error(src: &str, anchor: &str, code: &str, message: &str, end_delta: usize) {
    let start = src.find(anchor).expect("anchor must occur") as u32;
    let expected = (
        code.to_string(),
        message.to_string(),
        (start, start + end_delta as u32),
    );
    for generate in [GenerateMode::Client, GenerateMode::Server] {
        assert_eq!(
            diagnostic(src, generate),
            Some(expected.clone()),
            "for {src:?}"
        );
    }
}

const ARGUMENTS_MESSAGE: &str =
    "The arguments keyword cannot be used within the template or at the top level of a component";

#[test]
fn arguments_is_rejected_in_every_template_expression_host() {
    for src in [
        "{arguments}",
        "<div title={arguments}></div>",
        "{#if arguments}<p />{/if}",
        "{#each arguments as value}<p />{/each}",
        "{#if true}{@const value = arguments}{/if}",
        "{@html arguments}",
        "{arguments.value}",
        "{arguments['value']}",
        "{String(arguments)}",
        "{(() => arguments)()}",
    ] {
        assert_error(
            src,
            "arguments",
            "invalid_arguments_usage",
            ARGUMENTS_MESSAGE,
            "arguments".len(),
        );
    }
}

#[test]
fn an_ordinary_function_provides_arguments_but_an_arrow_does_not() {
    for src in [
        "{(function () { return arguments })()}",
        "{(function () { return () => arguments })()}",
    ] {
        for generate in [GenerateMode::Client, GenerateMode::Server] {
            assert_eq!(diagnostic(src, generate), None, "must compile: {src:?}");
        }
    }

    assert_error(
        "{(() => arguments)()}",
        "arguments",
        "invalid_arguments_usage",
        ARGUMENTS_MESSAGE,
        "arguments".len(),
    );
}

const RESERVED_ARGUMENTS: &str =
    "'arguments' is a reserved word in JavaScript and cannot be used here";
const RESERVED_EVAL: &str = "'eval' is a reserved word in JavaScript and cannot be used here";

#[test]
fn plain_template_bindings_use_the_reserved_word_diagnostic() {
    for (src, name, message) in [
        (
            "{#each [] as arguments}{/each}",
            "arguments",
            RESERVED_ARGUMENTS,
        ),
        ("{#each [] as value, eval}{/each}", "eval", RESERVED_EVAL),
        (
            "{#if true}{@const arguments = 1}{/if}",
            "arguments",
            RESERVED_ARGUMENTS,
        ),
        ("{#if true}{@const eval = 1}{/if}", "eval", RESERVED_EVAL),
    ] {
        assert_error(src, name, "unexpected_reserved_word", message, 0);
    }
}

#[test]
fn destructured_template_bindings_use_the_strict_assignment_diagnostic() {
    for (src, name) in [
        ("{#each [] as { arguments }}{/each}", "arguments"),
        ("{#each [] as [eval]}{/each}", "eval"),
        (
            "{#await Promise.resolve() then { arguments }}{/await}",
            "arguments",
        ),
        ("{#await Promise.reject() catch [eval]}{/await}", "eval"),
        ("{#if true}{@const { arguments } = {}}{/if}", "arguments"),
        ("{#if true}{@const [eval] = []}{/if}", "eval"),
    ] {
        assert_error(
            src,
            name,
            "js_parse_error",
            &format!("Assigning to {name} in strict mode"),
            0,
        );
    }
}

#[test]
fn neighbouring_ordinary_bindings_still_compile() {
    for src in [
        "{#each [] as value, index}{value}{index}{/each}",
        "{#each [] as { value }}{value}{/each}",
        "{#await Promise.resolve() then { value }}{value}{/await}",
        "{#if true}{@const { value } = { value: 1 }}{value}{/if}",
    ] {
        for generate in [GenerateMode::Client, GenerateMode::Server] {
            assert_eq!(diagnostic(src, generate), None, "must compile: {src:?}");
        }
    }
}
