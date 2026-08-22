//! #3254: the three `$$Generic` errors upstream `Generics.ts` raises, and the
//! `export type T = $$Generic` form its matcher reaches because `export` is a
//! modifier on the alias rather than a wrapper node.

use rsvelte_projection::svelte2tsx::{Svelte2TsxError, Svelte2TsxOptions, svelte2tsx};

fn convert(src: &str) -> Result<String, Svelte2TsxError> {
    let opts = Svelte2TsxOptions {
        filename: "Probe.svelte".to_string(),
        is_ts_file: true,
        ..Default::default()
    };
    svelte2tsx(src, opts).map(|result| result.code)
}

fn error_message(src: &str) -> String {
    match convert(src) {
        Ok(code) => panic!("expected a rejection, got:\n{code}"),
        Err(Svelte2TsxError::Script(message)) => message,
        Err(other) => panic!("expected a script error, got {other:?}"),
    }
}

#[test]
fn a_module_script_declaration_is_rejected() {
    for src in [
        "<script module lang=\"ts\">\n\ttype T = $$Generic;\n</script>\n",
        "<script module lang=\"ts\">\n\ttype T = $$Generic<string>;\n</script>\n",
        "<script module lang=\"ts\">\n\texport type T = $$Generic;\n</script>\n",
        // Upstream matches the type reference by NAME and never resolves it, so
        // a shadowing alias in the same script does not disarm the check.
        "<script module lang=\"ts\">\n\ttype $$Generic = 1;\n\ttype T = $$Generic;\n</script>\n",
    ] {
        assert_eq!(
            error_message(src),
            "$$Generic declarations are only allowed in the instance script"
        );
    }
}

#[test]
fn a_declaration_next_to_the_generics_attribute_is_rejected() {
    for src in [
        "<script lang=\"ts\" generics=\"U\">\n\ttype T = $$Generic;\n</script>\n",
        "<script lang=\"ts\" generics=\"U\">\n\texport type T = $$Generic;\n</script>\n",
    ] {
        assert_eq!(
            error_message(src),
            "Invalid $$Generic declaration: $$Generic definitions are not allowed when the generics attribute is present on the script tag"
        );
    }
}

#[test]
fn more_than_one_type_argument_is_rejected() {
    for src in [
        "<script lang=\"ts\">\n\ttype T = $$Generic<string, number>;\n</script>\n",
        "<script lang=\"ts\">\n\texport type T = $$Generic<string, number>;\n</script>\n",
    ] {
        assert_eq!(
            error_message(src),
            "Invalid $$Generic declaration: Only one type argument allowed"
        );
    }
}

/// The `export` modifier used to hide the alias from the matcher, so the
/// declaration survived into the `$$render()` body and the component never
/// became generic.
#[test]
fn an_exported_alias_becomes_a_type_parameter() {
    let code = convert(
        "<script lang=\"ts\">\n\texport type T = $$Generic;\n\tlet x: T = null as any; void x;\n</script>\n",
    )
    .expect("svelte2tsx ok");
    assert!(
        code.contains(
            "function $$render/*\u{03A9}ignore_start\u{03A9}*/<T>/*\u{03A9}ignore_end\u{03A9}*/()"
        ),
        "the alias must become a type parameter on $$render:\n{code}"
    );
    assert!(
        !code.contains("$$Generic"),
        "the alias must be removed from the render body:\n{code}"
    );
}

#[test]
fn an_exported_alias_keeps_its_constraint() {
    let code = convert(
        "<script lang=\"ts\">\n\texport type T = $$Generic<string>;\n\tlet x: T = null as any; void x;\n</script>\n",
    )
    .expect("svelte2tsx ok");
    assert!(
        code.contains("<T extends string>"),
        "the single type argument becomes the constraint:\n{code}"
    );
}

/// A qualified `ns.$$Generic` is a different type reference — upstream's
/// `is$$GenericType` requires a bare identifier.
#[test]
fn a_qualified_reference_is_not_a_dollar_generic() {
    let code = convert(
        "<script lang=\"ts\">\n\ttype T = ns.$$Generic;\n\tlet x: T = null as any; void x;\n</script>\n",
    )
    .expect("svelte2tsx ok");
    assert!(code.contains("ns.$$Generic"), "{code}");
}
