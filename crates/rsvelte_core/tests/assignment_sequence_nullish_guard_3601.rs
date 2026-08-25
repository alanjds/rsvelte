//! #3601: upstream `scope.evaluate` has no `AssignmentExpression` or
//! `SequenceExpression` arm. Both are UNKNOWN even when their result can be
//! inferred from a child, so a concatenated template chunk must retain its
//! `?? ''` guard. The shared evaluator introduced after the report already has
//! that behavior; this pins the complete measured matrix, including controls.

use rsvelte_core::{CompileOptions, GenerateMode, compile, compiler::CssMode};

fn client(initializer: &str, dev: bool) -> String {
    let source = format!(
        "<script>\n\tlet {{ cond, un }} = $props();\n\tlet obj = $state({{ a: 0 }});\n\tconst v = {initializer};\n</script>\n\n{{v}}{{cond}}\n"
    );

    compile(
        &source,
        CompileOptions {
            filename: Some("NullishGuard.svelte".to_string()),
            generate: GenerateMode::Client,
            dev,
            css: CssMode::External,
            ..Default::default()
        },
    )
    .expect("compile")
    .js
    .code
}

#[test]
fn assignment_and_sequence_initializers_stay_unknown() {
    let cases = [
        ("(obj.a = 1)", true),
        ("(obj.a = cond ? 1 : 2)", true),
        ("(obj.a = (un, 1))", true),
        ("(un, 1)", true),
        ("(obj.a = 1 || un)", true),
        ("(obj.a = un)", true),
        ("(cond ? 1 : 2)", false),
        ("obj.a", true),
        ("un", true),
    ];

    for dev in [false, true] {
        for (initializer, expected_guard) in cases {
            let out = client(initializer, dev);
            let has_guard = out.contains("${v ?? ''}");
            assert_eq!(
                has_guard, expected_guard,
                "wrong guard for `{initializer}` (dev={dev}):\n{out}"
            );
        }
    }
}
