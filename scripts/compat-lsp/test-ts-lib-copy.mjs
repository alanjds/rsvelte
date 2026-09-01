#!/usr/bin/env node
// Pins the `A `lib.d.ts` definition lands in the copy that answered it`
// entry of `compatibility/GATES.md#deliberate-divergences`, and with it the
// `ts-lib-copy` label of `compatibility/lsp-known-failures.json`.
//
// Three things this has to do that a naive version does not. The platform
// triple is never written down, because CI is linux-x64 and this machine is
// darwin-arm64. A non-lib definition is asserted to land on the SAME entity on
// both sides, so "every target is a different copy" cannot pass. And official
// is required to answer from the `typescript` package: the run-level oracle
// control that guards the rest of this gate does not run here, and a degraded
// official server would otherwise make "the two differ" quietly become "the two
// agree", which passes a test whose whole subject is a difference.
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { LspProcess } from "./protocol.mjs";
import { resolveTsgo } from "./tsgo.mjs";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const OFFICIAL = [
  "node",
  path.join(ROOT, "submodules/language-tools/packages/language-server/bin/server.js"),
  "--stdio",
];
// Same override the rest of this directory honours, so a release build can be
// pointed at without editing the test.
const RSVELTE = process.env.RSVELTE_LSP_COMMAND
  ? JSON.parse(process.env.RSVELTE_LSP_COMMAND)
  : [path.join(ROOT, "target/debug/rsvelte-language-server")];

// Resolved, because macOS's `/var` is a symlink and one server echoes the
// client's URI while the other resolves it — a fixture artifact, not the subject.
const dir = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "ts-lib-copy-")));
fs.writeFileSync(
  path.join(dir, "tsconfig.json"),
  JSON.stringify({
    compilerOptions: {
      target: "ES2022",
      module: "ESNext",
      moduleResolution: "Bundler",
      strict: true,
      skipLibCheck: true,
    },
  }),
);
const file = path.join(dir, "Probe.svelte");
const text = ['<script lang="ts">', "\tconst local = 1;", "\tconst upper = String(local).toUpperCase();", "\tconst echo = local;", "</script>", "", "{upper}{echo}", ""].join("\n");
fs.writeFileSync(file, text);
const uri = pathToFileURL(file).href;

// `toUpperCase` is declared in a `lib.*.d.ts`; `local` is declared in this file.
const LIB = { line: 2, character: 32 };
const LOCAL = { line: 3, character: 16 };

async function definitions(command, env) {
  const server = new LspProcess(path.basename(command[0]), command, { cwd: ROOT, env });
  let id = 0;
  const request = async (method, params) => {
    const current = ++id;
    server.send({ jsonrpc: "2.0", id: current, method, params });
    return (await server.response(current, () => ({}), 180000)).result;
  };
  await request("initialize", {
    processId: process.pid,
    rootUri: pathToFileURL(dir).href,
    workspaceFolders: [{ uri: pathToFileURL(dir).href, name: "probe" }],
    capabilities: { textDocument: { definition: { linkSupport: true } } },
    initializationOptions: {},
  });
  server.send({ jsonrpc: "2.0", method: "initialized", params: {} });
  server.send({
    jsonrpc: "2.0",
    method: "textDocument/didOpen",
    params: { textDocument: { uri, languageId: "svelte", version: 1, text } },
  });
  const at = async (position) => {
    const result = await request("textDocument/definition", { textDocument: { uri }, position });
    return Array.isArray(result) ? result : result ? [result] : [];
  };
  const out = { lib: await at(LIB), local: await at(LOCAL) };
  // `LspProcess` has no `kill`; without this the two servers outlive the run.
  await server.shutdown(++id, () => ({}));
  return out;
}

const targetOf = (entry) => entry?.targetUri ?? entry?.uri;
const rangeOf = (entry) => entry?.targetRange ?? entry?.range;
const libEntry = (list) => list.find((entry) => /\/lib\.[^/]+\.d\.ts$/.test(targetOf(entry) ?? ""));

const official = await definitions(OFFICIAL, {});
const rsvelte = await definitions(RSVELTE, { TSGO_BIN: resolveTsgo(ROOT) });

const officialLib = libEntry(official.lib);
const rsvelteLib = libEntry(rsvelte.lib);

// The oracle's own control, because the run-level one does not reach here.
assert.ok(
  officialLib && /\/typescript\/lib\/lib\.[^/]+\.d\.ts$/.test(targetOf(officialLib)),
  `official did not resolve \`toUpperCase\` into the \`typescript\` package's lib; the oracle is degraded and this test cannot say anything: ${JSON.stringify(official.lib)}`,
);
assert.ok(rsvelteLib, `rsvelte resolved no lib declaration: ${JSON.stringify(rsvelte.lib)}`);

// The divergence being pinned: the same declaration, in the copy that answered.
assert.equal(
  path.basename(new URL(targetOf(rsvelteLib)).pathname),
  path.basename(new URL(targetOf(officialLib)).pathname),
  "the two servers named different lib FILES, which is not the recorded divergence",
);
assert.deepEqual(
  rangeOf(rsvelteLib),
  rangeOf(officialLib),
  "the two lib copies disagree on where the declaration is; the recorded divergence is the path only",
);
// No platform triple: this string is `darwin-arm64` here and `linux-x64` in CI.
assert.match(
  targetOf(rsvelteLib),
  /native-preview/,
  "rsvelte's lib target is not inside the tsgo distribution",
);
assert.doesNotMatch(
  targetOf(officialLib),
  /native-preview/,
  "official's lib target is inside the tsgo distribution, so there is no divergence left to pin",
);

// The control: away from the lib files the two must name the same entity, or
// "every target is a different copy" would pass the assertions above.
const officialLocal = official.local[0];
const rsvelteLocal = rsvelte.local[0];
assert.ok(officialLocal && rsvelteLocal, "a local declaration resolved on neither or only one side");
assert.equal(
  decodeURIComponent(targetOf(rsvelteLocal)),
  decodeURIComponent(targetOf(officialLocal)),
  "a local declaration lands in different files, so the divergence is not confined to lib copies",
);
assert.deepEqual(rangeOf(rsvelteLocal), rangeOf(officialLocal));

fs.rmSync(dir, { recursive: true, force: true });
console.log(
  `[ts-lib-copy] pinned: both name ${path.basename(new URL(targetOf(officialLib)).pathname)} at the same range, official from \`typescript\` and rsvelte from the tsgo distribution; a local declaration agrees on both sides`,
);
