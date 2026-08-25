//! Regression coverage for #3590.
//!
//! A computed key in an object binding pattern is evaluated, so its identifiers
//! are references rather than declarations. The phase-2 declaration guard used
//! to classify every identifier inside `VariableDeclarator.id` as a declaration
//! and consequently dropped this warning entirely.

use rsvelte_core::{CompileOptions, GenerateMode, Warning, compile};

const SOURCE: &str =
    "<script>\n\tlet s = $state(1);\n\tconst { [s]: value } = {};\n</script>\n\n<b>1</b>";

fn state_warnings(mode: GenerateMode) -> Vec<Warning> {
    compile(
        SOURCE,
        CompileOptions {
            filename: Some("Main.svelte".to_string()),
            generate: mode,
            ..Default::default()
        },
    )
    .expect("compile")
    .warnings
    .into_iter()
    .filter(|warning| warning.code == "state_referenced_locally")
    .collect()
}

#[test]
fn computed_destructuring_key_is_a_reference_on_both_targets() {
    for mode in [GenerateMode::Client, GenerateMode::Server] {
        let warnings = state_warnings(mode);
        assert_eq!(
            warnings.len(),
            1,
            "expected one state warning: {warnings:#?}"
        );

        let start = warnings[0].start.as_ref().expect("warning has a start");
        let end = warnings[0].end.as_ref().expect("warning has an end");
        assert_eq!((start.line, start.column), (3, 9));
        assert_eq!((end.line, end.column), (3, 10));
    }
}
