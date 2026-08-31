//! An own-line comment inside a template-expression function body reached the
//! synthetic buffer through two channels — copied verbatim as its own chunk by
//! `push_own_line_comment_raws`, and again through the enclosing
//! `JsSourceAnchor` — so it was emitted twice. Official emits it once.
//!
//! Expectations are read off the pinned official compiler
//! (`submodules/svelte/.../src/compiler/index.js`), which is the entry point the
//! corpus gates use. They pin *multiplicity and attachment* rather than the whole
//! program, so unrelated codegen movement does not re-pin this class silently.

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use rsvelte_core::{CompileOptions, GenerateMode, compile};

const HEAD: &str = "<script>\n\tlet v = 1;\n\tfunction z(){}\n\tfunction act(){}\n</script>\n";

fn client(source: &str) -> String {
    let output = compile(
        source,
        CompileOptions {
            generate: GenerateMode::Client,
            filename: Some("C.svelte".into()),
            ..Default::default()
        },
    )
    .unwrap_or_else(|error| panic!("compile failed: {error:?}"))
    .js
    .code;

    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &output, SourceType::mjs()).parse();
    assert!(
        !parsed.panicked && parsed.diagnostics.is_empty(),
        "output must parse:\n{output}"
    );
    output
}

fn assert_attached_once(output: &str, comment: &str, indent: &str) {
    let count = output.matches(comment).count();
    assert_eq!(
        count, 1,
        "official emits `{comment}` once, got {count}:\n{output}"
    );
    let adjacent = format!("{indent}{comment}\n{indent}z();");
    assert!(
        output.contains(&adjacent),
        "`{comment}` must stay attached to the statement it precedes:\n{output}"
    );
}

#[test]
fn block_comment_leading_a_statement_in_a_delegated_handler_is_emitted_once() {
    let output = client(&format!(
        "{HEAD}<button onclick={{(e) => {{\n\t/* C */\n\tz();\n}}}}>x</button>\n"
    ));
    assert_attached_once(&output, "/* C */", "\t\t");
}

#[test]
fn line_comment_leading_a_statement_in_a_delegated_handler_is_emitted_once() {
    let output = client(&format!(
        "{HEAD}<button onclick={{(e) => {{\n\t// C\n\tz();\n}}}}>x</button>\n"
    ));
    assert_attached_once(&output, "// C", "\t\t");
}

#[test]
fn block_comment_in_a_handler_inside_an_each_block_is_emitted_once() {
    let output = client(&format!(
        "{HEAD}{{#each [1] as n}}<button onclick={{() => {{\n\t/* C */\n\tz();\n}}}}>x</button>{{/each}}\n"
    ));
    assert_attached_once(&output, "/* C */", "\t\t\t");
}

/// Control: a `use:` body is already byte-identical to official on this shape,
/// so it must not move.
#[test]
fn control_block_comment_in_a_use_directive_body_is_unchanged() {
    let output = client(&format!(
        "{HEAD}<div use:act={{() => {{\n\t/* C */\n\tz();\n}}}}></div>\n"
    ));
    assert_attached_once(&output, "/* C */", "\t\t");
}

/// Control: the same body written in `<script>` never reaches the template
/// channel at all, so it is the isomorphic detector for a regression in the
/// shared lowering.
#[test]
fn control_block_comment_in_a_script_function_body_is_unchanged() {
    let output = client(
        "<script>\n\tlet v = 1;\n\tfunction z(){}\n\tfunction h(){\n\t/* C */\n\tz();\n}\n</script>\n<button onclick={h}>x</button>\n",
    );
    assert_attached_once(&output, "/* C */", "\t\t");
}

/// A same-line comment in a `use:` body is a *separate*, still-open divergence:
/// official emits it twice (once in the action wrapper, once in the body) and
/// rsvelte drops it. This control only pins that the dedupe above does not push
/// it the other way by introducing a copy.
#[test]
fn control_same_line_comment_in_a_use_directive_body_is_not_duplicated() {
    let output = client(&format!(
        "{HEAD}<div use:act={{() => {{ /* C */ v; }}}}></div>\n"
    ));
    let count = output.matches("/* C */").count();
    assert!(
        count <= 1,
        "dedupe must not add a copy, got {count}:\n{output}"
    );
}
