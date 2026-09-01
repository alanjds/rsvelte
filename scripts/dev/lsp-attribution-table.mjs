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
	'ts-render-union-order': ['U', 'upstream_issues/tsgo-lsp-hover-renders-four-things-differently-from-tsc.md'],
	'ts-render-quote-style': ['U', 'upstream_issues/tsgo-lsp-hover-renders-four-things-differently-from-tsc.md'],
	'ts-render-local-modifier': ['U', 'upstream_issues/tsgo-lsp-hover-renders-four-things-differently-from-tsc.md'],
	'ts-render-jsdoc-tag': ['U', 'upstream_issues/tsgo-lsp-hover-renders-four-things-differently-from-tsc.md'],
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

const entries = JSON.parse(fs.readFileSync(RATCHET, 'utf8'));
const list = Array.isArray(entries) ? entries : Object.values(entries).flat();
const counts = new Map();
for (const key of list) {
	const match = /\|mech=([^|]+)/.exec(key);
	counts.set(match ? match[1] : '(no mech= segment)', (counts.get(match ? match[1] : '(no mech= segment)') ?? 0) + 1);
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
console.log(`\n${list.length} entries total; ${rows.length} mechanism labels.`);
if (owed.length) {
	console.log('\nLabels with no declared target (a decision is owed, not a row):');
	for (const [label, n] of owed) console.log(`  ${String(n).padStart(7)}  ${label}`);
}
const rLabels = rows.filter(([label]) => TARGETS[label]?.[0] === 'R');
if (rLabels.length) {
	console.log('\nLabels whose only end state is zero (R):');
	for (const [label, n] of rLabels) console.log(`  ${String(n).padStart(7)}  ${label}`);
}

// `n` bounds nothing in either direction. One label spans several mechanisms
// until it is split, and one mechanism spans several labels: at
// `hr.svelte:2:39` the item `elements` has `kind`, `sortText` and `filterText`
// all null on the official side, which produces a pairing-key key AND an
// item-set key from a single cause.
console.log(
	'\nNeither `n` nor the row count is an upper or a lower bound on the number of defects.',
);
