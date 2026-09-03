#!/usr/bin/env node
// `attribution-pending.json` says which ratchets have no attribution table yet.
// This refines that declaration from the file to the entry: which listed id has
// a located cause, and which issue tracks its elimination.
//
// It is NOT a fourth terminal state. `attribution-check.mjs` with no flag is the
// DoD and still counts every id recorded here as unattributed, because an
// rsvelte-side defect has exactly one end: elimination. A record says the work is
// located; an issue number is deferral, not achievement.
//
// It checks a RELATION -- one mechanism, one issue -- and says nothing about
// whether a `mechanism` string is true. Most of the first batch were wrong while
// passing here: written from the carrier's spelling, from a diff hunk read as
// order, naming one half of a conjunction, naming a symptom instead of a trigger,
// missing a condition, and once with the direction reversed. Only a cell reduced
// from the carrier and run against the oracle settles the content, and reducing
// from this string instead inherits its error; until then the issue says so.
//
// Nor can it see one entry carrying TWO mechanisms: the relation runs the other
// way, and such an entry retires only when both are fixed.
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const DIR = process.env.ATTRIBUTION_DIR ?? path.join(HERE, '..', '..', 'compatibility');
const PENDING = path.join(DIR, 'attribution-pending.json');
const PROGRESS = path.join(DIR, 'attribution-progress.json');

const problems = [];
const fail = (message) => problems.push(message);

const readJson = (file) => JSON.parse(fs.readFileSync(file, 'utf8'));

// A ratchet is either an array of ids or an object keyed by id.
const idsOf = (value) => new Set(Array.isArray(value) ? value : Object.keys(value));

function main() {
	if (!fs.existsSync(PENDING))
		throw new Error(
			`${PENDING} is missing; a progress record only refines a pending declaration`,
		);
	const pending = new Set(readJson(PENDING));
	// No progress file at all is a valid state: nothing is located yet.
	const progress = fs.existsSync(PROGRESS) ? readJson(PROGRESS) : {};

	for (const [file, records] of Object.entries(progress)) {
		if (!pending.has(file))
			fail(
				`${file} is not in attribution-pending.json; a ratchet that has an attribution table needs no progress record`,
			);
		const ratchetPath = path.join(DIR, file);
		if (!fs.existsSync(ratchetPath)) {
			fail(`${file} does not exist`);
			continue;
		}
		const listed = idsOf(readJson(ratchetPath));
		const seen = new Set();
		const byMechanism = new Map();
		for (const record of records) {
			const { id, issue, port, mechanism } = record;
			if (typeof id !== 'string' || !id) fail(`${file}: a record has no id`);
			else if (seen.has(id)) fail(`${file}: ${id} has two records`);
			// Stale in the sense the ratchet itself is two-sided: the entry this
			// record describes is gone, so the record describes nothing.
			else if (!listed.has(id))
				fail(`${file}: ${id} has a progress record and is not listed`);
			if (id) seen.add(id);
			if (!Number.isInteger(issue) || issue <= 0)
				fail(`${file}: ${id} cites issue ${JSON.stringify(issue)}; expected a positive integer`);
			if (typeof port !== 'string' || !port) fail(`${file}: ${id} has no port`);
			if (typeof mechanism !== 'string' || !mechanism) fail(`${file}: ${id} has no mechanism`);
			// Two entries of one mechanism in one port are one defect. Keying on
			// something the mechanism does not determine -- the direction a cell
			// happened to run in, say -- splits one defect into two backlogs.
			const key = `${port}\0${mechanism}`;
			const prior = byMechanism.get(key);
			if (prior === undefined) byMechanism.set(key, { issue, id });
			else if (prior.issue !== issue)
				fail(
					`${file}: ${id} and ${prior.id} share port ${JSON.stringify(port)} and mechanism ` +
						`${JSON.stringify(mechanism)} but cite issues ${issue} and ${prior.issue}`,
				);
		}
		console.log(`${file}: ${seen.size}/${listed.size} listed entries located`);
	}

	for (const file of pending)
		if (!Object.hasOwn(progress, file)) console.log(`${file}: 0 located`);

	if (problems.length) {
		for (const problem of problems) console.error(problem);
		console.error(
			`\n${problems.length} problem(s). A progress record locates an rsvelte-side defect and ` +
				'names the issue that will eliminate it. It is not an attribution and does not satisfy ' +
				'the DoD, which still requires the entry to be gone.',
		);
		process.exitCode = 1;
		return;
	}
	console.log(
		'[attribution-progress] every record names a listed entry, a port, a mechanism and an issue',
	);
}

main();
