# Sourcemap CSS samples

Extra `sourcemaps` samples, compiled by `scripts/fixtures/generate-fixtures.mjs`
alongside `packages/svelte/tests/sourcemaps/samples` and consumed by
`crates/rsvelte_core/tests/sourcemaps_gate.rs`.

They exist because adding the CSS arm to that gate measures nothing on its own.
Of the 13 upstream sourcemaps samples that contain a `<style>` block, **none has
a nested rule** (`&`, or a rule inside `@media`), and a nested rule is where
rsvelte's CSS map lost every segment (#3505). A comparison and a population that
can discriminate had to land together — the same shape as the `bind:` matrix
family, which measured nothing until `run.mjs` started comparing error *codes*.

`css-flat-rule` is the negative control: its mappings were already byte-identical
to the official compiler's, so it must stay that way.
