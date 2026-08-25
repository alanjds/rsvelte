//! Regression tests for #3694 — OXC accepts two expression forms that Acorn
//! rejects when Svelte parses template expressions as ES modules.
//!
//! These are AST checks, not text checks: `obj.await` is a legal property and
//! `super` remains legal in a class method. Every diagnostic below was measured
//! against the official compiler (Svelte v5.56.9).

use rsvelte_core::compiler::CompileError;
use rsvelte_core::{CompileOptions, GenerateMode, compile};

fn result(src: &str) -> Result<String, CompileError> {
    compile(
        src,
        CompileOptions {
            filename: Some("X.svelte".to_string()),
            generate: GenerateMode::Client,
            ..Default::default()
        },
    )
    .map(|compiled| compiled.js.code)
}

fn assert_parse_error(src: &str, token: &str, message: &str) {
    let err = result(src).expect_err("official rejects this expression");
    let CompileError::Parse(parse) = &err else {
        panic!("expected a parse error: {err:?}")
    };
    let text = format!("{parse:?}");
    assert!(text.contains("js_parse_error"), "{text}");
    assert!(text.contains(message), "{text}");
    assert_eq!(parse.span().0, src.find(token).expect("token is in source"));
}

#[test]
fn acorn_only_module_keywords_are_rejected_in_template_expressions() {
    for (src, token, message) in [
        ("{super.x}", "super", "'super' keyword outside a method"),
        (
            "{await}",
            "await",
            "Cannot use keyword 'await' outside an async function",
        ),
        (
            "{await.x}",
            "await",
            "Cannot use keyword 'await' outside an async function",
        ),
    ] {
        assert_parse_error(src, token, message);
    }
}

#[test]
fn the_expression_slot_does_not_change_the_answer() {
    for src in [
        "<div title={super.x}></div>",
        "{#if true}{@const x = super.x}<p>{x}</p>{/if}",
        "<div title={await}></div>",
        "{#if true}{@const x = await.x}<p>{x}</p>{/if}",
    ] {
        let token = if src.contains("super") {
            "super"
        } else {
            "await"
        };
        let message = if token == "super" {
            "'super' keyword outside a method"
        } else {
            "Cannot use keyword 'await' outside an async function"
        };
        assert_parse_error(src, token, message);
    }
}

#[test]
fn legal_property_and_nested_function_forms_stay_accepted() {
    for src in [
        "<p>{obj.await}</p>",
        "<p>{(async () => await value)()}</p>",
        "<p>{({ m() { return super.x; } }).m()}</p>",
        "<p>{({ get x() { return super.x; } }).x}</p>",
        "{@const C = class extends Base { m() { return super.m(); } }}<p>{C}</p>",
        "<script>class A extends B { m() { return super.m(); } }</script><p>ok</p>",
    ] {
        result(src).unwrap_or_else(|err| panic!("{src:?} must compile: {err:?}"));
    }
}
