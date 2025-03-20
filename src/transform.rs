use swc_common::Spanned;
use swc_core::ecma::{
    ast::{
        BlockStmt, CallExpr, Callee, ClassMember, ContentTagExpression, ContentTagMember, Expr,
        ExprStmt, Ident, StaticBlock, Stmt,
    },
    transforms::testing::test,
    visit::VisitMut,
    visit::VisitMutWith,
};

use swc_ecma_ast::{
    ContentTagContent, ExportDefaultExpr, ExprOrSpread, ModuleDecl, ModuleItem, Tpl, TplElement,
    TsSatisfiesExpr, TsType,
};

use swc_atoms::Atom;

pub struct TransformVisitor<'a> {
    template_identifier: Ident,
    found_it: Option<&'a mut bool>,
}

impl<'a> TransformVisitor<'a> {
    pub fn new(id: &Ident, found_it: Option<&'a mut bool>) -> Self {
        TransformVisitor {
            template_identifier: id.clone(),
            found_it,
        }
    }
    fn set_found_it(&mut self) {
        match self.found_it.as_mut() {
            Some(flag) => **flag = true,
            None => {}
        }
    }
    fn transform_tag_expression(&mut self, expr: &ContentTagExpression) -> Expr {
        let ContentTagExpression {
            span,
            contents,
            closing,
            ..
        } = expr;

        Expr::Call(CallExpr {
            span: *span,
            callee: Callee::Expr(Box::new(Expr::Ident(self.template_identifier.clone()))),
            args: vec![
                self.content_literal(contents),
                crate::snippets::scope_params(closing.span).into(),
            ],
            type_args: None,
        })
    }

    fn content_literal(&self, contents: &Box<ContentTagContent>) -> ExprOrSpread {
        Box::new(Expr::Tpl(Tpl {
            span: contents.span,
            exprs: vec![],
            quasis: vec![TplElement {
                span: contents.span,
                cooked: None,
                raw: escape_template_literal(&contents.value),
                tail: false,
            }],
        }))
        .into()
    }
}

fn escape_template_literal(input: &Atom) -> Atom {
    input
        .replace("\\", "\\\\")
        .replace("`", "\\`")
        .replace("$", "\\$")
        .into()
}

impl<'a> VisitMut for TransformVisitor<'a> {
    fn visit_mut_expr(&mut self, n: &mut Expr) {
        n.visit_mut_children_with(self);
        if let Expr::ContentTagExpression(expr) = n {
            *n = self.transform_tag_expression(expr);
            self.set_found_it();
        }
    }

    fn visit_mut_class_member(&mut self, n: &mut ClassMember) {
        n.visit_mut_children_with(self);
        if let ClassMember::ContentTagMember(ContentTagMember {
            span,
            opening,
            contents,
            closing,
        }) = n
        {
            let call_expr = Expr::Call(CallExpr {
                span: *span,
                callee: Callee::Expr(Box::new(Expr::Ident(self.template_identifier.clone()))),
                args: vec![
                    self.content_literal(contents),
                    crate::snippets::scope_params_with_this(closing.span).into(),
                ],
                type_args: None,
            });
            let call_statement = ExprStmt {
                span: *span,
                expr: Box::new(call_expr),
            };
            *n = ClassMember::StaticBlock(StaticBlock {
                span: opening.span,
                body: BlockStmt {
                    span: *span,
                    stmts: vec![Stmt::Expr(call_statement)],
                },
            });
            self.set_found_it();
        }
    }

    fn visit_mut_module_items(&mut self, items: &mut Vec<ModuleItem>) {
        let mut items_updated = Vec::with_capacity(items.len());
        for item in items.drain(..) {
            if let Some(content_tag) = content_tag_expression_statement(&item) {
                items_updated.push(ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultExpr(
                    ExportDefaultExpr {
                        span: content_tag.span,
                        expr: Box::new(self.transform_tag_expression(&content_tag)),
                    },
                )));
                self.set_found_it();
            } else if let Some(satisfies) = content_tag_satisfies_expression_statement(&item) {
                items_updated.push(ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultExpr(
                    ExportDefaultExpr {
                        span: satisfies.template.span,
                        expr: Box::new(Expr::TsSatisfies(TsSatisfiesExpr {
                            expr: Box::new(self.transform_tag_expression(&satisfies.template)),
                            type_ann: satisfies.ts_type.clone(),
                            span: satisfies.ts_type.span(),
                        })),
                    },
                )));
                self.set_found_it();
            } else {
                items_updated.push(item);
            }
        }

        *items = items_updated;
        items.visit_mut_children_with(self)
    }
}

fn content_tag_expression_statement(item: &ModuleItem) -> Option<&ContentTagExpression> {
    if let ModuleItem::Stmt(Stmt::Expr(ExprStmt {
        expr: box Expr::ContentTagExpression(content_tag),
        ..
    })) = item
    {
        Some(content_tag)
    } else {
        None
    }
}

struct TemplateSatisfies<'a> {
    pub template: &'a ContentTagExpression,
    pub ts_type: &'a Box<TsType>,
}

fn content_tag_satisfies_expression_statement(item: &ModuleItem) -> Option<TemplateSatisfies> {
    if let ModuleItem::Stmt(Stmt::Expr(ExprStmt {
        expr:
            box Expr::TsSatisfies(TsSatisfiesExpr {
                expr: box Expr::ContentTagExpression(content_tag),
                type_ann: ts_type,
                ..
            }),
        ..
    })) = item
    {
        Some(TemplateSatisfies {
            template: content_tag,
            ts_type,
        })
    } else {
        None
    }
}

#[cfg(test)]
use swc_core::ecma::visit::as_folder;

test!(
    Default::default(),
    |_| {
        as_folder(TransformVisitor::new(
            &Ident::new("template".into(), Default::default()),
            None,
        ))
    },
    content_tag_template_expression,
    r#"let x = <template>Hello</template>"#,
    r#"let x = template(`Hello`, { eval() { return eval(arguments[0]); }})"#
);

test!(
    Default::default(),
    |_| as_folder(TransformVisitor::new(
        &Ident::new("template".into(), Default::default()),
        None,
    )),
    content_tag_template_member,
    r#"class X { <template>Hello</template> } "#,
    r#"class X {
      static {
          template(`Hello`, { component: this, eval() { return eval(arguments[0]) }},);
      }
  }"#
);

test!(
    Default::default(),
    |_| as_folder(TransformVisitor::new(
        &Ident::new("template".into(), Default::default()),
        None,
    )),
    expression_inside_class_member,
    r#"class X { thing = <template>Hello</template> } "#,
    r#"class X {
        thing = template(`Hello`, { eval() { return eval(arguments[0]) }},);
    }"#
);

test!(
    Default::default(),
    |_| as_folder(TransformVisitor::new(
        &Ident::new("template".into(), Default::default()),
        None,
    )),
    class_member_inside_expression,
    r#"let x = class { <template>Hello</template> } "#,
    r#"let x = class {
        static {
            template(`Hello`, { component: this, eval() { return eval(arguments[0]) }},);
        }
    }"#
);

test!(
    Default::default(),
    |_| as_folder(TransformVisitor::new(
        &Ident::new("template".into(), Default::default()),
        None,
    )),
    content_tag_export_default,
    r#"<template>Hello</template>"#,
    r#"export default template(`Hello`, { eval() { return eval(arguments[0]) }},);"#
);

test!(
    Default::default(),
    |_| as_folder(TransformVisitor::new(
        &Ident::new("template".into(), Default::default()),
        None,
    )),
    inner_expression,
    r#"let x = doIt(<template>Hello</template>)"#,
    r#"let x = doIt(template(`Hello`, { eval() { return eval(arguments[0]) }}))"#
);

test!(
    Default::default(),
    |_| as_folder(TransformVisitor::new(
        &Ident::new("template".into(), Default::default()),
        None,
    )),
    backtick_in_template,
    r#"let x = <template>He`llo</template>"#,
    r#"let x = template(`He\`llo`, { eval() { return eval(arguments[0]) }})"#
);

test!(
    Default::default(),
    |_| as_folder(TransformVisitor::new(
        &Ident::new("template".into(), Default::default()),
        None,
    )),
    dollar_in_template,
    r#"let x = <template>He${ll}o</template>"#,
    r#"let x = template(`He\${ll}o`, { eval() { return eval(arguments[0]) }})"#
);

test!(
    Default::default(),
    |_| as_folder(TransformVisitor::new(
        &Ident::new("template".into(), Default::default()),
        None,
    )),
    do_not_interpret_js_escapes_in_hbs,
    r#"let x = <template>Hello\nWorld\u1234</template>"#,
    r#"let x = template(`Hello\\nWorld\\u1234`, { eval() { return eval(arguments[0]) }})"#
);
