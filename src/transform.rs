use swc_core::ecma::{
    ast::{
        BlockStmt, CallExpr, Callee, ClassMember, Expr, ExprOrSpread, ExprStmt,
        GlimmerTemplateExpression, GlimmerTemplateMember, Ident, Lit, StaticBlock, Stmt, ThisExpr,
    },
    transforms::testing::test,
    visit::VisitMut,
};

pub struct TransformVisitor;

impl VisitMut for TransformVisitor {
    fn visit_mut_expr(&mut self, n: &mut Expr) {
        if let Expr::GlimmerTemplateExpression(GlimmerTemplateExpression { span, contents }) = n {
            let content_literal = ExprOrSpread {
                spread: None,
                expr: Box::new(Expr::Lit(Lit::Str(contents.clone().into()))),
            };
            *n = Expr::Call(CallExpr {
                span: *span,
                callee: Callee::Expr(Box::new(Expr::Ident(Ident {
                    span: *span,
                    sym: "_GLIMMER_TEMPLATE_".into(),
                    optional: false,
                }))),
                args: vec![content_literal],
                type_args: None,
            })
        }
    }

    fn visit_mut_class_member(&mut self, n: &mut ClassMember) {
        if let ClassMember::GlimmerTemplateMember(GlimmerTemplateMember { span, contents }) = n {
            let content_literal = ExprOrSpread {
                spread: None,
                expr: Box::new(Expr::Lit(Lit::Str(contents.clone().into()))),
            };
            let this = ExprOrSpread {
                spread: None,
                expr: Box::new(Expr::This(ThisExpr { span: *span })),
            };
            let call_expr = Expr::Call(CallExpr {
                span: *span,
                callee: Callee::Expr(Box::new(Expr::Ident(Ident {
                    span: *span,
                    sym: "_GLIMMER_TEMPLATE_".into(),
                    optional: false,
                }))),
                args: vec![content_literal, this],
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
            })
        }
    }
}

#[cfg(test)]
use swc_core::ecma::visit::as_folder;

test!(
    Default::default(),
    |_| as_folder(TransformVisitor),
    glimmer_template_expression,
    r#"let x = <template>Hello</template>"#,
    r#"let x = _GLIMMER_TEMPLATE_("Hello")"#
);

test!(
    Default::default(),
    |_| as_folder(TransformVisitor),
    glimmer_template_member,
    r#"class X { <template>Hello</template> } "#,
    r#"class X {
      static {
          _GLIMMER_TEMPLATE_("Hello", this);
      }
  }"#
);
