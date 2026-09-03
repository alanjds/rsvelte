---
'@rsvelte/compiler': patch
---

A legacy `$$props` read inside a prop's default value now resolves against `$$sanitized_props`. Upstream rewrites `$$props` reads through the AST, so the binding position a generated `$.prop` / `$.bind_prop` / `$.legacy_rest_props` call occupies is left alone without a rule; rsvelte skipped every line carrying one of those calls, which also skipped the genuine read in a default-value thunk.
