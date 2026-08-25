---
"@rsvelte/compiler": patch
---

Fold client template expressions through the shared typed evaluator so `void` of an unknown operand has the known value `undefined`.
