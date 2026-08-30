#!/usr/bin/env node
// DoD for `compatibility/deliberate-divergences.md`: a divergence recorded there is a
// decision not to close, so it must be held in place by a test. A section with prose
// and no pin is a claim nothing re-checks — the next refactor changes the behaviour
// and the document keeps asserting the old one.
//
// One section is one `## ` heading. A pin is a repository path this file names that
// exists on disk and is a test: something under a `tests/` directory, a corpus pattern,
// or a `scripts/**/test-*.mjs` harness — the checker's first run rejected a section that
// WAS pinned, by a harness under `scripts/dev/`, so the shape has to be read off the
// tree rather than assumed.
import fs from 'node:fs';
import path from 'node:path';

const DOC = 'compatibility/deliberate-divergences.md';
const text = fs.readFileSync(DOC, 'utf8');

const PIN = /`((?:crates|compatibility|apps|packages|scripts)\/[A-Za-z0-9._@/-]+?\.(?:rs|svelte|svelte\.js|svelte\.ts|mjs|ts))`/g;
const isPin = (p) =>
  /(^|\/)tests\//.test(p) ||
  p.startsWith('compatibility/pattern-corpus/') ||
  /(^|\/)test-[A-Za-z0-9._-]+\.mjs$/.test(p);

const lines = text.split('\n');
const sections = [];
let current = null;
let inFence = false;
for (const [i, line] of lines.entries()) {
  if (/^\s*(```|~~~)/.test(line)) inFence = !inFence;
  if (inFence) {
    if (current) current.body.push(line);
    continue;
  }
  if (/^## /.test(line)) {
    current = { title: line.slice(3).trim(), line: i + 1, body: [] };
    sections.push(current);
  } else if (current) {
    current.body.push(line);
  }
}

const problems = [];
for (const s of sections) {
  const body = s.body.join('\n');
  const cited = [...body.matchAll(PIN)].map((m) => m[1]);
  const pins = cited.filter(isPin);
  if (pins.length === 0) {
    problems.push(`${DOC}:${s.line}  "${s.title}" names no pin`);
    continue;
  }
  for (const p of pins) {
    if (!fs.existsSync(path.resolve(p))) {
      problems.push(`${DOC}:${s.line}  "${s.title}" cites a pin that does not exist: ${p}`);
    }
  }
}

if (sections.length === 0) {
  console.error('[deliberate-divergences-check] no `## ` sections found — the parser or the doc changed');
  process.exit(1);
}

if (problems.length) {
  console.error(problems.join('\n'));
  console.error(
    `\n[deliberate-divergences-check] ${problems.length} problem(s) across ${sections.length} recorded divergence(s).\n` +
      'Every recorded divergence needs a test that fails if the behaviour changes; cite it as a\n' +
      'backticked repository path under a `tests/` directory or in `compatibility/pattern-corpus/`.',
  );
  process.exit(1);
}

console.log(
  `[deliberate-divergences-check] ${sections.length} recorded divergence(s), each pinned by an existing test.`,
);
