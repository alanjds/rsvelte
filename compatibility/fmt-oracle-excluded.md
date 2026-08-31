# fmt-oracle-excluded.json — why each id is excluded

Justification for every id permanently excluded from the formatter-parity gate
(`fmt-oracle-excluded.json`). Excluded ids are removed from the comparison set
entirely (neither matched nor failed). Each entry carries a `"class"`
(`oracle-bug` | `invalid-input` | `migrate` | `engine-divergence`) and a
`"reason"`; this file records the class-level rationale.

**Current baseline: `fmt-oracle-excluded.json`, 28 entries.**

`fmt-verify.mjs` warns if an excluded id is no longer in the parity set (can be
deleted) and notices if an excluded id now matches byte-for-byte (the oracle bug
was fixed upstream, or rsvelte was wrongly changed to reproduce it — avoid the
latter).


### DoD-4 attribution

Attribution of `fmt-oracle-excluded.json`:

| n | target | cluster |
|---|---|---|
| 3 | [`deliberate-divergences`](deliberate-divergences.md#a-props-line-comment-keeps-the-separator-slot-the-compiler-reads) | the `$props()` comment slot the #3515 repros depend on |
| 4 | [`deliberate-divergences`](deliberate-divergences.md#the-formatters-javascript-engine-is-oxc-not-prettier) | `engine-divergence` — oxc's line-breaking, not prettier's |
| 6 | [`deliberate-divergences`](deliberate-divergences.md#the-formatter-declines-an-input-its-own-parser-rejects) | `invalid-input` and `migrate` — inputs no compiler accepts, and Svelte 4 migrator output |
| 2 | [`upstream_issues/3035-prettier-plugin-svelte-drops-a-nested-pattern-key-in-each.md`](../upstream_issues/3035-prettier-plugin-svelte-drops-a-nested-pattern-key-in-each.md) | `oracle-bug` — the `{#each}` head drops a nested pattern's property key |

The remaining **13** `oracle-bug` entries are **deliberately absent from this table**. Listing
them against a report that does not cover them would make the gate green on a citation, which is
the failure mode a citation is most likely to hide. Each gets a row when its own report is filed
with both outputs measured.

**Two facts about this set were measured on 2026-08-30 and neither was known when it was
written.** Re-running the pinned oracle (`oxfmt@0.64.0`, `fmt-corpus.oxfmtrc.json`) over all
sixteen and feeding each result back to `svelte@5.56.10`'s `parse({modern: true})`:

- **Exactly 2 of 16 still produce text the official compiler rejects** — the two
  `{#each}` nested-pattern files above, one cause, now filed. The other 14 produce
  output that *parses*, which is not the same as output that is correct: the recorded
  defects there are semantic (a dropped variable, collapsed whitespace-significant
  `<textarea>` content) or cosmetic (indentation), and **the parse oracle cannot see
  either class**. Read the 2 as "confirmed by an instrument", not the 14 as "cleared".
- **At least one stated reason no longer reproduces.**
  `runtime-legacy/samples/block-expression-assign/main.svelte` is recorded as "oxfmt drops
  the closing paren in `{@const x = (h = 0)}`, producing `{@const x = (h = 0}` — invalid".
  Under 0.64.0 the output is `{@const x = h = 0}`, which parses and is semantically
  identical (`=` is right-associative). Whether the entry would now *match* rsvelte-fmt
  byte-for-byte — and so should be deleted rather than re-worded — is unmeasured; it needs
  a built `rsvelte-fmt`.

**A second stated reason was falsified on 2026-08-31, and that entry left this file.**
`shadcn-svelte/docs/src/lib/components/theme-customizer-code.svelte` was excluded as
`oracle-bug` for "cross-platform non-determinism": the overflowing self-closing
`<ColorIndicator color={value} />` inside `<pre>` was recorded as *collapsed on macOS,
attribute-wrapped on Linux*, with rsvelte-fmt matching the macOS form — so byte-parity
was declared undefined. Re-measured on macOS with the pinned oracle (`oxfmt@0.64.0`,
`fmt-corpus.oxfmtrc.json`, run over the real corpus source), the oracle emits the
**attribute-wrapped** form — the one the reason ascribes to Linux — at all 20
`<ColorIndicator>` sites, byte-identically on 5 consecutive runs:

```
oracle (macOS)   >&nbsp;&nbsp;&nbsp;--{key}: <ColorIndicator
                   color={value}
                 /> {value};</span
rsvelte-fmt      >&nbsp;&nbsp;&nbsp;--{key}: <ColorIndicator color={value} />
                 {value};</span
```

The two platform descriptions now coincide, so nothing is left of the non-determinism
claim; what remains is an ordinary rsvelte-fmt line-breaking divergence, which belongs in
`fmt-known-failures.json` and is now there. **The ratchet growing by one is a
reclassification, not a regression** — this pair has always differed, it was merely
unobserved. Two controls from the same repository (`announcement.svelte`,
`block-viewer-code.svelte`) were run through the same staged invocation and came out
byte-identical, so the harness can produce a match. What is *not* measured is the Linux
oracle: that needs CI. If the Formatter-parity job reports this id as already passing, the
right correction is to delete it from both files, not to restore the exclusion.

**This exclusion list is permanent, so nothing re-checks it.** The ratchets are two-sided and a
listed entry that starts passing fails CI; an *exclusion* has no such pressure, and its
justification was written against whatever oxfmt version was installed that day. `fmt-verify.mjs`
warns when an excluded id matches byte-for-byte, which catches the strongest case and not this
one: a reason can go stale while the pair still differs.

## oracle-bug — the `oxfmt(svelte:true)` oracle output is itself wrong/corrupt

Matching the oracle would require rsvelte to emit broken output. rsvelte-fmt is
correct; file upstream at `oxformatter/oxfmt` or `prettier/prettier-plugin-svelte`.

- **Nested-rest destructuring dropped → `...undefined`.** `{#each a as [x, y, ...[z, ...{n}]]}`
  is mangled to `[x, y, ...undefined]`, silently erasing `z`/`n`/`length` (source
  corruption). — `each-block-destructured-array-nested-rest`,
  `await-then-destruct-array-nested-rest`.
- **`{@const x = (h = 0)}` closing paren dropped** → `{@const x = (h = 0}`, invalid
  Svelte. — `block-expression-assign`.
- **Nested object destructure with a default loses its key.** In an `{#each}`
  context, `{ id, meta: { tags: […] } = {} }` is emitted as
  `{ id, { tags: … } = { } }` — the `meta:` key vanishes and the output is not
  JavaScript. — `pattern/issues/3035-destructure-defaults`,
  `pattern/adversarial/control-flow/each-destructure-exotic`.
- **`<textarea>` whitespace collapse.** Whitespace-significant body (`\n  A\n  B\n`)
  collapsed to `A B`, with inconsistent per-case rules. — `textarea-content`,
  `textarea-end-tag` (adversarial split close-tags).
- **CSS selector-list indentation mixes tabs and spaces.** Inline comments cause
  raw tab characters to leak into continuation lines while the body uses 2 spaces
  (non-idempotent). — `comment-html`, `comments-after-last-selector`,
  `css-pseudo-classes` (`:is()` inner selectors tab-indented).
- **Malformed `</script  >` / `</style  >` close tag loses body.** Whitespace
  before `>` makes prettier-plugin-svelte treat the block as empty and discard its
  content. — `whitespace-after-script-tag`, `whitespace-after-style-tag`.
- **`--svelte` CSS path defects.** Double-spaces an empty custom-property value
  (`css-vars`); emits a single space before `{` after an escaped-unicode selector
  (`unicode-identifier`); wraps a deeply-nested `calc(...)` differently
  (`svelte.dev .../docs/[topic]/[...path]/+layout.svelte`).
- **oxfmt formats embedded CSS differently from standalone CSS.** For
  `--arr: [1, 2]` / `--sel: a > b ~ c`, `oxfmt x.css` prints `[1 , 2]` /
  `a > b ~ c` while `oxfmt --svelte` prints `[1, 2]` / `a > b ~c` — the same tool
  disagreeing with itself, because the svelte path uses prettier's CSS printer
  and the `.css` path the oxc engine. rsvelte-fmt reproduces oxfmt's own `.css`
  output byte-for-byte, so parity against the svelte path is undefined here (and
  the svelte path's `~c` changes the token stream the value substitutes). —
  `pattern/adversarial/css/css-custom-property-values`.
- **Nested object-destructure default in `{#each}` loses its key.**
  `{#each xs as { id, meta: { tags: [t = 'x'] } = {} }}` is mangled to
  `{ id, { tags: [t = 'x'] } = { } }` — the `meta:` property key is dropped,
  which is not JavaScript. — `pattern/issues/3035-destructure-defaults.svelte`,
  `pattern/adversarial/control-flow/each-destructure-exotic.svelte`.

## invalid-input — the input is invalid and rsvelte correctly rejects it

- **Snippet optional param with initializer** — `{#snippet c5(c?: number = 5)}` is
  illegal TypeScript (TS1015: a parameter cannot have both `?` and `= …`); oxc
  correctly rejects. — `snippet-typescript`.
- **Snippet rest parameter** — snippets do not support rest params
  (`snippet_invalid_rest_parameter`); rsvelte-fmt correctly rejects. —
  `snippet-rest-args`.
- **Genuinely-invalid Svelte-specific CSS** — a parser-modern edge `<style>` block
  with invalid `:nth` syntax. — `css-nth-syntax`.
- **At-rule inside `:global()`** — `:global(@keyframes shared)` is rejected by both
  compilers (`css_expected_identifier`, #3120); rsvelte-fmt leaves a stylesheet its
  parser rejects untouched. — `rejected-global-keyframes-selector`.

## migrate — Svelte 4→5 migrator output (out of scope per AGENTS.md)

Svelte-4 syntax (legacy `let:` directives, `slot=` attributes) that rsvelte's
Svelte-5 compiler formats differently. — `migrate/samples/slot-non-identifier/output.svelte`,
`migrate/samples/slot-usages/output.svelte`.

## engine-divergence — oxc vs prettier JS layout, both valid

Not oracle bugs and not rsvelte bugs: rsvelte formats embedded JS with the
`oxc_formatter` crate (a deliberate design choice for the 100x-perf / oxc
integration goals), which makes different-but-valid line-break choices than the
oracle's prettier-based JS printer. Reproducing them would mean abandoning oxc or
fragile prettier-mimicking string surgery (forbidden). The long-term fix is
aligning `oxc_formatter`'s break heuristics with prettier upstream.

- Ternary-condition break granularity in a long `class=` (`flowbite TimelineColor`).
- IIFE arrow parameter-list vs call-argument break point (`flowbite GitHubSourceList`).
- Template-literal `${}` substitution indentation inside `<script>` (`flowbite range/+page`).
- Member-chain-only vs `&&`/call-args break priority in an `{#if}` header
  (`flowbite forms/tags/Tags`).
- Line-comment attachment between a destructuring assignment and its initializer:
  prettier keeps `= // comment\n $props()` in that separator slot (and inserts a
  blank line), while oxc emits `= $props(); // comment`. Both are valid, but the
  original slot is deliberately retained in the three `3515-props-*-line-comment`
  pattern fixtures because it distinguishes the compiler comment-cursor defect.
