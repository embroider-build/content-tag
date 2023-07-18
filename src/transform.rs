use swc_core::ecma::{
    ast::{
        BlockStmt, CallExpr, Callee, ClassMember, ContentTagExpression, ContentTagMember, Expr,
        ExprStmt, Ident, Lit, StaticBlock, Stmt,
    },
    transforms::testing::test,
    visit::VisitMut,
    visit::VisitMutWith,
};

use swc_ecma_ast::{ExportDefaultExpr, ModuleDecl, ModuleItem};

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
        let ContentTagExpression { span, contents } = expr;
        let content_literal = Box::new(Expr::Lit(Lit::Str(contents.clone().into()))).into();
        Expr::Call(CallExpr {
            span: *span,
            callee: Callee::Expr(Box::new(Expr::Ident(self.template_identifier.clone()))),
            args: vec![
                content_literal,
                crate::snippets::SCOPE_PARAMS.clone().into(),
            ],
            type_args: None,
        })
    }
}

impl<'a> VisitMut for TransformVisitor<'a> {
    fn visit_mut_expr(&mut self, n: &mut Expr) {
        if let Expr::ContentTagExpression(expr) = n {
            *n = self.transform_tag_expression(expr);
            self.set_found_it();
        }
    }

    fn visit_mut_class_member(&mut self, n: &mut ClassMember) {
        if let ClassMember::ContentTagMember(ContentTagMember { span, contents }) = n {
            let content_literal = Box::new(Expr::Lit(Lit::Str(contents.clone().into()))).into();

            let call_expr = Expr::Call(CallExpr {
                span: *span,
                callee: Callee::Expr(Box::new(Expr::Ident(self.template_identifier.clone()))),
                args: vec![
                    content_literal,
                    crate::snippets::SCOPE_PARAMS_WITH_THIS.clone().into(),
                ],
                type_args: None,
            });
            let call_statement = ExprStmt {
                span: *span,
                expr: Box::new(call_expr),
            };
            *n = ClassMember::StaticBlock(StaticBlock {
                span: *span,
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
            if let Some(content_tag) = content_tag_expression_statement(&item) 
            {
                items_updated.push(ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultExpr(
                    ExportDefaultExpr {
                        span: content_tag.span,
                        expr: Box::new(self.transform_tag_expression(&content_tag)),
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
    })) = item {
        Some(content_tag)
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
    r#"let x = template("Hello", { eval() { return eval(arguments[0]); }})"#
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
          template("Hello", { component: this, eval() { return eval(arguments[0]) }},);
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
    r#"export default template("Hello", { eval() { return eval(arguments[0]) }},);"#
);
