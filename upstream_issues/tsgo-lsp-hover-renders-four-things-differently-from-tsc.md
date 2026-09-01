# `tsgo --lsp` renders four things in `textDocument/hover` differently from `tsc`'s quick info

`rsvelte-language-server` proxies a child `tsgo --lsp` for TypeScript features, while the official
`svelte-language-server` calls the bundled `typescript` package's `LanguageService` directly. The
two are meant to answer the same question, and for hover they mostly do — but four renderings
differ, and every one of them reaches a user as a different hover card for identical source.

The four are reported together because they are one component (the quick-info renderer) and one
input file reproduces all of them.

## Reproduction

`src/probe.ts`, checked with `{"target":"ES2022","module":"ESNext","moduleResolution":"Bundler","strict":true,"skipLibCheck":true}`:

```ts
export const inlineUnion: "value" | "highlighted" = "value";
export const inlineUnion2: "Movies & TV" | "Anime & Manga" | "Games" | "Music" = "Games";

export const singleQuoted: () => ReturnType<import('svelte').Snippet> = () => {
  throw new Error("probe");
};

export function outer() {
  function classes(list: string): string[] {
    return list.split(" ");
  }
  return classes;
}

/**
 * @default false
 */
export const flagged = false;
```

`tsc` side: `ts.createLanguageService(...).getQuickInfoAtPosition(file, offset)`, then
`ts.displayPartsToString(info.displayParts)` and `info.tags`, with `typescript@6.0.3` — the copy
`submodules/language-tools/packages/language-server` resolves.

`tsgo` side: `tsgo --lsp -stdio`, `initialize` + `didOpen` + `textDocument/hover` at the identical
position, reading `contents.value`.

## The four differences

| position | `tsc` 6.0.3 | `tsgo --lsp` |
|---|---|---|
| `inlineUnion` | `const inlineUnion: "value" \| "highlighted"` | `const inlineUnion: "highlighted" \| "value"` |
| `inlineUnion2` | `const inlineUnion2: "Movies & TV" \| "Anime & Manga" \| "Games" \| "Music"` | `const inlineUnion2: "Anime & Manga" \| "Games" \| "Movies & TV" \| "Music"` |
| `singleQuoted` | `const singleQuoted: () => ReturnType<import("svelte").Snippet>` | `const singleQuoted: () => ReturnType<import('svelte').Snippet>` |
| `classes` (its use on the `return` line) | `(local function) classes(list: string): string[]` | `function classes(list: string): string[]` |
| `flagged` | `const flagged: false` **plus** `tags = [{name: "default", text: "false"}]` | `const flagged: false` **plus** the literal text `*@default* — false` appended to the hover body |

1. **Union members are sorted.** `tsc` prints a union in declaration order; `tsgo` prints it
   alphabetically. Both examples above are ordinary string-literal unions with no `keyof`, no
   intersection and no conditional type.
2. **A dynamic import's module specifier keeps the source's quote spelling.** `tsc` normalizes to
   `"`; `tsgo` echoes whatever the source wrote. The same declaration written with `"` renders
   identically on both sides, which is why this only appears in sources that use `'`.
3. **The `(local function)` modifier is dropped.** `tsc` marks a function declared inside another
   function's body; `tsgo` renders it as a plain `function`.
4. **JSDoc tags are inlined into the hover body rather than returned separately.** This one is
   arguably a protocol-shape choice rather than a defect, but it means a client cannot render tags
   its own way, and the resulting markdown differs (`*@default* — false` versus a `tags` array).

## What does NOT differ

Two renderings that a plausible reading of the symptom would attribute here, and which this probe
shows are **not** tsgo/tsc differences — recorded so they are not attributed to this report:

- `(property) type: "boolean"` for `const literal = { type: "boolean" } as const;` — **identical**
  on both sides.
- `const bindings: Bindings` for a union behind a type alias — **identical** on both sides; the
  alias is not expanded by either, so the sorting difference above needs an *inline* union to be
  visible at all.

## Where rsvelte stands

These are `rsvelte-language-server` hover divergences against the official server in
`compatibility/lsp-known-failures.json`. rsvelte cannot fix them without re-rendering tsgo's
quick-info text, which would mean re-implementing the renderer it delegates to. The entries stay
listed and attributed here.

The first probe written for the quote-style row used `"` in the source and reproduced nothing on
either side, which is a fact about the probe rather than about the renderer — the row is only
reachable from a source that spells the specifier with `'`.
