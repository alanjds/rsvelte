//! #3599: `<svelte:head><title>` must use the same `scope.evaluate().is_defined`
//! answer as every other client text chunk. A binary `+` produces a number or
//! string and therefore needs no `?? ''`; an unmodelled prop/store read does.

use rsvelte_core::{CompileOptions, GenerateMode, compile, compiler::CssMode};

fn client(initializer: &str, dev: bool) -> String {
    let source = format!(
        "<script>\n\timport {{ writable }} from 'svelte/store';\n\tlet {{ a }} = $props();\n\tconst s = writable();\n\tlet b = {initializer};\n</script>\n<svelte:head><title>{{b}}</title></svelte:head>\n"
    );

    compile(
        &source,
        CompileOptions {
            filename: Some("Title.svelte".to_string()),
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
fn binary_title_values_drop_only_the_dead_nullish_guard() {
    for dev in [false, true] {
        for (initializer, expected_guard) in [
            ("a", true),
            ("a + 1", false),
            ("$s", true),
            ("$s + a", false),
        ] {
            let out = client(initializer, dev);
            let has_guard = out.contains("b ?? ''");
            assert_eq!(
                has_guard, expected_guard,
                "wrong title guard for `{initializer}` (dev={dev}):\n{out}"
            );
        }
    }
}
