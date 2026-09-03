//! Native CSS assistance for Svelte style blocks and static style attributes.

use lsp_types::{
    Color, ColorInformation, ColorPresentation, CompletionItem, CompletionItemKind,
    CompletionItemTag, CompletionList, CompletionTextEdit, Diagnostic, DiagnosticSeverity,
    Documentation, InsertTextFormat, MarkupContent, MarkupKind, NumberOrString, Range, TextEdit,
};

use rsvelte_lint::rules::data::known_css_properties::KNOWN_CSS_PROPERTIES;

use crate::css_data::documentation::documentation;
use crate::css_data::svelte_css::SVELTE_PSEUDO_CLASSES;
use crate::css_data::web::{
    AT_DIRECTIVES, Entry, HTML5_TAGS, PSEUDO_CLASSES, PSEUDO_ELEMENTS, SVG_ELEMENTS,
};

use crate::text::LineIndex;

#[must_use]
pub fn colors(text: &str) -> Vec<ColorInformation> {
    let index = LineIndex::new(text);
    text.match_indices('#')
        .filter_map(|(start, _)| {
            let end = start + 7;
            let hex = text.get(start + 1..end)?;
            if !(style_body(text, start) || static_style_value(text, start))
                || !hex.as_bytes().iter().all(u8::is_ascii_hexdigit)
            {
                return None;
            }
            Some(ColorInformation {
                range: Range::new(index.position(text, start), index.position(text, end)),
                color: Color {
                    red: u8::from_str_radix(&hex[..2], 16).ok()? as f32 / 255.0,
                    green: u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0,
                    blue: u8::from_str_radix(&hex[4..], 16).ok()? as f32 / 255.0,
                    alpha: 1.0,
                },
            })
        })
        .collect()
}

#[must_use]
pub fn diagnostics(text: &str) -> Vec<Diagnostic> {
    let index = LineIndex::new(text);
    let mut diagnostics = Vec::new();
    let mut from = 0;
    while let Some(open) = text[from..].find("<style") {
        let open = from + open;
        let Some(start) = text[open..].find('>').map(|at| open + at + 1) else {
            break;
        };
        let end = text[start..]
            .find("</style")
            .map_or(text.len(), |at| start + at);
        let body = &text[start..end];
        let mut line_offset = 0;
        for line in body.split_inclusive('\n') {
            let current_line_offset = line_offset;
            line_offset += line.len();
            let Some(colon) = line.find(':') else {
                continue;
            };
            let property = line[..colon]
                .rsplit(['{', '}', ';'])
                .next()
                .unwrap_or("")
                .trim();
            if property.is_empty()
                || property.starts_with("--")
                || !property
                    .bytes()
                    .all(|byte| byte.is_ascii_alphabetic() || byte == b'-')
                || KNOWN_CSS_PROPERTIES.contains(&property)
            {
                continue;
            }
            let property_start = start + current_line_offset + line.find(property).unwrap_or(0);
            diagnostics.push(Diagnostic {
                range: Range::new(
                    index.position(text, property_start),
                    index.position(text, property_start + property.len()),
                ),
                severity: Some(DiagnosticSeverity::WARNING),
                code: Some(NumberOrString::String("css_unknown_property".to_string())),
                source: Some("rsvelte-css".to_string()),
                message: format!("Unknown CSS property `{property}`."),
                ..Diagnostic::default()
            });
        }
        from = end.saturating_add(8);
    }
    diagnostics
}

#[must_use]
pub fn color_presentations(color: Color) -> Vec<ColorPresentation> {
    let channel = |value: f32| (value.clamp(0.0, 1.0) * 255.0).round() as u8;
    vec![ColorPresentation {
        label: format!(
            "#{:02x}{:02x}{:02x}",
            channel(color.red),
            channel(color.green),
            channel(color.blue)
        ),
        ..ColorPresentation::default()
    }]
}

/// Selection expansion spans, innermost first, for a CSS declaration.
#[must_use]
pub fn selection_spans(text: &str, offset: usize) -> Vec<(u32, u32)> {
    let Some(body) = style_body_range(text, offset) else {
        return Vec::new();
    };
    let before = &text[body.start..offset.min(body.end)];
    let declaration_start = before
        .rfind([';', '{', '}'])
        .map_or(body.start, |i| body.start + i + 1);
    let declaration_end = text[offset.min(body.end)..body.end]
        .find([';', '}'])
        .map_or(body.end, |i| offset + i);
    let word = word_at(text, offset).and_then(|word| {
        let start = word.as_ptr() as usize - text.as_ptr() as usize;
        u32::try_from(start)
            .ok()
            .zip(u32::try_from(start + word.len()).ok())
    });
    word.into_iter()
        .chain(std::iter::once((
            declaration_start as u32,
            declaration_end as u32,
        )))
        .chain(std::iter::once((body.start as u32, body.end as u32)))
        .collect()
}

/// CSS completions at `offset`, when it is in a declaration name or value.
#[must_use]
pub fn completions(text: &str, offset: usize) -> Option<CompletionList> {
    let prefix = css_prefix(text, offset)?;
    let before = text.get(..offset)?;
    let prefix_start = prefix.as_ptr() as usize - before.as_ptr() as usize;
    if let Some(marker) = prefix_start
        .checked_sub(1)
        .and_then(|index| before.as_bytes().get(index))
        && matches!(marker, b'.' | b'#')
    {
        return Some(selector_completions(text, *marker as char, prefix));
    }
    if let Some(body) = style_body_range(text, offset)
        && brace_depth(&text[body.start..offset]) == 0
    {
        return Some(selector_context_completions(text, body.start, offset));
    }
    let value = before
        .rfind(':')
        .is_some_and(|colon| before[colon + 1..].find([';', '{', '}']).is_none());
    let items = if value {
        values(prefix)
    } else {
        KNOWN_CSS_PROPERTIES
            .iter()
            .copied()
            .filter(|property| property.starts_with(prefix))
            .map(property_item)
            .collect()
    };
    Some(CompletionList {
        is_incomplete: false,
        items,
    })
}

/// The CSS property under `offset`, including a compact native description.
#[must_use]
pub fn hover(text: &str, offset: usize) -> Option<String> {
    if text.get(..offset)?.ends_with(":global") || word_at(text, offset) == Some("global") {
        return Some("`:global(...)` prevents Svelte CSS scoping for a selector.".to_string());
    }
    let property = word_at(text, offset)?;
    KNOWN_CSS_PROPERTIES
        .contains(&property)
        .then(|| format!("`{property}` CSS property"))
}

/// Unbalanced `{` in `text`, so a cursor in a selector can be told from one in
/// a declaration block.
fn brace_depth(text: &str) -> usize {
    text.bytes().fold(0usize, |depth, byte| match byte {
        b'{' => depth + 1,
        b'}' => depth.saturating_sub(1),
        _ => depth,
    })
}

/// Where the selector token under the cursor begins. Upstream generates no node
/// for a bare `:`, so `getCompletionsForSelector` grows `currentWord` back over
/// the colons and the replace range grows with it.
fn selector_token_start(body: &str) -> usize {
    let bytes = body.as_bytes();
    let mut start = body.len();
    while start > 0
        && matches!(bytes[start - 1], b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_')
    {
        start -= 1;
    }
    while start > 0 && bytes[start - 1] == b':' {
        start -= 1;
    }
    if start > 0 && bytes[start - 1] == b'@' {
        start -= 1;
    }
    start
}

/// Whitespace and complete block comments only, which is what upstream's parser
/// skips before deciding a statement may begin here.
fn is_blank_css(text: &str) -> bool {
    let mut rest = text.trim_start();
    while let Some(after) = rest.strip_prefix("/*") {
        rest = after
            .find("*/")
            .map_or("", |end| &after[end + 2..])
            .trim_start();
    }
    rest.is_empty()
}

/// Upstream reaches its at-directive list only from `getCompletionForTopLevel`,
/// which needs a parse tree; this is the lexical stand-in, and the residue it
/// leaves is enumerated in `divergences_from_the_official_service_this_leaves`.
fn offers_at_directives(body: &str, token_start: usize) -> bool {
    let token = &body[token_start..];
    if !token.is_empty() && !token.starts_with('@') {
        return false;
    }
    let head = &body[..token_start];
    is_blank_css(head.rfind('}').map_or(head, |at| &head[at + 1..]))
}

fn entry_tags(entry: &Entry) -> Vec<CompletionItemTag> {
    if matches!(entry.status, Some("nonstandard" | "obsolete")) {
        vec![CompletionItemTag::DEPRECATED]
    } else {
        Vec::new()
    }
}

fn entry_documentation(entry: &Entry) -> Option<Documentation> {
    documentation(&entry.into(), true).map(|value| {
        Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value,
        })
    })
}

fn replace(range: Range, new_text: impl Into<String>) -> Option<CompletionTextEdit> {
    Some(CompletionTextEdit::Edit(TextEdit {
        range,
        new_text: new_text.into(),
    }))
}

fn at_directive_item(entry: &Entry, range: Range) -> CompletionItem {
    CompletionItem {
        label: entry.name.to_string(),
        kind: Some(CompletionItemKind::KEYWORD),
        documentation: entry_documentation(entry),
        tags: Some(entry_tags(entry)),
        text_edit: replace(range, entry.name),
        ..CompletionItem::default()
    }
}

/// `vendor` is the prefix that sorts an entry last: `:-` for a pseudo-class and
/// `::-` for a pseudo-element, so the two tables cannot share one test.
fn pseudo_item(entry: &Entry, range: Range, vendor: &str) -> CompletionItem {
    // `moveCursorInsideParenthesis`: a trailing `()` becomes a snippet stop.
    let (new_text, insert_text_format) = match entry.name.strip_suffix("()") {
        Some(head) => (format!("{head}($1)"), Some(InsertTextFormat::SNIPPET)),
        None => (entry.name.to_string(), None),
    };
    CompletionItem {
        label: entry.name.to_string(),
        kind: Some(CompletionItemKind::FUNCTION),
        documentation: entry_documentation(entry),
        tags: Some(entry_tags(entry)),
        text_edit: replace(range, new_text),
        insert_text_format,
        sort_text: entry.name.starts_with(vendor).then(|| "x".to_string()),
        ..CompletionItem::default()
    }
}

/// What the official service offers in a selector position: pseudo-classes,
/// Svelte's `:global()`, pseudo-elements, every HTML and SVG element name, and —
/// where a statement may begin — the at-directives.
fn selector_context_completions(text: &str, body_start: usize, offset: usize) -> CompletionList {
    let body = &text[body_start..offset];
    let token_start = selector_token_start(body);
    let index = LineIndex::new(text);
    let range = Range::new(
        index.position(text, body_start + token_start),
        index.position(text, offset),
    );

    let mut items: Vec<CompletionItem> = Vec::new();
    if offers_at_directives(body, token_start) {
        items.extend(
            AT_DIRECTIVES
                .iter()
                .map(|entry| at_directive_item(entry, range)),
        );
    }
    items.extend(
        PSEUDO_CLASSES
            .iter()
            .chain(SVELTE_PSEUDO_CLASSES)
            .map(|entry| pseudo_item(entry, range, ":-")),
    );
    items.extend(
        PSEUDO_ELEMENTS
            .iter()
            .map(|entry| pseudo_item(entry, range, "::-")),
    );
    items.extend(
        HTML5_TAGS
            .iter()
            .chain(SVG_ELEMENTS)
            .map(|name| CompletionItem {
                label: (*name).to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                text_edit: replace(range, *name),
                ..CompletionItem::default()
            }),
    );
    CompletionList {
        is_incomplete: false,
        items,
    }
}

fn css_prefix(text: &str, offset: usize) -> Option<&str> {
    let before = text.get(..offset)?;
    let in_style = style_body(text, offset) || static_style_value(text, offset);
    if !in_style {
        return None;
    }
    let start = before
        .char_indices()
        .rev()
        .find(|(_, c)| !matches!(c, 'a'..='z' | 'A'..='Z' | '-' | '_'))
        .map_or(0, |(index, c)| index + c.len_utf8());
    Some(&before[start..])
}

fn style_body(text: &str, offset: usize) -> bool {
    style_body_range(text, offset).is_some()
}

fn style_body_range(text: &str, offset: usize) -> Option<std::ops::Range<usize>> {
    let before = text.get(..offset.min(text.len()))?;
    let open = before.rfind("<style")?;
    let start = before[open..].find('>')? + open + 1;
    let end = text[start..]
        .find("</style")
        .map_or(text.len(), |index| start + index);
    (start <= offset && offset <= end).then_some(start..end)
}

fn static_style_value(text: &str, offset: usize) -> bool {
    let Some(before) = text.get(..offset.min(text.len())) else {
        return false;
    };
    let quote = before
        .rfind("style=\"")
        .map(|i| (i + 7, '"'))
        .or_else(|| before.rfind("style='").map(|i| (i + 7, '\'')));
    quote.is_some_and(|(start, quote)| !before[start..].contains(quote))
}

fn selector_completions(text: &str, marker: char, prefix: &str) -> CompletionList {
    let attribute = if marker == '.' { "class" } else { "id" };
    let mut names = std::collections::BTreeSet::new();
    for quote in ['\'', '"'] {
        let needle = format!("{attribute}={quote}");
        for (start, _) in text.match_indices(&needle) {
            if let Some(value) = text[start + needle.len()..].split(quote).next() {
                for name in value.split_ascii_whitespace() {
                    if name.starts_with(prefix) {
                        names.insert(name);
                    }
                }
            }
        }
    }
    CompletionList {
        is_incomplete: false,
        items: names
            .into_iter()
            .map(|name| CompletionItem {
                label: format!("{marker}{name}"),
                kind: Some(CompletionItemKind::REFERENCE),
                ..CompletionItem::default()
            })
            .collect(),
    }
}

/// `getIdClassCompletion.ts`: a `class=` / `id=` value, and the name after
/// `class:`, are completed from the selectors the component's own `<style>`
/// declares. Upstream neither deduplicates nor filters by what has been typed,
/// and labels the selector without its `.` or `#`.
#[must_use]
pub fn id_class_completions(text: &str, node_type: &str) -> CompletionList {
    // Upstream reads `document.stylesheet`, which is parsed from the extracted
    // style region alone — a completion is asked for while the markup around it
    // is half-typed, so parsing the whole component would answer nothing.
    let style = style_element(text).unwrap_or("");
    let allocator = rsvelte_core::Allocator::default();
    // `defer_script_parse` would also defer the CSS rules, and the rules are
    // exactly what is being collected here.
    let options = rsvelte_core::ParseOptions {
        skip_expression_loc: true,
        lenient_script: true,
        ..rsvelte_core::ParseOptions::default()
    };
    let mut items = Vec::new();
    if let Ok(root) = rsvelte_core::parse(style, &allocator, options)
        && let Some(css) = root.css.as_deref()
    {
        for child in &css.children {
            collect_selectors(child, node_type, &mut items);
        }
    }
    CompletionList {
        is_incomplete: false,
        items,
    }
}

/// The whole `<style …>…</style>` element, which on its own is a valid
/// component however broken the markup it was lifted out of is.
fn style_element(text: &str) -> Option<&str> {
    let open = text.find("<style")?;
    let end = text[open..].find("</style").and_then(|at| {
        text[open + at..]
            .find('>')
            .map(|close| open + at + close + 1)
    })?;
    Some(&text[open..end])
}

fn collect_selectors(node: &serde_json::Value, node_type: &str, items: &mut Vec<CompletionItem>) {
    if node.get("type").and_then(serde_json::Value::as_str) == Some(node_type)
        && let Some(name) = node.get("name").and_then(serde_json::Value::as_str)
    {
        items.push(CompletionItem {
            label: name.to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            ..CompletionItem::default()
        });
    }
    match node {
        serde_json::Value::Array(children) => {
            for child in children {
                collect_selectors(child, node_type, items);
            }
        }
        serde_json::Value::Object(fields) => {
            for value in fields.values() {
                collect_selectors(value, node_type, items);
            }
        }
        _ => {}
    }
}

fn word_at(text: &str, offset: usize) -> Option<&str> {
    let start = text[..offset.min(text.len())]
        .char_indices()
        .rev()
        .find(|(_, c)| !matches!(c, 'a'..='z' | 'A'..='Z' | '-'))
        .map_or(0, |(index, c)| index + c.len_utf8());
    let end = text[offset.min(text.len())..]
        .find(|c: char| !matches!(c, 'a'..='z' | 'A'..='Z' | '-'))
        .map_or(text.len(), |index| offset + index);
    text.get(start..end).filter(|word| !word.is_empty())
}

fn property_item(property: &str) -> CompletionItem {
    CompletionItem {
        label: property.to_string(),
        kind: Some(CompletionItemKind::PROPERTY),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: format!("`{property}` CSS property"),
        })),
        ..CompletionItem::default()
    }
}

fn values(prefix: &str) -> Vec<CompletionItem> {
    [
        "auto",
        "block",
        "contents",
        "flex",
        "grid",
        "inherit",
        "initial",
        "none",
        "revert",
        "transparent",
        "unset",
    ]
    .into_iter()
    .filter(|value| value.starts_with(prefix))
    .map(|value| CompletionItem {
        label: value.to_string(),
        kind: Some(CompletionItemKind::VALUE),
        ..CompletionItem::default()
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use lsp_types::Position;

    use super::*;

    fn items(text: &str) -> Vec<CompletionItem> {
        completions(text, text.len()).unwrap().items
    }

    fn labels(text: &str) -> Vec<String> {
        completions(text, text.len())
            .unwrap()
            .items
            .into_iter()
            .map(|item| item.label)
            .collect()
    }

    #[test]
    fn completes_properties_in_style_blocks_and_static_attributes() {
        assert!(labels("<style>a { colo").contains(&"color".to_string()));
        assert!(labels("<div style=\"colo").contains(&"color".to_string()));
    }

    #[test]
    fn completes_common_values_and_hovers_properties() {
        assert!(labels("<style>a { display: fl").contains(&"flex".to_string()));
        assert_eq!(
            hover("<style>a { color: red }</style>", 13).as_deref(),
            Some("`color` CSS property")
        );
    }

    #[test]
    fn reports_hex_colours_only_in_css() {
        let colors =
            colors("<script>const x = '#ffffff'</script><style>a { color: #123456 }</style>");
        assert_eq!(colors.len(), 1);
        assert_eq!(color_presentations(colors[0].color)[0].label, "#123456");
    }

    #[test]
    fn completes_template_classes_and_ids_in_selectors() {
        assert!(
            labels("<div class=\"button primary\" id=\"main\"></div><style>.but")
                .contains(&".button".to_string())
        );
        assert!(
            labels("<div class=\"button\" id=\"main\"></div><style>#ma")
                .contains(&"#main".to_string())
        );
    }

    #[test]
    fn completes_a_class_and_an_id_attribute_from_the_stylesheet() {
        // Upstream returns every matching selector, unfiltered and undeduplicated,
        // labelled without its `.` or `#` — so each half is asserted on its own.
        let source = "<div></div><style>.a{} .b{} .a{} #c{}</style>";
        assert_eq!(
            id_class_completions(source, "ClassSelector")
                .items
                .iter()
                .map(|item| item.label.clone())
                .collect::<Vec<_>>(),
            vec!["a", "b", "a"],
        );
        assert_eq!(
            id_class_completions(source, "IdSelector")
                .items
                .iter()
                .map(|item| item.label.clone())
                .collect::<Vec<_>>(),
            vec!["c"],
        );
    }

    #[test]
    fn completes_a_stylesheet_the_broken_markup_around_it_cannot_be_parsed_with() {
        // The position a completion is asked from — `id=` with no value yet —
        // is itself what stops the component parsing, so a whole-document parse
        // answers nothing here.
        assert_eq!(
            id_class_completions("<div id=></div><style>#abc{}</style>", "IdSelector")
                .items
                .iter()
                .map(|item| item.label.clone())
                .collect::<Vec<_>>(),
            vec!["abc"],
        );
    }

    #[test]
    fn completes_and_documents_global_selectors() {
        let global = items("<style>:glo")
            .into_iter()
            .find(|item| item.label == ":global()")
            .expect(":global() is not offered");
        assert_eq!(global.kind, Some(CompletionItemKind::FUNCTION));
        assert_eq!(global.insert_text_format, Some(InsertTextFormat::SNIPPET));
        assert_eq!(
            global.text_edit,
            replace(
                Range::new(Position::new(0, 7), Position::new(0, 11)),
                ":global($1)"
            )
        );
        assert!(global.sort_text.is_none());
        assert!(
            hover("<style>:global(.external) {}</style>", 14)
                .unwrap()
                .contains("prevents")
        );
    }

    /// Every number here was printed by `createLanguageServices().css`, the
    /// service the official server runs, on the same text without `<style>`.
    #[test]
    fn offers_the_selector_context_the_official_service_offers() {
        for (source, total, at_directives) in [
            ("<style>", 402, 25),
            ("<style> ", 402, 25),
            ("<style>\n", 402, 25),
            ("<style>@", 402, 25),
            ("<style>@med", 402, 25),
            ("<style>a{}\n", 402, 25),
            ("<style>a{} ", 402, 25),
            ("<style>a{}\n@", 402, 25),
            ("<style>/*c*/", 402, 25),
            ("<style>/*c*/@", 402, 25),
            ("<style>:", 377, 0),
            ("<style>:glo", 377, 0),
            ("<style>a", 377, 0),
            ("<style>a:", 377, 0),
            ("<style>a:glo", 377, 0),
            ("<style>a::", 377, 0),
            ("<style>a,", 377, 0),
            ("<style>a, ", 377, 0),
            ("<style>a >", 377, 0),
            ("<style>a > ", 377, 0),
            ("<style>a{}\n:glo", 377, 0),
        ] {
            let items = items(source);
            assert_eq!(items.len(), total, "{source:?}");
            assert_eq!(
                items
                    .iter()
                    .filter(|item| item.label.starts_with('@'))
                    .count(),
                at_directives,
                "{source:?}"
            );
            assert_eq!(
                items
                    .iter()
                    .filter(|item| item.kind == Some(CompletionItemKind::FUNCTION))
                    .count(),
                199,
                "{source:?}"
            );
            // Only the vendor-prefixed pseudo entries are sorted last, and the
            // property completion's `d_`/`x_` scheme reaches none of them.
            assert_eq!(
                items.iter().filter(|item| item.sort_text.is_some()).count(),
                81,
                "{source:?}"
            );
            assert!(
                items
                    .iter()
                    .all(|item| item.sort_text.is_none() || item.sort_text.as_deref() == Some("x")),
                "{source:?}"
            );
        }
    }

    /// The residue of the lexical stand-in for `getCompletionForTopLevel`, over a
    /// 270-cell grid of which 240 reach this path: 212 agree and these 28 do not.
    /// Four of the five clusters are the official parser falling into a different
    /// branch on a half-typed token; the fifth is a list rsvelte does not build.
    #[test]
    fn divergences_from_the_official_service_this_leaves() {
        for (source, official, ours) in [
            // A lone `-` leaves the official parser with no node before the
            // offset, so it answers as if the statement had not started (10 cells).
            ("<style>-", 402, 377),
            ("<style>a{} -", 402, 377),
            // The same after a `,`, where an empty token answers 377 (6 cells).
            ("<style>a,@", 402, 377),
            ("<style>a, -", 402, 377),
            // The official service also offers every class selector the sheet
            // itself declares, which rsvelte does not collect (10 cells).
            ("<style>.x ", 378, 377),
            // A cursor exactly on the closing brace is still inside the
            // declarations upstream, so it completes properties (1 cell).
            ("<style>a{}", 844, 402),
            // And nothing at all after a closed at-rule block (1 cell).
            ("<style>@media (x){}\n", 0, 402),
        ] {
            assert_ne!(
                official, ours,
                "{source:?} no longer diverges by construction"
            );
            assert_eq!(items(source).len(), ours, "{source:?}");
        }
    }

    #[test]
    fn tags_a_deprecated_pseudo_entry_and_leaves_an_element_name_untagged() {
        let items = items("<style>");
        let deprecated: Vec<&str> = items
            .iter()
            .filter(|item| item.tags.as_deref().is_some_and(|tags| !tags.is_empty()))
            .map(|item| item.label.as_str())
            .collect();
        assert_eq!(
            deprecated,
            vec![
                "::-moz-range-progress",
                "::-moz-range-thumb",
                "::-moz-range-track",
                "::-webkit-progress-inner-value",
            ]
        );
        // Upstream builds an element name from a bare string, so it carries no
        // `tags` at all where every data-derived entry carries an empty one.
        assert_eq!(items.iter().filter(|item| item.tags.is_none()).count(), 178);
    }

    #[test]
    fn reports_unknown_css_properties() {
        let diagnostics = diagnostics("<style>a { colro: red; --theme: blue }</style>");
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].code,
            Some(NumberOrString::String("css_unknown_property".to_string()))
        );
    }
}
