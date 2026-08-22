//! JavaScript early errors acorn raises and OXC does not.
//!
//! Every one of these is syntactically shaped but illegal, and none is decidable
//! from the token stream alone: each needs the class, function or label context
//! the construct sits in. OXC leaves them to `SemanticBuilder`, which this
//! pipeline never runs, so without this pass they are copied into the output and
//! produce text no JS engine accepts.
//!
//! acorn is single-pass and non-recovering, so it throws on the first violation
//! it reaches and never sees any that follow — the caller takes the earliest by
//! position.

use oxc_ast::ast::{
    AccessorProperty, ArrowFunctionExpression, BindingPattern, CallExpression, Class, ClassElement,
    Expression, FormalParameters, Function, MethodDefinition, MethodDefinitionKind, ObjectProperty,
    Program, PropertyDefinition, PropertyKey, PropertyKind, Statement, StaticBlock, Super,
    TSModuleBlock,
};
use oxc_ast_visit::{Visit, walk};
use oxc_span::GetSpan;
use rustc_hash::{FxHashMap, FxHashSet};

/// The earliest early error in `program`, as `(offset, message)`.
pub fn find_violation(program: &Program<'_>, source: &str) -> Option<(u32, String)> {
    // Every check below needs one of these spellings in the source. `class`
    // covers the private-name and constructor families, `:` the labels.
    if !(source.contains("super")
        || source.contains("break")
        || source.contains("continue")
        || source.contains("import")
        || source.contains("export")
        || source.contains("class")
        || source.contains('#')
        || source.contains(':')
        || source.contains("use strict"))
    {
        return None;
    }
    let mut scan = Scan::default();
    scan.visit_program(program);
    scan.hits.into_iter().min_by_key(|(at, _)| *at)
}

/// What a label can be a destination for. acorn's `loopLabel` / `switchLabel`
/// carry no name; a labelled statement carries a name and takes its kind from
/// the statement it ultimately labels.
#[derive(Clone, Copy, PartialEq, Eq)]
enum LabelKind {
    Loop,
    Switch,
}

struct Label {
    name: Option<String>,
    kind: Option<LabelKind>,
}

impl Label {
    fn loop_label() -> Self {
        Label {
            name: None,
            kind: Some(LabelKind::Loop),
        }
    }

    fn switch_label() -> Self {
        Label {
            name: None,
            kind: Some(LabelKind::Switch),
        }
    }
}

/// One class body's private names: what it declares, and every `#name` read
/// inside it that has not been resolved yet.
#[derive(Default)]
struct PrivateFrame {
    declared: FxHashSet<String>,
    used: Vec<(String, u32)>,
}

#[derive(Default)]
struct Scan {
    hits: Vec<(u32, String)>,
    /// Set for the direct children of `Program` only, and consumed by the first
    /// `visit_statement` that follows — acorn's `topLevel` argument.
    top_level: bool,
    /// acorn-typescript parses a namespace body with its own statement rules,
    /// and upstream rejects the whole construct as `typescript_invalid_feature`
    /// before any of this can apply.
    in_ts_module: bool,
    super_allowed: bool,
    direct_super_allowed: bool,
    /// Handed to the next `Function` visited, which is how a method's body gets
    /// `super` while a plain function nested in the same place does not.
    pending_super: bool,
    pending_direct_super: bool,
    /// Whether the class currently being walked has an `extends` clause.
    class_has_super: Vec<bool>,
    labels: Vec<Label>,
    private_stack: Vec<PrivateFrame>,
}

impl Scan {
    fn hit(&mut self, at: u32, message: impl Into<String>) {
        self.hits.push((at, message.into()));
    }

    /// acorn's `parseBreakContinueStatement`: a destination is any label whose
    /// name matches (or any label at all, when the statement carries no name)
    /// and whose kind admits the keyword.
    fn check_break_continue(&mut self, at: u32, label: Option<&str>, is_break: bool) {
        let reachable = self.labels.iter().any(|entry| {
            if label.is_some() && entry.name.as_deref() != label {
                return false;
            }
            if entry.kind.is_some() && (is_break || entry.kind == Some(LabelKind::Loop)) {
                return true;
            }
            label.is_some() && is_break
        });
        if !reachable {
            let keyword = if is_break { "break" } else { "continue" };
            self.hit(at, format!("Unsyntactic {keyword}"));
        }
    }

    /// A `#name` read. With no enclosing class acorn raises immediately;
    /// otherwise the read is recorded and resolved when the class body closes.
    fn use_private_name(&mut self, name: &str, at: u32, in_expression: bool) {
        if let Some(frame) = self.private_stack.last_mut() {
            frame.used.push((name.to_string(), at));
        } else if in_expression {
            // `#a in o` outside a class never reaches `parsePrivateIdent`.
            self.hit(at, "Unexpected token");
        } else {
            self.hit(at, undeclared_private_message(name));
        }
    }

    /// acorn's `parseClassBody`: at most one constructor, and each private name
    /// declared once — with a getter/setter pair of the same staticness the one
    /// legal repetition.
    fn check_class_members(&mut self, class: &Class<'_>) -> FxHashSet<String> {
        let mut declared = FxHashSet::default();
        let mut slots: FxHashMap<String, PrivateSlot> = FxHashMap::default();
        let mut had_constructor = false;

        for element in &class.body.body {
            let (key, slot, is_overload) = match element {
                ClassElement::MethodDefinition(method) => {
                    if method.kind == MethodDefinitionKind::Constructor {
                        // A TypeScript overload signature carries no body and
                        // is not the class's constructor.
                        if method.value.body.is_some() {
                            if had_constructor {
                                self.hit(
                                    method.span.start,
                                    "Duplicate constructor in the same class",
                                );
                            }
                            had_constructor = true;
                        }
                        continue;
                    }
                    (
                        &method.key,
                        method_slot(method),
                        method.value.body.is_none(),
                    )
                }
                ClassElement::PropertyDefinition(property) => {
                    (&property.key, PrivateSlot::Plain, false)
                }
                ClassElement::AccessorProperty(accessor) => {
                    (&accessor.key, PrivateSlot::Plain, false)
                }
                _ => continue,
            };

            let PropertyKey::PrivateIdentifier(id) = key else {
                continue;
            };
            let name = id.name.as_str();
            declared.insert(name.to_string());
            // A TypeScript overload signature names the member without defining
            // it, so it neither claims the slot nor collides with the body.
            if !is_overload && is_private_name_conflicted(&mut slots, name, slot) {
                self.hit(
                    id.span.start,
                    format!("Identifier '#{name}' has already been declared"),
                );
            }
        }
        declared
    }
}

/// acorn's `privateNameMap` values: an accessor records which half it is so the
/// matching half may follow it, and anything else occupies the name outright.
#[derive(Clone, Copy, PartialEq, Eq)]
enum PrivateSlot {
    Plain,
    Get { r#static: bool },
    Set { r#static: bool },
}

fn method_slot(method: &MethodDefinition<'_>) -> PrivateSlot {
    match method.kind {
        MethodDefinitionKind::Get => PrivateSlot::Get {
            r#static: method.r#static,
        },
        MethodDefinitionKind::Set => PrivateSlot::Set {
            r#static: method.r#static,
        },
        _ => PrivateSlot::Plain,
    }
}

fn is_private_name_conflicted(
    slots: &mut FxHashMap<String, PrivateSlot>,
    name: &str,
    next: PrivateSlot,
) -> bool {
    let Some(&current) = slots.get(name) else {
        slots.insert(name.to_string(), next);
        return false;
    };
    let complementary = matches!(
        (current, next),
        (PrivateSlot::Get { r#static: a }, PrivateSlot::Set { r#static: b })
            | (PrivateSlot::Set { r#static: a }, PrivateSlot::Get { r#static: b })
            if a == b
    );
    if complementary {
        slots.insert(name.to_string(), PrivateSlot::Plain);
        return false;
    }
    true
}

const USE_STRICT_MESSAGE: &str =
    "Illegal 'use strict' directive in function with non-simple parameter list";

/// acorn's `isSimpleParamList`: every parameter is a bare identifier. A default,
/// a rest element and a destructuring pattern each make the list non-simple.
fn is_simple_parameter_list(params: &FormalParameters<'_>) -> bool {
    params.rest.is_none()
        && params.items.iter().all(|param| {
            param.initializer.is_none()
                && matches!(param.pattern, BindingPattern::BindingIdentifier(_))
        })
}

fn undeclared_private_message(name: &str) -> String {
    format!("Private field '#{name}' must be declared in an enclosing class")
}

/// The kind a labelled statement takes, following a chain of labels down to the
/// statement they all label.
fn label_kind(body: &Statement<'_>) -> Option<LabelKind> {
    match body {
        Statement::ForStatement(_)
        | Statement::ForInStatement(_)
        | Statement::ForOfStatement(_)
        | Statement::WhileStatement(_)
        | Statement::DoWhileStatement(_) => Some(LabelKind::Loop),
        Statement::SwitchStatement(_) => Some(LabelKind::Switch),
        Statement::LabeledStatement(inner) => label_kind(&inner.body),
        _ => None,
    }
}

impl<'a> Visit<'a> for Scan {
    fn visit_program(&mut self, program: &Program<'a>) {
        for stmt in &program.body {
            self.top_level = true;
            self.visit_statement(stmt);
        }
        self.top_level = false;
    }

    fn visit_statement(&mut self, stmt: &Statement<'a>) {
        let top_level = std::mem::take(&mut self.top_level);
        if !top_level && !self.in_ts_module && stmt.is_module_declaration() {
            self.hit(
                stmt.span().start,
                "'import' and 'export' may only appear at the top level",
            );
        }
        walk::walk_statement(self, stmt);
    }

    fn visit_ts_module_block(&mut self, block: &TSModuleBlock<'a>) {
        let outer = std::mem::replace(&mut self.in_ts_module, true);
        walk::walk_ts_module_block(self, block);
        self.in_ts_module = outer;
    }

    fn visit_super(&mut self, node: &Super) {
        if !self.super_allowed {
            self.hit(node.span.start, "'super' keyword outside a method");
        }
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        // acorn raises "outside a method" first and never reaches this check,
        // so gate on the same condition rather than on position order.
        if self.super_allowed
            && !self.direct_super_allowed
            && matches!(call.callee, Expression::Super(_))
        {
            self.hit(
                call.callee.span().start,
                "super() call outside constructor of a subclass",
            );
        }
        walk::walk_call_expression(self, call);
    }

    fn visit_unary_expression(&mut self, expr: &oxc_ast::ast::UnaryExpression<'a>) {
        if expr.operator == oxc_syntax::operator::UnaryOperator::Delete {
            let mut target = &expr.argument;
            while let Expression::ParenthesizedExpression(inner) = target {
                target = &inner.expression;
            }
            if matches!(target, Expression::PrivateFieldExpression(_)) {
                self.hit(expr.span.start, "Private fields can not be deleted");
            }
        }
        walk::walk_unary_expression(self, expr);
    }

    fn visit_function(&mut self, func: &Function<'a>, flags: oxc_semantic::ScopeFlags) {
        if func.has_use_strict_directive() && !is_simple_parameter_list(&func.params) {
            self.hit(func.span.start, USE_STRICT_MESSAGE);
        }
        let super_allowed = std::mem::take(&mut self.pending_super);
        let direct_super_allowed = std::mem::take(&mut self.pending_direct_super);
        let outer_super = std::mem::replace(&mut self.super_allowed, super_allowed);
        let outer_direct = std::mem::replace(&mut self.direct_super_allowed, direct_super_allowed);
        let outer_labels = std::mem::take(&mut self.labels);
        walk::walk_function(self, func, flags);
        self.super_allowed = outer_super;
        self.direct_super_allowed = outer_direct;
        self.labels = outer_labels;
    }

    fn visit_arrow_function_expression(&mut self, func: &ArrowFunctionExpression<'a>) {
        // A concise body has no directive prologue to carry `'use strict'`.
        if func.has_use_strict_directive() && !is_simple_parameter_list(&func.params) {
            self.hit(func.span.start, USE_STRICT_MESSAGE);
        }
        // An arrow has no `this` scope of its own, so `super` carries through;
        // its body is still a function body, so the labels do not.
        let outer_labels = std::mem::take(&mut self.labels);
        walk::walk_arrow_function_expression(self, func);
        self.labels = outer_labels;
    }

    fn visit_class(&mut self, class: &Class<'a>) {
        let declared = self.check_class_members(class);
        self.private_stack.push(PrivateFrame {
            declared,
            used: Vec::new(),
        });
        self.class_has_super.push(class.heritage.is_some());

        walk::walk_class(self, class);

        self.class_has_super.pop();
        let frame = self.private_stack.pop().unwrap_or_default();
        let unresolved: Vec<(String, u32)> = frame
            .used
            .into_iter()
            .filter(|(name, _)| !frame.declared.contains(name))
            .collect();
        match self.private_stack.last_mut() {
            Some(parent) => parent.used.extend(unresolved),
            None => {
                for (name, at) in unresolved {
                    self.hit(at, undeclared_private_message(&name));
                }
            }
        }
    }

    fn visit_method_definition(&mut self, method: &MethodDefinition<'a>) {
        self.pending_super = true;
        self.pending_direct_super = method.kind == MethodDefinitionKind::Constructor
            && self.class_has_super.last().copied().unwrap_or(false);
        walk::walk_method_definition(self, method);
        self.pending_super = false;
        self.pending_direct_super = false;
    }

    fn visit_property_definition(&mut self, property: &PropertyDefinition<'a>) {
        let outer_super = std::mem::replace(&mut self.super_allowed, true);
        let outer_direct = std::mem::replace(&mut self.direct_super_allowed, false);
        walk::walk_property_definition(self, property);
        self.super_allowed = outer_super;
        self.direct_super_allowed = outer_direct;
    }

    fn visit_accessor_property(&mut self, property: &AccessorProperty<'a>) {
        let outer_super = std::mem::replace(&mut self.super_allowed, true);
        let outer_direct = std::mem::replace(&mut self.direct_super_allowed, false);
        walk::walk_accessor_property(self, property);
        self.super_allowed = outer_super;
        self.direct_super_allowed = outer_direct;
    }

    fn visit_static_block(&mut self, block: &StaticBlock<'a>) {
        let outer_super = std::mem::replace(&mut self.super_allowed, true);
        let outer_direct = std::mem::replace(&mut self.direct_super_allowed, false);
        let outer_labels = std::mem::take(&mut self.labels);
        walk::walk_static_block(self, block);
        self.super_allowed = outer_super;
        self.direct_super_allowed = outer_direct;
        self.labels = outer_labels;
    }

    fn visit_object_property(&mut self, property: &ObjectProperty<'a>) {
        // A shorthand or a plain data property holds an ordinary expression; a
        // method and an accessor are parsed by acorn's `parseMethod`.
        if property.method || property.kind != PropertyKind::Init {
            self.pending_super = true;
            self.pending_direct_super = false;
        }
        walk::walk_object_property(self, property);
        self.pending_super = false;
        self.pending_direct_super = false;
    }

    fn visit_private_field_expression(&mut self, expr: &oxc_ast::ast::PrivateFieldExpression<'a>) {
        self.use_private_name(expr.field.name.as_str(), expr.field.span.start, false);
        walk::walk_private_field_expression(self, expr);
    }

    fn visit_private_in_expression(&mut self, expr: &oxc_ast::ast::PrivateInExpression<'a>) {
        self.use_private_name(expr.left.name.as_str(), expr.left.span.start, true);
        walk::walk_private_in_expression(self, expr);
    }

    fn visit_labeled_statement(&mut self, stmt: &oxc_ast::ast::LabeledStatement<'a>) {
        let name = stmt.label.name.as_str();
        if self
            .labels
            .iter()
            .any(|entry| entry.name.as_deref() == Some(name))
        {
            self.hit(
                stmt.label.span.start,
                format!("Label '{name}' is already declared"),
            );
        }
        self.labels.push(Label {
            name: Some(name.to_string()),
            kind: label_kind(&stmt.body),
        });
        walk::walk_labeled_statement(self, stmt);
        self.labels.pop();
    }

    fn visit_break_statement(&mut self, stmt: &oxc_ast::ast::BreakStatement<'a>) {
        self.check_break_continue(
            stmt.span.start,
            stmt.label.as_ref().map(|l| l.name.as_str()),
            true,
        );
    }

    fn visit_continue_statement(&mut self, stmt: &oxc_ast::ast::ContinueStatement<'a>) {
        self.check_break_continue(
            stmt.span.start,
            stmt.label.as_ref().map(|l| l.name.as_str()),
            false,
        );
    }

    fn visit_for_statement(&mut self, stmt: &oxc_ast::ast::ForStatement<'a>) {
        self.labels.push(Label::loop_label());
        walk::walk_for_statement(self, stmt);
        self.labels.pop();
    }

    fn visit_for_in_statement(&mut self, stmt: &oxc_ast::ast::ForInStatement<'a>) {
        self.labels.push(Label::loop_label());
        walk::walk_for_in_statement(self, stmt);
        self.labels.pop();
    }

    fn visit_for_of_statement(&mut self, stmt: &oxc_ast::ast::ForOfStatement<'a>) {
        self.labels.push(Label::loop_label());
        walk::walk_for_of_statement(self, stmt);
        self.labels.pop();
    }

    fn visit_while_statement(&mut self, stmt: &oxc_ast::ast::WhileStatement<'a>) {
        self.labels.push(Label::loop_label());
        walk::walk_while_statement(self, stmt);
        self.labels.pop();
    }

    fn visit_do_while_statement(&mut self, stmt: &oxc_ast::ast::DoWhileStatement<'a>) {
        self.labels.push(Label::loop_label());
        walk::walk_do_while_statement(self, stmt);
        self.labels.pop();
    }

    fn visit_switch_statement(&mut self, stmt: &oxc_ast::ast::SwitchStatement<'a>) {
        self.labels.push(Label::switch_label());
        walk::walk_switch_statement(self, stmt);
        self.labels.pop();
    }
}
