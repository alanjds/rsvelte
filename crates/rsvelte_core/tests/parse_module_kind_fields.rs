//! `importKind` / `exportKind` and `attributes` are decided by the PARSER.
//!
//! acorn-typescript stamps a kind on every import and export and emits no
//! import attributes; acorn does the exact opposite. rsvelte emitted the kind
//! only for a `type` form and `attributes` unconditionally, so a `lang="ts"`
//! script disagreed with official's `parse()` on both fields at once.

use rsvelte_core::Allocator;
use rsvelte_core::ast::arena::with_serialize_arena;
use rsvelte_core::compiler::phases::phase1_parse::{ParseOptions, parse};
use serde_json::Value;

fn ast(src: &str) -> Value {
    let allocator = Allocator::default();
    let parsed = parse(src, &allocator, ParseOptions::public_api()).expect("parses");
    with_serialize_arena(&parsed.arena, || {
        serde_json::to_value(&parsed).expect("serializes")
    })
}

/// Every node of `ty`, in document order.
fn nodes_of<'a>(value: &'a Value, ty: &str, out: &mut Vec<&'a Value>) {
    match value {
        Value::Object(map) => {
            if map.get("type").and_then(Value::as_str) == Some(ty) {
                out.push(value);
            }
            for v in map.values() {
                nodes_of(v, ty, out);
            }
        }
        Value::Array(items) => {
            for v in items {
                nodes_of(v, ty, out);
            }
        }
        _ => {}
    }
}

fn kinds(src: &str, ty: &str, field: &str) -> Vec<Option<String>> {
    let tree = ast(src);
    let mut found = Vec::new();
    nodes_of(&tree, ty, &mut found);
    assert!(!found.is_empty(), "no {ty} in the tree");
    found
        .iter()
        .map(|n| {
            n.get(field)
                .and_then(Value::as_str)
                .map(std::string::ToString::to_string)
        })
        .collect()
}

fn has_attributes(src: &str, ty: &str) -> Vec<bool> {
    let tree = ast(src);
    let mut found = Vec::new();
    nodes_of(&tree, ty, &mut found);
    assert!(!found.is_empty(), "no {ty} in the tree");
    found
        .iter()
        .map(|n| n.get("attributes").is_some())
        .collect()
}

const TS: &str = "<script lang=\"ts\">\n\timport { A } from './a';\n\timport type { B } from './b';\n\timport { type C, D } from './c';\n\texport const x = 1;\n\texport { x as y };\n\texport type { A };\n</script>\n";
const JS: &str = "<script>\n\timport { A } from './a';\n\texport const x = 1;\n\texport { x as y };\n</script>\n";
/// A TypeScript script that DOES write import attributes. The first version of
/// this fix suppressed `attributes` on every TS import and regressed here.
const TS_ATTRS: &str = "<script lang=\"ts\">\n\timport data from './d.json' assert { type: 'json' };\n\tconst held: unknown = data;\n</script>\n";

/// The anchors come from the official compiler on these exact sources.
#[test]
fn a_typescript_script_stamps_a_kind_on_every_import_and_export() {
    let v = |s: &str| Some(s.to_string());
    assert_eq!(
        kinds(TS, "ImportDeclaration", "importKind"),
        vec![v("value"), v("type"), v("value")]
    );
    assert_eq!(
        kinds(TS, "ImportSpecifier", "importKind"),
        vec![v("value"), v("value"), v("type"), v("value")]
    );
    assert_eq!(
        kinds(TS, "ExportNamedDeclaration", "exportKind"),
        vec![v("value"), v("value"), v("type")]
    );
    assert_eq!(
        kinds(TS, "ExportSpecifier", "exportKind"),
        vec![v("value"), v("value")]
    );
}

/// The control: a plain script must gain no kind at all. A fix that stamps one
/// unconditionally passes the test above and breaks every JavaScript component.
#[test]
fn a_plain_script_stamps_no_kind() {
    assert_eq!(kinds(JS, "ImportDeclaration", "importKind"), vec![None]);
    assert_eq!(kinds(JS, "ImportSpecifier", "importKind"), vec![None]);
    assert_eq!(
        kinds(JS, "ExportNamedDeclaration", "exportKind"),
        vec![None, None]
    );
    assert_eq!(kinds(JS, "ExportSpecifier", "exportKind"), vec![None]);
}

/// `attributes` is where the two parsers disagree in the other direction, and
/// rsvelte matches NEITHER: it serializes an always-empty list. Pinned as it is
/// so the next change here has to say which of the two it is moving towards —
/// acorn always emits the list, acorn-typescript emits it only where the source
/// wrote an `assert`/`with` clause, and rsvelte never populates it at all.
#[test]
fn attributes_is_an_always_empty_list_on_both_parsers() {
    assert_eq!(
        has_attributes(TS, "ImportDeclaration"),
        vec![true, true, true]
    );
    assert_eq!(has_attributes(JS, "ImportDeclaration"), vec![true]);
    assert_eq!(has_attributes(TS_ATTRS, "ImportDeclaration"), vec![true]);
    // The clause the source wrote reaches nothing.
    let tree = ast(TS_ATTRS);
    let mut found = Vec::new();
    nodes_of(&tree, "ImportDeclaration", &mut found);
    assert_eq!(
        found[0]
            .get("attributes")
            .and_then(Value::as_array)
            .map(Vec::len),
        Some(0)
    );
}
