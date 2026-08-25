//! Regression coverage for #3705: a declaration tag may contain a single
//! `let` declarator without an initializer.
//!
//! Upstream parses the whole tag body as a statement and accepts it whenever
//! the result is a `VariableDeclaration`. rsvelte's single-declarator path
//! instead required a top-level `=`, even though its multi-declarator builder
//! already represented a missing initializer as ESTree `null`.

use rsvelte_core::ast::arena::with_serialize_arena;
use rsvelte_core::{CompileOptions, GenerateMode, ParseOptions, compile, parse};

fn ast_json(source: &str) -> serde_json::Value {
    let ast = parse(
        source,
        &oxc_allocator::Allocator::default(),
        ParseOptions::default(),
    )
    .expect("declaration tag should parse");
    let json = with_serialize_arena(&ast.arena, || serde_json::to_string(&ast).unwrap());
    serde_json::from_str(&json).unwrap()
}

fn find_declaration_tag(value: &serde_json::Value) -> Option<&serde_json::Value> {
    match value {
        serde_json::Value::Object(object) => {
            if object.get("type").and_then(|value| value.as_str()) == Some("DeclarationTag") {
                return Some(value);
            }
            object.values().find_map(find_declaration_tag)
        }
        serde_json::Value::Array(items) => items.iter().find_map(find_declaration_tag),
        _ => None,
    }
}

#[test]
fn a_bare_let_declarator_has_a_null_initializer_and_exact_spans() {
    let source = "{let x}";
    let json = ast_json(source);
    let tag = find_declaration_tag(&json).expect("DeclarationTag");
    let declaration = &tag["declaration"];
    let declarator = &declaration["declarations"][0];

    assert_eq!(tag["start"], 0);
    assert_eq!(tag["end"], 7);
    assert_eq!(declaration["type"], "VariableDeclaration");
    assert_eq!(declaration["kind"], "let");
    assert_eq!(declaration["start"], 1);
    assert_eq!(declaration["end"], 6);
    assert_eq!(declarator["id"]["type"], "Identifier");
    assert_eq!(declarator["id"]["name"], "x");
    assert_eq!(declarator["id"]["start"], 5);
    assert_eq!(declarator["id"]["end"], 6);
    assert!(declarator["init"].is_null());
}

#[test]
fn a_bare_let_compiles_in_every_template_slot_and_target() {
    for source in [
        "{let x}<p>ok</p>",
        "{#if true}{let x}<p>ok</p>{/if}",
        "{#each [1] as item}{let x}<p>{item}</p>{/each}",
    ] {
        for generate in [GenerateMode::Client, GenerateMode::Server] {
            compile(
                source,
                CompileOptions {
                    filename: Some("Component.svelte".to_string()),
                    generate,
                    ..Default::default()
                },
            )
            .unwrap_or_else(|error| panic!("{source:?} must compile for {generate:?}: {error:?}"));
        }
    }
}

#[test]
fn const_without_an_initializer_remains_invalid() {
    let error = compile(
        "{const x}",
        CompileOptions {
            filename: Some("Component.svelte".to_string()),
            generate: GenerateMode::Client,
            ..Default::default()
        },
    )
    .expect_err("const declarations still require an initializer");

    assert!(format!("{error:?}").contains("js_parse_error"), "{error:?}");
}
