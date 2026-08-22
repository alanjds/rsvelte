---
'@rsvelte/compiler': patch
---

Reject the JavaScript early errors that need the surrounding class, function or label context to decide — a duplicate constructor, `super` outside a method, `super()` outside a derived constructor, an unsyntactic `break` / `continue`, a duplicate label, an undeclared or duplicated private name, `delete` on a private field, a nested `import` / `export`, and a `'use strict'` directive in a function with a non-simple parameter list — which OXC leaves to a semantic pass this pipeline never runs, so each was copied into output no JS parser accepts
