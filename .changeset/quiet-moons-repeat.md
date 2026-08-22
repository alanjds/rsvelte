---
'@rsvelte/compiler': patch
---

fix(analyze): declare the synthetic `$$restProps` binding in legacy mode

Upstream declares both `$$props` and `$$restProps` as synthetic `rest_prop`
bindings before the legacy-mode walks. rsvelte declared only `$$props`, so a
`$$restProps` reference contributed no dependency: a pure call over it was
never memoized into the `$.template_effect` dependency array, and an
`{#each Object.keys($$restProps)}` did not lower its item to a signal.
