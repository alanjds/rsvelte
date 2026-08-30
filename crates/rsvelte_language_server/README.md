# `rsvelte_language_server`

The Svelte language server, as a Rust binary calling `rsvelte_core` directly.

## Vendored data

The HTML surface (tags, attributes, value sets, and the prose each carries) is
not written here — it is generated from the data the **official** language
server itself loads, so a completion's documentation is the same text on both
sides rather than a paraphrase.

| | |
|---|---|
| Package | [`vscode-html-languageservice`](https://github.com/microsoft/vscode-html-languageservice) |
| Version | `5.4.0`, the version `submodules/language-tools/pnpm-lock.yaml` pins |
| Licence | MIT (Microsoft Corporation) |
| Generated file | `src/html_data/web.rs` |
| Oracle fixture | `tests/data/html-documentation.json` |

Read from the `umd` build, which is what the package's `package.json` `main`
resolves to and therefore what the official server loads — the `esm` copy of the
same data hashes differently:

| File | SHA-256 |
|---|---|
| `lib/umd/languageFacts/data/webCustomData.js` | `34c1cf092562346e6a40a50567b6b22f0139981fe07f46d7f357820e4d2ecfd5` |
| `lib/umd/languageFacts/dataProvider.js` | `ae8c30b8cc165afd538198dac6b607f8a46b9d98624ee6811cc8ca86982be0d4` |

Regenerate both the table and the fixture with:

```bash
node scripts/dev/generate-html-data.mjs
```

It reads the version out of the lockfile and refuses to run against a package
that disagrees with it, so the pin lives in the repository rather than in
whatever happens to be installed. The package must be installed under
`submodules/language-tools` (`pnpm install` there); `--package-root <dir>`
points it elsewhere.

`generateDocumentation` and the baseline helpers it calls are **ported**, in
`src/html_data/documentation.rs`. Ports of one upstream function are the defect
class this repository has paid for most often, so the port is compared to the
function itself over all 607 entries the data holds, in both documentation
formats, by `tests/html_documentation.rs`.
