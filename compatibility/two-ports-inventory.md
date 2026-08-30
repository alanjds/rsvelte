# One upstream decision, N rsvelte implementations

A companion to [`gate-coverage.md`](gate-coverage.md). That document is indexed by **gate** and
asks what each gate does not look at. This one is indexed by **decision** and asks a question no
gate here is shaped to ask:

> The official compiler answers this question in one function. How many times does rsvelte
> answer it, which inputs reach which answer, and **is there anything that compares the answers
> to each other?**

Every gate in this repository compares rsvelte to *upstream* on some population. None compares
rsvelte to *itself*. So when one upstream function is ported twice, the second port is only ever
exercised on whatever inputs a real file happens to supply, and a shape that separates the two
has to be published before anyone sees it. That is the mechanism behind #3027, and on
**2026-08-22 four more instances were reported on the same day by four different people working
in four different files** — #3403 (CSS matching), #3427 (CSS pruning across phases), #3472
(console-argument shape), #3569 (`has_call`'s writers). This file exists because that is a
recurring class and not a coincidence.

## How to read a row

Each row carries an **evidence grade**, and the grades are not interchangeable:

| grade | means | what it takes |
|---|---|---|
| **[S]** structural | two implementations of one decision exist | file:line citations for each |
| **[D]** demonstrated | the two provably answer differently | the differing code **and a named input** |
| **[M]** measured | both were run and compared on real inputs | a harness, a denominator, a result |

The letters extend [`gate-coverage.md`](gate-coverage.md)'s vocabulary rather than
competing with it: **[S]** is its structural argument from code and **[D]** is its
discriminating case, one level down (the case discriminates two *ports* instead of a gate's
green from a correct gate's red). **[M]** has no counterpart there, because that file's rows
describe what a gate cannot see and this file's rows describe something nobody has run.

**"There are two ports" and "the two disagree" are separate claims** — the first is an argument
from code, the second needs an input. Do not soften an [S] into a [D] because a divergence looks
likely; write `未測定` for the divergence and leave the row at [S]. An unsupported claim here is
worse than a blank, because the next person reads the row as surveyed.

**No row below is [M], and that is the finding rather than an omission.** Nothing in this tree
runs two ports of one decision against each other and compares the results — with exactly one
exception, § *The one place this is already defended*, which is the template for closing a row.

Grading a row [D] from code alone is deliberate and it is weaker than it looks: it says the two
functions *would* answer differently on that input, not that the input is reachable through the
compiler's own routing. **Reachability is a separate question from correctness** — several rows
below name an input whose reachability is untested, and they say so.

## The one place this is already defended

`expression_has_reactive_state` (`3_transform/client/visitors/shared/utils.rs:5063`), its typed
front end `typed_has_reactive_state` (`:5486`) and the JSON walk `has_reactive_state_json`
(`:5654`) are three implementations of one decision — and a test runs two of them on the same
input and compares:

```rust
fn both_has_reactive_state(expr_src: &str) -> (bool, bool) { … }

#[test]
fn typed_reactive_state_front_end_agrees_with_the_json_walk() {
    // (expression, expected answer) — expectations are spelled out as well
    // as compared, so a front end that always says `false` can't pass by
    // agreeing with an equally broken oracle.
```

Two properties make it worth copying rather than admiring. It compares the **ports to each
other**, which no gate does. And it **also pins the expected answer independently**, so the test
cannot pass by having both ports be broken in the same direction — the failure mode that a
port-vs-port comparison has and an upstream-vs-rsvelte comparison does not. A differential test
whose oracle is the other implementation is only as good as its independent expectations.

## Inventory

| # | decision | ports | grade | closed? |
|---|---|---|---|---|
| [1](#1-which-estree-object-does-a-function-declaration-serialize-to--d) | Which ESTree object does a `function` declaration serialize to? | 4 | **[D]** | no |
| [2](#2-is-this-callee-a-rune-and-which-one--d) | Is this callee a rune, and which one? | 3 name tables (+ ≥7 lookup impls) | **[D]** | no |
| [3](#3-is-this-assignments-rhs-a-known-primitive--d) | Is this assignment's RHS a known primitive? | 3 | **[D]** | no |
| [4](#4-which-trailing-global-are-truncated-before-matching--d) | Which trailing `:global(...)` are truncated before matching? | 2 | **[D]** | no |
| [5](#5-is-this-fragment-standalone--d) | Is this fragment standalone? | 2 | **[D]** | no |
| [6](#6-is-this-byte-code-or-comment--string--template--regex--d) | Is this byte code, or comment / string / template / regex? | 2 predicates + ≥8 inline copies | **[D]** | no |
| [7](#7-does-this-element-match-this-selector--d-one-pair-closed) | Does this element match this selector? | 4 in phase 2 | **[D]** | #3403 fixed one pair |
| [8](#8-where-does-the-scoping-class-go-inside-a-compound--d-open-as-3402) | Where does the scoping class go inside a compound? | 2 | **[D]** | #3402 open |
| [9](#9-is-this-expressions-value-known--defined--d) | Is this expression's value known / defined? | ≥6 | **[D]** | no |
| [10](#10-which-line-and-column-is-byte-offset-n-on--d) | Which line and column is byte offset N on? | 4 tables | **[D]** | no |
| [11](#11-does-this-expression-contain-a-call--s) | Does this expression contain a call? | 4 | **[S]** | #3569 open |
| [12](#12-selector-unused-and-element-scoped-are-two-engines-over-two-element-models--s) | "Selector unused" vs "element scoped" | 2 engines, 2 element models | **[S]** | no |
| [13](#13-what-does-a-call-to-one-of-upstreams-globals-keypaths-evaluate-to--d-closed-by-degree-1) | What does a call to one of upstream's `globals` keypaths evaluate to? | 2 tables | **[D]** | closed by #3471 (degree 1) |
| [14](#14-what-options-does-the-public-parse-run-with--d) | What options does the public `parse()` run with? | 2 bindings | **[D]** | #3688 open |
| [15](#15-how-are-public-compile-options-validated--d) | How are public compile options validated? | 3 bindings | **[D]** | #3664 defended at degree 2 |
| [16](#16-what-is-the-read-form-of-a-name-inside-an-invalidate_inner_signals-body--d) | What is the read form of a name inside an `$.invalidate_inner_signals` body? | 2 | **[D]** | no |
| [17](#17-does-this-write-target-resolve-to-the-components-binding-or-to-a-shadow--d) | Does this write target resolve to the component's binding, or to a shadow? | 44 rewrite passes, 8 scope-aware | **[D]** | 4 ports closed at degree 1 |

---

### 1. Which ESTree object does a `function` declaration serialize to? — [D]

**Upstream:** one `acorn.parse` (`phases/1-parse/acorn.js:25`). Position in the source cannot
change the shape of the node it produces.

**Ports.** `convert_function_declaration_as_node`
(`1_parse/read/expression.rs:8344`) has exactly two call sites, and only one of them is guarded:

- `:7502` — `convert_statement_for_program`, the path every `function` declaration inside a
  `<script>` takes. **Unguarded.**
- `:8508` — `convert_declaration_for_program_as_node`, the `export`ed path, guarded by
  `&& func_decl.params.rest.is_none()`, which falls through to the Value form
  `convert_declaration_for_program` (`:8578`) when a rest parameter is present.

**The disagreement is documented in the tree, by both sides.** The typed converter's own doc
comment says rest parameters are not emitted and that callers needing them must route through the
Value form; the guard that routes around it says the typed path "emits only `params.items`, so a
rest parameter would be dropped relative to the Value form — keep Raw in that case."

So `export function f(...a) {}` serializes with a `RestElement` in `params`
(`expression.rs:8622-8639`) and `function f(...a) {}` — the same source minus one keyword — does
not. Two further converters answer the same question: the expression-context arm (`:6202`, which
*does* emit the rest element) and the `export default` arm (`:7548`, which does not).

**Who reads it.** The serialized program is what `rsvelte_lint`'s JSON-walking rules and
svelte2tsx consume; codegen is unaffected. The blast radius is every rule that inspects a
function's parameters.

Closing this means one converter, not four — or, short of that, a test that serializes the same
body in all four positions and asserts the `params` arrays are equal.

### 2. Is this callee a rune, and which one? — [D]

**Upstream:** one `RUNES` array and one `is_rune` in `src/utils.js:437`, with `get_rune`
(`phases/scope.js:1433`) applying one shadowing rule. **18 names.**

**Ports — three tables, and only one of them is upstream's:**

| table | file | missing relative to upstream |
|---|---|---|
| phase 2 | `2_analyze/visitors/shared/function.rs:84` `is_rune` | — (all 18 present) |
| phase 3 client | `3_transform/client/visitors/expression_converter.rs:2141` `RUNES` | `$props.id`, `$bindable`, `$inspect.trace` |
| server | `3_transform/server/evaluate.rs:642` `is_rune` | `$inspect().with`, `$inspect.trace` |

**The two phase-3 tables are not subsets of each other**: the client has `$inspect().with` and
not `$bindable`; the server has `$bindable` and not `$inspect().with`. Only `$inspect.trace` is
missing from both.

Both non-conforming tables carry a comment asserting the equality they break — the server's says
"The full rune list (mirrors `is_rune` in utils.js)", the client's "This function mirrors the
official Svelte compiler's `get_rune`". **A comment claiming fidelity is not evidence of it**,
and here it marks the opposite twice.

Named inputs: `let id = $props.id();` — phase 2 classifies the callee as a rune, the client's
`get_rune_from_call` returns `None`. `$inspect.trace()` — phase 2 says rune, client and server
both say not-a-rune. Whether either shape reaches both sites in one compile is `未測定`.

Above the tables there are at least seven implementations of the *lookup* itself
(`call_expression.rs:21` / `:217`, `shared/utils.rs:733` / `:1171`, `class_body.rs:86`,
`expression_converter.rs:2168` / `:6222`), differing in their shadowing rules —
`class_body.rs:86` has none at all. Those are `未測定`; the table divergence above is not.

### 3. Is this assignment's RHS a known primitive? — [D]

**Upstream:** `Evaluation.is_primitive` (`phases/scope.js:242`), read once, at
`client/visitors/AssignmentExpression.js:180`.

**Ports — three, and one of them states the invariant the other two break:**

- `3_transform/client/assign_dev_ast.rs:56` `is_known_primitive` (oxc `Expression`) — has
  `ConditionalExpression`, `LogicalExpression` and `SequenceExpression` arms.
- `3_transform/client/visitors/expression_converter.rs:5129` `is_known_primitive_json` — has
  none of the three; falls to `_ => false`.
- same file `:5212` `is_known_primitive_jsnode` — likewise none.

The first one's doc comment reads:

> `scope.evaluate(right).is_primitive`, approximated by shape exactly as the template path's
> `is_known_primitive_json` does — **the two must agree or the same source would be wrapped on
> one path and not the other.**

They do not agree. On `obj.x = cond ? 1 : 2` the oxc path skips the dev-mode `$.assign` wrap and
both template paths emit it. **The invariant is written down, the violation is one `match` arm
away, and nothing runs the two functions on one input** — the whole class in a single row.

### 4. Which trailing `:global(...)` are truncated before matching? — [D]

**Upstream:** `css-prune.js:209` `truncate`, one function, one caller
(`get_relative_selectors:172`), which is the single entry point for every matching call in
`prune()`. When every relative selector is global, `findLastIndex` returns `-1` and it returns
the **empty** array.

**Ports — two, with opposite behaviour in exactly that case:**

| | file | all-global input | global predicate |
|---|---|---|---|
| phase 2 | `2_analyze/css_scoping.rs:1184` `truncate_globals` | `&[]` — matches upstream | `is_relative_selector_global:1024` |
| phase 3 | `3_transform/css.rs:2704` `truncate_trailing_globals` | **the input unchanged** | `relative_selector_is_outer_global:2674` |

Both doc-comment themselves as ports of `truncate`; the phase-3 one says so and then documents
its own deviation ("if every selector is global, returns the input unchanged"). On
`:global(.a) :global(.b)` phase 2 truncates to nothing and its callers bail; phase 3 keeps both
relatives and goes on to match `.b` against local elements.

Neither port implements upstream's third behaviour, the `:root…:has()` `.map()` at
`css-prune.js:220-231`. And in `3_transform/css.rs` truncation is **not on the path at all** for
five of the deciders `is_complex_selector_unused_impl` calls — upstream funnels all of them
through `truncate`.

### 5. Is this fragment standalone? — [D]

**Upstream:** `phases/3-transform/utils.js:126` `clean_nodes`, imported by all four visitors —
client `Fragment`, client `RegularElement`, server `Fragment`, server `RegularElement`.

**Ports.** rsvelte's `clean_node_list` (`3_transform/utils.rs:672`) is client-only: every
`clean_nodes` occurrence under `3_transform/server/` is a **comment referring to upstream**, not
a call. The server answers the same question in `3_transform/server/ast/mod.rs:636`
`is_standalone_fragment`, and it differs in two fields:

| | upstream / client | server |
|---|---|---|
| comments | dropped only when `!preserve_comments` (`utils.rs:706`) | `TemplateNode::Comment(_) => false`, **unconditional** (`mod.rs:655`) |
| `DebugTag` | hoisted (`utils.js:157`, `utils.rs:713`) | **absent from the hoist list**, so `_ => true` counts it as a meaningful sibling |

Named inputs: `{#if x}<!-- c --><Foo />{/if}` with `preserveComments: true` — client not
standalone, server standalone. `{#if x}{@debug y}<Foo />{/if}` — client standalone, server not.
Which output each produces for those inputs is `未測定`; the branch difference is not.

This is adjacent to #3376, where a `{@debug}` with no identifiers left a fragment static on the
client. `DebugTag` is a node two independent lists must both remember to name, and one of them
has already forgotten once.

### 6. Is this byte code, or comment / string / template / regex? — [D]

**Upstream:** n/a. Upstream never re-scans raw text; this is a consequence of rsvelte's
text-rewriting pipeline and the reason AGENTS.md carries three separate rows about it.

**Ports.** `3_transform/shared/js_scan.rs:146` `skip_opaque` is one shared predicate with ~30
callers, `class_body::find_class_header` among them — that is a shared helper, **not** an
instance, and it is the shape the other copies should be folded into.

The instance is that **the phase-2 `$`-reference scanner does not use it**.
`2_analyze/store_subscriptions.rs:971` `collect_dollar_identifiers_pass` carries its own
`&[char]` state machine with `in_string`, `in_line_comment`, `in_block_comment`,
`template_stack` and `class_bodies` — and **no regex-literal branch at all**. Measured as a grep
carrying its own positive control in the same invocation: `js_scan.rs` names `regex` 20+ times,
`store_subscriptions.rs` names it **0** times.

Named input: `const r = /\$foo/;` — `js_scan` treats `$foo` as non-code, the store scanner
records it as a store reference. This is the shape of **#2988**, which was fixed by routing the
module rune loop through `js_scan::find_code`; the phase-2 scan answers the same question and
never received that fix. It has already been patched once for a *different* missing case (#3127,
class bodies), which is what an unshared predicate costs: each gap has to be found separately.

`store_subscriptions.rs:1236` `class_body_open` is a third answer to "where does a class body
start", independent of both `skip_opaque` and `find_class_header`, and
`3_transform/server/transform_store.rs` and `server/helpers.rs` carry at least eight more inline
`in_string` / `in_comment` machines. Their input ranges are `undetermined`.

**A fourth pair is worth recording for the opposite reason: the two copies AGREED, and both were
wrong.** `client/class_transforms.rs` splits a class body into member blocks line by line, and
until 2026-08-29 both `parse_section_members` (`is_plain_field`, which excluded a line beginning
`//` or `/*`) and `rejoin_class_members` (which refused to terminate a block on the same two
prefixes) asked "is this line comment text" **per line**. So the continuation lines of anything
spanning lines were members of their own on both, and the two failure modes are different
depending on what spans:

- a multi-line `/* … */` leaves its opening `/**` on the block above, that block is an
  unterminated comment, `private_class_assign_ast` cannot parse it, and every rewrite it owns is
  skipped in silence — on sveltekit's `query/instance.svelte.js` the `??=` lowering of a private
  `$state.raw` field, emitting `$.get(this.#promise) ??= this.#run()`, which no JS parser accepts;
- a multi-line **template literal** parses fine and changes *value*: the member blocks are
  re-emitted with esrap's margins, so a blank line lands inside the string
  (`` `a ${1} b⏎⏎c ${2} d` `` where the source has one line break).

Both are fixed by routing the two through one cross-line predicate,
`js_scan::line_starts_outside_opaque`, which is built on the same `skip_opaque` this row names as
the shape the copies should fold into — so `class_transforms.rs` is now a *user* of that
predicate rather than a further copy of it. Measured over the 589 corpus sources holding both
`class` and a rune (293 compiled by both compilers): the comment half moved 40 files from
divergent to byte-identical on client and 1 on client-dev, and took the population's unparseable
outputs from 1 to 0; folding onto the shared predicate then moved 2 more on client-dev, 0 on
client, and 0 either way in the other direction.

The reusable part is the grade this pair would have earned. It is **[S]**, never [D]: no input
separates the two, because they answered the same question the same wrong way — which is
precisely the failure mode § *The one place this is already defended* names for a port-vs-port
oracle. **A row at [S] whose two ports provably agree is not a closed row**; it is a row whose
divergence test cannot exist, and only an independently pinned expectation (here: the official
compiler's output) can grade it.

One defect this uncovered is **not** in this file's scope and is recorded so it is not
rediscovered here: once a chunk containing a multi-line template literal reaches the in-place AST
rewrite, the reprint **re-indents the template's interior lines**, which is another silent value
change. It reproduces on a binary built before any of today's fixes, so it is pre-existing and
belongs to the printer rather than to the member scan.

### 7. Does this element match this selector? — [D], one pair closed

**Upstream:** `css-prune.js:243` `apply_selector` + `:291` `apply_combinator` + `:436`
`relative_selector_might_apply_to_node`. One implementation, called for every
`(element, selector)` pair.

**Ports — four, in `2_analyze/css_scoping.rs`, partitioned by *filters* rather than by design:**

1. `GMatcher::apply_selector` (`:3220`) — graph-based, faithful. Reached **only** by selectors
   passing `has_sibling_combinator || selector_contains_has || selector_contains_complex_not`
   (`:3629`). A plain `div .a` never reaches it.
2. `complex_selector_matches_element` (`:1699`) → `element_matches_simple_selectors` (`:1097`) —
   element-walking. Reached by everything **except** `:has()` (`:1461`).
3. `static_relative_might_apply` (`:3525`) — a simplified third copy for exactly-two-part sibling
   selectors.
4. `element_is_ancestor_in_matching_selector` (`:1870`) — a fourth, for the ancestor pass;
   upstream has no separate function, it marks ancestors inside `apply_selector`.

**The two filters are not complements**, so a selector with a sibling combinator runs through
both #1 and #2 and the results are OR-ed. And #2 returns `false` outright for `+`/`~` (`:1855`),
deferring to #1 — so #1's filter is load-bearing for #2's correctness.

**#3403 is the demonstrated divergence** and is fixed (PR #3581): #1 truncates globals and falls
back to "assume a match" for a multi-part `:is()` argument, while #2 tested the argument's last
compound. Ports 3 and 4 bottom out in #2 and inherited its answer. The remaining pairs are
`未測定`.

### 8. Where does the scoping class go inside a compound? — [D], open as #3402

**Upstream:** `phases/3-transform/css/index.js:336-365` — **one** loop walking the compound
backwards, emitting the modifier once and `break`ing.

**Ports — two, in `3_transform/css.rs`:**

- `transform_complex_selector` (`:6696`) — iterates **forwards**, with a `*` arm at `:7166` that
  is **positionally unconditional**, plus a second modifier emission at `:7229` gated on the last
  non-pseudo index. Handles every compound **outside** a functional pseudo-class.
- `transform_is_not_complex_selector` (`:7636`), reached from
  `format_simple_selector_with_scope:7393` → `transform_is_not_args:7559` — its `*` arm at
  `:7805` **is** guarded by `Some(idx) == last_non_pseudo_idx`. Handles the `:is()` / `:where()`
  / `:has()` / `:not()` interior.

#3402 measures the consequence: `*.a` prints as `.svelte-X.a:where(.svelte-X)` (the modifier
twice) while `:is(*.a)` prints correctly. **The issue's own control list is the two-ports
signature** — "the identical compound inside `:is()` is handled correctly" means one of the two
ports is already right, and names which one.

### 9. Is this expression's value known / defined? — [D]

**Upstream:** one `Scope#evaluate` returning one `Evaluation` object (`phases/scope.js:198`),
whose `is_known` / `is_defined` / `is_primitive` fields are read at a handful of sites.

**Ports.** #3027 already split this once — the client fold now goes through the server's
`EvalValue` — but the *neighbouring* predicates did not follow:

- `3_transform/server/evaluate.rs:37` `EvalValue` — a real abstract-value lattice, server only.
- `client/visitors/shared/utils.rs:6734` `is_expression_known_json` — a JSON walk with binding
  resolution.
- same file `:6656` `is_initial_value_literal_or_known` — answers by
  `memchr::memmem::find(s.as_bytes(), b"Literal")` over `binding.initial`, a string that may hold
  **either** serialized AST JSON **or** raw source text. So `let x = "a Literal string"` is
  "known", and any JSON containing a nested `Literal` anywhere — `f(1)` — is too, while
  `is_expression_known_json` reaches its call arm and says no.
- `client/visitors/title_element.rs:469` `is_known_defined_expr` — matches `Some("Literal")` and
  `Some("TemplateLiteral")` and nothing else, while `client/visitors/shared/utils.rs:4677`
  `is_expression_defined_json` resolves identifiers and unions conditional branches. On
  `{cond ? 'a' : 'b'}` the `<title>` path emits `?? ""` and the ordinary-text path does not;
  upstream answers both from one `evaluate` that handles `ConditionalExpression`
  (`scope.js:375`), so the `<title>` path is the deviant one.
- `client/visitors/regular_element.rs:2140` `is_value_known_defined` — a fifth, for
  `<option>` / `<select>`'s `node.__value`, with its own scope-root resolution and its own
  `JsExpr::Raw` string heuristic.
- `2_analyze/visitors/variable_declarator.rs:268` `is_expression_defined_typed` — a sixth, whose
  answer is frozen into `binding.initial_is_defined` at analyze time.

AGENTS.md already names three of these as "the next instalment" after #3027. The `<title>` and
`<option>` ports are not in that list.

The `globals` **table** underneath these predicates was a seventh port until #3471; it is
row [13](#13-what-does-a-call-to-one-of-upstreams-globals-keypaths-evaluate-to--d-closed-by-degree-1),
and it is the one instance in this file where the two ports were shown to render different text
from the same source.

**Two of these ports are closed as of 2026-08-29, and the divergence they carried ran in BOTH
directions — which is what makes the row worth re-reading rather than ticking off.** The
`?? ''` guard on a template hole, on `$.document.title` and on `option.value` is one upstream
decision, `scope.evaluate(value).is_defined`, read at three sites. rsvelte answered it with the
shared estree walk in some places and with `identifier_is_defined`, a hand-written table of
binding shapes, in others. The table admitted no function binding and no `$state` binding that is
never written, so `{fn}`, `{arrow}` and `<option value={n || 'a'}>` were guarded where upstream
leaves them bare; and `<title>` graded the **source** expression rather than the value it had
just built, so a legacy `$.untrack(…)` wrapper never made the chunk unknown and the guard was
omitted where upstream adds it. `identifier_is_defined` now delegates to `evaluate_binding_initial`
and `title_element` grades the built value, so both sites read the one walk; the walk itself
gained upstream's FUNCTION case, which no port had.

The measurement is the reason to state the directions separately. Over a 5,041-component
population (a deterministic 4,000-file sample of the 33,792 corpus components plus every one of
the 1,210 holding a `<title>`, `<option>` or `<select>`), the change moved **12 client outputs and
12 client-dev outputs and 0 server outputs**; graded against the official compiler, 11 of the 12
go divergent → byte-identical on each target and **none** move the other way, the twelfth
shrinking from 15 to 11 divergent lines with the residue in comment placement. A fix measured on
one direction's population would have scored a one-directional patch green.

Still open in this row: `is_expression_known_json`, `is_initial_value_literal_or_known` (the
`memmem::find(json, b"Literal")` one), `is_value_known_defined` and `is_expression_defined_typed`
— four `is_known` ports, untouched here, and `is_js_expr_defined` remains a structural second
walk over the built `JsExpr` whose leaves now call the shared one.

### 10. Which line and column is byte offset N on? — [D]

**Upstream:** `state.js:57` — one `getLocator(source)` stored on `state.locator` and read
everywhere in the compiler. One table.

**Ports — four, in two crates:**

| | file | line terminators | column unit |
|---|---|---|---|
| T1 | `1_parse/mod.rs:197` `compute_line_offsets` | `\n` only | **bytes** |
| T2 | `rsvelte_lint/src/line_index.rs:50` | `\n`, `\r\n`, lone `\r` | **UTF-16** |
| T3 | `rsvelte_lint/src/line_index.rs:22` `js_line_starts` | T2 + U+2028 / U+2029 | UTF-16 |
| T4 | `rsvelte_lint/src/suppression.rs:215` `line_of` | `\n` only | n/a |

T2/T3 are the pair already reasoned about once: `LintDiagnostic::report_span` picks between them
per rule, with four upstream-measured verdicts pinned as a test. **T4 was not part of that.**
`runner.rs:295` filters a diagnostic whose line came from T2 or T3 against a suppression map
whose keys came from T4, and T4 does not split on a lone `\r`. Named input: a `\r`-delimited file
where an `eslint-disable-next-line` sits on T2's line 2 and T4's line 1 — the directive does not
suppress. `line_index.rs:203` tests T2 on exactly this shape; nothing compares it to T4.

T1 vs T2 is a **unit** difference rather than a terminator one, and the two meet in one output
array: `json_api.rs:120` emits byte columns for compiler warnings and `:141` emits UTF-16 columns
for native rules, into the same field. Any line with a non-ASCII character before the finding
gives two different columns for one offset.

Inside the parser, `get_line_column` (`read/expression.rs:6593`) and
`get_line_column_for_binding` (`:6605`) answer the same question about the same offset
differently by construction — the latter measures the column from the *previous* line's start
when that line is empty. Which one runs depends only on which `create_typed_loc*` the caller
picked.

### 11. Does this expression contain a call? — [S]

Filed as **#3569**; recorded here so the inventory is complete rather than restated.
`ast/template.rs` `set_has_call` has three reachable phase-2 writers. When the issue was filed,
phase 3 re-derived the same bit in the generic element walker twice — `json_contains_call` and
`walk_metadata_flags` (the latter additionally counted a `SpreadElement`) — and in
`shared/utils.rs` `expression_has_call`.

Upstream computes it once in phase 2 into `node.metadata.expression.has_call`; phase 3 only reads
it. Whether the reachable copies disagree on an input: `未測定` — see #3569.

Three phase-2 writes listed when #3569 was opened were structurally unreachable and were removed:
the `SpreadElement` and `TaggedTemplateExpression` arms in the typed script walker, and the typed
`CallExpression` visitor. `VisitorContext.expression` starts as `None`; the only site that installs
`Some` is the `{#if}` visitor, and it walks its condition through `walk_js_expression_node`, not the
typed script walker. This is a static reachability result, not an ablation result: deleting those
three writes cannot change output while that single producer and consumer remain disjoint. The
remaining phase-2 writers are the reachable call, object-spread and top-level-spread arms in the
template-expression walker.

The migration slices now attach and consume that Phase 2 metadata for `AttachTag`,
`SpreadAttribute`, `StyleDirective`, the expressions inside a regular `style=` attribute, and
every generic attribute-value chunk, generic event attribute and component CSS custom property.
The old generic attribute
`walk_metadata_flags` / `json_contains_call` implementations and the tests that only compared
those unused walkers were then removed. The component CSS-property migration also removed the
last production caller and definition of the shared `expression_has_call` helper, so Phase 3 no
longer independently answers this question for generic attribute values. The shared text
template-chunk builder now also reads `has_call` from each expression tag's Phase 2 metadata,
rather than calculating a fourth answer while lowering text content. `shared/events.rs` still
asks the broader "contains any call" question for `OnDirective`, so the inventory row remains
open for that separate path.

### 12. "Selector unused" and "element scoped" are two engines over two element models — [S]

**Upstream:** `css-prune.js:130` `prune()` sets `complex_selector.metadata.used` **and**
`element.metadata.scoped` from the **same** `apply_selector` call.
`3-transform/css/index.js` only *reads* `metadata.used`; it contains no matching logic.

**Ports.** rsvelte splits the two:

- `2_analyze/css_scoping.rs:1331` `mark_elements_scoped` produces `metadata.scoped`, over an
  `ElementInfo` / `SGraph` model.
- `3_transform/css.rs:1467` `is_complex_selector_unused_impl` produces the `used` bit at print
  time, over a *different* model (`CssDomElement` / `DomStructure`), through a cascade of ~10
  independent sub-deciders each with its own traversal.
- `2_analyze/css/prune.rs:11` `prune_css` is a **third**, name-set-only port whose result is
  discarded on the spot (`let _used = …`). #3574 proposes deleting it.

The structural claim is solid — two element models built by two passes and consumed by two
matcher families can only agree by coincidence, and each has a bail the other lacks. Whether they
**do** disagree on a real component is `未測定`, and it is the most expensive row here to measure,
because it needs both engines instrumented in one run. #3427 is the same shape one level over and
did produce a number, so it is measurable in principle.

---

### 13. What does a call to one of upstream's `globals` keypaths evaluate to? — [D], closed by degree 1

**Upstream:** one `globals` table in `phases/scope.js:26` — 46 keypaths, each `[type, fn?]`.
`scope.evaluate`'s `CallExpression` arm calls `fn(...args)` when every argument is known and adds
the `NUMBER` / `STRING` marker otherwise. One table, one arm, one set of JS semantics.

**Ports — two, and they disagreed on a value both computed:**

- `3_transform/server/evaluate.rs:487` `eval_global_call` — all 46 keypaths, JS semantics
  (`Math.round` as `(n + 0.5).floor()`, which is JS's half-**up**), returning a typed `EvalValue`.
- `client/visitors/shared/utils.rs`, `get_literal_value_complex`'s `CallExpression` arm — a
  private list of **eight** `Math` names (`max`/`min`/`floor`/`ceil`/`round`/`abs`/`sqrt`/`pow`),
  no `String`, no `Number`, no `Number.*`, no `String.*`, no shadow guard, no `SpreadElement`
  guard — and `Math.round` as Rust's `f64::round`, which rounds half **away from zero**.

**The discriminating input is one line**, and it needs no state at all:

```svelte
<b>{Math.round(-0.5)}</b>
```

The client inlined `b.textContent = '-1'`; the server inlined `<b>0</b>`; official is `0` on both.
So a single source rendered a different number depending on which port read it, in output that
parses cleanly and has no reactivity symptom. `Math.round(-1.5)` is the second instance (`-2` vs
`-1`). No gate saw it: the corpus compares each target to *upstream* independently, so a
client-only wrong value is one entry's client column and nothing cross-checks it against the
server column of the same entry.

**Reachability is not in question here** — unlike several rows above, the input is an ordinary
template expression and the client fold is on its default path.

The second-order cost was larger than the wrong value: because the client's table was private, it
was also *small*, so `String(n)`, `Number(n)`, `Math.sign(n)` and 30 more names silently lost the
`textContent` fast path (#3471, 61 divergent cells of 124 measured).

**Closed at degree 1:** the client's arm was deleted and now calls the server's table through
`eval_known_global_call`. There is no second answer left to compare, which is why this row is
recorded rather than tracked. What it does **not** buy: the surrounding predicates in row 9 are
untouched, and nothing new compares any two of *them*.

### 14. What options does the public `parse()` run with? — [D]

Filed as **#3688**; the divergence is one field today and the shape is why it is here.

**Upstream:** one answer, in `compiler/index.js` — `parse(source, { modern, loose } = {})` calls
`_parse(source, loose)` and `to_public_ast(source, ast, modern)`. There is no second construction
of the parse configuration anywhere in `svelte/compiler`.

**Ports.** rsvelte builds it independently in each binding:

- `crates/rsvelte_napi/src/lib.rs:201-217` sets `capture_comments: true`, with a comment
  asserting fidelity — *"The public AST API mirrors svelte/compiler `parse()`, which keeps
  `leadingComments`/`trailingComments` on nodes."*
- `crates/rsvelte_lint_bindings/src/compiler_wasm/mod.rs:87-89` takes `ParseOptions::default()`,
  which leaves `capture_comments` **false**, and accepts no options from its caller at all.

**The named input** is any component with a comment inside `<script>`: the NAPI AST carries the
node comments and the wasm AST does not. Graded **[D] from code** rather than **[M]** — the wasm
build was not executed, and a local `cargo` never builds the wasm features, which is part of why
this went unobserved.

**Nothing compares them.** The `parse()` AST parity gate (#3389) drives the NAPI port only; that
is gate-coverage **39g**. Corpus growth cannot reach the wasm port, because it is in no gate's
population. And the wasm build is what `@rsvelte/compiler` and the playground ship, so the port a
user installs is the unmeasured one.

### 15. How are public compile options validated? — [D]

**Upstream:** `packages/svelte/src/compiler/validate-options.js` owns one ordered schema for
`compile` and `compileModule`, including parametric values, removed-option errors and process-wide
legacy warnings.

**Ports.** The NAPI conversion in `crates/rsvelte_napi/src/lib.rs`, the C ABI JSON conversion in
`crates/rsvelte_capi/src/lib.rs`, and the wasm conversion in
`crates/rsvelte_lint_bindings/src/compiler_wasm/mod.rs` each implement that schema. #3664 recorded
demonstrated disagreements on unknown keys, wrong scalar types, nested keys, aliases, removed
options and truthy `runes` values.

**Defended at degree 2.** `scripts/dev/test-wasm-compile-options.mjs` now compares representative
rejections directly with official Svelte and pins the warning and parametric cases independently;
the C ABI suite spells the same exact messages and behaviours as independent expectations. The
ports remain separate because their value domains differ (JS callbacks versus JSON and native
callbacks), so this closes the demonstrated cells rather than removing the row. A new option or
validator kind still has to be added to all three ports and their boundary gates.

### 16. What is the read form of a name inside an `$.invalidate_inner_signals` body? — [D]

**Upstream:** one `build_getter(node, state)` (`3-transform/client/utils.js:33`), called once per
indirect binding from `AssignmentExpression.js:145-182`. It reads `state.transform[name].read`,
so the answer is a property of the **site** the body is emitted at, not of the binding.

**Ports.**

- `client/mod.rs` `prop_invalidate_bodies` — precomputes one body **string** per binding from a
  `BindingKind` table (`Prop`/`BindableProp` that is a prop source and `StoreSub` → `name()`;
  `State`/`RawState`/`Derived`/`LegacyReactive` → `$.get(name)`; otherwise bare). Consumed by the
  instance-script text pipeline and by `legacy_state_member_mutate_ast` /
  `prop_member_mutate_ast`, which splice it as text.
- `client/visitors/expression_converter.rs` `wrap_with_legacy_invalidate` — a second copy of that
  same table, for the template AST path.

**Demonstrated.** The kind table has no site, and a name's read form is not a function of its
kind alone: in `adventurelog`'s `LocationVisits.svelte`, `visit` is an instance-script function
parameter *and* an each item, so official emits bare `visit;` in `handleGpxFileChange` and
`$.get(visit);` inside the each block — from the same `legacy_indirect_bindings` list. The AST
port answered `visit` at both, because the table cannot see the each scope. It now consults
`context.state.transform` first and falls back to the table; the string port still has only the
table.

Two things the divergence was hiding, both found in the same file and both fixed:
`prop_source_reads_ast` walked **into** the spliced body and wrapped the prop read a second time
(`trails()` → `trails()()`), because the body arrives already in final read form and nothing said
so; and the legacy-state arm of a component `bind:` setter
(`visitors/shared/component.rs`, the `$.mutate(root, …)` branch) never called
`wrap_with_legacy_invalidate` at all, so `<Comp bind:tz={activityForm.tz} />` dropped the
invalidation the element arm emits. `compatibility/pattern-corpus/legacy-invalidate-inner-signals-site.svelte`
carries all three shapes.

**Not closed.** The string port cannot be made site-aware without a printer, and the AST port
cannot be made to produce the text the per-line pipeline splices. Closing this at degree 1 means
retiring the text splice — the client instance-script pipeline AGENTS.md already names as the
correctness hazard.

### 17. Does this write target resolve to the component's binding, or to a shadow? — [D]

**Upstream:** every write lowering reaches its binding through **one** `context.state.scope.get(name)`
— `build_assignment` (`3-transform/client/visitors/AssignmentExpression.js:120`) and
`validate_mutation` (`.../shared/utils.js:402`) both do, and a name that resolves to a nested
declaration returns a binding whose `kind` is `normal`, so nothing is rewritten.

**Ports.** rsvelte answers it once per rewrite pass. Of the 44 `*_ast.rs` passes under
`3_transform/client/`, **8** consulted `oxc_semantic` and 36 compared the identifier's **text**
against a `Vec<String>` of binding names. Four of the text ones were binding-keyed write
lowerings (the count is 12 / 32 after fixing them):

- `prop_member_mutate_ast.rs` — `prop.x = v` → `prop(prop().x = v, true)`
- `state_member_mutate_ast.rs` — `state.x = v` → `$.mutate(state, $.get(state).x = v)`, the
  reactive-body twin of `legacy_state_member_mutate_ast.rs`, which has resolved through
  `find_state_var_symbols` since it was written and carries
  `skips_parameter_shadow_but_rewrites_captured_state` as a test
- `state_set_reactive_ast.rs` — `state = v` → `$.set(state, v)`
- `reactive_update_ast.rs` — `x++` → `$.update(x)` / `$.update_prop(x)`

**Demonstrated.** `huly`'s `FilterTypePopup.svelte` writes `filter.group` inside
`for (const filter of filters)` where `filter` is also a prop, and `musicat`'s `AnalyticsView.svelte`
writes `stats.totalPlays` inside `songs.reduce((stats, song) => …)` where `stats` is also legacy
reactive state. Official emits the plain write in both; rsvelte emitted the setter call. The
second is the one that names the *pair* rather than one port: the identical source inside a
plain instance function was already correct, because that path runs the scope-aware twin.

**What made the reactive ports get it wrong** is worth keeping: a `$:` body is handed to its
transforms **without** the component-level declarations, so the state variable is an *unresolved*
name there. `is_locally_shadowed` — "resolves to a declaration below the root scope" — is the
predicate that is right for both input shapes: unresolved (fragment) and root-scope (whole
script) both mean "the component's binding", and only a shadow is below the root.

**These four now route the decision through one primitive** (`scope_analysis::is_locally_shadowed`,
with `shadowed_reference_starts` for the in-place rewriters, which cannot hold a `Semantic`). That
is degree 1 for the *shadow* question and not for the row: the instance twin
`legacy_state_member_mutate_ast` still answers through `find_state_var_symbols` /
`is_state_var_reference_or_unresolved`, a second primitive with a second rule, and nothing compares
the two.

**Four of the remaining text-keyed passes were probed and are clean**: a `$`-prefixed parameter
shadowing a store (`function bump($count) { $count = 1; $count++; $count.x = 1 }`, reaching
`store_assign_ast` / `store_update_ast` / `store_member_mutate_ast`) and a parameter shadowing a
rest-props binding (`function read(rest) { return rest.foo }`, `rest_prop_member_access_ast`)
both compile byte-identical to official. `state_eager_ast` and `state_raw_frozen_ast` are keyed
on the rune **call**, not on a binding name, so they are not instances of this row at all — an
earlier draft of this row listed them and was wrong.

**The same probe found a live one, which is why the row stays open.** A function-local
`let n = $state(5)` that IS reassigned, shadowing a top-level `let n = $state(0)` that is NOT,
compiles to

```js
let n = 0;
function make() { let n = 5; $.set(n, 6); return n; }   // official: $.state(5) / $.get(n)
```

— `$.set` on a plain number, so the output is broken at run time rather than merely different.
**Its reachability is 0 on the collected corpus**: 5,521 of 34,709 sources declare a `$state`, 16
declare one name twice, 13 of those are `.svelte.(js|ts)` modules (which run the module pipeline,
where the escape hatch below already exists) and the 3 real components all compile byte-identical
on all four targets. Correctness and reachability are separate questions; this row records both.
The classification is a `Vec<String>` of non-reactive **names** (`client/mod.rs:7094`), so the
top-level binding's "never reassigned" answer reaches the inner declaration and its reads, while
the write goes through a pass that resolves correctly. The module pipeline already has the escape
hatch for exactly this — `ambiguous_state_names` (`client/mod.rs:5429`) re-asks
`binding.reassigned` per symbol whenever one name carries two `$state` bindings that disagree, and
`state_call_ast::is_non_reactive` consumes it — while the component pipeline neither computes it
nor reaches that lowering, which makes the `$state(…)` lowering itself a second pair.

**A battery of ten shadow probes then measured what the gate cannot.** One input per binding kind
— a store, a store subscription, a rest prop, `$state.raw`, `$state.snapshot`, an arrow parameter
over a `$state`, a `$derived`, an each item, a prop called as a function, a `$`-prefixed local —
each shadowing the component's binding inside a nested scope, compared to official on all four
targets. **Nine of ten were already correct; the tenth was live.** Upstream's `EachBlock`
`assign` / `mutate` transforms set `uses_index` on the owning block, forcing the `$$index`
callback parameter even where nothing reads it, and they reach the item through `scope.get`;
rsvelte looked the root up in `each_item_name_flags` by NAME, at two sites (the typed and the JSON
assignment paths), so a handler declaring `let row = …` over the item emitted a `$$index`
parameter official does not. That divergence is **client-only** — the server emits no such
parameter — so a probe run on one target would have scored it clean.

Two things the battery is worth for beyond the one defect. **The nine passes are now a measured
`[D]`, not an assumption**: `store_assign_ast`, `store_update_ast`, `store_member_mutate_ast` and
`store_unsub_wrap_ast` carry 37 `&[String]` parameters between them and answer correctly anyway,
because a `$`-prefixed name cannot be redeclared in Svelte and the plain store name is not what
they key on. And the flag site is **not** an `*_ast.rs` pass — it is in the expression converter —
so the "44 passes" denominator this row keeps quoting is not the population. Grep for the
question, not for the file naming convention.

**Crossing the entry point multiplied the yield.** A generated matrix — 6 binding kinds x 6 entry
points x 5 shadow shapes, 165 inputs x 4 targets — reported **72** divergences on its first run,
against 1 for the ten hand-written probes that varied only the binding kind. Three causes, and the
first is closed: the expression converter's shadow set held a bare `let` and a function parameter
and nothing else. Its registrar said so — *"destructuring patterns are ignored (they rarely shadow
a prop name and the code is cleaner without the extra complexity)"* — and a `catch` clause and a
`for…of` head bound nothing at all. **A comment recording a deliberate simplification is the same
hiding place as a comment asserting fidelity.** Closing it took 72 to 48, and the reusable part is
that all three constructs bind for their body only and must hide **both** the read transform and
`shadowed_prop_names`: the pre-existing `for…of` code removed the transform and not the second, so
a prop read inside the loop still became `$$props.v`.

The second is closed too, and it is the one with real-world reach.
`transform_legacy_state_declarations` finds `let <name> =` by text, and its caller hands it one
top-level instance statement at a time — so `function go() { let v = …; }` arrives as a single
input and the LOCAL declaration was lowered to `$.mutable_source`, allocating a signal per call.
Upstream promotes only a top-level `let`, so the rewrite is refused unless the match sits at the
statement's own brace depth. **Every other shadow fix in this batch moved 0 of 34,728 corpus
entries; this one moves 3**, and takes `musicat/src/lib/views/AlbumsView.svelte` from a listed
failure on `client` and `client-dev` to a 4-target match. Reachability is a property of the
defect, not of the class.

The third is the reason this row keeps a **server** paragraph, and it corrects a claim an earlier
draft made here. That draft called the 44 remaining divergences "one cause, outside phase 3";
**8 of them were phase 3**, in a port this row had not looked at. `server/ast/read_wrap.rs`
decides whether an identifier read is a derived / store binding from a `shadowed` stack, and its
own doc comment says the stack is populated "from function / arrow parameter patterns (the only
shadowing the store-cluster fixtures exercise)" — the second deliberate-simplification comment in
one row, and the second one to be load-bearing. A `catch` clause, a `for…of` / `for…in` head and a
`for (let …;;)` head bind names and none was collected, so `catch (v) { v.n = 2 }` emitted
`v().n = 2` and `for (let v = 0; v < 2; v++)` emitted
`for (let v = 0; v() < 2; $.update_derived(v))` — a runtime helper called on a loop counter. The
client had been fixed for the same five shapes one commit earlier and the server had not, which is
the row's own subject: **fixing one port is not fixing the question**, and only a probe that
compares all four targets separates the two. Blast radius 0 of 34,728 corpus entries on `server`
and `server-dev`, and the four hunks are independently necessary (ablated one at a time: 6 / 2 /
2 / 4 divergent lines).

**The predicate this row introduced then over-fired, and what caught it was a unit test rather
than any gate here.** `reference_is_plain_local` asks the `scope_root` bindings which one owns a
reference and whether its kind is `Normal` — and phase 2 records a **second, `Normal`** entry for a
rune declared inside a template expression's function body (the #3233 shape). So
`let counter = $state(1); counter = 2` in an event handler answered "plain local",
`try_transform_assignment` bailed, and the fallback emitted `$.set(counter, 2, true)` where
official emits `$.set(counter, 2)`. **The corpus could not see it**: the client hash sweep moved 0
of 34,728 entries across the whole series, and `template_function_rune_3233.rs` — a committed
repro from an earlier fix — is what went red. A property gate and a corpus are both populations;
a test written for the shape is not.

The discriminator is the scope chain: a component binding is declared at instance depth and a
local signal one function deeper, so the veto is `State` / `RawState` / `Derived` at
`function_depth >= 2`. **Restricting it to those three kinds is load-bearing** — the first
narrowing vetoed on any nested non-`Normal` binding, which is also true of an each item, and put
the `$$index` parameter back on the repro two rows above. A predicate fix needs the whole set of
repros the predicate serves re-run, not only the one that failed.

**A sweep of the shadow shapes the 165-probe matrix did NOT enumerate then found the same question
answered wrongly in THREE more places at once, and the count is the point: `const f = function v() { … }`
binds `v` inside its own body, and every implementation that had to know said otherwise.** `server/ast/read_wrap.rs` never put the
id in its frame; `client/ast_state_transform.rs` carries a comment saying named function
expressions "bind only in their own scope, so they are excluded" — correct about the *enclosing*
scope, and it then never declared the name in the function's own scope either; and the template
walker's `LocalScope` collected parameters and block declarations and not the id. So `typeof v`
came out `v()` on the server, `$.get(v)` in the instance script and `$$props.w` for a shadowed
prop, with the instance script and a template event handler being two separate ports of the client
half. Each hunk is independently necessary (2 / 4 / 2 divergent lines ablated one at a time) and
the blast radius is 0 of 34,728 corpus entries on all four targets. **A row that says "two ports" is a lower bound
until somebody counts**; the sweep that found this one also found `for (let v = 0; …)` above, and
neither shape was an axis value the generated family's author wrote.

Three things that sweep turned up are recorded rather than fixed. A named **class** expression is
the same shape and **upstream emits output no JS parser accepts** for it — `const C = class $.get(v) {`
on the client and `class v() {` on the server, both rejected by acorn — while rsvelte emits the
correct `class v {`; that is
[`upstream_issues/svelte-named-class-expression-shadowing-a-rune-emits-unparseable-output.md`](../upstream_issues/svelte-named-class-expression-shadowing-a-rune-emits-unparseable-output.md),
and no pattern-corpus file can carry it while byte equality is the goal. `function $y() {}` is
rejected by official with `dollar_prefix_invalid` and accepted here — the over-acceptance shape,
in phase 2. The opposite direction turned up too: upstream creates no scope for a class
`static {}` block, so `class C { static { const v = 2; … } }` beside a top-level `let v` is
rejected with `declaration_duplicate` while a method body, a function body and a plain block all
compile — legal JavaScript refused, which no collected corpus can hold either
([`upstream_issues/svelte-class-static-block-shares-the-instance-scope.md`](../upstream_issues/svelte-class-static-block-shares-the-instance-scope.md)). And a `$derived` name reused as a **destructured default parameter**
(`function go({ v } = { v: 0 })`) made the client emit
`function go(($$value) => { v = $$value.v; return $$value; })({ v: 0 }) { … }` — text no parser
accepts, with the component's own `$state` / `$derived` declarations left unlowered beside it.
`destructure_transforms.rs` finds a destructuring assignment by scanning for `} =` / `] =`, and
its one guard asks "is this inside ANOTHER pattern" — which a formal parameter list is not. What
separates the two spellings is the enclosing paren: a parameter list's `)` is followed by `=>` or
by the body's `{`, and a control-flow head is the one other paren that closes before a `{`. That
is fixed.

The next defect in the same scanner was `is_standalone`, and it is the sharpest statement of what
this row is about: upstream computes it as `context.path.at(-1).type.endsWith('Statement')` — a
**parent node type** — while rsvelte read the punctuation around the expression, which recognizes
an expression statement and nothing else. So every other statement whose child the assignment
actually is kept a trailing value: `if (({ v } = o))` came out `if (($.set(v, o.v, true), o))`
against official's `if (($.set(v, o.v, true)))`, and where the right-hand side is cached the IIFE
gained a `return $$value;` official does not emit. The population is not one shape — ten head
slots (`if` / `while` / `do…while` / `switch`, all three `for` slots, `return`, `throw`), three
keyword-introduced statement bodies (`else`, `case …:`, `default:`) and a redundant paren layer,
38 divergent comparisons over 33 probes. It is fixed by asking the same question from text, and
**three things about that translation are worth keeping**. A redundant paren layer is no node at
all — acorn drops it — so every layer has to be asked the question *innermost first*; peeling the
layers off before deciding strips the head's OWN parens and loses `if (({ a } = o))`, which the
first version did. The rule is not "a `)` follows": `if (1 && ({ a } = o))` closes on the same
`)`, so a head slot has to be delimited on **both** sides — by the head's own parentheses or by
the `;` between two `for` slots. And a `:` is a statement boundary in `case …:` / `default:` and
an expression's punctuation in a ternary or an object property, which is decided by scanning back
for the keyword at depth 0 rather than by the character. The one thing a text rule still cannot do
is name the node: `foo(({ a } = o))` and `if (({ a } = o))` differ only in the token before the
paren, so this stays an approximation of a parent-type test, not the test.

Underneath that scanner sits a plainer question the same row keeps asking — **which statements bind
a name** — and the two client registrars each knew a different half. `ast_state_transform.rs` had a
`visit_function` arm declaring a function declaration's id in the enclosing scope and **no class
hook at all**; the template walker's `register_block_local_vars` matched
`JsStatement::VariableDeclaration` and nothing else. So `class v {}` inside a function read
`typeof $.get(v)` on both paths and `function v() {}` inside an event handler did too. Both are
fixed. What sized the work honestly was refusing to price it off the three probes that reported
it: a grid of declaration kind (`function` / `class` / `let` / `const` / `var`) × where the
reference sits relative to the declaration × host (instance-script body / template handler /
prop-named binding) is **30 divergences over 96 comparisons**, against the 6 divergent lines the
original probes showed. The declaration-kind fix takes 12 of those; the residue is two further
causes, recorded rather than claimed. **Hoisting**: the instance-script port declares a name when
the walk reaches it, so `const r = typeof v; function v() {}` still reads the component binding —
upstream resolves against a scope that already holds every declaration of the block, and the same
is true of `let` and `var`, which is why the residue is 12 comparisons and not just the function
one. The template port already pre-scans its block, so this half is one port, not two. And **`var`
is function-scoped**: `{ var v = 2; } return typeof v;` binds `v` in the enclosing function, while
every registrar here treats a block's declarations as the block's — that one is 6 comparisons and
is the only member of this family that **also reproduces on the server**.

The hoisting half is fixed too, and the interesting part is what the repro found rather than what
the fix does. `ast_state_transform.rs` now registers a block's declarations in a pre-pass over the
statement list, through the same method the walk uses — a second copy of "which declarations
register no names" is exactly the shape this row exists to catch, so the `$props()` guard is
extracted from the rewrite that owns it and both callers read it. All four declaration kinds are
registered, not only the genuinely hoisted `function` / `var`: a read above a `let` or a `class` is
a TDZ error, but upstream still resolves it to the local, and byte equality is the goal. Ablated,
the variable half and the function/class half are 6 comparisons each. **And the repro's first draft
found a live defect in a third port that none of this touches**: rsvelte wraps `console.log(a)` in
`$.log_if_contains_state` for a handler-LOCAL `a`, where official wraps only an argument that
references a component binding — `const a = 1; console.log(a)` reproduces it with no shadowing
anywhere, and `console.log(v)` on the real `$derived` matches, so the divergence is
over-instrumentation of a local rather than a scope-resolution error. It is dev-mode only, it is
not in any probe set written for this row, and it is recorded here rather than fixed.

The `var` half closes the family, and it is the largest single instance this row has produced.
A `var` outlives its block, so `{ var v = 2; } typeof v` resolves to the local — and **all three**
phase-3 shadow registrars scoped it to the block. The server's `read_wrap.rs` carried the tell:
its `collect_block_decl_names` doc said collecting `let`/`const`/`var`/`function`/`class` "at every
block boundary is conservatively correct", which is false for exactly one of those five, because
the frame is *popped* when the block ends. **A comment asserting fidelity is where this class
hides** — the same shape as `assign_dev_ast.rs:56` and the server rune table. The grid put every
`var` site except a function's own top level wrong on client and server: a block, an `if`
consequent, a `for` init, a `for…of` head, a `try` block, a `case` arm, a `while` body, a doubly
nested block — **42 of 56 comparisons**, against the 6 the original probe showed. Ablated per port:
18 server, 18 instance-script, 8 template. The server and the instance-script pass walk the same
oxc AST and asked the same question, so they now share one `shared::hoisted_vars` walk instead of
a copy each; the template port reads the phase-3 IR and keeps its own, documented as the twin.

Two things it leaves. The negative control is load-bearing and is what stops the fix from being
"collect every `var` anywhere": a `var` inside a **nested function** must not leak out, so the walk
declines to enter a function or class body. And the residue names a **fourth** answer to this row's
question: `for (var v = 0; v < 1; v++)` in a template handler now reads `typeof v` correctly while
`v++` still lowers to `$.update(v)`, because that decision is made in `expression_converter.rs`
from `reference_is_plain_local` — a predicate driven by **phase 2's** scope data rather than by any
phase-3 registrar. Three registrars agreeing does not make the compiler agree with itself.

The 36 that remain are one cause, **in phase 2**, and every one is `client` or `client-dev`. A
write through a `catch` parameter or a `for…of` binding is recorded on the *component's* binding,
which shows up as a different `$.prop` flag word (24 vs 28, 19 vs 23), a `$$ownership_validator`
upstream does not emit, and a store declared as `$.mutable_source(writable(…))`; recorded here
rather than fixed.

The remaining ~28 text-keyed passes are **未測定**. Degree 3 is available here and is the right
shape for it: "no rewrite pass claims an identifier that resolves inside its own input" is a
property, not a comparison, so the corpus becomes the detector at whatever size it is.

**That gate now exists — `RSVELTE_ASSERT_SIGNAL_DISCIPLINE`
(`3_transform/client/signal_discipline.rs`) — and what it cost to make it discriminate is worth
more than the gate.** The first formulation asserted that no signal sink's first argument may
resolve to a symbol the same program declares as a plain value. It reported 9 violations on the
corpus, of which 4 components are byte-identical to official; narrowing it until the corpus
reported 0 took two rules — a `const` cannot be judged, because upstream emits `const st = 1`
beside a `$.set(st, …)` in the accessor generated for `export const st = $state(1)`, and an
initialiser that is an identifier cannot, because `let i = $$index_4` receives a signal. **A
property gate that reads 0 on the corpus is exactly what a property gate that sees nothing reads,
and this one saw nothing**: ablating the five shadow guards above and recompiling this row's own
repro produced `$.mutate(stats, …)` / `$.set(count, 1)` / `$.update(count)` with the gate armed
and silent, because `stats` and `count` are *parameters* of a user callback and the rule skipped
every parameter as unknown provenance. The defect's own container was inside the exclusion.

Two changes make it discriminate, and each is a distinction the first version collapsed. A
parameter is unjudgeable only when its function is **passed directly to a runtime helper** —
`$.each(…, ($$anchor, item, $$index) => …)` really does hand over signals — and that is not
answerable by nesting depth, because `$.set(s, xs.reduce((acc) => …))` puts a user callback inside
a runtime call's argument. And a prop write has its own sink: the generated shape is
`name(name().x = v, true)`, so that callee must be a `$.prop` / `$.rest_props` accessor. Ablated,
the gate now reports all six wrong writes across the two repros; restored, it is silent on all
three.

**Its first clean run found a live defect, in a file no output gate could have reported it from.**
`sparrow-app/…/TeamSidePanel.svelte` has `export let data` shadowed by a `let data = await …`
inside a template event handler, and rsvelte emitted `data(data().isNewInvite = false, true)`
where official emits `data.isNewInvite = false`. That id is already a listed entry on
`known-failures.{client,client-dev,server}.json` for two unrelated divergences (a scoping class
argument, a lost comment), so the output ratchet suppressed this one — the
"a ratchet entry suppresses everything its key cannot tell apart" rule, observed from the other
side. The fix is the same shadow question one entry point over: an event handler's body is
lowered by the expression converter, whose scope is the *template's*, so the name lookup reaches
the prop. It is **two** lowerings — `try_transform_assignment` and `try_transform_update` — and
fixing only the first left `data.count++` wrapped, which the gate then reported against the
repro written for the first half.

**The predicate is the part to copy carefully.** `reference_is_shadowed_non_prop` reads like the
right question and is not: it is true of a top-level `$state` too, because every kind but a prop
counts as "not a prop" there. Using it as the bail changed **736** corpus outputs, 724 of them
files that were passing, turning `$.set(layout, "…")` into `$.set(layout, "…", true)` across the
corpus. `reference_is_plain_local` — the reference uniquely belongs to a `BindingKind::Normal`
declaration — changes exactly **1**, the file the gate flagged, with 0 violations over 34,728
entries × client + client-dev.

What the gate cannot see is the **read** side, and that half had to be found by reading the fix
rather than by running it: in the same handler `items.selected = data` emitted
`items(items().selected = data(), true)` where official emits `data`, because the RHS is
transformed eagerly — before the outer walk that would have built a scope for it — with an empty
`LocalScope`. A read has no sink, so no signal-discipline violation exists to report.

**The position for a read cannot come from where the write's came from.** `JsExpr::Spanned` is
attached only when `enable_sourcemap` is true (`expression_converter.rs:156`), so keying a codegen
decision on it would make the generated program depend on whether a map was asked for — the same
option split that hides regressions from CodSpeed. An expression has many identifiers and the
converted `JsExpr` carries none of their positions, but its **source range** is on both paths, so
the bindings are asked which plain locals they declare inside it
(`plain_local_names_in_range`). Reachability of the read half is **0 of 34,728 corpus entries**:
correct, and it moves no real-world output.

## Adding a row, and closing one

**Finding a candidate.** Start from *one upstream function*, not from a rsvelte symbol. Grep the
Svelte submodule for a function with several importers, then find rsvelte's answer(s) and check
whether the callers split into independent paths. A rsvelte-side grep finds duplicated *names*;
it does not find the case where the second port was given a different name, which is the case
that hides.

**Two warnings that cost time here.**

A negative grep is not evidence. `grep` in this shell is a ugrep wrapper that skips gitignored
paths, and `cargo fmt` wraps comments across lines, so a multi-word literal needle encodes a
formatting assumption. **Put a positive control in the same invocation as the real needle** — a
different invocation cannot rule out that something changed in between.

A helper with many callers is **not** an instance. `js_scan::skip_opaque` (~30 callers) and
`clean_nodes` / `clean_nodes_refs` (two signatures over one body) were both checked and dropped.
The instance is two *separate* code paths each carrying their own logic.

**Closing a row** has three degrees, in increasing order of what it buys:

1. Make one port call the other. Removes the row.
2. Keep both and add a port-vs-port test with **independently spelled expectations** — the
   `typed_reactive_state_front_end_agrees_with_the_json_walk` shape. This is the only pattern in
   the tree today that defends the class.
3. Assert the property at runtime under an env flag and let the corpus find the violations, the
   way `RSVELTE_ASSERT_TRANSFORM_IDEMPOTENT` does. A property gate is bounded neither by a
   collected population nor by an author's axis values — which is why it found 37,352 violations
   in a corpus that scored 0 output divergences.

Degree 3 is worth reaching for whenever the decision is cheap to recompute, because it turns the
corpus you already have into a detector for this class **at whatever size it happens to be**.
