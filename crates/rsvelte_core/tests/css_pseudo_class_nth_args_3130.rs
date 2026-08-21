//! An `An+B` token is a selector inside the argument list of ANY pseudo-class,
//! and which spellings are one is decided by upstream's `REGEX_NTH_OF`
//! (`1-parse/read/style.js`), not by a heuristic. rsvelte entered its `Nth` path
//! only for the four `nth-*` names and then accepted anything that looked
//! numeric, so `:is(2n)` was rejected while `:nth-child(2foo)` compiled.
//!
//! Every expectation below is the official compiler's, taken from a
//! 10 pseudo-class names x 24 argument grid.

use rsvelte_core::{CompileOptions, GenerateMode, compile, compiler::CssMode};

fn compile_css(src: &str) -> Result<String, String> {
    let source =
        format!("<div class=\"x\"><b class=\"a\">y</b></div>\n<style>\n\t{src}\n</style>\n");
    match compile(
        &source,
        CompileOptions {
            filename: Some("Test.svelte".to_string()),
            generate: GenerateMode::Client,
            dev: false,
            css: CssMode::External,
            ..Default::default()
        },
    ) {
        Ok(result) => Ok(result.css.map(|c| c.code).unwrap_or_default()),
        Err(err) => Err(format!("{err:?}")),
    }
}

/// Upstream gates the `Nth` branch on `inside_pseudo_class` alone, so an `An+B`
/// token is legal in the argument list of every pseudo-class.
#[test]
fn an_plus_b_is_accepted_in_any_pseudo_class() {
    for rule in [
        ".x:is(2n) { color: red }",
        ".x:where(5) { color: red }",
        ".x:not(2n) { color: red }",
        ".x:has(2n) { color: red }",
        ".x:global(2n) { color: red }",
        ".x:hover(even) { color: red }",
        ".x:is(-n+3) { color: red }",
        ".x:where(+3) { color: red }",
    ] {
        assert!(
            compile_css(rule).is_ok(),
            "expected `{rule}` to compile, got {:?}",
            compile_css(rule)
        );
    }
}

/// The nine spellings the old heuristic over-accepted. Every one is rejected by
/// the official compiler at the token start.
#[test]
fn near_miss_an_plus_b_spellings_are_rejected() {
    for arg in [
        "-2n-1",
        "-1",
        "2foo",
        "2n /* t */",
        "n+",
        "2n+",
        "2N",
        "2e",
        "3 n",
    ] {
        let rule = format!(".x:nth-child({arg}) {{ color: red }}");
        assert!(
            compile_css(&rule).is_err(),
            "expected `{rule}` to be rejected, got {:?}",
            compile_css(&rule)
        );
    }
}

/// When the regex does not match, upstream falls through to `read_identifier`,
/// so `-n-1` is a type selector and compiles while `-1` does not.
#[test]
fn unmatched_an_plus_b_falls_back_to_an_identifier() {
    assert!(compile_css(".x:nth-child(-n-1) { color: red }").is_ok());
    assert!(compile_css(".x:is(-n-1) { color: red }").is_ok());
    assert!(compile_css(".x:nth-child(-1) { color: red }").is_err());
}

/// `\s+of\s+` is part of the match, so the whitespace around `of` is not a
/// descendant combinator however much of it there is.
#[test]
fn of_clause_is_part_of_the_nth_token() {
    for rule in [
        ".x:nth-child(2n of .a) { color: red }",
        ".x:nth-child(2n  of  .a) { color: red }",
        ".x:nth-child(even of .a) { color: red }",
        ".x:nth-child(n of .a) { color: red }",
    ] {
        let out = compile_css(rule).unwrap_or_else(|e| panic!("`{rule}` failed: {e}"));
        assert!(
            out.contains(" of "),
            "expected the `of` clause to survive in `{rule}`, got:\n{out}"
        );
    }
}

/// A `+` inside `2n + 1` is part of the token, not a sibling combinator.
#[test]
fn signed_an_plus_b_is_not_split_on_the_combinator() {
    let out = compile_css(".x:nth-child(2n + 1) { color: red }").expect("compile");
    assert!(
        out.contains(":nth-child(2n + 1)"),
        "expected `:nth-child(2n + 1)`, got:\n{out}"
    );
}
