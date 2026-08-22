---
"@rsvelte/compiler": patch
---

Keep a second `this=` on `<svelte:element>` / `<svelte:component>` instead of dropping it. Upstream's parser `splice`s exactly one `this` attribute out of the list and uses it as the tag / component; anything after it stays an ordinary attribute, so `<svelte:element this={tag} this={tag2}>` renders `this="span"` and `<svelte:component this={C} this={C} />` passes a `this` prop. rsvelte filtered the attribute list by name, so every `this` was removed and the second one disappeared from the output with no error from either compiler. The `svelte_element_invalid_this` warning is scoped the same way — it is asked of the spliced definition only, so a second, non-expression `this` no longer warns.
