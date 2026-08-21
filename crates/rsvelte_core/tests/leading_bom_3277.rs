//! Regression tests for #3277 — a leading UTF-8 BOM was emitted into the
//! generated template as document content.
//!
//! Upstream strips it in `compile` / `compileModule` / `parse` before anything
//! sees the source (`compiler/index.js` `remove_bom`), so the contract is not a
//! particular output string: it is that a BOM-prefixed source compiles to
//! exactly what the same source without the BOM compiles to. That is what these
//! tests assert, so no expected output is written by hand.
//!
//! What made it more than byte parity: the BOM became a template node, so the
//! extra-node flag and the `$.next()` / `var fragment = root()` shape changed
//! with it and a hydrating client walked a different node count than the server
//! produced.

use rsvelte_core::{CompileOptions, GenerateMode, ModuleCompileOptions, compile, compile_module};

const BOM: &str = "\u{feff}";

fn compile_component(src: &str, generate: GenerateMode, dev: bool) -> String {
    compile(
        src,
        CompileOptions {
            generate,
            dev,
            filename: Some("App.svelte".to_string()),
            ..Default::default()
        },
    )
    .expect("component should compile")
    .js
    .code
}

#[track_caller]
fn assert_bom_is_transparent(src: &str) {
    let with_bom = format!("{BOM}{src}");
    for generate in [GenerateMode::Client, GenerateMode::Server] {
        for dev in [false, true] {
            assert_eq!(
                compile_component(src, generate, dev),
                compile_component(&with_bom, generate, dev),
                "a leading BOM changed the output ({generate:?}, dev={dev}) for:\n{src}"
            );
        }
    }
}

#[test]
fn markup_only_component() {
    assert_bom_is_transparent("<p>x</p>\n");
}

#[test]
fn component_with_a_script() {
    assert_bom_is_transparent("<script>\n\tlet s = $state(1);\n</script>\n\n<p>{s}</p>\n");
}

#[test]
fn component_with_a_style_block() {
    assert_bom_is_transparent(
        "<p class=\"a\">x</p>\n\n<style>\n\t.a {\n\t\tcolor: red;\n\t}\n</style>\n",
    );
}

/// A BOM anywhere but the leading position is ordinary content and must survive
/// — the control that keeps the fix from becoming "strip U+FEFF everywhere".
#[test]
fn a_non_leading_bom_is_content() {
    let src = format!("<p>a{BOM}b</p>\n");
    let out = compile_component(&src, GenerateMode::Client, false);
    assert!(
        out.contains(BOM),
        "a U+FEFF inside the template is content and must be preserved:\n{out}"
    );
}

#[test]
fn module_entry_point() {
    let src = "export let n = $state(1);\n";
    let with_bom = format!("{BOM}{src}");
    for generate in [GenerateMode::Client, GenerateMode::Server] {
        let one = |source: &str| {
            compile_module(
                source,
                ModuleCompileOptions {
                    generate,
                    filename: Some("m.svelte.js".to_string()),
                    ..Default::default()
                },
            )
            .expect("module should compile")
            .js
            .code
        };
        assert_eq!(one(src), one(&with_bom));
    }
}
