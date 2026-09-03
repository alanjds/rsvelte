//! The `$$props` → `$$sanitized_props` rewrite is about READS, not about lines.
//!
//! Upstream runs it as an AST read transform
//! (`read: (node) => ({ ...node, name: '$$sanitized_props' })`), so the first
//! argument of a generated `$.prop` / `$.bind_prop` / `$.legacy_rest_props` call
//! — a binding position, not a read — is left alone without any rule for it.
//! rsvelte approximated that by skipping every LINE carrying one of those calls,
//! which also skipped a genuine read inside a default-value thunk.
//!
//! Every expected string is the official compiler's output for the same source.
//! The rows that already passed are kept: they are what says the narrowing did
//! not start rewriting a binding position.

use rsvelte_core::{CompileOptions, GenerateMode, compile};

fn client(source: &str, dev: bool) -> String {
    compile(
        source,
        CompileOptions {
            filename: Some("X.svelte".to_string()),
            generate: GenerateMode::Client,
            dev,
            ..Default::default()
        },
    )
    .expect("compile component")
    .js
    .code
}

#[track_caller]
fn has(out: &str, expected: &str) {
    assert!(out.contains(expected), "expected `{expected}`. Got:\n{out}");
}

#[track_caller]
fn lacks(out: &str, unexpected: &str) {
    assert!(
        !out.contains(unexpected),
        "did not expect `{unexpected}`. Got:\n{out}"
    );
}

/// The two binding positions every legacy cell here carries. `$.push($$props,`
/// takes a third argument in dev, so it is matched by prefix.
#[track_caller]
fn binding_positions_keep_the_raw_name(out: &str) {
    has(
        out,
        "const $$sanitized_props = $.legacy_rest_props($$props, ['children', '$$slots', '$$events', '$$legacy']);",
    );
    has(out, "$.push($$props, false");
    has(out, "$.prop($$props, 'a',");
}

#[test]
fn a_read_in_a_prop_default_resolves_against_the_sanitized_object() {
    for dev in [false, true] {
        let out = client(
            "<script>\n\texport let a = $$props.b;\n</script>\n<div {...$$restProps}>{a}</div>",
            dev,
        );
        has(
            out.as_str(),
            "let a = $.prop($$props, 'a', 24, () => $$sanitized_props.b);",
        );
        binding_positions_keep_the_raw_name(&out);
    }
}

#[test]
fn every_read_on_the_line_resolves_not_just_the_first() {
    for dev in [false, true] {
        let out = client(
            "<script>\n\texport let a = $$props.b ?? $$props.c;\n</script>\n<div {...$$restProps}>{a}</div>",
            dev,
        );
        has(
            out.as_str(),
            "let a = $.prop($$props, 'a', 24, () => $$sanitized_props.b ?? $$sanitized_props.c);",
        );
        binding_positions_keep_the_raw_name(&out);
    }
}

#[test]
fn a_bindable_prop_default_resolves_too() {
    for dev in [false, true] {
        let out = client(
            "<script>\n\timport C from './C.svelte';\n\texport let a = $$props.b;\n</script>\n<C bind:a />\n<div {...$$restProps}></div>",
            dev,
        );
        has(
            out.as_str(),
            "let a = $.prop($$props, 'a', 28, () => $$sanitized_props.b);",
        );
        binding_positions_keep_the_raw_name(&out);
    }
}

#[test]
fn a_component_with_no_rest_props_resolves_the_same_way() {
    for dev in [false, true] {
        let out = client("<script>\n\texport let a = $$props.b;\n</script>\n{a}", dev);
        has(
            out.as_str(),
            "let a = $.prop($$props, 'a', 24, () => $$sanitized_props.b);",
        );
        binding_positions_keep_the_raw_name(&out);
    }
}

/// The control that names the scanning unit rather than the rule: the same read,
/// on a line the skip list never matched, was already correct.
#[test]
fn a_read_in_the_body_was_already_correct_and_stays_correct() {
    for dev in [false, true] {
        let out = client(
            "<script>\n\texport let a = 1;\n\tconst z = $$props.b;\n</script>\n<div {...$$restProps}>{a}{z}</div>",
            dev,
        );
        has(out.as_str(), "const z = $$sanitized_props.b;");
        has(out.as_str(), "let a = $.prop($$props, 'a', 8, 1);");
        binding_positions_keep_the_raw_name(&out);
    }
}

/// A `{...$$props}` spread is a read and upstream rewrites it; a runes component
/// never declares `$$sanitized_props` at all. Both were already correct and are
/// the cells a narrowing could most easily break in opposite directions.
#[test]
fn a_whole_props_spread_is_a_read_and_a_runes_component_is_not_rewritten() {
    for dev in [false, true] {
        let spread = client(
            "<script>\n\texport let a = 1;\n</script>\n<div {...$$props}>{a}</div>",
            dev,
        );
        has(
            spread.as_str(),
            "$.attribute_effect(div, () => ({ ...$$sanitized_props }));",
        );
        has(spread.as_str(), "let a = $.prop($$props, 'a', 8, 1);");

        let runes = client("<script>\n\tlet { a } = $props();\n</script>\n{a}", dev);
        lacks(runes.as_str(), "$$sanitized_props");
        has(runes.as_str(), "$$props.a");
    }
}
