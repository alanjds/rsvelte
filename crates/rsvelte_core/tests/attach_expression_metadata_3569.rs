//! Issue #3569: `AttachTag` owns expression metadata populated by Phase 2, so
//! Phase 3 must not maintain a second implementation of the same decisions.

use rsvelte_core::{
    CompileOptions, ParseOptions,
    ast::{
        arena::SerializeArenaGuard,
        template::{Attribute, TemplateNode},
    },
    compiler::phases::analyze_component,
    parse,
};

fn attach_flags(source: &str) -> (bool, bool) {
    let mut root = parse(
        source,
        &oxc_allocator::Allocator::default(),
        ParseOptions::default(),
    )
    .expect("parse");
    // SAFETY: `root.arena` outlives the guard and analysis below.
    let _arena_guard = unsafe { SerializeArenaGuard::new(&raw const root.arena) };
    analyze_component(&mut root, source, &CompileOptions::default()).expect("analyze");

    let TemplateNode::RegularElement(element) = &root.fragment.nodes[0] else {
        panic!("expected regular element");
    };
    let Attribute::AttachTag(attach) = &element.attributes[0] else {
        panic!("expected attach tag");
    };

    (
        attach.metadata.expression.has_state(),
        attach.metadata.expression.has_call(),
    )
}

#[test]
fn local_call_is_stateful_and_impure() {
    let source = "<script>function make() {}</script><div {@attach make()}></div>";
    assert_eq!(attach_flags(source), (true, true));
}

#[test]
fn global_call_is_not_promoted_to_an_impure_call() {
    let source = "<div {@attach globalThis.make()}></div>";
    assert_eq!(attach_flags(source), (false, false));
}

#[test]
fn state_reference_does_not_invent_a_call() {
    let source = "<script>let attachment = $state();</script><div {@attach attachment}></div>";
    assert_eq!(attach_flags(source), (true, false));
}
