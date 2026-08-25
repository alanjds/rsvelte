use rsvelte_formatter::{FormatOptions, format};

fn assert_fmt(src: &str, expected: &str) {
    let out = format(src, &FormatOptions::default()).expect("format ok");
    assert_eq!(out, expected, "got:\n{out}");
    assert_eq!(
        format(&out, &FormatOptions::default()).expect("second format ok"),
        out,
        "formatter output must be a fixed point"
    );
}

#[test]
fn issue_3675_hugs_a_long_inline_element_glued_inside_flow_blocks() {
    let text = "aaaaaaaa bbbbbbbb cccccccc dddddddd eeeeeeee ffffffff gggggggg";
    for (open, close) in [
        ("{#if flag}", "{/if}"),
        ("{#each items as item}", "{/each}"),
        ("{#key value}", "{/key}"),
        ("{#await promise}", "{/await}"),
    ] {
        let src = format!("{open}<span>{text}</span>{close}\n");
        let expected = format!("{open}<span\n    >{text}</span\n  >{close}\n");
        assert_fmt(&src, &expected);
    }
}

#[test]
fn issue_3675_hugs_an_unbreakable_expression_run() {
    assert_fmt(
        "{#if flag}<span>{o.aaaaaaaa}{o.bbbbbbbb}{o.cccccccc}{o.dddddddd}{o.eeeeeeee}{o.ffffffff}</span>{/if}\n",
        "{#if flag}<span\n    >{o.aaaaaaaa}{o.bbbbbbbb}{o.cccccccc}{o.dddddddd}{o.eeeeeeee}{o.ffffffff}</span\n  >{/if}\n",
    );
}

#[test]
fn a_short_element_keeps_its_open_bracket_on_the_block_line() {
    assert_fmt(
        "          {#each group.breadcrumbs as breadcrumb}<span>{breadcrumb}</span>{/each}\n",
        "          {#each group.breadcrumbs as breadcrumb}<span>{breadcrumb}</span\n            >{/each}\n",
    );
}
