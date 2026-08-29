//! A class body is split into member blocks line by line, and a `/* … */`
//! spanning several lines was one member per line. Its opening `/**` then
//! joined the block above, so the AST assignment pass could not parse that
//! block and every rewrite it owns — here the `??=` lowering of a private
//! `$state.raw` field — was silently skipped, emitting text no JS parser
//! accepts. Reproduced from sveltekit's `remote-functions/query/instance`.

use rsvelte_core::{ModuleCompileOptions, compile_module};

fn module(src: &str) -> String {
    compile_module(
        src,
        ModuleCompileOptions {
            filename: Some("T.svelte.js".into()),
            ..Default::default()
        },
    )
    .map(|r| r.js.code)
    .unwrap_or_else(|e| format!("COMPILE_ERROR: {e:?}"))
}

const BODY: &str = "\t#promise = $state.raw(null);\n\n\t#run() {\n\t\treturn Promise.resolve();\n\t}\n\n\t#get_promise() {\n\t\tvoid (this.#promise ??= this.#run());\n\t\treturn this.#promise;\n\t}\n";

#[test]
fn a_multiline_block_comment_does_not_split_the_class_member_before_it() {
    // The comment is placed *after* the `??=` method: only then does its
    // opening line land on the block that carries the assignment.
    let out = module(&format!(
        "export class Q {{\n{BODY}\n\t/**\n\t * text\n\t */\n\tzzz() {{}}\n}}\n"
    ));
    assert!(!out.contains("COMPILE_ERROR"), "{out}");
    assert!(
        out.contains("$.get(this.#promise) ?? $.set(this.#promise, this.#run())"),
        "{out}"
    );
    assert!(!out.contains("??="), "{out}");
}

/// The control: with the comment *before* the method the split was harmless,
/// so this shape has always been lowered and must stay lowered.
#[test]
fn a_block_comment_before_the_member_keeps_lowering_the_assignment() {
    let out = module(&format!(
        "export class Q {{\n\t/**\n\t * text\n\t */\n{BODY}}}\n"
    ));
    assert!(!out.contains("COMPILE_ERROR"), "{out}");
    assert!(
        out.contains("$.get(this.#promise) ?? $.set(this.#promise, this.#run())"),
        "{out}"
    );
}
