//! `<script generics="…">` parsing and the type-text analysis that decides
//! whether the synthesised `$$ComponentProps` alias can be hoisted above
//! `function $$render()`. Mirrors `svelte2tsx/nodes/Generics.ts`.

use super::super::utils::lexical::is_ascii_ident_byte;

/// Return true if `type_text` mentions any of `names` as a whole identifier
/// (i.e. surrounded by non-identifier characters on both sides).
///
/// Used to detect when a `$$ComponentProps` body references a type/interface
/// or value declared at the top level of the instance script — in which case
/// the synthesised `;type $$ComponentProps = ...;` cannot be hoisted above
/// `function $$render()`.
pub fn type_text_references_any(
    type_text: &str,
    names: &std::collections::HashSet<String>,
) -> bool {
    if names.is_empty() {
        return false;
    }
    let bytes = type_text.as_bytes();
    for name in names {
        if name.is_empty() {
            continue;
        }
        let nbytes = name.as_bytes();
        let mut i = 0usize;
        while i + nbytes.len() <= bytes.len() {
            if &bytes[i..i + nbytes.len()] == nbytes {
                let before_ok = i == 0 || !is_ascii_ident_byte(bytes[i - 1]);
                let after_idx = i + nbytes.len();
                let after_ok = after_idx == bytes.len() || !is_ascii_ident_byte(bytes[after_idx]);
                if before_ok && after_ok {
                    return true;
                }
            }
            i += 1;
        }
    }
    false
}

/// Return true if `type_text` contains a `typeof X` value-query whose root
/// identifier `X` is a value **declared in the instance script** (and not an
/// import).
///
/// This mirrors upstream's `HoistableInterfaces` hoistability test for the
/// synthesised `$$ComponentProps` alias: a `typeof X` adds `X` to the props
/// interface's `value_deps`, and the alias can only be hoisted above
/// `function $$render()` when every value dep is an *allowed reference*
/// (`isAllowedReference`) — i.e. NOT a locally-declared instance value. A
/// `typeof` of an **imported** binding (e.g. `ComponentProps<typeof Button>`
/// where `Button` is `import`ed) stays hoistable, because the import lives at
/// module scope above `$$render`. The previous heuristic forced *every*
/// `typeof` inside `$$render`, which wrongly nested the alias for the very
/// common imported-component case.
pub fn type_text_typeof_references_local_value(
    type_text: &str,
    instance_value_names: &std::collections::HashSet<String>,
    instance_import_names: &std::collections::HashSet<String>,
    module_import_names: &std::collections::HashSet<String>,
    module_value_names: &std::collections::HashSet<String>,
) -> bool {
    let bytes = type_text.as_bytes();
    let kw = b"typeof";
    let mut i = 0usize;
    while i + kw.len() <= bytes.len() {
        if &bytes[i..i + kw.len()] == kw {
            let before_ok = i == 0 || !is_ascii_ident_byte(bytes[i - 1]);
            let mut j = i + kw.len();
            // `typeof` must be followed by whitespace (a value query), not be a
            // prefix of a longer identifier like `typeofX`.
            let has_ws = j < bytes.len() && bytes[j].is_ascii_whitespace();
            if before_ok && has_ws {
                while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                    j += 1;
                }
                // Capture the root identifier (stop at `.` / non-ident), e.g.
                // `foo.bar.baz` -> `foo`.
                let start = j;
                while j < bytes.len() && is_ascii_ident_byte(bytes[j]) {
                    j += 1;
                }
                if j > start {
                    let root = &type_text[start..j];
                    // A `$name` query auto-subscribes, and upstream feeds every
                    // accessed store into `disallowed_values` regardless of how
                    // `name` is bound — so the import exception does not apply.
                    if let Some(base) = root.strip_prefix('$').filter(|b| !b.starts_with('$')) {
                        if !base.is_empty()
                            && (instance_value_names.contains(base)
                                || instance_import_names.contains(base)
                                || module_import_names.contains(base)
                                || module_value_names.contains(base))
                        {
                            return true;
                        }
                    } else {
                        let is_import = instance_import_names.contains(root)
                            || module_import_names.contains(root);
                        if !is_import && instance_value_names.contains(root) {
                            return true;
                        }
                    }
                }
                i = j;
                continue;
            }
        }
        i += 1;
    }
    false
}

/// The type parameters declared by `<script generics="…">` — mirrors
/// `svelte2tsx/nodes/Generics.ts`. The `$$Generic` alias form is mutually
/// exclusive with the attribute (upstream throws when both are present) and is
/// collected into `ExportedNames::dollar_generics` instead.
///
/// Upstream never splits the attribute textually: it parses `<{raw}>() => {}`
/// and reads the arrow function's type parameters, so a comma inside an object
/// type, a tuple, a parameter list or a string literal is the parser's problem
/// rather than the scanner's.
#[derive(Debug, Default, Clone)]
pub struct Generics {
    /// The whole `T extends boolean`.
    definitions: Vec<String>,
    /// The `T` in `T extends boolean`.
    references: Vec<String>,
    /// The raw attribute value, kept only when non-empty. `createRenderFunction`
    /// splices it onto `$$render` verbatim even when it does not parse.
    raw: Option<String>,
}

impl Generics {
    /// Parse the raw `generics` attribute value. An empty value is treated as
    /// no attribute at all, exactly like upstream's `if (generics)` guard.
    #[must_use]
    pub fn from_attribute(raw: Option<&str>) -> Self {
        let Some(raw) = raw.filter(|value| !value.is_empty()) else {
            return Self::default();
        };
        let (definitions, references) = parse_type_parameters(raw);
        Self {
            definitions,
            references,
            raw: Some(raw.to_string()),
        }
    }

    /// Whether the script tag declared a `generics` attribute at all — the
    /// predicate `createRenderFunction` keys on.
    #[must_use]
    pub const fn has_attribute(&self) -> bool {
        self.raw.is_some()
    }

    /// The raw attribute value (empty when there is no attribute).
    #[must_use]
    pub fn raw(&self) -> &str {
        self.raw.as_deref().unwrap_or_default()
    }

    /// Whether any type parameter was actually recognised — the predicate
    /// `addComponentExport` keys on.
    #[must_use]
    pub fn has(&self) -> bool {
        !self.definitions.is_empty()
    }

    /// `T extends boolean,U` — the definitions joined for a declaration site.
    #[must_use]
    pub fn definition_list(&self) -> String {
        self.definitions.join(",")
    }

    /// `T,U` — the names joined for a reference site.
    #[must_use]
    pub fn reference_list(&self) -> String {
        self.references.join(",")
    }

    /// The declared type parameter names.
    #[must_use]
    pub fn references(&self) -> &[String] {
        &self.references
    }
}

/// Parse `<{raw}>() => {}` and return `(definitions, references)` read off the
/// arrow function's type parameters, mirroring `Generics.getGenericTypeParameters`.
fn parse_type_parameters(raw: &str) -> (Vec<String>, Vec<String>) {
    use oxc_allocator::Allocator;
    use oxc_ast::ast::{Expression, Statement};
    use oxc_parser::Parser;
    use oxc_span::SourceType;

    let probe = format!("<{raw}>() => {{}}");
    let allocator = Allocator::default();
    // `.ts`, not `.tsx`: `<T>() => {}` must lex as a type parameter list.
    let parsed = Parser::new(&allocator, &probe, SourceType::ts()).parse();
    if parsed.panicked {
        return (Vec::new(), Vec::new());
    }
    let Some(Statement::ExpressionStatement(statement)) = parsed.program.body.first() else {
        return (Vec::new(), Vec::new());
    };
    let Expression::ArrowFunctionExpression(arrow) = &statement.expression else {
        return (Vec::new(), Vec::new());
    };
    let Some(type_parameters) = arrow.type_parameters.as_ref() else {
        return (Vec::new(), Vec::new());
    };
    let mut definitions = Vec::with_capacity(type_parameters.params.len());
    let mut references = Vec::with_capacity(type_parameters.params.len());
    for param in &type_parameters.params {
        let (start, end) = (param.span.start as usize, param.span.end as usize);
        let Some(text) = probe.get(start..end) else {
            return (Vec::new(), Vec::new());
        };
        definitions.push(text.to_string());
        references.push(param.name.name.to_string());
    }
    (definitions, references)
}

/// Extract the `generics` attribute value from a script tag text.
pub fn extract_generics_from_script_tag(tag_text: &str) -> Option<String> {
    if let Some(pos) = tag_text.find("generics=") {
        let after = &tag_text[pos + 9..];
        let trimmed = after.trim_start();
        if let Some(quote_char) = trimmed.chars().next() {
            if quote_char == '"' || quote_char == '\'' {
                let content = &trimmed[1..];
                if let Some(end) = content.find(quote_char) {
                    return Some(content[..end].to_string());
                }
            } else {
                // Unquoted value: take until whitespace or `>`
                let end = trimmed
                    .find(|c: char| c.is_whitespace() || c == '>')
                    .unwrap_or(trimmed.len());
                if end > 0 {
                    return Some(trimmed[..end].to_string());
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    fn set(names: &[&str]) -> HashSet<String> {
        names.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn typeof_local_value_blocks_hoist_but_import_does_not() {
        // `typeof Button` where Button is imported → hoistable (false).
        let imports = set(&["Button"]);
        assert!(!type_text_typeof_references_local_value(
            "ComponentProps<typeof Button>",
            &set(&[]),
            &imports,
            &set(&[]),
            &set(&[]),
        ));
        // `typeof localVal` where localVal is an instance-script value → block (true).
        assert!(type_text_typeof_references_local_value(
            "typeof localVal",
            &set(&["localVal"]),
            &set(&[]),
            &set(&[]),
            &set(&[]),
        ));
        // No `typeof` at all → false.
        assert!(!type_text_typeof_references_local_value(
            "{ a: string; b: number }",
            &set(&["localVal"]),
            &set(&[]),
            &set(&[]),
            &set(&[]),
        ));
        // A module-scope import via `typeof` is also hoistable (false).
        assert!(!type_text_typeof_references_local_value(
            "typeof ModuleThing",
            &set(&[]),
            &set(&[]),
            &set(&["ModuleThing"]),
            &set(&[]),
        ));
    }

    #[test]
    fn typeof_store_blocks_hoist_for_every_binding_kind() {
        // `$store` auto-subscribes, so the underlying binding is disallowed
        // whether it is an instance value, an instance import or a module one.
        for (values, imports, module_imports, module_values) in [
            (set(&["store"]), set(&[]), set(&[]), set(&[])),
            (set(&[]), set(&["store"]), set(&[]), set(&[])),
            (set(&[]), set(&[]), set(&["store"]), set(&[])),
            (set(&[]), set(&[]), set(&[]), set(&["store"])),
        ] {
            assert!(type_text_typeof_references_local_value(
                "{ someProp: typeof $store }",
                &values,
                &imports,
                &module_imports,
                &module_values,
            ));
        }
        // `$$props` & friends are not store references.
        assert!(!type_text_typeof_references_local_value(
            "typeof $$props",
            &set(&["$props"]),
            &set(&[]),
            &set(&[]),
            &set(&[]),
        ));
    }
}
