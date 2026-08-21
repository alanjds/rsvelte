//! Upstream edits a copy of the stylesheet, so whatever the source has between
//! two `:is()` / `:not()` / `:has()` / `:where()` arguments survives into the
//! output and one `/* (unused) … */` spans a whole run of pruned arguments.
//! rsvelte rebuilt the list from the transformed arguments and joined them with
//! `", "`, which dropped every comment and opened one comment per pruned
//! argument.
//!
//! Every expectation below is the official compiler's.

use rsvelte_core::{CompileOptions, GenerateMode, compile, compiler::CssMode};

fn css_of(markup: &str, rule: &str) -> String {
    let source = format!("{markup}\n<style>\n\t{rule}\n</style>\n");
    let result = compile(
        &source,
        CompileOptions {
            filename: Some("Test.svelte".to_string()),
            generate: GenerateMode::Client,
            dev: false,
            css: CssMode::External,
            ..Default::default()
        },
    )
    .expect("compile");
    let code = result.css.map(|c| c.code).unwrap_or_default();
    // The hash is derived from the filename and is not what these assert.
    let hash = format!(".{}", result_hash(&code));
    code.replace(&hash, ".H").trim().to_string()
}

fn result_hash(code: &str) -> String {
    code.split("svelte-")
        .nth(1)
        .map(|rest| {
            let end = rest
                .find(|c: char| !c.is_ascii_alphanumeric())
                .unwrap_or(rest.len());
            format!("svelte-{}", &rest[..end])
        })
        .unwrap_or_default()
}

#[test]
fn comments_inside_argument_lists_survive() {
    let both = "<b class=\"a\">x</b><i class=\"b\">y</i>";
    assert_eq!(
        css_of(both, ":is(.a /* t */) { color: red }"),
        ":is(.a.H /* t */) { color: red }"
    );
    assert_eq!(
        css_of(both, ":is(/* l */ .a) { color: red }"),
        ":is(/* l */ .a.H) { color: red }"
    );
    assert_eq!(
        css_of(both, ":is(.a /* m */, .b) { color: red }"),
        ":is(.a.H /* m */, .b.H) { color: red }"
    );
    assert_eq!(
        css_of(both, ":not(.a /* t */) { color: red }"),
        ".H:not(.a /* t */) { color: red }"
    );
    assert_eq!(
        css_of(both, ":where(.a /* t */) { color: red }"),
        ":where(.a.H /* t */) { color: red }"
    );
}

/// One comment spans a run of pruned arguments, not one per argument, and the
/// separator that survives is whatever the source wrote.
#[test]
fn a_run_of_pruned_arguments_is_one_comment() {
    let only_a = "<b class=\"a\">x</b>";
    assert_eq!(
        css_of(only_a, ":is(.a, .zz, .yy) { color: red }"),
        ":is(.a.H /* (unused) .zz, .yy*/) { color: red }"
    );
    assert_eq!(
        css_of(only_a, ":is(.zz, .yy, .a) { color: red }"),
        ":is(/* (unused) .zz, .yy,*/ .a.H) { color: red }"
    );
    assert_eq!(
        css_of(only_a, ":is(.zz, .a, .yy) { color: red }"),
        ":is(/* (unused) .zz,*/ .a.H /* (unused) .yy*/) { color: red }"
    );
    assert_eq!(
        css_of(only_a, ":is(.a,.b) { color: red }"),
        ":is(.a.H /* (unused) .b*/) { color: red }"
    );
    assert_eq!(
        css_of(only_a, ":is(.a  ,  .zz) { color: red }"),
        ":is(.a.H /* (unused) .zz*/) { color: red }"
    );
    assert_eq!(
        css_of(only_a, ":is(.zz /* c */, .a) { color: red }"),
        ":is(/* (unused) .zz /* c */,*/ .a.H) { color: red }"
    );
}
