---
"@rsvelte/compiler": patch
---

Lower a class expression reached through a rune argument or through an `extends` heritage clause. `held = $state(class { deep = $state(1) })` copied its argument text through verbatim, so the nested body never reached the class-field transform and the `$.proxy` wrapper was dropped; `class Sub extends class { … } { … }` mistook the heritage body's `{` for the subclass's, so the subclass's own rune fields stayed plain public fields. The class-header scan is now the shared lexical one (a `class` in a comment or a string is not a header) and it consumes an inline heritage body before looking for the real one; both nested positions are re-scanned as classes of their own, as upstream's ordinary walk reaches them.

The printer no longer parenthesises a class / function / object expression in a heritage position: those are valid `LeftHandSideExpression`s there, and esrap adds no parentheses of its own.

A `$derived` whose argument is a compile-time known value is no longer treated as reactive, so `{rd}` over `$derived(1)` writes `textContent` once instead of templating a text node and a `$.template_effect` — the template string itself differed, so the two hydrated against different DOM. A binding stores a literal initializer as its own source text rather than as node JSON, and the "is this value known" check only understood the JSON form.

A production-mode `$inspect(…)` in a value position keeps its slot filled with `undefined` instead of leaving `let v = ;`, which no JS parser accepts.
