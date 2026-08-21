//! Regression tests for #3256 and #3259 — client attribute ROUTING.
//!
//! Each case below is a decision about which codegen route an attribute takes,
//! and every one of them is scored byte-for-byte against the official compiler
//! by the `pattern/issues/3256-*` / `3259-*` corpus entries. These assertions
//! restate the route so a `cargo test` run reports which decision moved.

use rsvelte_core::{CompileOptions, GenerateMode, compile};

fn client(src: &str) -> String {
    compile(
        src,
        CompileOptions {
            generate: GenerateMode::Client,
            filename: Some("App.svelte".to_string()),
            ..Default::default()
        },
    )
    .expect("component should compile")
    .js
    .code
}

const STATE: &str = "<script>let s = $state('a');</script>";

/// `needs_clsx` is decided in phase 2 off the RAW name (`node.name === 'class'`),
/// while the `set_class` route is decided off the normalized one — so a
/// non-lowercase `class` reaches `$.set_class` without the wrap.
#[test]
fn uppercase_class_expression_is_not_wrapped_in_clsx() {
    let out = client(&format!("{STATE}<div CLASS={{s}}>x</div>"));
    assert!(out.contains("$.set_class("), "{out}");
    assert!(!out.contains("$.clsx("), "{out}");

    let lowercase = client(&format!("{STATE}<div class={{s}}>x</div>"));
    assert!(lowercase.contains("$.clsx("), "{lowercase}");
}

/// `inert` is not in upstream's `NON_STATIC_PROPERTIES`, and the list is
/// consulted with the raw name — so `MUTED` stays a template attribute while
/// `muted` does not.
#[test]
fn static_non_static_properties_use_the_raw_name() {
    for src in [
        "<div inert=\"a\">x</div>",
        "<div inert>x</div>",
        "<div INERT=\"a\">x</div>",
        "<div MUTED=\"a\">x</div>",
        "<div DEFAULTVALUE=\"a\">x</div>",
    ] {
        let out = client(src);
        assert!(
            !out.contains("div.inert")
                && !out.contains("div.muted")
                && !out.contains("defaultValue"),
            "{src} took the DOM-property route:\n{out}"
        );
    }

    let lowercase = client("<div muted=\"a\">x</div>");
    assert!(lowercase.contains("div.muted"), "{lowercase}");
}

/// `is` is consumed by the parser at element-creation time, so it goes into the
/// template whenever its BUILT value is a string literal — a foldable
/// concatenation included.
#[test]
fn foldable_is_attribute_goes_into_the_template() {
    let out = client(&format!("{STATE}<div is=\"a{{s}}b\">x</div>"));
    assert!(out.contains("is=\"aab\""), "{out}");
    assert!(!out.contains("set_custom_element_data"), "{out}");

    let dynamic = client("<script>let s = $state('a'); s = 'b';</script><div is={s}>x</div>");
    assert!(dynamic.contains("set_custom_element_data"), "{dynamic}");
}

/// `innerHTML` / `textContent` / `innerText` are not DOM properties upstream.
/// Only the SVG namespace can observe it: elsewhere `get_attribute_name`
/// lowercases the name before the list is consulted.
#[test]
fn svg_content_attributes_use_set_attribute() {
    for name in ["innerHTML", "textContent", "innerText"] {
        let out = client(&format!("{STATE}<svg><rect {name}={{s}} /></svg>"));
        assert!(
            out.contains(&format!("$.set_attribute(rect, '{name}'")),
            "{name} took the DOM-property route:\n{out}"
        );
    }
}

/// The `?? ''` guard follows `scope.evaluate(value).is_defined`, whose value
/// domain covers functions, binary results and every unary but `void`.
#[test]
fn option_value_guard_follows_evaluate() {
    for value in [
        "() => 1",
        "function () {}",
        "1 + 1",
        "-1",
        "typeof s",
        "!s",
        "s === 1",
    ] {
        let out = client(&format!(
            "{STATE}<select><option value={{{value}}}>x</option></select>"
        ));
        assert!(
            !out.contains("?? ''") && !out.contains("?? \"\""),
            "`{value}` is known to be defined, so the guard must be dropped:\n{out}"
        );
    }

    // `s` is deliberately absent: a never-reassigned `$state('a')` evaluates to
    // its initializer upstream, so it is DEFINED — the guard is dropped there too.
    for value in ["void 0", "null", "undefined", "s.k"] {
        let out = client(&format!(
            "{STATE}<select><option value={{{value}}}>x</option></select>"
        ));
        assert!(
            out.contains("?? ''") || out.contains("?? \"\""),
            "`{value}` can be nullish, so the guard must stay:\n{out}"
        );
    }
}
