//! Issue #3569: a standalone regular attribute expression carries the Phase 2
//! metadata consumed by the client attribute transform.

use rsvelte_core::{
    CompileOptions, ParseOptions,
    ast::{
        arena::SerializeArenaGuard,
        template::{Attribute, AttributeValue, AttributeValuePart, TemplateNode},
    },
    compiler::phases::analyze_component,
    parse,
};

#[derive(Debug, PartialEq, Eq)]
struct Flags {
    has_state: bool,
    has_call: bool,
    has_member_expression: bool,
    quoted: bool,
}

fn attribute_flags(source: &str) -> Flags {
    let mut root = parse(
        source,
        &oxc_allocator::Allocator::default(),
        ParseOptions::default(),
    )
    .expect("parse");
    // SAFETY: `root.arena` outlives the guard and analysis below.
    let _arena_guard = unsafe { SerializeArenaGuard::new(&raw const root.arena) };
    analyze_component(&mut root, source, &CompileOptions::default()).expect("analyze");

    let attributes = match &root.fragment.nodes[0] {
        TemplateNode::RegularElement(element) => &element.attributes,
        TemplateNode::Component(component) => &component.attributes,
        TemplateNode::SvelteComponent(component) => &component.attributes,
        TemplateNode::SvelteElement(element) => &element.attributes,
        TemplateNode::SlotElement(element) => &element.attributes,
        other => panic!("expected an element host, got {other:?}"),
    };
    let Some(Attribute::Attribute(attribute)) = attributes.last() else {
        panic!("expected regular attribute");
    };

    let (metadata, quoted) = match &attribute.value {
        AttributeValue::Expression(tag) => (&tag.metadata.expression, false),
        AttributeValue::Sequence(parts) => {
            let [AttributeValuePart::ExpressionTag(tag)] = parts.as_slice() else {
                panic!("expected one expression chunk");
            };
            (&tag.metadata.expression, true)
        }
        AttributeValue::True(_) => panic!("expected attribute expression"),
    };

    Flags {
        has_state: metadata.has_state(),
        has_call: metadata.has_call(),
        has_member_expression: metadata.has_member_expression(),
        quoted,
    }
}

#[test]
fn every_legal_host_populates_local_call_metadata() {
    for source in [
        "<script>function make() {}</script><div title={make()}></div>",
        "<script>function make() {}</script><Widget prop={make()} />",
        "<script>function make() {}</script><svelte:component this={Widget} prop={make()} />",
        "<script>function make() {}</script><svelte:element this=\"div\" title={make()} />",
        "<script>function make() {}</script><slot title={make()} />",
    ] {
        assert_eq!(
            attribute_flags(source),
            Flags {
                has_state: true,
                has_call: true,
                has_member_expression: false,
                quoted: false,
            },
            "{source}"
        );
    }
}

#[test]
fn one_part_quoted_value_keeps_call_and_member_metadata() {
    let source = "<script>function make() {}</script><div title=\"{make().value}\"></div>";
    assert_eq!(
        attribute_flags(source),
        Flags {
            has_state: true,
            has_call: true,
            has_member_expression: true,
            quoted: true,
        }
    );
}

#[test]
fn global_call_is_not_promoted_to_an_impure_call() {
    assert_eq!(
        attribute_flags("<div title={globalThis.make()}></div>"),
        Flags {
            has_state: false,
            has_call: false,
            has_member_expression: true,
            quoted: false,
        }
    );
}

#[test]
fn state_reference_does_not_invent_a_call() {
    let source = "<script>let title = $state();</script><div title={title}></div>";
    assert_eq!(
        attribute_flags(source),
        Flags {
            has_state: true,
            has_call: false,
            has_member_expression: false,
            quoted: false,
        }
    );
}
