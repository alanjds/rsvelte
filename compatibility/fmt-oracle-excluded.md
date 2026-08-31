# fmt-oracle-excluded.json — why each id is excluded

Justification for every id permanently excluded from the formatter-parity gate
(`fmt-oracle-excluded.json`). Excluded ids are removed from the comparison set
entirely (neither matched nor failed). Each entry carries a `"class"`
(`oracle-bug` | `invalid-input` | `migrate` | `engine-divergence`) and a
`"reason"`; this file records the class-level rationale.

**Current baseline: `fmt-oracle-excluded.json`, 27 entries.**

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
| 5 | [`deliberate-divergences`](deliberate-divergences.md#the-formatter-declines-an-input-its-own-parser-rejects) | `invalid-input` and `migrate` — inputs no compiler accepts, and Svelte 4 migrator output |
| 5 | [`deliberate-divergences`](deliberate-divergences.md#a-formatter-difference-the-compiler-cannot-see) | both texts compile to byte-identical client and server `js` **and** `css` |
| 3 | [`deliberate-divergences`](deliberate-divergences.md#the-formatters-css-engine-is-oxc-not-prettiers-postcss) | rsvelte reproduces `oxfmt <file>.css` byte-for-byte; the oracle's Svelte path disagrees with oxfmt itself |
| 2 | [`upstream_issues/3035-prettier-plugin-svelte-drops-a-nested-pattern-key-in-each.md`](../upstream_issues/3035-prettier-plugin-svelte-drops-a-nested-pattern-key-in-each.md) | `oracle-bug` — the `{#each}` head drops a nested pattern's property key |
| 1 | [`upstream_issues/oxfmt-svelte-css-eats-a-css-escape-terminator-space.md`](../upstream_issues/oxfmt-svelte-css-eats-a-css-escape-terminator-space.md) | `oracle-bug` — a CSS escape's terminator space is eaten, and a live rule becomes dead |
| 3 | [`upstream_issues/oxfmt-svelte-css-keeps-source-tabs-around-a-selector-comment.md`](../upstream_issues/oxfmt-svelte-css-keeps-source-tabs-around-a-selector-comment.md) | `oracle-bug` — source tabs survive on a comment-bearing selector under `useTabs: false` |

**One entry carries no target: `shadcn-svelte/.../theme-customizer-code.svelte`, and its
recorded `reason` was falsified on 2026-08-31.** The JSON calls it an `oracle-bug` — "on macOS it
collapses the overflowing self-closing `<ColorIndicator color={value} />` inside `<pre>` to one
line, on Linux it attribute-wraps it … rsvelte deterministically matches the macOS form". Measured
on macOS with oxfmt 0.64.0, three runs each, both formatters byte-stable: **the oracle
attribute-wraps and rsvelte collapses** — the opposite assignment. So whatever the cross-platform
behaviour is, the sentence "rsvelte matches the macOS form" is wrong, and the entry is not
justified by the reason it carries.

What the divergence actually is, reduced to a 19-line repro that reproduces it:

```svelte
<pre class="…overflowing class list…">
  <code>
    {#each Object.entries(o?.light || {}) as [key, value] (key)}
      <span data-line class="line text-code-foreground"
        >&nbsp;&nbsp;&nbsp;--{key}: <ColorIndicator color={value} /> {value};</span
      >
    {/each}
  </code>
</pre>
```

The `<span>` line overflows `printWidth: 80` at that depth, so both formatters must break it:

| | break point | rendered text between `/>` and `{value}` |
|---|---|---|
| oracle | inside the `<ColorIndicator>` tag (`<ColorIndicator⏎ color={value}⏎/> {value};`) | the source's single space |
| rsvelte | at the text whitespace (`<ColorIndicator color={value} />⏎ {value};`) | a newline + 14 spaces |

Inside `<pre>` that whitespace is content, so rsvelte's choice changes what the page renders.
**The control that makes this rsvelte's defect rather than a shared one: replace the `<pre>` with a
`<div>` and the two outputs are byte-identical** — both then take rsvelte's break, which is safe
where whitespace collapses. prettier-plugin-svelte suppresses a text-position break under
`isPreTagContent`; `rsvelte_formatter` does not.

Compiled three ways with `submodules/svelte/…/compiler/index.js` (`css: 'external'`), the full
file separates into **two divergences, only one of which is shared**:

- **Shared, and not a parity failure:** source-vs-oracle differs on **28 server** and **8 client**
  lines, and **every one of them differs only in leading horizontal whitespace** — the
  `useTabs: false` reindentation of `<pre>` content, which both formatters perform and which does
  change the rendering. `css.code` is byte-identical on all three texts and both targets.
- **rsvelte-only, and the reason the entry exists:** the break above. It is why oracle-vs-rsvelte
  differ (server 552 → 570 lines, client 932 → 969), and the oracle side is the one that preserves
  the source's rendering.

**The DoD-4 answer is therefore "eliminate", not a citation.** There is a correct side, so this is
neither an `upstream_issues/` item nor a deliberate divergence — a pin would pin the wrong
behaviour. It also cannot be eliminated while it stays here: an excluded id is removed from the
comparison set entirely, so fixing the formatter can never show up as a match. The entry belongs
in the `fmt-known-failures.json` burndown that already owns 788 of these. `fmt-*` is Agent-C's
ratchet, so the JSON edit is theirs to make; this section records the measurement.

## Re-measured twice: **six reasons did not reproduce on 2026-08-30, and the 2026-08-31 pass closed nine of the ten**

On 2026-08-31 the ten entries that had carried no attribution target were run through
`scripts/compat-corpus/fmt-one.mjs` against the current `rsvelte-fmt`, and both texts of each were
compiled for `client` and `server` and compared on `js.code` **and** `css.code`:

- **`textarea-content` now matches the oracle byte-for-byte** (720 bytes on each side) and has been
  removed from the list. CI's Linux run reports the same for it and for `snippet-rest-args`
  (`[fmt-verify] NOTICE: excluded id now matches oracle`), which is also removed — 29 entries → 27.
- **Five compile to byte-identical output on all four comparisons** and are now recorded under
  [`deliberate-divergences`](deliberate-divergences.md#a-formatter-difference-the-compiler-cannot-see).
- **Three are the CSS engine**, and `rsvelte-fmt` reproduces `oxfmt <file>.css` byte-for-byte on
  every one, so the oracle is the same tool answering differently —
  [`deliberate-divergences`](deliberate-divergences.md#the-formatters-css-engine-is-oxc-not-prettiers-postcss).
- **One is left open**, for the reason stated above the table.

## The 2026-08-30 pass: **six recorded reasons do not reproduce**

Every `oracle-bug` entry was re-run through the pinned oracle (`oxfmt@0.64.0` with
`scripts/fixtures/fmt-corpus.oxfmtrc.json`, the same in-place invocation `fmt.mjs` uses), and
where the recorded reason claimed a *semantic* loss the two texts were additionally compiled with
`submodules/svelte/packages/svelte/src/compiler/index.js` and their outputs compared.

| entry | recorded reason | measured 2026-08-30 |
|---|---|---|
| `await-then-destruct-array-nested-rest` | drops nested rest → `...[...undefined]` | **does not reproduce** — `{:then [a, b, ...[, , c, ...{ length }]]}` is preserved |
| `block-expression-assign` | emits invalid `{@const x = (h = 0}` | **does not reproduce** — emits valid `{@const x = h = 0}`; it *adds* parens to `{#if a = 0}` |
| `textarea-content` | collapses whitespace-significant content | **does not reproduce** |
| `whitespace-after-script-tag` | reads an empty script and **loses the body** | **does not reproduce** — `let name = "world";` survives |
| `whitespace-after-style-tag` | loses `div { color: red; }` | **does not reproduce** — it survives |
| `parser-legacy/textarea-end-tag` | collapses whitespace the textarea renders | **does not reproduce as a semantic defect.** Trailing `</textarea` text and blank lines *are* deleted from the file, but both texts compile to a **byte-identical** `<textarea>` body — the deleted run is past where Svelte closes the element |
| `css/comment-html`, `comments-after-last-selector`, `parser-modern/css-pseudo-classes` | mixed tab/space indentation | **reproduces**, and it is one cause, now filed — see the table above |
| `css/unicode-identifier` | a cosmetic space before `{` | **reproduces, and is worse than recorded** — the escape-terminator collapse turns a used scoped rule into a pruned one; filed |
| `css/css-vars` | `--bar: !important;` gains a second space | **reproduces**; the compiled CSS differs only in that space |
| `adversarial/css/css-custom-property-values` | the same value formatted two ways | **reproduces**, and is cosmetic: `--sel: a > b ~ c` → `a > b ~c` (whitespace around a combinator is optional, so the selector is unchanged) and `url('/x.png')` → `url("/x.png")` |
| `shadcn-svelte/theme-customizer-code` | platform-dependent output | the **platform axis was not re-measured**; the output does carry 61 tab-bearing lines under `useTabs: false` |
| `svelte.dev/.../+layout.svelte` | `calc()` wrap position | **not re-verified in this pass** |

Two things this cost, and both generalize. **A reason can be stale in either direction**: five of
the six above overstate the defect, and `unicode-identifier` *under*states it — the entry was
filed as a space before a brace and is in fact a selector whose meaning changes. And **"the
formatter deleted text" is not "the output is wrong"**: `textarea-end-tag` reads as content loss
and compiles identically, which only a compile of both texts can tell you.

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
- **Cross-platform non-determinism.** oxfmt produces different output on macOS vs
  Linux for the same input (an overflowing self-closing component inside `<pre>` is
  collapsed on macOS, attribute-wrapped on Linux), so byte-parity is undefined. —
  `shadcn-svelte .../theme-customizer-code.svelte`.
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
