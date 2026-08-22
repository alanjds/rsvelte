//! `class`-attribute whitespace normalization (#3096).
//!
//! Ported from prettier-plugin-svelte's `Text` printer, which applies two
//! ordered replacements to the raw text of a `class` value — and only when the
//! host is a `RegularElement`. Every expectation below was read off the
//! `oxfmt(svelte: true)` oracle the corpus gate uses.

use rsvelte_formatter::{FormatOptions, format};

fn fmt(src: &str) -> String {
    let out = format(src, &FormatOptions::default()).expect("format ok");
    out.strip_suffix('\n').map(str::to_string).unwrap_or(out)
}

#[test]
fn collapses_interior_runs_but_only_for_class() {
    assert_eq!(
        fmt(r#"<div class="a  b   c" title="x  y"></div>"#),
        r#"<div class="a b c" title="x  y"></div>"#
    );
}

#[test]
fn collapses_tabs_too() {
    assert_eq!(fmt("<div class=\"a\t\tb\"></div>"), r#"<div class="a b"></div>"#);
}

#[test]
fn keeps_leading_whitespace_and_drops_the_trailing_run() {
    assert_eq!(
        fmt(r#"<div class="  lead and trail  "></div>"#),
        r#"<div class="  lead and trail"></div>"#
    );
}

#[test]
fn an_all_whitespace_value_is_left_alone() {
    // Neither pass has a `[^ \t\n]` character to anchor on.
    assert_eq!(fmt(r#"<div class="   "></div>"#), r#"<div class="   "></div>"#);
}

#[test]
fn a_run_before_a_newline_vanishes_and_the_lines_survive() {
    assert_eq!(
        fmt("<div class=\"a  \n   b   \n  c  \"></div>"),
        "<div\n  class=\"a\n   b\n  c\"\n></div>"
    );
}

#[test]
fn a_trailing_run_on_its_own_line_is_left_alone() {
    // The end-of-string run is preceded by a newline, so neither pass's
    // `[^ \t\n]` anchor matches.
    assert_eq!(
        fmt("<div class=\"a\n  \"></div>"),
        "<div\n  class=\"a\n  \"\n></div>"
    );
}

#[test]
fn a_run_before_an_interpolation_collapses_to_one_space() {
    // The text part is not last in the value, so the second pass shrinks the
    // trailing run to a single space instead of removing it.
    assert_eq!(
        fmt(r#"<div class="a  {b}"></div>"#),
        r#"<div class="a {b}"></div>"#
    );
}

#[test]
fn whitespace_between_two_interpolations_is_untouched() {
    // The text part between them starts with whitespace, so no `[^ \t\n]`
    // anchors either pass.
    assert_eq!(
        fmt(r#"<div class="{a}  {b}"></div>"#),
        r#"<div class="{a}  {b}"></div>"#
    );
}

#[test]
fn a_pre_element_is_still_a_regular_element() {
    assert_eq!(fmt(r#"<pre class="a  b   c">x</pre>"#), r#"<pre class="a b c">x</pre>"#);
}

#[test]
fn a_root_title_is_a_regular_element() {
    // `TitleElement` is only produced inside `<svelte:head>`; a root `<title>`
    // is a `RegularElement`, so it normalizes.
    assert_eq!(
        fmt(r#"<title class="a  b   c">t</title>"#),
        r#"<title class="a b c">t</title>"#
    );
}

#[test]
fn a_head_title_is_printed_verbatim() {
    assert_eq!(
        fmt(r#"<svelte:head><title class="a  b   c">t</title></svelte:head>"#),
        r#"<svelte:head><title class="a  b   c">t</title></svelte:head>"#
    );
}

#[test]
fn a_slot_class_is_printed_verbatim() {
    assert_eq!(fmt(r#"<slot class="a  b   c" />"#), r#"<slot class="a  b   c" />"#);
}

#[test]
fn a_component_class_is_printed_verbatim() {
    assert_eq!(fmt(r#"<Foo class="a  b   c" />"#), r#"<Foo class="a  b   c" />"#);
}

#[test]
fn a_svelte_element_class_is_printed_verbatim() {
    assert_eq!(
        fmt(r#"<svelte:element this="div" class="a  b   c"></svelte:element>"#),
        r#"<svelte:element this="div" class="a  b   c"></svelte:element>"#
    );
}

#[test]
fn a_svelte_body_class_is_printed_verbatim() {
    assert_eq!(
        fmt(r#"<svelte:body class="a  b   c" />"#),
        r#"<svelte:body class="a  b   c" />"#
    );
}
