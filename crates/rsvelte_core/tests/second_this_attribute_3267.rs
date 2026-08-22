//! `<svelte:element>` / `<svelte:component>` consume ONE `this` attribute as
//! their tag; upstream's parser `splice(index, 1)`s exactly that one and leaves
//! any further `this=` in `attributes`, where it is rendered as an ordinary
//! attribute (or passed as a prop). rsvelte filtered the list by name, so every
//! `this` was dropped and the second one vanished from the output (issue #3267).
//!
//! The `svelte_element_invalid_this` warning is scoped the same way: upstream
//! asks it of the spliced definition only, so a second, non-expression `this`
//! is an ordinary attribute and warns about nothing.

use rsvelte_core::{CompileOptions, GenerateMode, compile, compiler::CssMode};

fn compile_ok(src: &str, generate: GenerateMode) -> (String, Vec<String>) {
    let result = compile(
        src,
        CompileOptions {
            filename: Some("Test.svelte".to_string()),
            generate,
            dev: false,
            css: CssMode::External,
            ..Default::default()
        },
    )
    .expect("must compile");
    let codes = result.warnings.iter().map(|w| w.code.clone()).collect();
    (result.js.code, codes)
}

const SCRIPT: &str = "<script>\n\timport C from './C.svelte';\n\tlet tag = $state('div');\n\tlet tag2 = $state('span');\n</script>\n";

#[test]
fn a_second_this_on_svelte_element_is_an_ordinary_attribute() {
    let (code, _) = compile_ok(
        &format!("{SCRIPT}<svelte:element this={{tag}} this={{tag2}}>x</svelte:element>"),
        GenerateMode::Client,
    );
    assert!(
        code.contains("$.attribute_effect($$element, () => ({ this: tag2 }))"),
        "the second `this` must reach the attribute effect: {code}"
    );
    // …and the FIRST one is still the tag.
    assert!(
        code.contains("$.element(node, () => tag, false"),
        "the first `this` must still be the tag: {code}"
    );
}

#[test]
fn a_second_this_on_svelte_component_is_a_prop() {
    let (code, _) = compile_ok(
        &format!("{SCRIPT}<svelte:component this={{C}} this={{tag2}} />"),
        GenerateMode::Client,
    );
    assert!(
        code.contains("this"),
        "the second `this` must be passed as a prop: {code}"
    );
    assert!(
        !code.contains("$$component($$anchor, {})"),
        "the prop object must not be empty: {code}"
    );
}

/// The warning is asked of the definition, not of every `this`. Both directions,
/// because a fix that simply stops warning would pass the second row alone.
#[test]
fn svelte_element_invalid_this_is_scoped_to_the_definition() {
    for generate in [GenerateMode::Client, GenerateMode::Server] {
        let (_, warned) = compile_ok(
            &format!("{SCRIPT}<svelte:element this=\"a\">x</svelte:element>"),
            generate,
        );
        assert!(
            warned.iter().any(|c| c == "svelte_element_invalid_this"),
            "a lone string `this` must warn ({generate:?}): {warned:?}"
        );

        let (_, not_warned) = compile_ok(
            &format!("{SCRIPT}<svelte:element this={{tag}} this=\"a\">x</svelte:element>"),
            generate,
        );
        assert!(
            !not_warned
                .iter()
                .any(|c| c == "svelte_element_invalid_this"),
            "a SECOND string `this` is an ordinary attribute and must not warn ({generate:?}): {not_warned:?}"
        );
    }
}

/// A valueless `this` is still the definition even when a valid one follows, so
/// the error is unchanged — `splice` takes the first match regardless of value.
#[test]
fn a_valueless_first_this_still_errors() {
    let err = compile(
        &format!("{SCRIPT}<svelte:element this this={{tag}}>x</svelte:element>"),
        CompileOptions {
            filename: Some("Test.svelte".to_string()),
            generate: GenerateMode::Client,
            dev: false,
            css: CssMode::External,
            ..Default::default()
        },
    )
    .err()
    .map(|e| format!("{e:?}"))
    .expect("a valueless `this` must not compile");
    assert!(
        err.contains("svelte_element_missing_this"),
        "expected svelte_element_missing_this, got: {err}"
    );
}
