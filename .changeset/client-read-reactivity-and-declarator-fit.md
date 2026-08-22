---
"@rsvelte/compiler": patch
---

Client output now matches upstream on four shapes: a synthesized `$state()` destructuring declaration breaks across lines at the same 50-column boundary esrap uses; a `{@const}` bound to a function keeps its `template_effect` (and its text placeholder); a never-written `$state` / `$derived` read under `customElement` is written once instead of through an effect; and a `{@const}` reading an enclosing `{@const}` through a pure global (`String(w)`) folds to a direct `nodeValue` assignment.
