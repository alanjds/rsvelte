---
"@rsvelte/compiler": patch
---

Recognise a legacy `$:` label whose `$` and `:` are separated by whitespace or a comment. The client pipeline matched the literal two bytes `$:`, while the official compiler matches a `LabeledStatement` whose label is `$` — so `$ : x = a` stayed a bare label in the output, ran once at init instead of reactively, and shifted every later reactive statement's dependency list by one.
