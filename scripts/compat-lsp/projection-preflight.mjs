// Can the official server project the documents this run is about to compare?
//
// `DocumentSnapshot.ts:241` hands `svelte2tsx` the `parse` and `version` of the
// Svelte the server resolved; when it throws, `:291` replaces the projection
// with the instance script alone — no template — and every completion for that
// document is built with `isIncomplete: true`. The response is well formed, so
// the divergence it produces enrols into a shrink-only ratchet as a legitimate
// entry. This is the same predicate `gate-coverage.md` 27m measures with.
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";

/// Fraction of `.svelte` cases the official server cannot project.
///
/// `serverScript` locates both the `svelte2tsx` the server would call and the
/// `svelte` it would call it with, so the answer is a property of the oracle
/// this run actually launched rather than of the checkout.
export function projectionFailures(serverScript, cases) {
  const require = createRequire(path.resolve(serverScript));
  const { svelte2tsx } = require(require.resolve("svelte2tsx"));
  const compiler = require(
    require.resolve("svelte/compiler", {
      paths: [path.dirname(path.resolve(serverScript))],
    }),
  );
  const version = JSON.parse(
    fs.readFileSync(
      require.resolve("svelte/package.json", {
        paths: [path.dirname(path.resolve(serverScript))],
      }),
      "utf8",
    ),
  ).version;
  const failures = [];
  let total = 0;
  for (const entry of cases) {
    const file = entry.file ?? entry.path;
    if (!file || !file.endsWith(".svelte")) continue;
    let text;
    try {
      text = entry.text ?? fs.readFileSync(file, "utf8");
    } catch {
      continue;
    }
    total++;
    try {
      svelte2tsx(text, {
        filename: file,
        isTsFile: true,
        mode: "ts",
        parse: compiler.parse,
        version,
      });
    } catch {
      failures.push(entry.id ?? file);
    }
  }
  return { failures, total, version };
}
