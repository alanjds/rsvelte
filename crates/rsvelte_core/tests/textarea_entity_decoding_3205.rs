//! Regression test for #3205 — `<textarea>` content decoded a semicolon-less
//! legacy named reference that upstream leaves literal.
//!
//! Upstream reads a `<textarea>` body through `read_sequence`, which decodes
//! with `decode_character_references(raw, /* is_attribute_value */ true)`. That
//! flag appends `\b(?!=)` to every semicolon-less entity name, so `&notreal;`
//! never matches `&not` — while ordinary text (`is_attribute_value = false`)
//! does decode it. rsvelte passed `false` for the textarea body.

use rsvelte_core::{CompileOptions, GenerateMode, compile};

fn server(src: &str) -> String {
    compile(
        src,
        CompileOptions {
            generate: GenerateMode::Server,
            filename: Some("App.svelte".to_string()),
            ..Default::default()
        },
    )
    .expect("component should compile")
    .js
    .code
}

#[test]
fn semicolon_less_prefix_stays_literal_in_a_textarea() {
    for src in [
        "<textarea>a&notreal;b</textarea>",
        "<textarea>a&ampx;b</textarea>",
    ] {
        let out = server(src);
        assert!(
            !out.contains('\u{ac}') && !out.contains("&amp;x;"),
            "{src} decoded a legacy prefix:\n{out}"
        );
    }
}

/// The other side of the same rule: an entity that terminates properly, or one
/// followed by a non-word character, still decodes.
#[test]
fn real_entities_still_decode_in_a_textarea() {
    assert!(server("<textarea>a&not;b</textarea>").contains('\u{ac}'));
    assert!(server("<textarea>a&copy b</textarea>").contains('\u{a9}'));
    assert!(server("<textarea>a&nbsp;b</textarea>").contains('\u{a0}'));
}

/// Ordinary text keeps the content rule, where the prefix DOES decode.
#[test]
fn ordinary_text_still_decodes_the_prefix() {
    assert!(server("<div>a&notreal;b</div>").contains('\u{ac}'));
    assert!(server("<pre>a&notreal;b</pre>").contains('\u{ac}'));
}
