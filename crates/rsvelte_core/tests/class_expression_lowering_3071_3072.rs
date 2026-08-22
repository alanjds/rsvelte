//! Issues #3071 / #3072: a class expression reached through a rune argument or
//! through an `extends` heritage clause is a class of its own, so upstream's
//! ordinary walk lowers its rune fields too
//! (`3-transform/client/visitors/ClassBody.js`).

use rsvelte_core::{CompileOptions, GenerateMode, ModuleCompileOptions, compile, compile_module};

fn module(src: &str) -> String {
    compile_module(
        src,
        ModuleCompileOptions {
            filename: Some("T.svelte.js".into()),
            generate: GenerateMode::Client,
            dev: false,
            ..Default::default()
        },
    )
    .map(|r| r.js.code)
    .unwrap_or_else(|e| format!("COMPILE_ERROR: {e:?}"))
}

/// Everything after the generated-by banner and the runtime import, so a Svelte
/// version bump does not rewrite the expected text.
fn body(out: &str) -> String {
    match out.split_once("import * as $ from 'svelte/internal/client';\n") {
        Some((_, rest)) => rest.trim_start_matches('\n').to_string(),
        None => out.to_string(),
    }
}

fn component(src: &str) -> String {
    compile(
        src,
        CompileOptions {
            filename: Some("T.svelte".into()),
            generate: GenerateMode::Client,
            dev: false,
            ..Default::default()
        },
    )
    .map(|r| r.js.code)
    .unwrap_or_else(|e| format!("COMPILE_ERROR: {e:?}"))
}

/// #3071: `$state(class { … })` — the nested body is lowered, and the class
/// expression is proxied (`should_proxy` has no `ClassExpression` arm, so it
/// returns true).
#[test]
fn a_class_expression_in_a_rune_argument_is_lowered_and_proxied() {
    let out =
        module("export class A {\n\theld = $state(class {\n\t\tdeep = $state(1);\n\t});\n}\n");
    assert_eq!(
        body(&out),
        "export class A {\n\
         \t#held = $.state($.proxy(class {\n\
         \t\t#deep = $.state(1);\n\
         \n\
         \t\tget deep() {\n\
         \t\t\treturn $.get(this.#deep);\n\
         \t\t}\n\
         \n\
         \t\tset deep(value) {\n\
         \t\t\t$.set(this.#deep, value, true);\n\
         \t\t}\n\
         \t}));\n\
         \n\
         \tget held() {\n\
         \t\treturn $.get(this.#held);\n\
         \t}\n\
         \n\
         \tset held(value) {\n\
         \t\t$.set(this.#held, value, true);\n\
         \t}\n\
         }"
    );
}

/// The same shape inside a component's instance script.
#[test]
fn a_class_expression_in_a_rune_argument_is_lowered_in_an_instance_script() {
    let out = component(
        "<script>\nclass A {\n\theld = $state(class {\n\t\tdeep = $state(1);\n\t});\n}\nnew A();\n</script>\n",
    );
    assert!(out.contains("$.state($.proxy(class {"), "{out}");
    assert!(out.contains("#deep = $.state(1);"), "{out}");
    assert!(!out.contains("deep = $state(1)"), "{out}");
}

/// `new (class { … })()` in a rune argument reaches the same walk.
#[test]
fn a_new_class_expression_in_a_rune_argument_is_lowered() {
    let out = module(
        "export class A {\n\theld = $state(new (class {\n\t\tdeep = $state(1);\n\t})());\n}\n",
    );
    assert!(out.contains("$.state($.proxy(new (class {"), "{out}");
    assert!(out.contains("#deep = $.state(1);"), "{out}");
}

/// #3072: the inline heritage body is not the subclass's body, so the
/// subclass's own rune fields are lowered — and esrap adds no parentheses
/// around a heritage expression.
#[test]
fn an_inline_heritage_class_keeps_its_own_body_and_the_subclass_is_lowered() {
    let out = module(
        "export class Sub extends class {\n\tinline = $state('i');\n} {\n\town = $derived(this.inline + '!');\n}\n",
    );
    assert_eq!(
        body(&out),
        "export class Sub extends class {\n\
         \t#inline = $.state('i');\n\
         \n\
         \tget inline() {\n\
         \t\treturn $.get(this.#inline);\n\
         \t}\n\
         \n\
         \tset inline(value) {\n\
         \t\t$.set(this.#inline, value, true);\n\
         \t}\n\
         } {\n\
         \t#own = $.derived(() => this.inline + '!');\n\
         \n\
         \tget own() {\n\
         \t\treturn $.get(this.#own);\n\
         \t}\n\
         \n\
         \tset own(value) {\n\
         \t\t$.set(this.#own, value);\n\
         \t}\n\
         }"
    );
}

/// A heritage class whose subclass declares no runes still gets lowered.
#[test]
fn an_inline_heritage_class_is_lowered_without_subclass_runes() {
    let out =
        module("export class Sub extends class {\n\tinline = $state('i');\n} {\n\tplain = 1;\n}\n");
    assert!(out.contains("extends class {"), "{out}");
    assert!(!out.contains("extends (class"), "{out}");
    assert!(out.contains("#inline = $.state('i');"), "{out}");
}

/// The same shape inside a component's instance script.
#[test]
fn an_inline_heritage_class_is_lowered_in_an_instance_script() {
    let out = component(
        "<script>\nclass Sub extends class {\n\tinline = $state('i');\n} {\n\town = $derived(this.inline + '!');\n}\nnew Sub();\n</script>\n",
    );
    assert!(out.contains("class Sub extends class {"), "{out}");
    assert!(
        out.contains("#own = $.derived(() => this.inline + '!');"),
        "{out}"
    );
}
