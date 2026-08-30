# `grass` divides a slash-separated number list under a nested `:not()`

`grid-row: 2/5` is a slash-separated list in modern Sass, not a division: dart-sass 1.103.1
emits it unchanged. `grass` 0.13.4 emits it unchanged almost everywhere too — and evaluates
it as division when the declaration sits in a rule that is **nested inside another rule** and
whose own compound selector carries **`:not(...)`**. The result, `0.4`, is not a valid
`grid-row` value, so the browser drops the declaration.

## Reproduction

```scss
.p { .q:not(.r) { grid-row: 2/5; } }   /* dart-sass: 2/5 — grass: 0.4 */
```

Both conditions are load-bearing, and each of these agrees on both sides:

```scss
.p { .q      { grid-row: 2/5; } }   /* nested, no :not()          -> 2/5 */
.q:not(.r)   { grid-row: 2/5; }     /* :not(), not nested         -> 2/5 */
.q { &:not(.r) { grid-row: 2/5; } } /* :not() reached through `&` -> 2/5 */
.p:not(.r) { .q { grid-row: 2/5; } }/* :not() on the parent       -> 2/5 */
```

`:not` is the only pseudo-class that does it: `:is()`, `:where()`, `:has()`, `:nth-child()`,
`:hover` and an `[attr]` selector in the same slot all keep the list. The property is not
involved (`margin: 2/5` diverges identically), and three levels of nesting behave as two.

Measured with dart-sass 1.103.1 and `grass` 0.13.4 through
`crates/rsvelte_preprocess/tests/grass_serialisation.rs`.

Two neighbouring cases agree and so are **not** part of this report, though each looks like it
should be: `$n: 2; a { grid-row: $n/5; }` divides on both sides (dart-sass with a `slash-div`
deprecation warning), and `calc(2/5)` folds to `0.4` on both. A report reduced to "grass
divides a slash" would be describing those.

## Where it shows up

1 of the 2 `content-differs` units in the `scss-known-failures` ratchet
(`musicat/src/App.svelte`, two declarations, both under `.queue:not(.panel.queue)` and
`.wiki:not(.panel.wiki)`). It is the only entry in that ratchet whose divergence produces CSS
a browser rejects.
