---
"@rsvelte/svelte2tsx": patch
"@rsvelte/compiler": patch
---

svelte2tsx: read `<script generics="…">` by parsing `<{raw}>() => {}` the way upstream `Generics.ts` does, instead of splitting the raw text on commas that only angle brackets protect. A comma at top level of an object type, a tuple, a parameter list or a string literal used to split the constraint, and the fragments were emitted as extra type *arguments* — `ReturnType<typeof $$render<T,b:>>`, text no TypeScript parser accepts, from ordinary TypeScript. "Does this component have generics?" is now the same two decisions upstream makes: the raw attribute reaches `$$render`, while the component export keys on the type parameters the parse recognised, so an attribute that is not a type parameter list no longer invents a parameter name out of its leading token.

`$$Generic` also raises the three errors upstream raises — in a module script, next to a `generics` attribute, and with more than one type argument — and `export type T = $$Generic` is recognised, so the alias is stripped and `T` becomes a type parameter instead of surviving into the render body.

A standalone `{#snippet}` whose header has anything between the name and its first parameter (a formatted multi-line parameter list, a type parameter list, a space) is padded by the number of gaps upstream's `transform()` collapses rather than by a measurement of the region before `}` — it was one space short, which shifted every later source-map mapping.
