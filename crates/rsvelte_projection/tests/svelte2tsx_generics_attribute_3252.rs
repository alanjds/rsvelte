//! #3252 / #3253: `generics="…"` is read by parsing `<{raw}>() => {}`, the way
//! upstream `Generics.ts` does — not by splitting the raw text on commas that
//! only angle brackets protect.

use rsvelte_projection::svelte2tsx::{Svelte2TsxOptions, svelte2tsx};

fn convert(generics: &str) -> String {
    let src = format!(
        "<script lang=\"ts\" generics=\"{generics}\">\n\tlet x: T = null as any; void x;\n</script>\n"
    );
    let opts = Svelte2TsxOptions {
        filename: "Probe.svelte".to_string(),
        is_ts_file: true,
        ..Default::default()
    };
    svelte2tsx(&src, opts).expect("svelte2tsx ok").code
}

/// A top-level comma in any bracket kind other than `<…>` used to split the
/// constraint, so the fragments were emitted as extra type *arguments* —
/// `ReturnType<typeof $$render<T,b:>>`, which no TypeScript parser accepts.
#[test]
fn a_comma_outside_angle_brackets_does_not_split_the_type_parameter() {
    for generics in [
        "T extends { a: string, b: number }",
        "T extends [a, b]",
        "T extends [a: 1, b: 2]",
        "T extends (a: 1, b: 2) => void",
        "T = { a: 1, b: 2 }",
        "T extends 'a,b'",
    ] {
        let code = convert(generics);
        assert!(
            code.contains(&format!("class __sveltets_Render<{generics}> {{")),
            "definition must round-trip verbatim for {generics:?}:\n{code}"
        );
        assert!(
            code.contains("$$render<T>()"),
            "the only type argument is `T` for {generics:?}:\n{code}"
        );
        assert!(
            !code.contains("$$render<T,"),
            "no fragment of the constraint may become a type argument for {generics:?}:\n{code}"
        );
    }
}

/// Several type parameters are still separated, and the definition list is
/// joined the way upstream joins `param.getText()`.
#[test]
fn several_type_parameters_are_still_separated() {
    let code = convert("A, B extends keyof A, C extends boolean");
    assert!(
        code.contains("class __sveltets_Render<A,B extends keyof A,C extends boolean> {"),
        "{code}"
    );
    assert!(code.contains("$$render<A,B,C>()"), "{code}");
}

/// #3253: `createRenderFunction` splices the RAW attribute onto `$$render`,
/// while `addComponentExport` keys on the type parameters the parse recognised
/// — so an attribute that is not a type parameter list yields the raw generics
/// on `$$render` *and* the non-generic component export.
#[test]
fn an_unparseable_attribute_yields_the_non_generic_component_export() {
    let code = convert("T extends string ? 1 : 2");
    assert!(
        code.contains(";function $$render<T extends string ? 1 : 2>() {"),
        "the raw attribute still reaches $$render:\n{code}"
    );
    assert!(
        code.contains(
            "const Probe__SvelteComponent_ = __sveltets_2_isomorphic_component(__sveltets_2_with_any_event($$render()));"
        ),
        "the component export must be the non-generic form:\n{code}"
    );
    assert!(
        !code.contains("__sveltets_Render"),
        "no generic component class may be emitted:\n{code}"
    );
    assert!(
        !code.contains("<extends"),
        "no type parameter name may be invented from the leading token:\n{code}"
    );
}

/// The same predicate, one token further out: rsvelte used to invent a type
/// parameter name out of the attribute's leading token, so `in T` became `<in>`
/// and `extends string` became `<extends>`.
#[test]
fn a_leading_keyword_is_not_a_type_parameter_name() {
    for generics in ["in T", "extends string", "1", ",T"] {
        let code = convert(generics);
        assert!(
            !code.contains("__sveltets_Render"),
            "{generics:?} declares no type parameter:\n{code}"
        );
    }
    // `out T` is the control: an arrow function's type parameter list DOES
    // accept the `out` variance modifier, so it stays generic on both sides.
    let code = convert("out T");
    assert!(code.contains("class __sveltets_Render<out T> {"), "{code}");
}

/// An empty attribute is no attribute at all (upstream's `if (generics)` guard).
#[test]
fn an_empty_attribute_emits_no_type_parameter_list() {
    let src = "<script lang=\"ts\" generics=\"\">\n\tlet x = 1; void x;\n</script>\n";
    let opts = Svelte2TsxOptions {
        filename: "Probe.svelte".to_string(),
        is_ts_file: true,
        ..Default::default()
    };
    let code = svelte2tsx(src, opts).expect("svelte2tsx ok").code;
    assert!(
        code.contains(";function $$render() {"),
        "no `<>` may be emitted:\n{code}"
    );
}
