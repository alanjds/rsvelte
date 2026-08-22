---
'@rsvelte/compiler': patch
---

Fix injected CSS minification and CSS source maps for nested rules

The CSS printer reconstructed a nested or minified block instead of copying it
out of the source. Reconstruction produced two defects that no gate could see at
once, because they live in different outputs of the same code path:

- the injected stylesheet (`css: "injected"`, and every `customElement`
  component) gained a synthesized `;` after every declaration and, for a nested
  rule, an extra `{` with no matching `}` — the stylesheet was not parseable CSS
- every byte emitted by the nested path carried no source position, so `css.map`
  lost every segment inside a nested rule while `css.code` stayed byte-identical

Nested blocks, at-rules and `:global {}` bodies now copy from the source with the
mapped writer, and minification is upstream's four `remove_preceding_whitespace`
edits plus the run after `property:` rather than a rebuild.
