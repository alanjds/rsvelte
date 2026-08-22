//! A name declared INSIDE a `$:` statement belongs to a child scope, so upstream's
//! `scope.references` for the `$:` scope never holds it. rsvelte reached those names
//! through three separate walks that had no notion of a nested scope: the cycle graph
//! (`reactive_declaration_cycle` on a `catch` parameter or a block `let`), the
//! dependency thunk (a `catch` parameter became a tracked dependency), and the server's
//! topological reorder (which resolves read names against the instance scope BY NAME).
//!
//! Every expectation below was read off the official compiler at the pinned version.

use rsvelte_core::{CompileOptions, GenerateMode, compile};

fn compile_to(src: &str, generate: GenerateMode) -> String {
    compile(
        src,
        CompileOptions {
            filename: Some("T.svelte".into()),
            generate,
            dev: false,
            ..Default::default()
        },
    )
    .map(|r| r.js.code)
    .unwrap_or_else(|e| format!("COMPILE_ERROR: {e:?}"))
}

fn client(src: &str) -> String {
    compile_to(src, GenerateMode::Client)
}

fn server(src: &str) -> String {
    compile_to(src, GenerateMode::Server)
}

const CATCH_PARAM: &str = "<script>\n\texport let a = 1;\n\tlet d = 0;\n\tlet e = 0;\n\t$: try { d = a; } catch (e) { d = 0; }\n\t$: e = d + 1;\n</script>\n<b>{d}{e}</b>\n";

const BLOCK_LET: &str = "<script>\n\texport let a = 1;\n\tlet d = 0;\n\tlet e = 0;\n\t$: { let e = a; d = e; }\n\t$: e = d + 1;\n</script>\n<b>{d}{e}</b>\n";

const FOR_OF_LOCAL: &str = "<script>\n\texport let a = 1;\n\tlet d = 0;\n\tlet g = 0;\n\t$: for (const g of [a]) { d = g; }\n\t$: g = d + 1;\n</script>\n<b>{d}{g}</b>\n";

const SWITCH_CASE_LOCAL: &str = "<script>\n\texport let a = 1;\n\tlet d = 0;\n\tlet g = 0;\n\t$: switch (a) { case 1: let g = 2; d = g; }\n\t$: g = d + 1;\n</script>\n<b>{d}{g}</b>\n";

/// Official compiles all four; none of them writes the outer `e` / `g` from the
/// first statement, so there is no `d → e → d` edge.
#[test]
fn a_name_declared_inside_a_reactive_statement_is_not_an_outer_assignment() {
    for src in [CATCH_PARAM, BLOCK_LET, FOR_OF_LOCAL, SWITCH_CASE_LOCAL] {
        for out in [client(src), server(src)] {
            assert!(
                !out.contains("COMPILE_ERROR"),
                "false reactive_declaration_cycle: {out}"
            );
        }
    }
}

/// A genuine cycle must still be reported — the control that shows the fix removed
/// the false edge rather than the check.
#[test]
fn a_real_cycle_is_still_reported() {
    let src = "<script>\n\tlet d = 0;\n\tlet e = 0;\n\t$: d = e;\n\t$: e = d + 1;\n</script>\n<b>{d}{e}</b>\n";
    let out = client(src);
    assert!(out.contains("reactive_declaration_cycle"), "{out}");
    assert!(out.contains("d \u{2192} e \u{2192} d"), "{out}");
}

/// The `catch` parameter is a declaration, so it is not a dependency of the
/// statement — official's thunk reads `a` alone even when the outer `e` is state.
#[test]
fn a_catch_parameter_is_not_a_tracked_dependency() {
    let src = "<script>\n\texport let a = 1;\n\tlet d = 0;\n\tlet e = 0;\n\tfunction bump() { e++; }\n\t$: try { d = a; } catch (e) { d = 0; }\n</script>\n<b>{d}{e}</b><button on:click={bump}>x</button>\n";
    let out = client(src);
    assert!(
        out.contains("$.legacy_pre_effect(() => ($.deep_read_state(a())), () => {"),
        "{out}"
    );
}

/// The server orders the reactive run by dependency. A block-local `let e` read is
/// not a read of the component's `e`, so the statements keep their source order.
#[test]
fn the_server_reorder_does_not_read_a_block_local_as_the_outer_binding() {
    let out = server(BLOCK_LET);
    let block = out.find("$: {").expect("block statement kept");
    let assign = out.find("$: e = d + 1;").expect("assignment kept");
    assert!(block < assign, "statements were reordered: {out}");
}

/// A function parameter was always scoped correctly, which is what made this a
/// scoping bug rather than a missing feature — it must stay that way.
#[test]
fn a_function_parameter_is_still_scoped() {
    let src = "<script>\n\texport let a = 1;\n\tlet d = 0;\n\tlet e = 0;\n\t$: d = ((e) => e)(a);\n\t$: e = d + 1;\n</script>\n<b>{d}{e}</b>\n";
    for out in [client(src), server(src)] {
        assert!(!out.contains("COMPILE_ERROR"), "{out}");
    }
}
