//! Regression coverage for the two reactive statement forms whose generated
//! dependency thunk was reprinted by the state-assignment pass (issues #3411
//! and #3579). Upstream builds even one dependency as a sequence, so its
//! redundant-looking parentheses are part of output parity.

use rsvelte_core::{CompileOptions, GenerateMode, compile, compiler::CssMode};

fn client(src: &str) -> String {
    compile(
        src,
        CompileOptions {
            filename: Some("main.svelte".to_string()),
            generate: GenerateMode::Client,
            dev: false,
            css: CssMode::External,
            ..Default::default()
        },
    )
    .expect("compile")
    .js
    .code
}

#[test]
fn compound_assignment_keeps_single_dependency_parens() {
    let out = client(
        r#"<script>
	let q;
	$: q += 1;
</script>
<p>{q}</p>
"#,
    );

    assert!(
        out.contains("$.legacy_pre_effect(() => ($.get(q)), () => {")
            && out.contains("$.set(q, $.get(q) + 1);"),
        "generated reactive statement diverged:\n{out}"
    );
}

#[test]
fn bare_block_keeps_single_dependency_parens() {
    let out = client(
        r#"<script>
	let d = 0;
	$: {
		let x = 1;
		d += x;
	}
</script>
<b>{d}</b>
"#,
    );

    assert!(
        out.contains("$.legacy_pre_effect(() => ($.get(d)), () => {")
            && out.contains("$.set(d, $.get(d) + x);"),
        "generated reactive block diverged:\n{out}"
    );
}
