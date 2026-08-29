use rsvelte_core::{CompileOptions, GenerateMode, compile};

#[test]
fn unprocessed_indented_sass_errors_after_the_first_property_colon() {
    let source = "<style lang=\"sass\">\n\t.card\n\t\tdisplay: block\n</style>";
    let expected = u32::try_from(source.find("display:").unwrap() + "display:".len()).unwrap();

    for generate in [GenerateMode::Client, GenerateMode::Server] {
        let diagnostic = compile(
            source,
            CompileOptions {
                generate,
                ..Default::default()
            },
        )
        .expect_err("official rejects Sass that has not been preprocessed")
        .diagnostic();

        assert_eq!(
            (diagnostic.code.as_deref(), diagnostic.span,),
            (Some("css_expected_identifier"), Some((expected, expected)))
        );
    }
}

fn assert_error_at(source: &str, expected: usize) {
    let expected = u32::try_from(expected).expect("fixture offset fits in u32");

    for generate in [GenerateMode::Client, GenerateMode::Server] {
        let diagnostic = compile(
            source,
            CompileOptions {
                generate,
                ..Default::default()
            },
        )
        .expect_err("official rejects this CSS")
        .diagnostic();

        assert_eq!(
            (diagnostic.code.as_deref(), diagnostic.span),
            (Some("css_expected_identifier"), Some((expected, expected)))
        );
    }
}

#[test]
fn a_pseudo_class_colon_is_not_the_first_property_colon() {
    let source =
        "<style lang=\"sass\">\n\t:global(:root)\n\t\t--primary: #1a79ff\n\t\tcolor: red\n</style>";
    assert_error_at(
        source,
        source.find("--primary:").unwrap() + "--primary:".len(),
    );
}

#[test]
fn a_later_line_comment_does_not_outrank_an_earlier_identifier_error() {
    let source = "<style lang=\"sass\">\n\t:global(:root)\n\t\t--primary: #1a79ff\n\n\t\t// for demo editing\n\t\t--bg: #fff\n</style>";
    assert!(source.find("//").unwrap() > source.find("--primary:").unwrap());
    assert_error_at(
        source,
        source.find("--primary:").unwrap() + "--primary:".len(),
    );
}

#[test]
fn content_that_forms_no_rule_at_all_still_reports_at_the_block_end() {
    let source = "<style>\n\tthis is not css\n</style>";
    assert_error_at(source, source.find("</style>").unwrap());
}
