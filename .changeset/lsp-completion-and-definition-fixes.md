---
'@rsvelte/language-server': patch
---

Four `rsvelte-language-server` answers now match the official server. An open-tag completion is no longer filtered by the name already typed, because `collectOpenTagSuggestions` never filters there and leaves the client to do it; a tag completion's `textEdit` replaces the whole name token rather than the part before the cursor; a definition asked on `const`, `let`, `var` or `enum` answers with nothing, where tsgo answers with the enclosing declaration and TypeScript resolves no symbol; and the `style` arm of the trigger-suggest condition is back inside its guard — `htmlCompletion.js:204-211` tests the name only after `attr.valueSet !== 'v' && value.length`, and `&&` binds tighter than `||`, so an attribute that already carried a value was still asking the editor to suggest one.
