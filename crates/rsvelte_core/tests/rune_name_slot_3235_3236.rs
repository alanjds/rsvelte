//! Regression tests for #3235 and #3236 — a rune-spelled name is decided by
//! the bytes in the source rather than by the identifier the parser built.
//!
//! #3235: a NAME slot holding a rune spelling — `o.$derived(1)`, `o?.$derived`,
//! a class or object member `$derived(v) {}` — was rewritten as the rune.
//! Four of the shapes emitted text no JS parser accepts, because the call was
//! deleted rather than rewritten (`export const a = o.`).
//!
//! #3236: the opposite direction of the same premise. `\u0024state` and
//! `$st\u0061te` ARE the identifier `$state`, so upstream lowers them; rsvelte
//! matched the source spelling, leaving the rune in the generated module
//! (a `ReferenceError` at import) or rejecting the identifier outright.
//!
//! Both are checked by a twin: the same source with the rune spelling swapped
//! for a same-length name that is provably NOT a rune (#3235) or for the plain
//! spelling of the same identifier (#3236). The two outputs must be the same
//! text. A byte comparison alone cannot tell "wrong text" from "text that is
//! not JavaScript", so every output is also handed to a JS parser.

use rsvelte_core::{CompileOptions, GenerateMode, ModuleCompileOptions, compile, compile_module};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Target {
    Client,
    Server,
}

impl Target {
    fn generate(self) -> GenerateMode {
        match self {
            Target::Client => GenerateMode::Client,
            Target::Server => GenerateMode::Server,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Target::Client => "client",
            Target::Server => "server",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Entry {
    ModuleJs,
    ModuleTs,
    Instance,
    ScriptModule,
}

impl Entry {
    fn label(self) -> &'static str {
        match self {
            Entry::ModuleJs => "m.svelte.js",
            Entry::ModuleTs => "m.svelte.ts",
            Entry::Instance => "instance",
            Entry::ScriptModule => "<script module>",
        }
    }

    /// Wrap a module-shaped body for this entry point.
    fn wrap(self, body: &str) -> String {
        match self {
            Entry::ModuleJs | Entry::ModuleTs => format!("{body}\n"),
            Entry::Instance => format!(
                "<script>\n{}\n</script>\n<div>x</div>\n",
                strip_exports(body)
            ),
            Entry::ScriptModule => format!(
                "<script module>\n{}\n</script>\n<div>x</div>\n",
                strip_exports(body)
            ),
        }
    }
}

fn strip_exports(body: &str) -> String {
    body.lines()
        .map(|line| line.strip_prefix("export ").unwrap_or(line))
        .collect::<Vec<_>>()
        .join("\n")
}

fn compile_at(entry: Entry, target: Target, source: &str) -> Result<String, String> {
    match entry {
        Entry::ModuleJs | Entry::ModuleTs => compile_module(
            source,
            ModuleCompileOptions {
                generate: target.generate(),
                filename: Some(entry.label().to_string()),
                ..Default::default()
            },
        )
        .map(|r| r.js.code)
        .map_err(|e| format!("{e:?}")),
        Entry::Instance | Entry::ScriptModule => compile(
            source,
            CompileOptions {
                generate: target.generate(),
                filename: Some("C.svelte".to_string()),
                ..Default::default()
            },
        )
        .map(|r| r.js.code)
        .map_err(|e| format!("{e:?}")),
    }
}

#[track_caller]
fn assert_parses(code: &str, what: &str) {
    let allocator = oxc_allocator::Allocator::default();
    let ret = oxc_parser::Parser::new(&allocator, code, oxc_span::SourceType::mjs()).parse();
    assert!(
        ret.diagnostics.is_empty(),
        "{what}: emitted JS does not parse: {:?}\n--- output ---\n{code}",
        ret.diagnostics
    );
}

/// Compile `probe` and `twin` at every entry point / target and require the two
/// outputs to agree once `twin_name` is read back as `rune_name`.
///
/// `twin_name` is the same LENGTH as `rune_name` so the printer's line-breaking
/// decisions cannot differ for a reason other than the rune.
#[track_caller]
fn assert_twin(entries: &[Entry], probe: &str, twin: &str, twin_name: &str, rune_name: &str) {
    assert_eq!(
        twin_name.len(),
        rune_name.len(),
        "the twin name must not change the printed width"
    );
    for &entry in entries {
        for target in [Target::Client, Target::Server] {
            let what = format!("{} | {} | {rune_name}", entry.label(), target.label());
            let probe_out = compile_at(entry, target, &entry.wrap(probe))
                .unwrap_or_else(|e| panic!("{what}: the probe was rejected: {e}"));
            let twin_out = compile_at(entry, target, &entry.wrap(twin))
                .unwrap_or_else(|e| panic!("{what}: the twin was rejected: {e}"));
            assert_parses(&probe_out, &what);
            assert_parses(&twin_out, &what);
            assert_eq!(
                probe_out,
                twin_out.replace(twin_name, rune_name),
                "{what}: the rune spelling changed the output"
            );
        }
    }
}

const MODULES: &[Entry] = &[Entry::ModuleJs, Entry::ModuleTs];
const EVERY_ENTRY: &[Entry] = &[
    Entry::ModuleJs,
    Entry::ModuleTs,
    Entry::Instance,
    Entry::ScriptModule,
];

// ---------------------------------------------------------------- #3235 ----

/// `(probe body, twin body)` for one member-slot shape, with `$derived`
/// swapped for the eight-character non-rune `qderived`.
fn derived_shapes() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        (
            "member access",
            "const o = { $derived: (v) => v };\nexport const a = o.$derived(1);",
            "const o = { qderived: (v) => v };\nexport const a = o.qderived(1);",
        ),
        (
            "optional member access",
            "const o = { $derived: (v) => v };\nexport const a = o?.$derived(1);",
            "const o = { qderived: (v) => v };\nexport const a = o?.qderived(1);",
        ),
        (
            "nested member access",
            "const o = { p: { $derived: (v) => v } };\nexport const a = o.p.$derived(1);",
            "const o = { p: { qderived: (v) => v } };\nexport const a = o.p.qderived(1);",
        ),
        (
            "member access on the next line",
            "const o = { $derived: (v) => v };\nexport const a = o\n\t.$derived(1);",
            "const o = { qderived: (v) => v };\nexport const a = o\n\t.qderived(1);",
        ),
        (
            "class member name",
            "export class C {\n\t$derived(v) { return v; }\n\tm() { return this.$derived(1); }\n}",
            "export class C {\n\tqderived(v) { return v; }\n\tm() { return this.qderived(1); }\n}",
        ),
        (
            "object member name",
            "export const o = {\n\t$derived(v) { return v; }\n};",
            "export const o = {\n\tqderived(v) { return v; }\n};",
        ),
    ]
}

#[test]
fn a_derived_spelled_name_in_a_member_slot_is_not_the_rune() {
    for (what, probe, twin) in derived_shapes() {
        eprintln!("shape: {what}");
        assert_twin(EVERY_ENTRY, probe, twin, "qderived", "$derived");
    }
}

#[test]
fn a_state_spelled_name_in_a_member_slot_is_not_the_rune() {
    assert_twin(
        EVERY_ENTRY,
        "const o = { $state: (v) => v };\nexport const a = o.$state(1);",
        "const o = { qstate: (v) => v };\nexport const a = o.qstate(1);",
        "qstate",
        "$state",
    );
}

#[test]
fn an_effect_spelled_name_in_a_member_slot_is_not_the_rune() {
    assert_twin(
        EVERY_ENTRY,
        "const o = { $effect: (f) => f };\nexport const a = o.$effect(() => {});",
        "const o = { qeffect: (f) => f };\nexport const a = o.qeffect(() => {});",
        "qeffect",
        "$effect",
    );
}

#[test]
fn an_inspect_spelled_name_in_a_member_slot_is_not_the_rune() {
    assert_twin(
        EVERY_ENTRY,
        "const o = { $inspect: (v) => v };\nexport const a = o.$inspect(1);",
        "const o = { qinspect: (v) => v };\nexport const a = o.qinspect(1);",
        "qinspect",
        "$inspect",
    );
}

/// The control the twin comparison needs: the rune itself still lowers. Without
/// it, "never treat a `$derived` as a rune" would pass every test above.
#[test]
fn the_rune_itself_still_lowers() {
    let out = compile_at(
        Entry::ModuleJs,
        Target::Client,
        "let a = $state(1);\nlet d = $derived(a);\nexport const read = () => d;\n",
    )
    .expect("module should compile");
    assert!(
        out.contains("$.derived(") && !out.contains("$derived("),
        "the rune was left unlowered:\n{out}"
    );
}

/// #3235's four unparseable rows: the call was deleted rather than rewritten.
#[test]
fn a_member_slot_call_survives_verbatim() {
    for (source, needle) in [
        (
            "const o = { $effect: (f) => f };\nexport const a = o.$effect(() => {});\n",
            "o.$effect(",
        ),
        (
            "const o = { $inspect: (v) => v };\nexport const a = o.$inspect(1);\n",
            "o.$inspect(",
        ),
    ] {
        for entry in [Entry::ModuleJs, Entry::ModuleTs] {
            for target in [Target::Client, Target::Server] {
                let out = compile_at(entry, target, source).expect("module should compile");
                assert_parses(&out, needle);
                assert!(
                    out.contains(needle),
                    "{} | {}: `{needle}` was rewritten away:\n{out}",
                    entry.label(),
                    target.label()
                );
            }
        }
    }
}

// ---------------------------------------------------------------- #3236 ----

/// `\u0024` is the six source characters that spell `$`, so `\u0024state` IS
/// the identifier `$state`. `assert_twin` cannot be reused: the two spellings
/// have different widths on purpose, and the identifier is consumed by the
/// lowering, so the outputs are expected to be equal with no substitution.
#[track_caller]
fn assert_same_as_plain(entries: &[Entry], escaped: &str, plain: &str) {
    for &entry in entries {
        for target in [Target::Client, Target::Server] {
            let what = format!("{} | {} | {escaped}", entry.label(), target.label());
            let escaped_out = compile_at(entry, target, &entry.wrap(escaped))
                .unwrap_or_else(|e| panic!("{what}: the escaped spelling was rejected: {e}"));
            let plain_out = compile_at(entry, target, &entry.wrap(plain))
                .unwrap_or_else(|e| panic!("{what}: the plain spelling was rejected: {e}"));
            assert_parses(&escaped_out, &what);
            assert_eq!(
                escaped_out, plain_out,
                "{what}: the escape changed the output"
            );
        }
    }
}

#[test]
fn an_escaped_dollar_is_the_rune() {
    assert_same_as_plain(
        EVERY_ENTRY,
        r"export let a = \u0024state(1);",
        "export let a = $state(1);",
    );
    assert_same_as_plain(
        EVERY_ENTRY,
        "export let a = $state(1);\nlet d = \\u0024derived(a);\nexport const read = () => d;",
        "export let a = $state(1);\nlet d = $derived(a);\nexport const read = () => d;",
    );
    assert_same_as_plain(
        EVERY_ENTRY,
        "export let a = $state(1);\nlet d = \\u0024derived.by(() => a);\nexport const read = () => d;",
        "export let a = $state(1);\nlet d = $derived.by(() => a);\nexport const read = () => d;",
    );
    assert_same_as_plain(
        MODULES,
        r"export let a = \u0024state.raw(1);",
        "export let a = $state.raw(1);",
    );
}

#[test]
fn an_escape_inside_the_rune_name_is_the_rune() {
    assert_same_as_plain(
        EVERY_ENTRY,
        r"export let a = $st\u0061te(1);",
        "export let a = $state(1);",
    );
}

/// `\u0024props()` carries no `$` byte at all, so the runes-mode probe closed on
/// it and the component was compiled as Svelte 4 — where `$props()` is a hard
/// error.
#[test]
fn an_escaped_props_rune_still_turns_runes_mode_on() {
    for target in [Target::Client, Target::Server] {
        let escaped = compile_at(
            Entry::Instance,
            target,
            "<script>\n\tlet { p } = \\u0024props();\n</script>\n<div>{p}</div>\n",
        )
        .expect("the escaped `$props()` should compile");
        let plain = compile_at(
            Entry::Instance,
            target,
            "<script>\n\tlet { p } = $props();\n</script>\n<div>{p}</div>\n",
        )
        .expect("the plain `$props()` should compile");
        assert_parses(&escaped, "escaped $props()");
        assert_eq!(escaped, plain, "the escape changed the output");
    }
}

/// The negative control for the `\u` probe widening: an escaped name that is
/// NOT a rune must stay a plain identifier, and must not flip runes mode on.
#[test]
fn an_escaped_non_rune_name_is_left_alone() {
    let out = compile_at(
        Entry::Instance,
        Target::Client,
        "<script>\n\texport let p = 1;\n\tlet \\u0061bc = p;\n</script>\n<div>{\\u0061bc}</div>\n",
    )
    .expect("a legacy component with an escaped local should compile");
    assert_parses(&out, "escaped non-rune");
    assert!(
        out.contains("abc"),
        "the escaped local vanished from the output:\n{out}"
    );
}
