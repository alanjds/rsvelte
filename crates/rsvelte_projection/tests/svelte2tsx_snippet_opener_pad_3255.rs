//! #3255: the pad before a standalone `{#snippet}`'s `const` is the number of
//! non-empty gaps between the source ranges upstream's `transform()` keeps —
//! not a measurement of the region left before `}`. Every value below was read
//! off the official `svelte2tsx` for the same header.
//!
//! svelte2tsx is MagicString-based, so one missing character shifts every
//! mapping after it; this is a source-map assertion as much as a text one.

use rsvelte_projection::svelte2tsx::{Svelte2TsxOptions, svelte2tsx};

fn opener_pad(header: &str) -> usize {
    let src = format!("<script lang=\"ts\">\n</script>\n{header}x{{/snippet}}");
    let opts = Svelte2TsxOptions {
        filename: "S.svelte".to_string(),
        is_ts_file: true,
        ..Default::default()
    };
    let code = svelte2tsx(&src, opts).expect("svelte2tsx ok").code;
    let anchor = code
        .find("const s/*")
        .unwrap_or_else(|| panic!("no snippet declaration in:\n{code}"));
    code[..anchor]
        .chars()
        .rev()
        .take_while(|character| *character == ' ')
        .count()
}

#[test]
fn the_pad_counts_the_gaps_between_the_kept_ranges() {
    // One gap: `{#snippet ` only — the name range and the parameter range are
    // adjacent because the widening consumed the `(`.
    for header in [
        "{#snippet s(a)}",
        "{#snippet s(a, b)}",
        "{#snippet s(a , b)}",
        "{#snippet s(a: string)}",
        "{#snippet s(/* c */ a)}",
    ] {
        assert_eq!(opener_pad(header), 1, "{header}");
    }

    // Two gaps: anything at all between the name and the first parameter, or a
    // tail that still has something left after the deleted character.
    for header in [
        "{#snippet s (a)}",
        "{#snippet s\t(a)}",
        "{#snippet s( a)}",
        "{#snippet s(  a)}",
        "{#snippet s( /* c */ a)}",
        "{#snippet s<T>(a)}",
        "{#snippet s<T,U>(a)}",
        "{#snippet s<T extends string>(a)}",
        "{#snippet s<T,>(a)}",
        "{#snippet s<T>()}",
        "{#snippet s()}",
        "{#snippet s( )}",
        "{#snippet s(a )}",
        "{#snippet s(a  )}",
        "{#snippet s(a,)}",
    ] {
        assert_eq!(opener_pad(header), 2, "{header}");
    }

    // Three: a formatted multi-line parameter list opens the middle gap AND
    // leaves a tail — the shape the old tail-measuring rule could not reach.
    for header in ["{#snippet s(\n\ta\n)}", "{#snippet s<T>(\n\ta: T\n)}"] {
        assert_eq!(opener_pad(header), 3, "{header}");
    }
}
