//! Where a rune-spelled token in a script is actually a rune.
//!
//! The module-script lowerings find their work with byte searches over raw
//! text, so anything spelled like a rune matched — including the slots where
//! the same spelling is a name rather than a reference: a member property
//! (`o.$derived(1)`), a class or object member (`class C { $derived(v) {} }`).
//! Upstream never has the question, because `get_rune` is handed a resolved
//! callee. Only an `IdentifierReference` can be a rune, and the parser already
//! separates those from every naming slot.
//!
//! Two filters, cheapest first. The previous significant code byte settles a
//! member *access* and an identifier tail with no parse at all; only a hit
//! whose neighbourhood also admits a member *name* is worth an AST.

use std::cell::RefCell;

use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_ast_visit::Visit;
use oxc_ast_visit::walk;
use oxc_parser::ParseOptions;
use oxc_span::SourceType;
use rustc_hash::FxHashSet;

use super::ast_rewrite;
use super::js_scan::{blocked_by_previous_token, skip_opaque};

thread_local! {
    static RUNE_SITES_ALLOC: RefCell<Allocator> = RefCell::new(Allocator::default());
}

/// The rune spellings the raw scans search for. `$bindable` is included because
/// it is a rune name even though no scan lowers it on its own.
const RUNE_NAMES: &[&str] = &[
    "$state",
    "$derived",
    "$effect",
    "$props",
    "$bindable",
    "$inspect",
    "$host",
];

/// A lazily built, text-keyed index of the rune-reference offsets in a script.
///
/// The scans it serves rewrite their own input, so the offsets move under them;
/// the cache is keyed by the text it was built from and rebuilt when that
/// changes. A script that never reaches the ambiguous case never parses.
pub(crate) struct RuneSites {
    is_ts: bool,
    cached: Option<(u64, Option<FxHashSet<u32>>)>,
}

impl RuneSites {
    pub(crate) fn new(is_ts: bool) -> Self {
        Self {
            is_ts,
            cached: None,
        }
    }

    /// The first occurrence of `needle` in `text` that is code, begins an
    /// identifier token, and is a rune reference rather than a member name.
    ///
    /// `needle` must start with the rune's `$`.
    pub(crate) fn find(&mut self, text: &str, needle: &[u8]) -> Option<usize> {
        let mut first = None;
        self.scan(text, needle, |at| {
            first = Some(at);
            false
        });
        first
    }

    /// Every such occurrence, ascending.
    pub(crate) fn positions(&mut self, text: &str, needle: &[u8]) -> Vec<usize> {
        let mut out = Vec::new();
        self.scan(text, needle, |at| {
            out.push(at);
            true
        });
        out
    }

    /// Walk the code occurrences of `needle`, calling `on_hit` for each rune
    /// reference until it returns `false`.
    fn scan(&mut self, text: &str, needle: &[u8], mut on_hit: impl FnMut(usize) -> bool) {
        let bytes = text.as_bytes();
        let mut candidates = memchr::memmem::find_iter(bytes, needle);
        let Some(mut candidate) = candidates.next() else {
            return;
        };
        let mut i = 0usize;
        let mut prev: Option<u8> = None;
        while i < bytes.len() {
            if let Some((next, is_comment)) = skip_opaque(bytes, i, prev) {
                while candidate < next {
                    match candidates.next() {
                        Some(next_candidate) => candidate = next_candidate,
                        None => return,
                    }
                }
                if !is_comment {
                    prev = Some(b'x');
                }
                i = next;
                continue;
            }
            if i == candidate {
                if !blocked_by_previous_token(prev)
                    && self.is_reference(text, candidate, prev)
                    && !on_hit(candidate)
                {
                    return;
                }
                match candidates.next() {
                    Some(next_candidate) => candidate = next_candidate,
                    None => return,
                }
            }
            if !bytes[i].is_ascii_whitespace() {
                prev = Some(bytes[i]);
            }
            i += 1;
        }
    }

    /// Is the token at `at` a rune reference? Answered from the AST only when
    /// `prev` leaves a member *name* possible — every other predecessor puts
    /// the token in expression position, where a name cannot appear.
    fn is_reference(&mut self, text: &str, at: usize, prev: Option<u8>) -> bool {
        if !can_precede_member_name(prev) {
            return true;
        }
        match self.offsets(text) {
            // An unparseable intermediate answers nothing; leave the scan as
            // permissive as it was before this filter existed.
            None => true,
            Some(offsets) => offsets.contains(&(at as u32)),
        }
    }

    fn offsets(&mut self, text: &str) -> Option<&FxHashSet<u32>> {
        let key = hash_text(text);
        if !matches!(self.cached, Some((cached_key, _)) if cached_key == key) {
            self.cached = Some((key, rune_reference_offsets(text, self.is_ts)));
        }
        self.cached.as_ref().and_then(|(_, set)| set.as_ref())
    }
}

/// Can a class / object member NAME start after `prev`? A member name follows
/// the body's `{`, a previous member's `}` / `;` / `,`, or a generator `*`;
/// `static` / `get` / `set` / `async` end in identifier bytes, which
/// [`blocked_by_previous_token`] already rejects.
fn can_precede_member_name(prev: Option<u8>) -> bool {
    matches!(prev, None | Some(b'{' | b'}' | b';' | b',' | b'*'))
}

fn hash_text(text: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = rustc_hash::FxHasher::default();
    text.hash(&mut hasher);
    hasher.finish()
}

/// Byte offsets at which a rune-spelled `IdentifierReference` starts.
///
/// `None` when `source` does not parse — the caller must not read that as
/// "no rune here".
fn rune_reference_offsets(source: &str, is_ts: bool) -> Option<FxHashSet<u32>> {
    ast_rewrite::with_program(
        &RUNE_SITES_ALLOC,
        source,
        if is_ts {
            SourceType::ts().with_module(true)
        } else {
            SourceType::mjs()
        },
        ParseOptions {
            allow_return_outside_function: true,
            ..ParseOptions::default()
        },
        |program| {
            let mut collector = RuneReferenceCollector {
                offsets: FxHashSet::default(),
            };
            collector.visit_program(program);
            Some(collector.offsets)
        },
    )
}

/// Rewrite every rune-spelled reference that the source spells with a unicode
/// escape to the plain name. Returns `None` when there is nothing to do.
///
/// A rune name may be written with a unicode escape, which denotes the same
/// identifier to a parser, and upstream lowers it because it reads the
/// parser's name. Every
/// raw scan downstream compares source bytes, so the module path left the rune
/// in the generated code and the module threw on import (#3236). Normalizing the
/// spelling once is what makes those scans see the identifier the parser built;
/// the escape carries no other meaning, since the name is what the value is
/// bound to.
pub(crate) fn normalize_rune_spellings(source: &str, is_ts: bool) -> Option<String> {
    memchr::memmem::find(source.as_bytes(), br"\u")?;
    ast_rewrite::rewrite_once(
        &RUNE_SITES_ALLOC,
        source,
        if is_ts {
            SourceType::ts().with_module(true)
        } else {
            SourceType::mjs()
        },
        ParseOptions {
            allow_return_outside_function: true,
            ..ParseOptions::default()
        },
        false,
        |program| {
            let mut collector = EscapedRuneCollector {
                source,
                edits: Vec::new(),
            };
            collector.visit_program(program);
            collector.edits
        },
    )
}

struct EscapedRuneCollector<'s> {
    source: &'s str,
    edits: Vec<ast_rewrite::Edit>,
}

impl<'a> Visit<'a> for EscapedRuneCollector<'_> {
    fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
        walk::walk_identifier_reference(self, it);
        let name = it.name.as_str();
        if !RUNE_NAMES.contains(&name) {
            return;
        }
        let (start, end) = (it.span.start as usize, it.span.end as usize);
        if self.source.get(start..end) != Some(name) {
            self.edits
                .push((it.span.start, it.span.end, name.to_string()));
        }
    }
}

struct RuneReferenceCollector {
    offsets: FxHashSet<u32>,
}

impl<'a> Visit<'a> for RuneReferenceCollector {
    fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
        walk::walk_identifier_reference(self, it);
        if RUNE_NAMES.contains(&it.name.as_str()) {
            self.offsets.insert(it.span.start);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn find(src: &str, needle: &str) -> Option<usize> {
        RuneSites::new(false).find(src, needle.as_bytes())
    }

    #[test]
    fn a_plain_call_is_a_rune() {
        assert_eq!(find("let a = $state(1);", "$state("), Some(8));
        assert_eq!(find("let d = $derived(a);", "$derived("), Some(8));
    }

    #[test]
    fn a_statement_position_call_is_a_rune() {
        assert_eq!(find("$effect(() => {});", "$effect("), Some(0));
        assert_eq!(
            find("function f() {\n\t$effect(() => {});\n}", "$effect("),
            Some(16)
        );
    }

    #[test]
    fn a_member_property_is_not_a_rune() {
        assert!(
            find(
                "const o = { $derived: (v) => v };\no.$derived(1);",
                "$derived("
            )
            .is_none()
        );
        assert!(find("o?.$derived(1);", "$derived(").is_none());
        assert!(find("o.p.$derived(1);", "$derived(").is_none());
        assert!(find("o\n\t.$derived(1);", "$derived(").is_none());
        assert!(find("this.$derived(1);", "$derived(").is_none());
    }

    #[test]
    fn a_class_member_name_is_not_a_rune() {
        assert!(find("class C {\n\t$derived(v) { return v; }\n}", "$derived(").is_none());
        assert!(find("class C {\n\tstatic $state(v) { return v; }\n}", "$state(").is_none());
        assert!(find("class C {\n\t*$state(v) { yield v; }\n}", "$state(").is_none());
    }

    #[test]
    fn an_object_member_name_is_not_a_rune() {
        assert!(find("const o = {\n\t$derived(v) { return v; }\n};", "$derived(").is_none());
    }

    #[test]
    fn a_longer_identifier_is_not_a_rune() {
        assert!(find("let a = my$state(1);", "$state(").is_none());
    }

    #[test]
    fn an_opaque_carrier_is_not_a_rune() {
        assert!(find("const c = '$state(';", "$state(").is_none());
        assert!(find("// $state(1)\n", "$state(").is_none());
    }

    /// A real call after a decoy still has to be found.
    #[test]
    fn a_decoy_does_not_hide_the_real_call() {
        let src = "const o = { $derived: (v) => v };\no.$derived(1);\nlet d = $derived(1);";
        let at = find(src, "$derived(").expect("the real call");
        assert_eq!(&src[at..at + 9], "$derived(");
        assert!(at > src.find("o.$derived").unwrap());
    }

    /// An unparseable intermediate must leave the scan exactly as it was.
    #[test]
    fn an_unparseable_intermediate_reports_the_hit() {
        assert_eq!(find("{ $effect(() => {}); ", "$effect("), Some(2));
    }
}
