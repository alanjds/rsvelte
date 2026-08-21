//! `typescript_invalid_feature` for a `TSModuleDeclaration`.
//!
//! Upstream's `remove_typescript_nodes` visitor keys on the **body** alone: it
//! visits every entry and errors unless every one of them was erased. rsvelte
//! instead answered from the statement's shape, which made three modifiers
//! decide the verdict — an `export` wrapper and a `declare` modifier both made
//! the namespace vanish before the body was ever looked at (issue #3244) — and
//! two body shapes lie in the other direction (an already-empty statement and a
//! nested `enum`, both of which upstream rejects).

use rsvelte_core::{CompileOptions, GenerateMode, compile, compiler::CssMode};

fn error(body: &str) -> Option<String> {
    let src = format!("<script lang=\"ts\">\n{body}\n</script>\n\n<div>x</div>\n");
    compile(
        &src,
        CompileOptions {
            filename: Some("Test.svelte".to_string()),
            generate: GenerateMode::Client,
            dev: false,
            css: CssMode::External,
            ..Default::default()
        },
    )
    .err()
    .map(|e| format!("{e:?}"))
}

/// `(body, feature, offset of the reported node inside `body`)`. Every offset
/// and message below is the one `svelte.compile` reports for the same source.
const REJECTED: &[(&str, &str, usize)] = &[
    (
        "namespace N { export const a = 1; }",
        "namespaces with non-type nodes",
        0,
    ),
    (
        "export namespace N { export const a = 1; }",
        "namespaces with non-type nodes",
        7,
    ),
    (
        "declare module \"x\" { export const a: number; }",
        "namespaces with non-type nodes",
        0,
    ),
    (
        "declare global { const g: number }",
        "namespaces with non-type nodes",
        0,
    ),
    (
        "module N { export const a = 1; }",
        "namespaces with non-type nodes",
        0,
    ),
    (
        "namespace N { class C {} }",
        "namespaces with non-type nodes",
        0,
    ),
    ("namespace N { ; }", "namespaces with non-type nodes", 0),
    (
        "namespace N { namespace M { export const a = 1; } }",
        "namespaces with non-type nodes",
        14,
    ),
    ("namespace N { enum E { A } }", "enums", 14),
];

/// Every shape upstream compiles. The `export` / `declare` rows are the
/// over-acceptance's own negative control: they must stay legal once the
/// modifier stops deciding the verdict.
const ACCEPTED: &[&str] = &[
    "namespace N { export type A = 1; }",
    "export namespace N { export type A = 1; }",
    "namespace N { type T = 1; }",
    "namespace N { export interface I { a: 1 } }",
    "namespace N { declare const a: number; }",
    "namespace N { namespace M { export type T = 1; } }",
    "declare namespace N { export type T = 1; }",
    "declare global { interface Window { a: 1 } }",
    "declare module \"x\" { export type T = 1; }",
    "declare module \"x\" { export function f(): void; }",
    "declare module \"x\" {}",
    "declare module \"x\";",
    "export declare namespace N { export type T = 1; }",
];

#[test]
fn a_module_declaration_with_non_type_nodes_is_rejected_however_it_is_spelled() {
    // `<script lang="ts">\n` — the body starts one line in.
    let prefix = "<script lang=\"ts\">\n".len();
    for (body, feature, at) in REJECTED {
        let err = error(body).unwrap_or_else(|| panic!("{body:?} must not compile"));
        assert!(
            err.contains("typescript_invalid_feature"),
            "expected typescript_invalid_feature for {body:?}, got: {err}"
        );
        assert!(
            err.contains(feature),
            "expected the {feature:?} message for {body:?}, got: {err}"
        );
        let start = prefix + at;
        assert!(
            err.contains(&format!("span: ({start},")),
            "span must start at {start} for {body:?}, got: {err}"
        );
    }
}

#[test]
fn every_type_only_module_declaration_still_compiles() {
    for body in ACCEPTED {
        assert!(
            error(body).is_none(),
            "must still compile: {body:?} — got {:?}",
            error(body)
        );
    }
}
