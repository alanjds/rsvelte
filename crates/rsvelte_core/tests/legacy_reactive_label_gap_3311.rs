//! Issue #3311 — whitespace or a comment between `$` and `:`.
//!
//! Three text scanners in the client pipeline recognised a legacy reactive
//! statement by the literal two bytes `$:`, while upstream matches a
//! `LabeledStatement` whose label is `$` — so anything JavaScript allows
//! between the name and its colon left the statement unlowered.
//!
//! Every expectation below was measured against the official compiler
//! (`submodules/svelte/.../compiler/index.js`) on the same source. The server
//! target emits `$: out = a + 1;` for these with and without the gap, so it is
//! a control that *cannot move* — it is not evidence the parse is fine.

use rsvelte_core::{CompileOptions, GenerateMode, compile};

fn compile_to(source: &str, generate: GenerateMode) -> String {
    compile(
        source,
        CompileOptions {
            filename: Some("A.svelte".to_string()),
            generate,
            ..Default::default()
        },
    )
    .expect("compile failed")
    .js
    .code
}

fn with_gap(gap: &str) -> String {
    format!(
        "<script>\n\texport let a = 1;\n\tlet out = 0;\n\t${gap}: out = a + 1;\n</script>\n<p>{{out}}</p>\n"
    )
}

/// The axis is "is there a gap", not which character sits in it.
const GAPS: &[(&str, &str)] = &[
    ("space", " "),
    ("two spaces", "  "),
    ("tab", "\t"),
    ("newline", "\n"),
    ("CRLF", "\r\n"),
    ("CR", "\r"),
    ("block comment", "/*c*/"),
    ("padded block comment", " /*c*/ "),
    ("NBSP", "\u{a0}"),
    ("form feed", "\u{c}"),
    ("vertical tab", "\u{b}"),
    ("U+2028", "\u{2028}"),
];

#[test]
fn a_gap_between_the_dollar_and_the_colon_still_lowers_the_statement() {
    for (name, gap) in GAPS {
        let out = compile_to(&with_gap(gap), GenerateMode::Client);
        assert!(
            out.contains("$.legacy_pre_effect(() => ($.deep_read_state(a())), () => {"),
            "gap `{name}` left the statement unlowered:\n{out}"
        );
        assert!(
            !out.contains("$: $.set"),
            "gap `{name}` emitted a bare `$:` label:\n{out}"
        );
    }
}

/// The control: no gap at all was already correct and must stay so.
#[test]
fn no_gap_is_unchanged() {
    let out = compile_to(&with_gap(""), GenerateMode::Client);
    assert!(
        out.contains("$.legacy_pre_effect(() => ($.deep_read_state(a())), () => {"),
        "the no-gap control moved:\n{out}"
    );
}

/// A `$` that is not a reactive label must not become one. `$x:` is an ordinary
/// label and `$ = 1` is an assignment; official emits neither effect.
#[test]
fn a_dollar_that_is_not_a_label_is_left_alone() {
    let other_label =
        "<script>\n\tlet out = 0;\n\t$x: out = 1;\n\tvoid out;\n</script>\n<p>{out}</p>\n";
    let out = compile_to(other_label, GenerateMode::Client);
    assert!(
        !out.contains("$.legacy_pre_effect("),
        "an ordinary `$x:` label was lowered as a reactive statement:\n{out}"
    );
}

/// Phase 2 counts labels from the AST and Phase 3 counted them by this scan, so
/// a missed label also shifted every later statement's dependency list — the
/// second effect here was built from the *first* statement's dependencies.
#[test]
fn a_missed_label_does_not_shift_the_dependency_list_of_the_next_one() {
    let source = "<script>\n\texport let a = 1;\n\texport let b = 2;\n\tlet one = 0;\n\tlet two = 0;\n\t$ : one = a + 1;\n\t$: two = b + 1;\n</script>\n<p>{one}{two}</p>\n";
    let out = compile_to(source, GenerateMode::Client);
    assert!(
        out.contains("$.legacy_pre_effect(() => ($.deep_read_state(b())), () => {"),
        "the second statement did not get its own dependency list:\n{out}"
    );
}

/// The server runs the statements in order, so the label is inert there. This
/// asserts the control did not move, not that the server is right.
#[test]
fn the_server_is_unchanged_with_and_without_the_gap() {
    let plain = compile_to(&with_gap(""), GenerateMode::Server);
    let spaced = compile_to(&with_gap(" "), GenerateMode::Server);
    assert!(
        plain.contains("out = a + 1;"),
        "server control moved:\n{plain}"
    );
    assert!(
        spaced.contains("out = a + 1;"),
        "server control moved:\n{spaced}"
    );
}
