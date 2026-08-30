#!/usr/bin/env node
// A shrink-only ratchet is shrink-only in its KEY count, and several keys here carry a
// content hash (`[count=…,hash=…]`, `[official=…,rsvelte=…]`). A re-baseline that improves
// a divergence without eliminating it therefore retires one key and enrols another, so the
// key diff cannot tell "improved, still divergent" from "newly broken" — and a net shrink
// can contain a new defect. Diff the UNIT (the key with its trailing bracket stripped).
//
//   node scripts/ci/ratchet-unit-delta.mjs <ratchet.json> [base-ref]
//
// Exits 1 when a unit is genuinely new, so this can gate a re-baseline.
import fs from 'node:fs';
import { execFileSync } from 'node:child_process';

const [, , file, baseRef = 'origin/main'] = process.argv;
if (!file) {
	console.error('usage: ratchet-unit-delta.mjs <ratchet.json> [base-ref]');
	process.exit(2);
}

const keysOf = (text) => {
	const j = JSON.parse(text);
	return Array.isArray(j) ? j.map(String) : Object.keys(j);
};
// Both bracket shapes end the key, so one anchored strip covers them.
const unit = (k) => k.replace(/\[[^[\]]*\]$/, '');

let base;
try {
	base = keysOf(execFileSync('git', ['show', `${baseRef}:${file}`], { maxBuffer: 1 << 30 }).toString());
} catch {
	console.error(`[ratchet-unit-delta] ${file} does not exist at ${baseRef} — nothing to compare`);
	process.exit(0);
}
const head = keysOf(fs.readFileSync(file, 'utf8'));

const kb = new Set(base);
const kh = new Set(head);
const ub = new Set(base.map(unit));
const uh = new Set(head.map(unit));

const removed = base.filter((k) => !kh.has(k));
const added = head.filter((k) => !kb.has(k));
const churnIn = added.filter((k) => ub.has(unit(k)));
const churnOut = removed.filter((k) => uh.has(unit(k)));
const newUnits = [...new Set(added.filter((k) => !ub.has(unit(k))).map(unit))];
const goneUnits = [...new Set(removed.filter((k) => !uh.has(unit(k))).map(unit))];

console.log(`${file}  (${baseRef} -> working tree)`);
console.log(`  keys    ${base.length} -> ${head.length}   (${head.length - base.length >= 0 ? '+' : ''}${head.length - base.length})`);
console.log(`  units   ${ub.size} -> ${uh.size}   (${uh.size - ub.size >= 0 ? '+' : ''}${uh.size - ub.size})`);
console.log(`  removed keys ${removed.length}, of which ${churnOut.length} are units still listed (content churn)`);
console.log(`  added   keys ${added.length}, of which ${churnIn.length} are units already listed (content churn)`);
console.log(`  units eliminated ${goneUnits.length}`);
console.log(`  units genuinely NEW ${newUnits.length}`);
for (const u of newUnits.slice(0, 20)) console.log(`    + ${u}`);
if (newUnits.length > 20) console.log(`    … ${newUnits.length - 20} more`);

process.exit(newUnits.length ? 1 : 0);
