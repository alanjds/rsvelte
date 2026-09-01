// One mechanism label per divergence, drawn from a closed vocabulary.
//
// The label goes into the ratchet key, so it must be derived from the OBSERVED
// pair alone and must not encode the measured content: a key carrying a digest
// of the difference changes the moment a mechanism is partly fixed, and CI
// reads the new key as a NEW failure instead of as progress.
export const MECHANISMS = [
  // Architectural: rsvelte proxies tsgo, official bundles `typescript`.
  "ts-lib-copy",
  "ts-render",
  "ts-symbol-kind",
  "ts-symbol-name",
  // official answers about a `*.svelte.ts` shadow that exists in no editor.
  "official-defect-svelte-ts-shadow",
  // Language-data providers (CSS / HTML) rather than TypeScript.
  "css-data",
  "html-data",
  "provider-routing",
  // One side declines to answer.
  "rsvelte-empty",
  "official-empty",
  // rsvelte's `.svelte` <-> `.tsx` position projection.
  "projection-origin-range",
  "projection-target-position",
  "projection-response-range",
  "target-file-mismatch",
  // completion payload fields.
  "completion-item-set",
  // Measured on melt-ui: the arrays differ (18.9% of label-paired items, upstream
  // omits the `(` at a new-identifier location) and upstream omits the field
  // outright (8.1%) are two mechanisms one label hid.
  "completion-commit-characters-value",
  "completion-commit-characters-presence",
  "completion-command",
  "completion-text-edit",
  "completion-item-data",
  "completion-is-incomplete",
  "completion-item-detail",
  "unclassified",
];

const MECHANISM_SET = new Set(MECHANISMS);

const isEmptyResult = (value) =>
  value === null ||
  value === undefined ||
  (Array.isArray(value) && value.length === 0);

const asList = (value) =>
  Array.isArray(value) ? value : value === null || value === undefined ? [] : [value];

const targetUri = (item) => String(item.targetUri ?? item.uri ?? "");
const targetStart = (item) => (item.targetSelectionRange ?? item.range ?? {}).start;

// The two servers load two different copies of the TypeScript lib: official
// resolves `typescript/lib/lib.*.d.ts`, rsvelte gets the copy tsgo ships.
const isLibFile = (uri) => /\/lib\/lib\.[^/]*\.d\.ts$/.test(uri);
const isSvelteShadow = (uri) => /\.svelte\.ts$/.test(uri);

// rsvelte's CSS hover is a name-only stub; official serves the MDN description.
const RSVELTE_CSS_STUB = /^`[^`]+` CSS property$|^`:global\(\.\.\.\)` prevents/;
const plaintextOf = (contents) =>
  contents && typeof contents === "object" && !Array.isArray(contents) &&
  contents.kind === "plaintext"
    ? contents.value
    : null;

function hoverDataKind(hover) {
  if (!hover) return null;
  const contents = hover.contents;
  // A CSS selector hover is upstream's only array-shaped payload.
  if (Array.isArray(contents)) return "css";
  const text = plaintextOf(contents);
  if (text === null) return null;
  if (RSVELTE_CSS_STUB.test(text)) return "css";
  if (text.includes("developer.mozilla.org/docs/Web/CSS/")) return "css";
  if (text.includes("developer.mozilla.org/docs/Web/HTML/")) return "html";
  // The MDN reference line is the only reliable discriminator; a plaintext
  // payload without one is language data of an unknown flavour.
  return "data";
}

// `(method) String.replace`, `var undefined`, `module "svelte/elements"` — the
// leading declaration line names the symbol the server resolved, which
// separates "the same symbol rendered differently" from "a different symbol".
function declarationHead(text) {
  const fenced = /```typescript\n([\s\S]*?)(?:\n```|$)/.exec(text);
  const first = (fenced ? fenced[1] : text).split("\n")[0];
  const tagged = /^\(([^)]*)\)\s*(.*)$/.exec(first);
  if (tagged) {
    const name = /^([A-Za-z_$][\w$]*(?:\.[A-Za-z_$][\w$]*)*)/.exec(tagged[2]);
    return { kind: tagged[1], name: name ? name[1] : "" };
  }
  const keyword =
    /^(var|let|const|function|class|interface|type|namespace|module|enum|import|new|abstract class)\s+(.*)$/.exec(
      first,
    );
  if (keyword) {
    const name = /^(["'][^"']*["']|[A-Za-z_$][\w$]*)/.exec(keyword[2]);
    return { kind: keyword[1], name: name ? name[1] : "" };
  }
  return { kind: "?", name: first.slice(0, 32) };
}

function classifyHover(official, rsvelte) {
  if (isEmptyResult(official) && !isEmptyResult(rsvelte)) return "official-empty";
  if (!isEmptyResult(official) && isEmptyResult(rsvelte)) {
    const kind = hoverDataKind(official);
    if (kind === "css") return "css-data";
    if (kind === "html") return "html-data";
    return "rsvelte-empty";
  }
  if (isEmptyResult(official) || isEmptyResult(rsvelte)) return "unclassified";
  const left = official.contents;
  const right = rsvelte.contents;
  const leftKind = hoverDataKind(official);
  const rightKind = hoverDataKind(rsvelte);
  if (JSON.stringify(left) === JSON.stringify(right)) return "projection-response-range";
  // One side answered with language data and the other with TypeScript.
  if ((leftKind === null) !== (rightKind === null)) return "provider-routing";
  if (leftKind !== null) {
    if (leftKind === "css" || rightKind === "css") return "css-data";
    if (leftKind === "html" || rightKind === "html") return "html-data";
    return "unclassified";
  }
  if (typeof left !== "string" || typeof right !== "string") return "unclassified";
  const leftHead = declarationHead(left);
  const rightHead = declarationHead(right);
  if (leftHead.name !== rightHead.name) return "ts-symbol-name";
  if (leftHead.kind !== rightHead.kind) return "ts-symbol-kind";
  return "ts-render";
}

function classifyDefinition(official, rsvelte, difference) {
  if (isEmptyResult(official) && !isEmptyResult(rsvelte)) return "official-empty";
  if (!isEmptyResult(official) && isEmptyResult(rsvelte)) return "rsvelte-empty";
  const left = asList(official);
  const right = asList(rsvelte);
  if (!left.length || !right.length) return "unclassified";
  // A field-level pointer names the field directly.
  if (difference.includes("originSelectionRange")) return "projection-origin-range";
  const identity = (item) => {
    const start = targetStart(item) ?? {};
    return `${targetUri(item)}|${start.line}:${start.character}`;
  };
  const leftKeys = new Set(left.map(identity));
  const rightKeys = new Set(right.map(identity));
  const onlyLeft = [...leftKeys].filter((key) => !rightKeys.has(key));
  const onlyRight = [...rightKeys].filter((key) => !leftKeys.has(key));
  if (!onlyLeft.length && !onlyRight.length) return "projection-origin-range";
  const uriOf = (key) => key.slice(0, key.lastIndexOf("|"));
  const all = [...onlyLeft, ...onlyRight];
  if (all.every((key) => isLibFile(uriOf(key)))) return "ts-lib-copy";
  if (onlyLeft.some((key) => isSvelteShadow(uriOf(key))))
    return "official-defect-svelte-ts-shadow";
  if (new Set(all.map(uriOf)).size === 1) return "projection-target-position";
  return "target-file-mismatch";
}

const COMPLETION_POINTERS = [
  [/\/commitCharacters:(extra|missing)-rsvelte-field/, "completion-commit-characters-presence"],
  [/\/commitCharacters(:|$)/, "completion-commit-characters-value"],
  [/\/command(:|$)/, "completion-command"],
  [/\/(textEdit|additionalTextEdits)(\/|:|$)/, "completion-text-edit"],
  [/\/data(\/|:|$)/, "completion-item-data"],
  [/^\/isIncomplete:/, "completion-is-incomplete"],
  [/\/(detail|documentation|labelDetails)(\/|:|$)/, "completion-item-detail"],
  [/^\/items:(extra|missing)-rsvelte/, "completion-item-set"],
];

function classifyCompletion(official, rsvelte, difference) {
  for (const [pattern, label] of COMPLETION_POINTERS)
    if (pattern.test(difference)) return label;
  if (isEmptyResult(official?.items) && !isEmptyResult(rsvelte?.items))
    return "official-empty";
  if (!isEmptyResult(official?.items) && isEmptyResult(rsvelte?.items))
    return "rsvelte-empty";
  return "unclassified";
}

export function classifyDivergence(method, official, rsvelte, difference) {
  let label;
  if (method === "textDocument/hover") label = classifyHover(official, rsvelte);
  else if (method === "textDocument/definition")
    label = classifyDefinition(official, rsvelte, difference);
  else if (method === "textDocument/completion")
    label = classifyCompletion(official, rsvelte, difference);
  else label = "unclassified";
  // A label outside the vocabulary would silently create ratchet keys nobody
  // can enumerate, so it is a defect in this module rather than a new class.
  if (!MECHANISM_SET.has(label))
    throw new Error(`mechanism "${label}" is not in the declared vocabulary`);
  return label;
}

export function classifyDivergences(method, official, rsvelte, differences) {
  const labels = new Set();
  for (const difference of differences)
    labels.add(classifyDivergence(method, official, rsvelte, difference));
  return [...labels].sort();
}
