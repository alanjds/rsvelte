//! What `grass` 0.13.4 emits where dart-sass 1.103.1 emits something else.
//!
//! `scss-known-failures.json` lists 315 units, and `compatibility/deliberate-divergences.md`
//! decides that the render-neutral ones stay listed rather than being normalised away. That
//! decision is only enforceable if the behaviour it describes is pinned: each case below
//! records dart-sass's output beside the assertion, so a `grass` upgrade that converges (or
//! that moves a case from one class to another) fails here and the document gets re-read.
#![cfg(feature = "sass")]

use rsvelte_core::compiler::preprocess::types::{AttributeValue, PreprocessAttributeMap as Map};
use rsvelte_preprocess::filter::FilterOptions;
use rsvelte_preprocess::sass::{SassOptions, preprocess_sass};

fn scss(src: &str) -> String {
    let mut attrs = Map::default();
    attrs.insert(
        "lang".to_string(),
        AttributeValue::String("scss".to_string()),
    );
    preprocess_sass(
        &SassOptions::default(),
        &FilterOptions::default(),
        Some("./src/App.svelte"),
        src,
        &attrs,
    )
    .expect("compiles")
    .expect("not filtered out")
    .code
}

/// dart-sass: `color: rgb(91.3333333333%, 91.3333333333%, 91.3333333333%)`.
/// Same colour once each channel is rounded to 8 bits, which is what the classifier folds to.
#[test]
fn a_computed_colour_prints_in_the_legacy_shortest_form() {
    assert_eq!(
        scss("@use 'sass:color';\na { color: color.adjust(#eee, $lightness: -2%); }"),
        "a {\n  color: #e9e9e9;\n}"
    );
    // dart-sass: `color: rgb(100%, 41.3333333333%, 20%)` — 105.4 against 105 on the green channel.
    assert_eq!(
        scss("a { color: lighten(#f40, 10%); }"),
        "a {\n  color: #ff6933;\n}"
    );
}

/// dart-sass keeps the comment on the declaration's own line; `grass` moves it to the next.
#[test]
fn a_trailing_comment_moves_to_its_own_line() {
    assert_eq!(
        scss("a { color: red; /* keep */ }"),
        "a {\n  color: red;\n  /* keep */\n}"
    );
}

/// dart-sass indents every line of a wrapped selector list to the block; `grass` indents the first.
#[test]
fn a_wrapped_selector_list_inside_media_loses_its_indentation() {
    assert_eq!(
        scss("@media (min-width: 1px) {\n  a,\n  b {\n    color: red;\n  }\n}"),
        "@media (min-width: 1px) {\n  a,\nb {\n    color: red;\n  }\n}"
    );
}

/// Not render-neutral — this one changes the cascade.
/// dart-sass (since 1.77, the `mixed-decls` change): `.b a { color: red; }` then `.b { background: none; }`.
/// Reported in `upstream_issues/grass-hoists-a-declaration-written-after-a-nested-rule.md`.
#[test]
fn a_declaration_after_a_nested_rule_is_hoisted() {
    assert_eq!(
        scss(".b { a { color: red; } background: none; }"),
        ".b {\n  background: none;\n}\n.b a {\n  color: red;\n}"
    );
}

/// Not render-neutral — `0.4` is not a valid `grid-row`, so the browser drops the declaration.
/// Reported in `upstream_issues/grass-slash-list-divided-inside-a-nested-rule.md`.
#[test]
fn a_slash_list_is_divided_under_a_nested_not() {
    // dart-sass emits `grid-row: 2/5` here. Both conditions are load-bearing.
    assert_eq!(
        scss(".p { .q:not(.r) { grid-row: 2/5; } }"),
        ".p .q:not(.r) {\n  grid-row: 0.4;\n}"
    );
    // Drop either one and the two agree, so neither alone would discriminate.
    assert_eq!(
        scss(".p { .q { grid-row: 2/5; } }"),
        ".p .q {\n  grid-row: 2/5;\n}"
    );
    assert_eq!(
        scss(".q:not(.r) { grid-row: 2/5; }"),
        ".q:not(.r) {\n  grid-row: 2/5;\n}"
    );
    // `:not` is the only pseudo-class that does it.
    assert_eq!(
        scss(".p { .q:is(.r) { grid-row: 2/5; } }"),
        ".p .q:is(.r) {\n  grid-row: 2/5;\n}"
    );
}
