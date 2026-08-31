//! `textDocument/hover` for Svelte template tags and event modifiers.
//!
//! A port of the official language server's
//! `plugins/svelte/features/getHoverInfo.ts`.

use lsp_types::{Hover, HoverContents, MarkedString, MarkupContent, MarkupKind};

use crate::context::{EmbeddedRegions, attribute_context};
use crate::html_data::documentation::{Entry, documentation};
use crate::html_data::provider;
use crate::modifiers::MODIFIERS;
use crate::tags::{SvelteTag, latest_opening_tag};

/// How far either side of the cursor a tag may start.
const WINDOW: usize = 10;

/// The spellings that identify a tag. `:else` comes last because it belongs to
/// whichever block is open around it.
const TAG_SPELLINGS: &[(SvelteTag, &[&str])] = &[
    (SvelteTag::If, &["#if", "/if", ":else if"]),
    (SvelteTag::Each, &["#each", "/each"]),
    (SvelteTag::Await, &["#await", "/await", ":then", ":catch"]),
    (SvelteTag::Key, &["#key", "/key"]),
    (SvelteTag::Snippet, &["#snippet", "/snippet"]),
    (SvelteTag::Html, &["@html"]),
    (SvelteTag::Debug, &["@debug"]),
    (SvelteTag::Const, &["@const"]),
    (SvelteTag::Render, &["@render"]),
    (SvelteTag::Attach, &["@attach"]),
];

const ELSE: &str = ":else";

#[must_use]
pub fn hover(text: &str, offset: usize) -> Option<Hover> {
    let embedded = EmbeddedRegions::new(text);
    if let Some(style) = embedded.style_at(offset) {
        // `shouldExcludeHover` (`CSSPlugin.ts:606-616`).
        if matches!(style.language.as_deref(), Some("sass" | "stylus" | "styl")) {
            return None;
        }
        return crate::css::hover(text, offset).map(markdown);
    }
    // A script body belongs to tsgo; answering it here spells an import path as
    // a CSS property.
    if embedded.in_script(offset) {
        return None;
    }
    let (window_start, window) = around(text, offset);

    if opens_a_tag(window) {
        let tag = tag_at(text, window_start, window, offset)?;
        return Some(plain(tag.documentation().to_string()));
    }

    let attribute = attribute_context(text, offset)?;
    if attribute.in_value && attribute.name == "style" {
        return crate::css::hover(text, offset).map(markdown);
    }
    // `HTMLPlugin.doHover` bails on `possiblyComponent(node)`, so a component's
    // attributes get no HTML description.
    if !attribute.in_value
        && !attribute.on_a_component()
        && let Some(provided) = provider::attributes(attribute.element_tag)
            .into_iter()
            .find(|provided| provided.name == attribute.name)
        && let Some(value) = documentation(
            &Entry {
                description: provided.data.description,
                status: provided.data.status.as_ref(),
                browsers: provided.data.browsers,
                references: provided.data.references,
            },
            true,
        )
    {
        return Some(markdown(value));
    }
    if !attribute.can_have_event_modifier() {
        return None;
    }
    let modifier = MODIFIERS.iter().find(|modifier| {
        around_offset(attribute.name_start, attribute.name, modifier.name, offset)
    })?;
    // `getModifierData` (`features/getModifierData.ts:52-62`) maps every entry's
    // documentation into a Markdown `MarkupContent`; only the TAG hover next to
    // it hands back a bare string.
    Some(markdown(modifier.documentation()))
}

/// The `WINDOW` characters before `offset` plus what follows them, as the
/// official plugin slices it.
fn around(text: &str, offset: usize) -> (usize, &str) {
    let before = text.get(..offset).unwrap_or(text);
    let start = before
        .char_indices()
        .nth_back(WINDOW - 1)
        .map_or(0, |(idx, _)| idx);
    let rest = &text[start..];
    let end = rest
        .char_indices()
        .nth(WINDOW * 2)
        .map_or(rest.len(), |(idx, _)| idx);
    (start, &rest[..end])
}

/// Whether the window holds a `{`, optional whitespace and a known tag that
/// ends at a word boundary.
fn opens_a_tag(window: &str) -> bool {
    window.match_indices('{').any(|(idx, _)| {
        let rest = window[idx + 1..].trim_start();
        TAG_SPELLINGS
            .iter()
            .flat_map(|(_, spellings)| spellings.iter())
            .chain(std::iter::once(&ELSE))
            .any(|spelling| {
                rest.strip_prefix(spelling)
                    .is_some_and(|after| after.starts_with(['}', ' ', '\t', '\n', '\r']))
            })
    })
}

/// The tag whose spelling covers `offset`.
fn tag_at(text: &str, window_start: usize, window: &str, offset: usize) -> Option<SvelteTag> {
    let found = TAG_SPELLINGS.iter().find(|(_, spellings)| {
        spellings
            .iter()
            .any(|spelling| around_offset(window_start, window, spelling, offset))
    });
    match found {
        Some((tag, _)) => Some(*tag),
        // `{:else}` belongs to the innermost open block.
        None if around_offset(window_start, window, ELSE, offset) => {
            latest_opening_tag(text, offset)
        }
        None => None,
    }
}

/// Whether the first `needle` in `haystack` — which starts at `haystack_start`
/// in the document — spans `offset`.
fn around_offset(haystack_start: usize, haystack: &str, needle: &str, offset: usize) -> bool {
    let Some(idx) = haystack.find(needle) else {
        return false;
    };
    let start = haystack_start + idx;
    start <= offset && start + needle.len() >= offset
}

/// `getHoverInfo.ts` hands back `{ contents: <string> }`; a `MarkupContent`
/// wrapper around the same text is a different response on the wire.
const fn plain(value: String) -> Hover {
    Hover {
        contents: HoverContents::Scalar(MarkedString::String(value)),
        range: None,
    }
}

const fn markdown(value: String) -> Hover {
    Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value,
        }),
        range: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hovered_tag(content: &str, offset: usize) -> Option<String> {
        let hover = hover(content, offset)?;
        match hover.contents {
            HoverContents::Scalar(MarkedString::String(value)) => Some(value),
            HoverContents::Markup(content) => {
                assert_eq!(content.kind, MarkupKind::Markdown);
                Some(content.value)
            }
            _ => panic!("expected a hover body"),
        }
    }

    fn expect_tag(content: &str, offset: usize, tag: SvelteTag) {
        assert_eq!(
            hovered_tag(content, offset).as_deref(),
            Some(tag.documentation()),
            "{content:?} at {offset}"
        );
    }

    fn expect_none(content: &str, offset: usize) {
        assert_eq!(hovered_tag(content, offset), None, "{content:?}");
    }

    #[test]
    fn nothing_inside_style_or_script() {
        assert_eq!(
            hovered_tag("<style>h1{color:blue;}</style><p>test</p>", 10).as_deref(),
            Some("`color` CSS property")
        );
        expect_none("<script>const a = true</script><p>test</p>", 10);
    }

    #[test]
    fn nothing_on_ordinary_content() {
        expect_none("{nope", 2);
        expect_none("not really", 2);
        expect_none("{#await.", 3);
    }

    #[test]
    fn else_needs_an_open_block() {
        expect_none("{:else}", 3);
        expect_none("{#if}{/if}{:else}", 15);
    }

    #[test]
    fn else_resolves_to_the_open_block() {
        expect_tag("{#if}{:else}", 8, SvelteTag::If);
        expect_tag("{#each}{:else}", 10, SvelteTag::Each);
    }

    #[test]
    fn closing_tags_hover() {
        for (spelling, tag) in [
            ("if", SvelteTag::If),
            ("each", SvelteTag::Each),
            ("await", SvelteTag::Await),
        ] {
            expect_tag(&format!("{{/{spelling}}}"), 3, tag);
            expect_tag(&format!("{{/{spelling} "), 3, tag);
        }
    }

    #[test]
    fn opening_tags_hover() {
        for (spelling, tag) in [
            ("if", SvelteTag::If),
            ("each", SvelteTag::Each),
            ("await", SvelteTag::Await),
            ("key", SvelteTag::Key),
            ("snippet", SvelteTag::Snippet),
        ] {
            expect_tag(&format!("{{#{spelling}}}"), 3, tag);
            expect_tag(&format!("{{#{spelling} "), 3, tag);
        }
    }

    #[test]
    fn at_tags_hover() {
        for (spelling, tag) in [
            ("debug", SvelteTag::Debug),
            ("html", SvelteTag::Html),
            ("const", SvelteTag::Const),
            ("render", SvelteTag::Render),
            ("attach", SvelteTag::Attach),
        ] {
            expect_tag(&format!("{{@{spelling}}}"), 3, tag);
            expect_tag(&format!("{{@{spelling} "), 3, tag);
        }
    }

    #[test]
    fn definite_continuations_hover() {
        expect_tag("{:else if}", 3, SvelteTag::If);
        expect_tag("{:else if ", 3, SvelteTag::If);
        expect_tag("{:then}", 3, SvelteTag::Await);
        expect_tag("{:catch}", 3, SvelteTag::Await);
    }

    #[test]
    fn an_event_modifier_hovers() {
        let hovered = hovered_tag("<div on:click|preventDefault />", 15).unwrap();
        assert!(hovered.starts_with("`preventDefault` event modifier"));
        assert!(hovered.contains("event.preventDefault()"));
    }

    /// The two producers in `getHoverInfo.ts` disagree on the wire shape: a tag
    /// is `{ contents: <string> }` (:56) and a modifier is a `MarkupContent`
    /// (`getModifierData.ts:52-62`). `hovered_tag` accepts either, so the shape
    /// needs its own assertion.
    #[test]
    fn a_tag_hover_is_a_string_and_a_modifier_hover_is_markup() {
        assert!(matches!(
            hover("{#if x}", 3).unwrap().contents,
            HoverContents::Scalar(MarkedString::String(_))
        ));
        assert!(matches!(
            hover("<div on:click|preventDefault />", 15)
                .unwrap()
                .contents,
            HoverContents::Markup(_)
        ));
    }

    #[test]
    fn a_second_modifier_hovers_too() {
        let hovered = hovered_tag("<div on:click|preventDefault|once />", 31).unwrap();
        assert!(hovered.starts_with("`once` event modifier"));
    }

    /// The prose is the vendored data's, not this crate's.
    #[test]
    fn a_plain_event_directive_has_no_modifier_hover() {
        assert_eq!(
            hovered_tag("<div on:click />", 12).as_deref(),
            Some("A pointing device button has been pressed and released on an element.")
        );
    }

    #[test]
    fn native_directives_and_bindings_hover() {
        assert_eq!(
            hovered_tag("<div transition: />", 8).as_deref(),
            Some(
                "A transition is triggered by an element entering or leaving the DOM as a result of a state change.\n\n[Svelte.dev Reference](https://svelte.dev/docs/svelte/transition)"
            )
        );
        assert_eq!(
            hovered_tag("<input bind:checked />", 13).as_deref(),
            Some("Available for type=\"checkbox\"")
        );
    }

    #[test]
    fn a_tag_inside_a_script_body_is_left_alone() {
        let text = "<script>\n  const a = `{#if}`;\n</script>";
        expect_none(text, text.find("{#if}").unwrap() + 3);
    }

    #[test]
    fn a_css_property_name_in_a_script_is_not_a_css_hover() {
        let text = "<script>\n  import type { A } from \"../types.js\";\n</script>";
        expect_none(text, text.find("types.js").unwrap() + 1);
    }

    /// `shouldExcludeHover` (`CSSPlugin.ts:606-616`) excludes `sass` too, which
    /// `shouldExcludeCompletion` does not.
    #[test]
    fn a_sass_or_stylus_block_gets_no_css_hover() {
        for lang in ["sass", "stylus", "styl"] {
            let text = format!("<style lang=\"{lang}\">\n  h1 {{ color: red }}\n</style>");
            expect_none(&text, text.find("color").unwrap() + 1);
        }
        // The positive control: `less` is not on upstream's list.
        let text = "<style lang=\"less\">\n  h1 { color: red }\n</style>";
        assert!(hovered_tag(text, text.find("color").unwrap() + 1).is_some());
    }

    #[test]
    fn a_style_body_is_still_answered_from_css() {
        let text = "<style>\n  h1 { color: red }\n</style>";
        assert!(hovered_tag(text, text.find("color").unwrap() + 1).is_some());
    }

    #[test]
    fn a_components_attribute_has_no_html_description() {
        let element = "<div class=\"a\">x</div>";
        assert!(hovered_tag(element, element.find("class").unwrap() + 1).is_some());
        let component = "<Widget class=\"a\">x</Widget>";
        expect_none(component, component.find("class").unwrap() + 1);
        // A `style` value is the CSS plugin's, which upstream does not gate on
        // the tag being a component.
        let styled = "<Widget style=\"color: red\">x</Widget>";
        assert!(hovered_tag(styled, styled.find("color").unwrap() + 1).is_some());
    }
}
