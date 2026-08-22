//! JavaScript early errors that acorn raises and OXC does not (issue #3243).
//!
//! None of these is decidable from the token stream alone — each needs the
//! class, function or label context around the construct — so OXC leaves them
//! to `SemanticBuilder`, which this pipeline never runs. Every one was accepted
//! and copied verbatim into output no JS parser will read.
//!
//! Each expectation below was measured against `svelte.compile` /
//! `svelte.compileModule`, message and byte offset included; a `@` in a source
//! marks the offset and is removed before compiling. The `legal_*` tests are
//! the other half of the same check: adding a rejection always risks an
//! over-rejection, and every shape there is one acorn accepts.

use rsvelte_core::{
    CompileOptions, GenerateMode, ModuleCompileOptions, compile, compile_module, compiler::CssMode,
};

/// `(code, message, start offset)` for a rejected input, or `None`.
fn component_error(src: &str) -> Option<(String, String, u32)> {
    diagnostic(
        compile(
            src,
            CompileOptions {
                filename: Some("Test.svelte".to_string()),
                generate: GenerateMode::Client,
                css: CssMode::External,
                ..Default::default()
            },
        )
        .err(),
    )
}

fn module_error(src: &str) -> Option<(String, String, u32)> {
    diagnostic(
        compile_module(
            src,
            ModuleCompileOptions {
                filename: Some("m.svelte.js".to_string()),
                generate: GenerateMode::Client,
                ..Default::default()
            },
        )
        .err(),
    )
}

fn diagnostic(err: Option<rsvelte_core::CompileError>) -> Option<(String, String, u32)> {
    err.map(|err| {
        let d = err.diagnostic();
        (
            d.code.unwrap_or_default(),
            d.message.lines().next().unwrap_or_default().to_string(),
            d.span.map(|(start, _)| start).unwrap_or(u32::MAX),
        )
    })
}

fn assert_rejected(
    src_with_marker: &str,
    message: &str,
    compiler: fn(&str) -> Option<(String, String, u32)>,
) {
    let at = src_with_marker
        .find('@')
        .unwrap_or_else(|| panic!("no `@` offset marker in {src_with_marker}"));
    let src = src_with_marker.replacen('@', "", 1);
    let (code, actual, start) =
        compiler(&src).unwrap_or_else(|| panic!("must not compile: {src_with_marker}"));
    assert_eq!(code, "js_parse_error", "{src_with_marker}");
    assert_eq!(actual, message, "{src_with_marker}");
    assert_eq!(start as usize, at, "offset for {src_with_marker}");
}

/// The same statement in a `.svelte.js` module and in a component's instance
/// script — the two entry points that reach `parse_program`.
fn assert_rejected_everywhere(statement_with_marker: &str, message: &str) {
    assert_rejected(
        &format!("export {statement_with_marker}\n"),
        message,
        module_error,
    );
    assert_rejected(
        &format!("<script>{statement_with_marker}</script>\n"),
        message,
        component_error,
    );
}

#[test]
fn duplicate_constructor() {
    assert_rejected_everywhere(
        "class K { constructor() {} @constructor() {} }",
        "Duplicate constructor in the same class",
    );
    assert_rejected_everywhere(
        "class K { \"constructor\"() {} @constructor() {} }",
        "Duplicate constructor in the same class",
    );
    assert_rejected_everywhere(
        "class K { m() { return class { constructor() {} @constructor() {} }; } }",
        "Duplicate constructor in the same class",
    );
}

#[test]
fn super_outside_a_method() {
    assert_rejected_everywhere(
        "function f() { @super(); }",
        "'super' keyword outside a method",
    );
    assert_rejected_everywhere(
        "class K extends Object { m() { return function () { return @super.toString(); }; } }",
        "'super' keyword outside a method",
    );
}

#[test]
fn super_call_outside_a_derived_constructor() {
    assert_rejected_everywhere(
        "class K { constructor() { @super(); } }",
        "super() call outside constructor of a subclass",
    );
    assert_rejected_everywhere(
        "class K extends Object { m() { @super(); } }",
        "super() call outside constructor of a subclass",
    );
}

#[test]
fn unsyntactic_break() {
    assert_rejected_everywhere("function f() { @break nope; }", "Unsyntactic break");
    // A function boundary is not crossed, by either spelling.
    assert_rejected_everywhere(
        "function f() { for (;;) { (function () { @break; })(); } }",
        "Unsyntactic break",
    );
    assert_rejected_everywhere(
        "function f() { for (;;) { (() => { @break; })(); } }",
        "Unsyntactic break",
    );
    // A class static block is a function body too.
    assert_rejected_everywhere(
        "function f() { a: for (;;) { class L { static { @break a; } } } }",
        "Unsyntactic break",
    );
}

#[test]
fn unsyntactic_continue() {
    assert_rejected_everywhere("function f() { @continue; }", "Unsyntactic continue");
    // A `switch` is a destination for `break` and not for `continue`.
    assert_rejected_everywhere(
        "function f(x) { switch (x) { case 1: @continue; } }",
        "Unsyntactic continue",
    );
    // So is a labelled block.
    assert_rejected_everywhere(
        "function f() { a: { @continue a; } }",
        "Unsyntactic continue",
    );
    assert_rejected_everywhere(
        "function f() { a: b: { @continue a; } }",
        "Unsyntactic continue",
    );
}

#[test]
fn duplicate_label() {
    assert_rejected_everywhere(
        "function f() { a: @a: for (;;) break a; }",
        "Label 'a' is already declared",
    );
    assert_rejected_everywhere(
        "function f() { a: for (;;) { @a: for (;;) break a; } }",
        "Label 'a' is already declared",
    );
}

#[test]
fn undeclared_private_name() {
    assert_rejected_everywhere(
        "class K { m() { return this.@#nope; } }",
        "Private field '#nope' must be declared in an enclosing class",
    );
    assert_rejected_everywhere(
        "class K { m() { return class { n(o) { return o.@#nope; } }; } }",
        "Private field '#nope' must be declared in an enclosing class",
    );
    assert_rejected_everywhere(
        "class K { static has(o) { return @#b in o; } }",
        "Private field '#b' must be declared in an enclosing class",
    );
}

#[test]
fn duplicate_private_name() {
    assert_rejected_everywhere(
        "class K { #a = 1; @#a = 2; }",
        "Identifier '#a' has already been declared",
    );
    assert_rejected_everywhere(
        "class K { #a() {} @#a = 1; }",
        "Identifier '#a' has already been declared",
    );
    // A getter/setter pair is the one legal repetition — but only when the two
    // halves agree on `static`.
    assert_rejected_everywhere(
        "class K { get #a() { return 1; } static set @#a(v) {} }",
        "Identifier '#a' has already been declared",
    );
}

#[test]
fn private_fields_can_not_be_deleted() {
    assert_rejected_everywhere(
        "class K { #a = 1; m() { @delete this.#a; } }",
        "Private fields can not be deleted",
    );
}

#[test]
fn import_and_export_only_at_the_top_level() {
    assert_rejected_everywhere(
        "function f() { @import \"./x.js\"; }",
        "'import' and 'export' may only appear at the top level",
    );
    assert_rejected_everywhere(
        "function f() { @export const a = 1; }",
        "'import' and 'export' may only appear at the top level",
    );
    assert_rejected_everywhere(
        "function f() { @export * from \"./x.js\"; }",
        "'import' and 'export' may only appear at the top level",
    );
    assert_rejected("{ @import \"./x.js\"; }\n", MODULE_LEVEL, module_error);
    assert_rejected("if (1) @export const a = 1;\n", MODULE_LEVEL, module_error);
}

const MODULE_LEVEL: &str = "'import' and 'export' may only appear at the top level";

#[test]
fn use_strict_with_a_non_simple_parameter_list() {
    for statement in [
        "@function f(a = 1) { 'use strict'; return a; }",
        "@function f(...a) { 'use strict'; return a; }",
        "@function f({ a }) { 'use strict'; return a; }",
        "@function f([a]) { 'use strict'; return a; }",
        "@function* g(a = 1) { 'use strict'; return a; }",
        "@async function f(a = 1) { 'use strict'; return a; }",
        // A later directive in the same prologue still counts.
        "@function f(a = 1) { 'other'; 'use strict'; return a; }",
    ] {
        assert_rejected_everywhere(statement, USE_STRICT);
    }
    assert_rejected(
        "export const f = @(a = 1) => { 'use strict'; return a; };\n",
        USE_STRICT,
        module_error,
    );
    assert_rejected(
        "export class K { m@(a = 1) { 'use strict'; return a; } }\n",
        USE_STRICT,
        module_error,
    );
}

const USE_STRICT: &str =
    "Illegal 'use strict' directive in function with non-simple parameter list";

/// Every shape below is accepted by `svelte.compile`, and each sits one step
/// away from a rejection above.
#[test]
fn legal_shapes_still_compile() {
    for statement in [
        "class K extends Object { constructor() { super(); } }",
        "class K extends Object { m() { return super.toString(); } }",
        "class K extends Object { a = super.toString(); }",
        "class K extends Object { a = () => super.toString(); }",
        "class K extends Object { static { super.toString(); } }",
        "class K extends Object { m() { return () => super.toString(); } }",
        "class K extends Object { m() { return { n() { return super.toString(); } }; } }",
        "const o = { __proto__: {}, m() { return super.toString(); } };",
        "class K { static constructor() {} constructor() {} }",
        "class K { [\"constructor\"]() {} constructor() {} }",
        "class K { #a = 1; m() { return this.#a; } }",
        "class K { #a = 1; static has(o) { return #a in o; } }",
        "class K { get #a() { return 1; } set #a(v) {} }",
        "class K { #a = 1; m() { class L { #a = 2; n() { return this.#a; } } return L; } }",
        "class K { #a = 1; m() { return class { n(o) { return o.#a; } }; } }",
        "function f() { a: for (;;) break a; }",
        "function f() { a: for (;;) break a; a: for (;;) break a; }",
        "function f() { a: b: for (;;) { continue a; } }",
        "function f() { a: { break a; } }",
        "function f(x) { switch (x) { case 1: break; } }",
        "function f(x) { a: switch (x) { case 1: break a; } }",
        "function f() { a: for (;;) { (function () { a: for (;;) break a; })(); break a; } }",
        "function f() { do { continue; } while (0); }",
        "function f() { return import(\"./x.js\"); }",
        "function f() { return import.meta.url; }",
        "class K { static { a: for (;;) break a; } }",
        "function f(a, b) { 'use strict'; return a + b; }",
        "function f() { 'use strict'; return 1; }",
        "const g = (a = 1) => a;",
        "function f(a = 1) { const b = 1; 'use strict'; return a + b; }",
        "const o = { get a() { 'use strict'; return 1; } };",
    ] {
        assert_eq!(
            module_error(&format!("export {statement}\n")),
            None,
            "module must accept: {statement}"
        );
        assert_eq!(
            component_error(&format!("<script>{statement}</script>\n")),
            None,
            "instance script must accept: {statement}"
        );
    }
}

/// A TypeScript overload signature repeats the member's name without defining
/// it, so neither the constructor count nor the private-name map may see it.
#[test]
fn legal_typescript_overloads() {
    for statement in [
        "class K { constructor(a: string); constructor(a: number); constructor(a: any) {} }",
        "class K { #m(a: string): void; #m(a: any) {} }",
        "class K { declare a: number; }",
        "abstract class K { abstract m(): void; }",
        // An optional parameter is still a plain identifier binding.
        "function f(a?: number) { 'use strict'; return a; }",
        "function f(a: number) { 'use strict'; return a; }",
    ] {
        assert_eq!(
            component_error(&format!("<script lang=\"ts\">{statement}</script>\n")),
            None,
            "lang=\"ts\" instance script must accept: {statement}"
        );
    }
}

/// The same rejections apply to `lang="ts"`, which upstream parses with
/// acorn-typescript — the base parser's early errors are still in force.
#[test]
fn typescript_scripts_are_rejected_too() {
    for (statement, message) in [
        (
            "class K { constructor() {} constructor() {} }",
            "Duplicate constructor in the same class",
        ),
        (
            "function f() { super(); }",
            "'super' keyword outside a method",
        ),
        ("function f() { break nope; }", "Unsyntactic break"),
        (
            "function f() { a: a: for (;;) break a; }",
            "Label 'a' is already declared",
        ),
        (
            "class K { #a = 1; #a = 2; }",
            "Identifier '#a' has already been declared",
        ),
        (
            "class K { m() { return this.#nope; } }",
            "Private field '#nope' must be declared in an enclosing class",
        ),
        (
            "function f() { import \"./x.js\"; }",
            "'import' and 'export' may only appear at the top level",
        ),
    ] {
        let src = format!("<script lang=\"ts\">{statement}</script>\n");
        let (code, actual, _) =
            component_error(&src).unwrap_or_else(|| panic!("lang=\"ts\" must reject: {statement}"));
        assert_eq!(code, "js_parse_error", "lang=\"ts\": {statement}");
        assert_eq!(actual, message, "lang=\"ts\": {statement}");
    }
}
