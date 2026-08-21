/// prettier-plugin-svelte's `class`-attribute text normalisation, ported from
/// its `Text` printer (`parent.name === 'class' && path.getParentNode(1).type
/// === 'RegularElement'`). Every other attribute — and `class` on a component
/// or a `<svelte:element>` — prints its text verbatim.
///
/// Two passes over the raw text, mirroring the two `String.replace` calls:
///
/// 1. `/([^ \t\n])(([ \t]+$)|([ \t]+(\r?\n))|[ \t]+)/g` — a run of spaces/tabs
///    that follows a non-whitespace character collapses to one space, or to
///    nothing when a line break follows, and is left alone when it ends the
///    string (pass 2 owns that case).
/// 2. `/([^ \t\n])[ \t]+$/` — the run that ends the string is dropped when this
///    text node is the value's last part and collapsed to one space otherwise
///    (something — a mustache — follows it on the value).
///
/// Leading whitespace has no preceding non-whitespace character, so it is never
/// touched; neither is whitespace that only a newline precedes, which is what
/// keeps a multi-line `class` value's own indentation.
pub(super) fn normalize_class_text(raw: &str, is_last_part: bool) -> String {
    let pass1 = collapse_inner_runs(raw);
    trim_trailing_run(&pass1, is_last_part)
}

fn is_space_or_tab(byte: u8) -> bool {
    byte == b' ' || byte == b'\t'
}

fn collapse_inner_runs(raw: &str) -> String {
    let bytes = raw.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(raw.len());
    let mut i = 0;
    while i < bytes.len() {
        let byte = bytes[i];
        out.push(byte);
        i += 1;
        // The regex's first group is `[^ \t\n]`, which a `\r` and every byte of
        // a multi-byte character satisfy — all are copied verbatim above, so
        // only the run that follows one matters here.
        if is_space_or_tab(byte) || byte == b'\n' {
            continue;
        }
        let run_start = i;
        while i < bytes.len() && is_space_or_tab(bytes[i]) {
            i += 1;
        }
        if i == run_start {
            continue;
        }
        if i == bytes.len() {
            // `[ \t]+$` — kept verbatim; pass 2 decides what happens to it.
            out.extend_from_slice(&bytes[run_start..i]);
        } else if bytes[i] == b'\n' {
            out.push(b'\n');
            i += 1;
        } else if bytes[i] == b'\r' && bytes.get(i + 1) == Some(&b'\n') {
            out.extend_from_slice(b"\r\n");
            i += 2;
        } else {
            out.push(b' ');
        }
    }
    String::from_utf8(out).unwrap_or_else(|_| raw.to_string())
}

fn trim_trailing_run(text: &str, is_last_part: bool) -> String {
    let bytes = text.as_bytes();
    let mut start = bytes.len();
    while start > 0 && is_space_or_tab(bytes[start - 1]) {
        start -= 1;
    }
    if start == bytes.len() || start == 0 {
        return text.to_string();
    }
    let before = bytes[start - 1];
    if is_space_or_tab(before) || before == b'\n' {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len());
    out.push_str(&text[..start]);
    if !is_last_part {
        out.push(' ');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::normalize_class_text;

    #[test]
    fn collapses_runs_between_words() {
        assert_eq!(normalize_class_text("a  b   c", true), "a b c");
        assert_eq!(normalize_class_text("a\tb\t\tc", true), "a b c");
    }

    #[test]
    fn keeps_leading_whitespace_and_drops_the_trailing_run() {
        assert_eq!(
            normalize_class_text("  lead and trail  ", true),
            "  lead and trail"
        );
    }

    #[test]
    fn a_non_final_part_keeps_one_trailing_space() {
        // `class="tail  {x}"` — a mustache follows, so the run collapses
        // instead of vanishing.
        assert_eq!(normalize_class_text("tail  ", false), "tail ");
    }

    #[test]
    fn drops_trailing_whitespace_on_a_continued_line() {
        assert_eq!(normalize_class_text("a  \n  b", true), "a\n  b");
        assert_eq!(normalize_class_text("a  \r\n  b", true), "a\r\n  b");
    }

    #[test]
    fn leaves_a_whitespace_only_value_alone() {
        assert_eq!(normalize_class_text("   ", true), "   ");
        assert_eq!(normalize_class_text("\n  ", true), "\n  ");
    }
}
