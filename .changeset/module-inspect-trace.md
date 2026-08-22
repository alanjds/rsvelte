---
'@rsvelte/compiler': patch
---

Lower `$inspect.trace(…)` in `.svelte.(js|ts)` modules

A module script had no dev-mode lowering for the rune at all, so `$inspect.trace(…)`
reached the output verbatim and threw `ReferenceError: $inspect is not defined`. The
enclosing function body is now rewritten to `{ return $.trace(label, () => { … }); }`
(awaited for an `async` function), with the default label taken from the function's own
AST parent and located in the source the user wrote. The non-dev removal was a `memmem`
scan that also deleted the same bytes out of a string literal, in both the module and the
component instance path; both are lexical now.
