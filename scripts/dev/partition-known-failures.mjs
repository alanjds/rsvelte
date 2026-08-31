// Partition the four output ratchets by WHY each listed entry diverges, without
// running a corpus sweep: every id is compiled in-process by both compilers and
// classified under the gate's own normalization (flattenTemplateHoles -> oxfmt
// -> stripBlankLines).
//
// The buckets that matter are `ALREADY-PASSES` (a two-sided ratchet fails on
// these, so they must be re-baselined) and `output-unparseable` (text no JS
// parser accepts, which a match/mismatch verdict cannot distinguish from
// ordinary wrong text).
//
//   node scripts/dev/partition-known-failures.mjs --binding path/to/rsvelte.node
//
// A git worktree usually has only some submodules populated, so `--corpus-root`
// points the SOURCE lookup at a checkout that has them (the ratchets, oracle and
// normalizer are still read from this checkout). Entries whose file is absent are
// reported as FILE-MISSING rather than silently skipped.
//
// The binding is a built `rsvelte_napi` cdylib; `cargo build --release -p
// rsvelte_napi` puts one at `target/release/librsvelte_napi.dylib` (`.so` on
// Linux). Reads only — it never writes into `scripts/compat-corpus/{sources,
// expected,actual}` or into any ratchet.

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { createRequire } from 'node:module';
import { execFileSync } from 'node:child_process';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const require = createRequire(import.meta.url);

function arg(name) {
	const i = process.argv.indexOf(`--${name}`);
	return i === -1 ? undefined : process.argv[i + 1];
}

const bindingArg = arg('binding') ?? process.env.BINDING;
if (!bindingArg) {
	console.error('need --binding <path to rsvelte_napi cdylib> (or $BINDING)');
	process.exit(2);
}
const binding = path.resolve(bindingArg);
if (!fs.existsSync(binding)) {
	console.error(`no such binding: ${binding}`);
	process.exit(2);
}

// The gates use the submodule SOURCE entry point; the npm build and the built
// submodule bundle disagree with it on real inputs.
const { OFFICIAL_COMPILER_REL } = await import(
	path.join(ROOT, 'scripts/compat-corpus/oracle.mjs')
);
const svelte = await import(path.join(ROOT, OFFICIAL_COMPILER_REL));
const rs = require(binding);
const acorn = require(path.join(ROOT, 'node_modules/acorn'));
const esbuild = require(path.join(ROOT, 'node_modules/esbuild'));
const { stripBlankLines, flattenTemplateHoles } = await import(
	path.join(ROOT, 'scripts/compat-corpus/normalize.mjs')
);

// Submodule sources may live in another checkout; everything else is local.
const CORPUS_ROOT = path.resolve(arg('corpus-root') ?? process.env.CORPUS_ROOT ?? ROOT);

const sources = JSON.parse(
	fs.readFileSync(path.join(ROOT, 'scripts/compat-corpus/corpus-sources.json'), 'utf8')
);
const repoDir = new Map(sources.map((s) => [s.id, s.path]));

const TARGETS = {
	client: { generate: 'client', dev: false, css: true },
	'client-dev': { generate: 'client', dev: true, css: true },
	server: { generate: 'server', dev: false, css: false },
	'server-dev': { generate: 'server', dev: true, css: false }
};

const work = fs.mkdtempSync(path.join(os.tmpdir(), 'rsvelte-partition-'));
process.on('exit', () => fs.rmSync(work, { recursive: true, force: true }));

if (CORPUS_ROOT !== ROOT) console.log(`corpus sources: ${CORPUS_ROOT}`);

for (const [name, target] of Object.entries(TARGETS)) {
	const ratchet = path.join(ROOT, `compatibility/known-failures.${name}.json`);
	const ids = JSON.parse(fs.readFileSync(ratchet, 'utf8'));
	const dir = path.join(work, name);
	fs.mkdirSync(path.join(dir, 'expected'), { recursive: true });
	fs.mkdirSync(path.join(dir, 'actual'), { recursive: true });

	const records = [];
	let threw = 0;
	for (const id of ids) {
		const repo = id.split('/')[0];
		const rel = repoDir.get(repo);
		if (!rel) {
			records.push({ id, verdict: 'NO-SOURCE' });
			continue;
		}
		let source;
		try {
			source = fs.readFileSync(path.join(CORPUS_ROOT, rel, id.slice(repo.length + 1)), 'utf8');
		} catch {
			records.push({ id, verdict: 'FILE-MISSING' });
			continue;
		}
		// A `.svelte.(js|ts)` is a MODULE: compiling it with `compile()` reports
		// spurious error-parity rather than whatever it actually diverges on.
		const isModule = /\.svelte\.[jt]s$/.test(id);
		if (id.endsWith('.svelte.ts')) {
			try {
				source = esbuild.transformSync(source, { loader: 'ts' }).code;
			} catch {
				/* leave the TS in and let the compilers report */
			}
		}
		const options = {
			generate: target.generate,
			dev: target.dev,
			filename: id,
			...(isModule ? {} : { css: 'external' })
		};

		let expected = null,
			actual = null,
			expectedError = null,
			actualError = null;
		try {
			expected = isModule ? svelte.compileModule(source, options) : svelte.compile(source, options);
		} catch (error) {
			expectedError = error;
		}
		try {
			actual = isModule ? rs.compileModule(source, options) : rs.compile(source, options);
		} catch (error) {
			actualError = error;
		}
		if (expectedError || actualError) {
			threw++;
			const verdict =
				expectedError && actualError
					? (expectedError.code ?? '?') === (actualError.code ?? '?')
						? 'error-parity'
						: 'error-mismatch'
					: 'compile-mismatch';
			records.push({ id, verdict });
			continue;
		}

		let unparseable = false;
		try {
			acorn.parse(actual.js.code, { ecmaVersion: 'latest', sourceType: 'module' });
		} catch {
			unparseable = true;
		}
		const slot = records.length;
		fs.writeFileSync(path.join(dir, 'expected', `${slot}.js`), flattenTemplateHoles(expected.js.code));
		fs.writeFileSync(path.join(dir, 'actual', `${slot}.js`), flattenTemplateHoles(actual.js.code));
		records.push({
			id,
			verdict: unparseable ? 'output-unparseable' : 'PENDING',
			expectedCss: target.css ? (expected.css?.code ?? '') : '',
			actualCss: target.css ? (actual.css?.code ?? '') : '',
			slot
		});
	}

	try {
		execFileSync(
			path.join(ROOT, 'node_modules/.bin/oxfmt'),
			['-c', path.join(ROOT, 'compatibility/.oxfmtrc.json'), path.join(dir, 'expected'), path.join(dir, 'actual')],
			{ stdio: 'ignore' }
		);
	} catch {
		// oxfmt exits non-zero on files it declines to format; the comparison
		// below still runs, on unformatted text for those.
	}

	const counts = {};
	for (const record of records) {
		let verdict = record.verdict;
		if (verdict === 'PENDING' || verdict === 'output-unparseable') {
			const e = stripBlankLines(fs.readFileSync(path.join(dir, 'expected', `${record.slot}.js`), 'utf8'));
			const a = stripBlankLines(fs.readFileSync(path.join(dir, 'actual', `${record.slot}.js`), 'utf8'));
			if (verdict !== 'output-unparseable') {
				verdict = e !== a ? 'js-mismatch' : record.expectedCss !== record.actualCss ? 'css-mismatch' : 'ALREADY-PASSES';
			}
		}
		record.verdict = verdict;
		counts[verdict] = (counts[verdict] || 0) + 1;
	}

	console.log(`\n### ${name} (${ids.length} entries)  [entries where a compiler threw: ${threw}]`);
	for (const [verdict, n] of Object.entries(counts).sort((x, y) => y[1] - x[1])) {
		console.log(`  ${String(n).padStart(4)}  ${verdict}`);
	}
	// `PARTITION_LIST=1` names each entry, which is what a re-baseline needs:
	// the `ALREADY-PASSES` ids are exactly the ones a two-sided ratchet fails on.
	if (process.env.PARTITION_LIST) {
		for (const record of records) console.log(`    ${record.verdict}\t${record.id}`);
	}
}
