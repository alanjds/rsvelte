#!/usr/bin/env node
// Counts `lsp-known-failures.json` by the `mech=` segment its corpus keys now
// carry and prints the `Attribution of ...` block `scripts/ci/attribution-check.mjs`
// requires. It is a generator, not a check: a label with no declared target is
// printed as a decision that is still owed rather than folded into a neighbour,
// because a target invented to make the sum work is exactly the failure the
// attribution gate exists to catch.
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const RATCHET = path.join(ROOT, 'compatibility/lsp-known-failures.json');

// `U` = attributed to a filed upstream report; `D` = deliberate, pinned by a
// test; `R` = an rsvelte defect, whose only end state is zero entries.
const TARGETS = {
	'ts-render-union-order': ['U', 'upstream_issues/tsgo-lsp-hover-renders-declarations-differently-from-tsc.md'],
	'ts-render-quote-style': ['U', 'upstream_issues/tsgo-lsp-hover-renders-declarations-differently-from-tsc.md'],
	'ts-render-local-modifier': ['U', 'upstream_issues/tsgo-lsp-hover-renders-declarations-differently-from-tsc.md'],
	'ts-render-jsdoc-tag': ['U', 'upstream_issues/tsgo-lsp-hover-renders-declarations-differently-from-tsc.md'],
	'ts-render-import-line': ['U', 'upstream_issues/tsgo-lsp-hover-renders-declarations-differently-from-tsc.md'],
	'ts-render-overload-count': ['U', 'upstream_issues/tsgo-lsp-hover-renders-declarations-differently-from-tsc.md'],
	'ts-render-declaration-order': ['U', 'upstream_issues/tsgo-lsp-hover-renders-declarations-differently-from-tsc.md'],
	'ts-render-multiple': ['U', 'upstream_issues/tsgo-lsp-hover-renders-declarations-differently-from-tsc.md'],
	'ts-type-any': ['R', null],
	'projection-target-position-declaration': ['U', 'upstream_issues/svelte-language-server-rune-definition-lands-inside-the-jsdoc.md'],
	'projection-target-position-workspace': ['R', null],
	'completion-item-detail-presence-rsvelte-only': ['R', null],
	'completion-text-edit-presence-rsvelte-only': ['R', null],
	'rsvelte-empty-import-only': ['U', 'upstream_issues/tsgo-lsp-hover-renders-declarations-differently-from-tsc.md'],
	'official-empty-target-is-the-request': ['R', null],
	'completion-item-pairing-key-filter-text-ts': ['R', null],
	'completion-text-edit-new-text-html': ['R', null],
	'completion-text-edit-new-text-ts': ['R', null],
	'completion-command-presence-official-only': ['R', null],
	'completion-text-edit-range-end': ['R', null],
	'completion-text-edit-range-start': ['R', null],
	'completion-additional-text-edits-presence-official-only': ['R', null],
	'ts-lib-copy': ['D', 'deliberate-divergences'],
	'official-defect-svelte-ts-shadow': ['U', 'upstream_issues/svelte-language-server-hovers-svelte2tsx-synthesized-render-function.md'],
	// Only the SINGLE-field, SINGLE-provider cell is attributable: the field set
	// and the provider are both unions over one response, so `kind+sort-text` and
	// `-mixed` each still hold two mechanisms.
	'completion-item-pairing-key-kind-ts': ['U', 'upstream_issues/tsgo-lsp-completion-item-omits-the-typescript-kind.md'],
	'completion-command-presence-rsvelte-only': ['R', null],
	'projection-response-range': ['R', null],
	'ts-symbol-name': ['R', null],
	'completion-item-set-missing-emmet': ['R', null],
	'completion-item-set-missing-html-close-tag': ['R', null],
	'target-component-vs-import': ['R', null],
	'completion-item-data-source-official-only': ['R', null],
	'completion-item-data-uri': ['R', null],
	'html-data': ['R', null],
	'css-data': ['R', null],
	'target-file-mismatch': ['R', null],
	'completion-commit-characters-presence-rsvelte-only': ['U', 'upstream_issues/tsgo-lsp-completion-omits-the-commit-character-inputs.md'],
	'completion-commit-characters-value-extra-paren': ['U', 'upstream_issues/tsgo-lsp-completion-omits-the-commit-character-inputs.md'],
	'completion-commit-characters-value-other': ['U', 'upstream_issues/tsgo-lsp-completion-omits-the-commit-character-inputs.md'],
};

const CLUSTERS = {
	'ts-render-union-order': 'tsgo sorts a union’s members; tsc prints them in declaration order',
	'ts-render-quote-style': 'tsgo echoes the source’s quote spelling in `import(…)`; tsc normalizes to `"`',
	'ts-render-local-modifier': 'tsc marks a nested function `(local function)`; tsgo does not',
	'ts-render-jsdoc-tag': 'tsc returns JSDoc tags separately; tsgo inlines them into the hover body',
	'ts-render-import-line': 'tsc names the import an alias came through on a second line; tsgo prints the declaration alone',
	'ts-render-overload-count': 'tsc appends `(+N overloads)` to a selected call signature; tsgo prints the signature alone',
	'ts-render-declaration-order': 'a merged symbol prints one line per declaration and the two disagree on the order; every rune is a function plus a namespace',
	'ts-render-multiple': 'two of the seven renderings in one hover — named for the pair, because a label a rule wins by its position in the table would make the ratchet key depend on that order',
	'ts-type-any': 'the same declaration typed `any` on the rsvelte side where official resolves a real type — not a rendering difference',
	'projection-target-position-declaration': 'official lands eight lines short of a rune’s `declare function`, inside its JSDoc; rsvelte lands on the declaration',
	'projection-target-position-workspace': 'rsvelte’s origin range covers the enclosing node where official’s covers the identifier — the ends agree and rsvelte’s start is earlier',
	'completion-item-detail-presence-rsvelte-only': 'official assigns `detail` only in `completionItem/resolve` (`CompletionProvider.ts:989`), so its initial list can never carry one; rsvelte’s tsgo proxy fills it there',
	'completion-text-edit-presence-rsvelte-only': 'official emits `textEdit` only where tsc returned a `replacementSpan` (`CompletionProvider.ts:693`); rsvelte emits one unconditionally',
	'rsvelte-empty-import-only': 'official’s entire hover is the `import <Name>` line tsgo drops, so dropping it leaves tsgo with nothing to answer',
	'official-empty-target-is-the-request': 'rsvelte answers a definition with the very token the request sat on — every sampled row is `restProps` inside a `{...restProps}` spread',
	'completion-item-pairing-key-filter-text-ts': 'rsvelte passes tsgo’s `filterText` through on a bracket-accessor completion; official’s only `filterText` setter is a Svelte `component` snippet',
	'completion-text-edit-new-text-html': 'upstream appends the `="$1"` value snippet to an attribute name (`htmlCompletion.js:216-217`); rsvelte inserts the bare name',
	'completion-text-edit-new-text-ts': 'official trims `newText` to the part after the word range (`CompletionProvider.ts:894`); rsvelte inserts the whole label',
	'completion-command-presence-official-only': 'the same `value.length` gate that appends `="$1"` also attaches the trigger-suggest command, and rsvelte reads the attribute name one token wider',
	'completion-text-edit-range-end': 'the same attribute-name boundary: official inserts at an empty range where rsvelte replaces the character under the cursor',
	'completion-text-edit-range-start': 'official’s word-range fixup moves `range.start` to the word start (`CompletionProvider.ts:895-898`) in the same block that trims `newText`',
	'completion-additional-text-edits-presence-official-only': 'the third assignment of that same block (`CompletionProvider.ts:899`) — the edit that restores the part `newText` no longer carries',
	'ts-lib-copy': 'each server names the `lib.d.ts` of the type checker that answered — pinned by `scripts/compat-lsp/test-ts-lib-copy.mjs`',
	'official-defect-svelte-ts-shadow': 'official answers about svelte2tsx’s generated `$$render` / `*.svelte.ts` shadow, which exists in no document the user has open',
	'completion-item-pairing-key-kind-ts': 'tsgo omits the TypeScript kind, so a `const` completes as `Variable` where tsc says `Constant`',
	'completion-command-presence-rsvelte-only': 'the `style` arm of the trigger-suggest condition escaped upstream’s outer guard (`&&` binds tighter than `||`)',
	'projection-response-range': 'the hover text agrees and rsvelte’s range does not cover the position that was asked about — a constant column shift, or a collapse to zero width',
	'ts-symbol-name': 'the same per-line column shift, landing on the neighbouring symbol instead of the same one',
	'completion-item-set-missing-emmet': 'official runs an emmet participant and rsvelte has none; every sampled row is official-only abbreviations against an empty rsvelte answer',
	'completion-item-set-missing-html-close-tag': 'rsvelte has no `collectCloseTagSuggestions` path at all',
	'target-component-vs-import': 'a component import resolves to the import specifier, not the component',
	'completion-item-data-source-official-only': 'tsgo sends `data.source`; `adopt_upstream_item_data` rebuilds `data` without it',
	'completion-item-data-uri': 'official re-serializes the document URI (`+` becomes `%2B`); rsvelte echoes the client string',
	'html-data': 'rsvelte answers nothing where upstream hovers an element from its HTML data',
	'css-data': 'rsvelte hovers a CSS property as a one-line stub where upstream renders MDN prose, syntax and Baseline',
	'target-file-mismatch': 'rsvelte resolves the directive name and lands inside the requesting document',
	'completion-commit-characters-presence-rsvelte-only': 'upstream omits the field where TypeScript gives none; tsgo gives none either way',
	'completion-commit-characters-value-extra-paren': 'upstream appends `(` only at a new-identifier location, which tsgo does not report',
	'completion-commit-characters-value-other': 'upstream passes TypeScript’s per-entry list through; tsgo sends no per-entry list',
};

// An optional path so the table can be produced from a `--write-current`
// artifact: the committed ratchet is only re-keyed by a baseline run, and the
// table's own arithmetic is otherwise never exercised.
const entries = JSON.parse(fs.readFileSync(process.argv[2] ?? RATCHET, 'utf8'));
const list = entries.current ?? (Array.isArray(entries) ? entries : Object.values(entries).flat());
const counts = new Map();
for (const key of list) {
	const match = /\|mech=([^|]+)/.exec(key);
	counts.set(match ? match[1] : '(no mech= segment)', (counts.get(match ? match[1] : '(no mech= segment)') ?? 0) + 1);
}

// `ratchet.mjs` builds the key without `mech=`, so every entry lands in one
// bucket and the table is empty by construction rather than by a missing
// target. A baseline run alone does not populate it: the key change and its
// re-baseline are one commit, because a key format with no baseline behind it
// makes every committed entry stale at once.
if (counts.size === 1 && counts.has('(no mech= segment)')) {
	console.log(
		`This ratchet predates the \`mech=\` re-keying: ${counts.get('(no mech= segment)')} entries carry no label. The table below populates when the key gains its \`mech=\` segment AND the ratchet is re-baselined in the same commit — a baseline run against the current key produces no labels.\n`,
	);
}

const rows = [...counts].sort((left, right) => right[1] - left[1]);
const owed = rows.filter(([label]) => !TARGETS[label]);
console.log(`Attribution of \`lsp-known-failures.json\`:\n`);
console.log('| n | target | cluster |');
console.log('|---|---|---|');
for (const [label, n] of rows) {
	const target = TARGETS[label];
	if (!target || !target[1]) continue;
	console.log(`| ${n} | \`${target[1]}\` | ${CLUSTERS[label] ?? label} |`);
}
// A signed label the artifact holds no carrier for is NOT n = 0: this artifact
// was written before the label existed, so the population is stale and nothing
// about that label was measured. A blank and a zero are indistinguishable, and
// a dropped row is worse than either.
for (const label of Object.keys(TARGETS)) {
	if (counts.has(label) || !TARGETS[label][1]) continue;
	console.log(`| STALE-POPULATION | \`${TARGETS[label][1]}\` | ${CLUSTERS[label] ?? label} |`);
}
// The label is IN the ratchet key, so one entry carries exactly one label and
// `n` partitions the file by construction -- no dominant-label rule is needed
// and none may be added, because it would double-count or drop entries.
const staleSigned = Object.keys(TARGETS).filter((label) => !counts.has(label));
if (staleSigned.length)
	console.log(
		`\n${staleSigned.length} signed label(s) have no carrier in this artifact and are printed as STALE-POPULATION, not 0: ${staleSigned.join(', ')}.`,
	);
const rLabels = rows.filter(([label]) => TARGETS[label]?.[0] === 'R');
const total = (subset) => subset.reduce((sum, [, n]) => sum + n, 0);
const attributed = total(rows.filter(([label]) => TARGETS[label]?.[1]));
const owedTotal = total(owed);
const rTotal = total(rLabels);
console.log(`\n${list.length} entries total; ${rows.length} mechanism labels.`);
console.log(
	`${attributed} attributed + ${rTotal} awaiting zero + ${owedTotal} undecided = ${attributed + rTotal + owedTotal}`,
);
if (attributed + rTotal + owedTotal !== list.length)
	console.log('WARNING: the three buckets do not partition the ratchet.');
if (owed.length) {
	console.log('\nLabels with no declared target (a decision is owed, not a row):');
	for (const [label, n] of owed) console.log(`  ${String(n).padStart(7)}  ${label}`);
}
if (rLabels.length) {
	// `attribution-check.mjs` takes an `upstream_issues/` path or
	// `deliberate-divergences` and nothing else, so an rsvelte defect is not a
	// row: it is the first of P3's three options, and the attribution sum stays
	// short by exactly this bucket until these labels reach zero.
	console.log(
		`\nLabels whose only end state is zero (${rTotal} entries; not attributable, so not rows):`,
	);
	for (const [label, n] of rLabels) console.log(`  ${String(n).padStart(7)}  ${label}`);
}

// P3 is complete when every entry is attributed, and an attribution is a
// FILE, not a label: one defect wearing three labels is one row someone has to
// write and one report someone has to read. Counting labels therefore
// overstates the remaining work by exactly the labels-per-ID factor, so the
// progress metric is the ID count and the factor is printed beside it.
const byTarget = new Map();
for (const [label, n] of rows) {
	const id = TARGETS[label]?.[1];
	if (!id) continue;
	const seen = byTarget.get(id) ?? { labels: [], entries: 0 };
	seen.labels.push(label);
	seen.entries += n;
	byTarget.set(id, seen);
}
for (const label of staleSigned) {
	const id = TARGETS[label][1];
	if (!id) continue;
	const seen = byTarget.get(id) ?? { labels: [], entries: 0 };
	seen.labels.push(label);
	byTarget.set(id, seen);
}
const ids = [...byTarget].sort((left, right) => right[1].entries - left[1].entries);
const signedLabels = ids.reduce((sum, [, seen]) => sum + seen.labels.length, 0);
if (ids.length) {
	console.log(`\n${ids.length} attribution ID(s) cover ${signedLabels} label(s):\n`);
	console.log('| labels | entries | attribution |');
	console.log('|---|---|---|');
	for (const [id, seen] of ids)
		console.log(`| ${seen.labels.length} | ${seen.entries} | \`${id}\` |`);
	// All N entries of an N-label defect must carry the SAME attribution, so a
	// multi-label ID is the shape to check by hand: the labels below are one
	// report, and a reader who lands on any of them must reach that report.
	for (const [id, seen] of ids)
		if (seen.labels.length > 1) console.log(`\n\`${id}\` <- ${seen.labels.join(', ')}`);
}

// `n` bounds nothing in either direction. One label spans several mechanisms
// until it is split, and one mechanism spans several labels: at
// `hr.svelte:2:39` the item `elements` has `kind`, `sortText` and `filterText`
// all null on the official side, which produces a pairing-key key AND an
// item-set key from a single cause.
console.log(
	'\nNeither `n` nor the row count is an upper or a lower bound on the number of defects.',
);
