---
"@rsvelte/compiler": patch
---

Fold client template expressions through the shared typed evaluator so `void` of an unknown operand has the known value `undefined` and dev equality expressions in binding initializers remain foldable.
