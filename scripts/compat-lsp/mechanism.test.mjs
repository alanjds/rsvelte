import test from "node:test";
import assert from "node:assert/strict";
import { MECHANISMS, classifyDivergence } from "./mechanism.mjs";

const hover = (contents) => ({ contents });
const ts = (text) => hover("```typescript\n" + text + "\n```");
const plain = (value) => hover({ kind: "plaintext", value });
const CSS_DOC = plain(
  "The scale CSS property ...\n\nMDN Reference: https://developer.mozilla.org/docs/Web/CSS/scale",
);
const HTML_DOC = plain(
  "The div element ...\n\nMDN Reference: https://developer.mozilla.org/docs/Web/HTML/Reference/Elements/div",
);
const classify = (method, left, right, difference = "/contents:value-mismatch") =>
  classifyDivergence(method, left, right, difference);

test("every label the classifier can emit is declared", () => {
  assert.equal(new Set(MECHANISMS).size, MECHANISMS.length);
  assert.ok(MECHANISMS.includes("unclassified"));
});

test("hover: the two language-data providers are told apart", () => {
  assert.equal(classify("textDocument/hover", CSS_DOC, plain("`scale` CSS property")), "css-data");
  assert.equal(classify("textDocument/hover", CSS_DOC, null), "css-data");
  assert.equal(classify("textDocument/hover", HTML_DOC, null), "html-data");
});

test("hover: language data on one side and TypeScript on the other is routing", () => {
  assert.equal(classify("textDocument/hover", HTML_DOC, ts("const a: number")), "provider-routing");
  assert.equal(classify("textDocument/hover", ts("const a: number"), HTML_DOC), "provider-routing");
});

test("hover: a different symbol is not a rendering difference", () => {
  assert.equal(
    classify("textDocument/hover", ts("(method) String.replace(): string"), ts("(method) String.replace(x): string")),
    "ts-render",
  );
  assert.equal(
    classify("textDocument/hover", ts("var undefined"), ts("(property) undefined: undefined")),
    "ts-symbol-kind",
  );
  assert.equal(
    classify("textDocument/hover", ts('module "svelte/elements.js"'), ts('module "svelte/elements"')),
    "ts-symbol-name",
  );
});

test("hover: an equal payload leaves only the response range", () => {
  assert.equal(
    classify(
      "textDocument/hover",
      { contents: "```typescript\nconst a: number\n```", range: { start: { line: 1, character: 0 } } },
      { contents: "```typescript\nconst a: number\n```", range: { start: { line: 1, character: 4 } } },
    ),
    "projection-response-range",
  );
});

const link = (uri, line, character) => ({
  targetUri: uri,
  targetRange: { start: { line, character }, end: { line, character } },
  targetSelectionRange: { start: { line, character }, end: { line, character } },
  originSelectionRange: { start: { line: 0, character: 0 }, end: { line: 0, character: 1 } },
});

test("definition: the TypeScript lib copy is not a target mismatch", () => {
  assert.equal(
    classify(
      "textDocument/definition",
      [link("file:///nm/typescript/lib/lib.es5.d.ts", 10, 4)],
      [link("file:///nm/native-preview/lib/lib.es5.d.ts", 10, 4)],
      ":extra-rsvelte",
    ),
    "ts-lib-copy",
  );
});

test("definition: a shadow official alone answers about is its own class", () => {
  assert.equal(
    classify(
      "textDocument/definition",
      [link("file:///ws/a.svelte.ts", 3, 1)],
      [link("file:///ws/a.svelte", 3, 1)],
      ":extra-rsvelte",
    ),
    "official-defect-svelte-ts-shadow",
  );
});

test("definition: same file is a position defect, another file is a target defect", () => {
  assert.equal(
    classify("textDocument/definition", [link("file:///ws/a.svelte", 71, 28)], [link("file:///ws/a.svelte", 71, 1)], ":extra-rsvelte"),
    "projection-target-position",
  );
  assert.equal(
    classify("textDocument/definition", [link("file:///ws/types.ts", 0, 12)], [link("file:///ws/a.svelte", 71, 1)], ":extra-rsvelte"),
    "target-file-mismatch",
  );
  assert.equal(classify("textDocument/definition", [], [link("file:///ws/a.svelte", 1, 1)], ":extra-rsvelte"), "official-empty");
  assert.equal(classify("textDocument/definition", [link("file:///ws/a.svelte", 1, 1)], [], ":missing-rsvelte"), "rsvelte-empty");
});

test("completion: the field the pointer names decides the label", () => {
  const both = { items: [{ label: "a" }] };
  assert.equal(
    classify("textDocument/completion", both, both, "/items/@x/commitCharacters:extra-rsvelte-element[count=1,hash=x]"),
    "completion-commit-characters-value",
  );
  assert.equal(
    classify("textDocument/completion", both, both, "/items/@x/commitCharacters:extra-rsvelte-field[hash=x]"),
    "completion-commit-characters-presence",
  );
  assert.equal(classify("textDocument/completion", both, both, "/items/@x/textEdit/range/end/character:value-mismatch"), "completion-text-edit");
  assert.equal(classify("textDocument/completion", both, both, "/items:missing-rsvelte-element[count=1,hash=x]"), "completion-item-set");
  assert.equal(classify("textDocument/completion", both, both, "/isIncomplete:value-mismatch"), "completion-is-incomplete");
});
