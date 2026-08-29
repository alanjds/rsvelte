# sourcemap-known-failures.json — why each entry is accepted

The source-map gate (`crates/rsvelte_core/tests/sourcemaps_gate.rs`) runs the 29
official `packages/svelte/tests/sourcemaps` samples through rsvelte and checks
the resulting `js.map` / `css.map`. Ground truth is the official compiler: the
`client.js` / `client.js.map` / `server.js` / `server.js.map` fixtures under
`fixtures/<sha>/sourcemaps/` come from `scripts/fixtures/generate-fixtures.mjs`
calling `submodules/svelte`'s own `compile()` on the same input with the same
options (`{ dev: false, generate, filename: 'input.svelte' }` — the gate asserts
each sample's recorded `metadata.json` still says exactly that).

| kind | id shape | meaning |
|---|---|---|
| `anchor` | `anchor\t<sample>\t<target>\t<index>\t<str>` | an official `_config.js` `client:` / `server:` / `css:` expectation that rsvelte's map does not satisfy |
| `map-parity` | `map-parity\t<sample>\t<target>\t<count>` | budget: official map segments that rsvelte does not reproduce, where the generated code is byte-identical (missing + wrong) |
| `out-of-range` | `out-of-range\t<sample>\t<target>\t<count>` | budget: out-of-range segments not also emitted by the official map at the same generated and original position |

**Current baseline: `sourcemap-known-failures.json`, 0 entries.** The
before/after tables further down record what one specific change did at the time
it landed; they are history, not the current size. Reading the newest number in
those tables as today's count is the mistake this line exists to prevent — the
`73` under the anchoring fix was correct when written (#2264 took the list 75 →
73), #2312 later took it to 74, and the location-less comment cursor brought it
back to 73.

Ratchet semantics, matching `fmt-verify.mjs` / `verify.mjs`:

- an `anchor` id **not** in this list fails CI;
- a `map-parity` / `out-of-range` count **above** its recorded budget fails CI;
- an entry that starts passing (or a count below its budget) only prints a
  reminder to shrink the list — the list may shrink, never grow.

Two things deliberately **cannot** be expressed as a known failure, because
"measured less" must never look like "passed":

- a budgeted `<sample>/<target>` that disappears from the measurement is a
  regression, not a win;
- an `anchor` id in this list whose entry no longer exists in the test's
  `ANCHORS` table is a regression, so anchors cannot be deleted to go green.

On top of that the gate holds hard floors — sample count, anchor count, and the
number of byte-identical outputs `map-parity` can observe — and panics rather
than skipping when a sample's `input.svelte` or `metadata.json` is unreadable.

Regenerate the whole list from a measurement (never hand-edit the counts):

```bash
UPDATE_SOURCEMAP_RATCHET=1 cargo test -p rsvelte_core --test sourcemaps_gate -- \
  --ignored --nocapture sourcemap_gate_measure
```

## After a Svelte bump

The four constants at the top of `sourcemaps_gate.rs` are the only things a bump
can touch beyond the ratchet itself. Raise a floor only *after* a measurement
justifies it — never to make a red run go green.

- **Upstream adds samples.** Nothing to do. The floors are `>=` lower bounds, so
  they stay satisfied, and a new sample has no ratchet entry — any failure it
  brings is correctly reported as a regression. Once it is triaged, regenerate
  the ratchet and raise `EXPECTED_SAMPLES` / `EXPECTED_ANCHOR_COUNT` /
  `EXPECTED_IDENTICAL_OUTPUTS` to the new measured values in the same commit.
- **Upstream removes or renames samples.** A floor trips, or `load_input`
  panics. That is the intended outcome — confirm against the upstream diff that
  the sample really is gone, then lower the floor and drop its ratchet entries.
  Never lower a floor without that confirmation: an unreadable sample and a
  deleted one look identical from here, and the first is a broken checkout.
- **Upstream adds a sourcemaps `_config.js` that the fixture generator can
  import.** `check_fixture_options` fails with "the comparison would be
  meaningless". This is a benign cause with a loud symptom: the generator now
  compiles that sample with options this test does not use, so the oracle and
  rsvelte are no longer comparable. Either teach `compile_sample` the same
  options, or exclude the sample — do not relax
  `EXPECTED_FIXTURE_COMPILE_OPTIONS` to paper over the divergence.
- **Anchors.** `_config.js` expectations are copied by hand into `ANCHORS`;
  re-read the changed ones on a bump, since nothing detects an upstream
  expectation that silently changed value.

## Baseline at the time this gate was added

Measured on Svelte `b29d7002ecf9`, 29 samples × {client, server} (55 of the 58
pairs are byte-identical to the official output, so 55 take part in
`map-parity`):

| measure | client | server | total |
|---|---|---|---|
| official segments reproduced | 0 / 480 | 164 / 284 | **164 / 764 (21.5%)** |
| — of which missing / wrong | 393 / 87 | 113 / 7 | 506 / 94 |
| out-of-range segments | 37 | 0 | **37 / 545 (6.8%)** |
| ported `_config.js` anchors passing | 0 / 12 | 9 / 10 | **10 / 23** (incl. 1 CSS) |

The split is nearly, but not entirely, along the client/server line:

- **Client maps reproduce nothing.** Every client sample scores `0 exact` — not
  one segment of the official client map is reproduced at the same generated
  position with the same origin — all 12 client anchors fail, and all 37
  out-of-range segments are client.
- **Server maps are directionally correct but coarser than official.** 164 of
  284 official server segments are reproduced exactly and no server map has an
  out-of-range segment, but 113 are *missing* (the official compiler emits
  segments rsvelte's printer does not) and 7 are *wrong* (in
  `preprocessed-styles` and `source-map-generator`). One server anchor fails:
  `sourcemap-empty-source` has no segment at the start of `let doubled`. So
  "the server side is fine" would be an overstatement — server is where the
  burndown is tractable, not where it is finished.
- The single CSS anchor passes: CSS maps come from a separate
  `string_wizard`-based path that the client JS refactor does not touch.

### First catch: #1772

The baseline above was re-measured when this branch was rebased onto a main that
had gained #1772 ("keep `<script>` comments on the direct-AST codegen path"),
and the gate moved. The delta is confined to the two sourcemaps samples that
have a `//` comment inside `<script>` — exactly the files #1772 switches from
the text generator to the direct-AST path:

| | before #1772 | after |
|---|---|---|
| `typescript` client — byte-identical to official | no | **yes** |
| `typescript` client — official segments reproduced | not measured | 0 / 52 (40 missing, 12 wrong) |
| `typescript` client — out-of-range | 0 | **4** |
| `sourcemap-offsets` client — out-of-range | 0 | **1** |

Both directions in one change: generated-code parity *improved* (54 → 55
byte-identical, which is why `typescript` newly qualifies for `map-parity` at
all), while map quality *regressed* (0 → 5 new out-of-range segments). Server
totals are byte-for-byte unchanged, confirming the change is client-only.

This is the degradation issue #1781 describes, and it is the reason this gate
exists: the same change passed every other suite. No other sample's counts
moved, so nothing else on main has touched source maps.

### Second catch: #1784

Same shape as the #1772 entry above. Fixing #1784 (a trailing
`<script>` comment now flushes at the next node upstream gives a location, not
at the end of the function body) made `sourcemap-offsets` client output
byte-identical to the official compiler for the first time, so it newly
qualifies for `map-parity` and reports its resolution loss: 8 official segments,
0 reproduced.

| | before #1784 | after |
|---|---|---|
| `sourcemap-offsets` client — byte-identical to official | no | **yes** |
| `sourcemap-offsets` client — official segments reproduced | not measured | 0 / 8 (8 missing, 0 wrong) |

`EXPECTED_IDENTICAL_OUTPUTS` rises 55 → 56 in the same commit. Nothing else
moved: no anchor changed, and no existing budget grew.

### Third catch: instance-script chunk anchor

The instance script chunk was anchored at `ScriptContent::start` — the byte
immediately after `<script>`, i.e. the newline ending that line. Every segment
derived from it therefore resolved to a column past the end of the `<script>`
line. Anchoring the chunk at the script's first non-whitespace byte instead
halved `out-of-range` and produced the first non-zero client `exact` count this
gate has ever recorded; generated code is unchanged (the offset only feeds the
map).

| | before | after |
|---|---|---|
| client `out-of-range` segments | 37 | **19** |
| samples with an `out-of-range` budget | 16 | **14** |
| client official segments reproduced | 0 / 488 | **9 / 488** |
| client `wrong` segments | 81 | **72** |
| ratchet entries | 75 | **73** |

### Fourth catch: location-less comment cursor

Marking synthesized client nodes as location-less removes the last
`sourcemap-offsets` client segment whose origin pointed past its source line.
Generated output and the sample's `map-parity` budget are unchanged.

| | before | after |
|---|---|---|
| `sourcemap-offsets` client — out-of-range | 1 | **0** |
| ratchet entries | 74 | **73** |

## Root cause

The client entries all shared one cause, tracked in issue #1781: the client AST
output path mapped an entire emitted *chunk* to the one source offset the chunk
started at (`js_ast/to_oxc.rs::take_chunk_region`), and the printer's column
arithmetic then accumulated on top of that single anchor. Individual nodes inside
a chunk lost their own provenance, which produced both symptoms at once —
segments that no longer existed (`missing`, the resolution loss) and segments
that addressed a column past the end of the anchor's line (`out-of-range`).

Two findings from the #1781 burndown sharpened this. First, the official map's
segments are overwhelmingly *identifier and literal* start/end pairs, emitted by
esrap's `Context.write(content, node)`; `rsvelte_esrap` only emitted anchors from
`Printer::write_source_keyword`, so it had none of them and reproduced 0 / 488
client segments. Second, adding those anchors did not help on its own: a
comment-free chunk is parsed in place (`to_oxc.rs::parse_chunk`), so its node
spans are *chunk-local* byte offsets that the printer then read as offsets into
the original `.svelte` file. Chunk-local offsets and real source offsets share
one number space with nothing to tell them apart, so per-node anchors resolved to
unrelated positions.

Both halves are now fixed. `Printer::write_node` ports esrap's
`Context.write(content, node)` — every source-backed identifier, literal, member
property and block brace is bracketed by anchors for its own span — and the
spans reaching it are real source offsets, carried through client and SSR
lowering rather than reconstructed from a chunk. That took the gate from 73
entries to 3, with the `anchor` and `out-of-range` classes eliminated entirely.

### Fifth catch: the empty baseline was never a measurement

#3896 replaced a 3-entry list with `[]` in the same commit that made parity pair
duplicate generated columns by occurrence. That pairing is right for `effects`
(server), whose two official segments at one column rsvelte reproduces in order,
but on its own it also reports a *redundant* official duplicate — the same
segment emitted twice at one column, which `basic`'s `let foo = $.prop(…)` line
carries — as a segment rsvelte failed to reproduce. Measured on #3896's own base
(`b734a16ac`, its `baseRefOid`), that commit's gate scores **47 missing, 7 wrong
over 33 ratchet keys**, so `[]` describes no tree the comparison has ever run on
— and no tree it *could* have run on, because matching a redundant duplicate
would mean reproducing the official map byte for byte.

Nothing caught it because the CI runs for #3896 and its three successors are all
`cancelled`. This is the worked example of the rule in `CLAUDE.md`: **a cancelled
run and a green run are indistinguishable in the branch header**, so a ratchet
merged behind one has never been checked against anything. The gate stayed red on
`main` for the 145 commits that followed, and the failure list at `main` is
identical to the one on a branch cut from it — which is how a branch inherits a
regression that reads as its own.

Both defects of the comparison are now fixed together. `counterpart` still pairs
by occurrence — an extra *leading* rsvelte segment shifts every occurrence and is
still `wrong`, which the unit test pins — but when official has more segments at
a column than rsvelte does, an exact match anywhere at that column satisfies the
surplus one. A redundant duplicate resolves to the same original position for
every consumer, so reproducing it once is reproducing it.

Two compiler defects were behind the rest, both found by comparing against the
official fixture maps rather than against the ratchet:

- **The `bind:` element identifier started at `<`.** Upstream stamps
  `element.name_loc` on the identifier it reuses for the declaration and every
  runtime use, so `$.bind_value(input, …)` maps to the *tag name*;
  `bind_directive.rs` spanned it from `element.start`. The sibling site
  (`$.remove_input_defaults`) already used `element.start + 1`, and the
  `--lib` unit test had pinned the wrong column rather than the fixture's.
- **A source without a trailing `;` lost its whole statement span.** The
  generated terminator has no copied byte behind it, so
  `RestoreRawMappedSpans::source_end_offset` could not map the statement's end
  and `visit_span` dropped the span entirely — leaving the statement in chunk
  coordinates, where it resolves to offset 0. `export let foo = 5` and
  `export let foo = 5;` differ in the map by exactly this one segment. The end
  now falls back to the last copied run at or before the offset, which is where
  upstream's own declaration span ends. An offset past the end of the chunk is
  excluded: a kept `;` for a removed `$inspect` marks itself with
  `span.end == u32::MAX`, and mapping that sentinel to a real position deletes
  the marker, so the `;;` upstream prints collapses to `;`.

| | before | after |
|---|---|---|
| official segments reproduced | 741 / 770 | **768 / 770** |
| — of which missing / wrong | 24 / 5 | **0 / 2** |
| out-of-range segments | 0 | 0 |
| ratchet entries | 0 (unattainable) | **2** |

### Sixth catch: the keyword anchor, ported twice and guarded once

The last two entries were `attached-sourcemap` on `client` and `server`, one
segment each, and they read as one defect: official emits two segments at one
generated column and rsvelte emitted only the second. They were **four**
defects, in two ports of one upstream function
(`write_source_keyword`, `esrap/src/languages/ts/index.js:113`), which anchors
`location(line, column)` / `location(line, column + keyword.length)` around a
fragment that *includes* the keyword's trailing space.

| # | where | what it did |
|---|---|---|
| A | `KeywordCursor::write`, `Printer::write_keyword` | dropped the end anchor when `column + keyword.len()` exceeded the source line's length. Upstream has no such test; `let` alone on a line is 3 wide and the anchor for `let ` is at column 4. |
| B | `Driver::push_mapping` | **overwrote** the previous mapping when the generated position matched. esrap pushes one segment per `Location` command, so two anchors at one generated column are two segments. |
| C | `keyword_cursor`, `write_keyword` | mapped a builder-made node's keyword. Upstream guards every keyword write on `node.loc`; rsvelte spells "no loc" as an empty or sentinel span and only `write_node` was checking it, so every synthesized `var root = …` / `import …` anchored at offset 0 of the `.svelte` file. |
| D | `generate_token_mappings_inner` (`3_transform/mod.rs`) | the **server** map is not built by esrap at all — `print_split` runs with `emit_locations: false` and a text token scan supplies the anchors. It anchored the 3-character token `let`, so its end anchor was one column short of upstream's. |

Each was measured on its own by restoring it and re-running the gate:

| restored | official segments reproduced | out-of-range | which sample |
|---|---|---|---|
| — (all four fixed) | **770 / 770** | **0 / 1634** | — |
| A | 769 / 770 | 0 / 1633 | `attached-sourcemap/client` |
| B | 769 / 770 | 0 / 1596 | `attached-sourcemap/client` |
| C | 758 / 770 | 3 / 1870 | 10 samples, all `client` |
| D | 769 / 770 | 0 / 1634 | `attached-sourcemap/server` |

Two things generalize past the fix.

**B was masking C.** The dedup made a spurious anchor invisible whenever a
correct one landed on the same generated column, which is exactly what happens
after `var ` in `var h1 = root();`. Removing the dedup alone takes the gate from
2 wrong to 13, and twelve of those thirteen are C, which had been there the whole
time — 236 spurious segments over the 29 samples (1870 with C restored against
1634 without it). A collapse rule that keeps "the last write wins" is not a
normalization — it is a repair that hides whatever it repaired.

**The two ports could not be compared to each other by anything.** The client's
anchors come from `rsvelte_esrap`, the server's from a text token scan in
`3_transform/mod.rs`, and every gate here compares each of them to *upstream* on
whatever inputs a sample happens to supply. `attached-sourcemap` is the one
sourcemaps sample whose `let` is alone on its source line, and it is the only
reason either half was visible. See `two-ports-inventory.md`.

Four independently-failing pins keep them apart:
`crates/rsvelte_esrap/tests/keyword_anchor_fidelity.rs` (A, B, C — one test
each, each failing only under its own ablation) and
`crates/rsvelte_core/tests/server_declaration_keyword_anchor.rs` (D). There is no
`compatibility/pattern-corpus/` repro because the corpus pipeline never writes a
`js.map`: `scripts/compat-corpus/compile.mjs` stores generated code only, so a
file added there would measure nothing about this class.

| | before | after |
|---|---|---|
| official segments reproduced | 768 / 770 | **770 / 770** |
| — of which missing / wrong | 0 / 2 | **0 / 0** |
| out-of-range segments | 0 / 1816 | **0 / 1634** |
| ratchet entries | 2 | **0** |

Total segments fall 1816 → 1634 because C's spurious anchors are gone; the
official map is reproduced in full at the same time, so the drop is
over-emission being removed, not resolution being lost.

## Entries

No entry is accepted as correct behaviour; all are burndown targets. The list is
currently **empty**: every official segment is reproduced and no segment points
outside its source. Unlike the empty list #3896 wrote (see the fifth catch), this
one is a measurement — `sourcemap_gate` asserts it, and the four ablations above
each turn it red.
