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
        let stripped_content = strip_indent(&contents.value);
        Box::new(Expr::Tpl(Tpl {
            span: contents.span,
            exprs: vec![],
            quasis: vec![TplElement {
                span: contents.span,
                cooked: None,
                raw: escape_template_literal(&stripped_content.into()),
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

fn strip_indent(input: &str) -> String {
    let lines: Vec<&str> = input.lines().collect();

    if lines.len() <= 1 {
        return input.to_string();
    }

    let start = lines.iter().position(|l| !l.trim().is_empty()).unwrap_or(lines.len());
    let end = lines.iter().rposition(|l| !l.trim().is_empty()).map(|i| i + 1).unwrap_or(0);

    if start >= end {
        return String::new();
    }

    let lines = &lines[start..end];

    let mut min_indent: Option<usize> = None;
    let mut has_spaces = false;
    let mut has_tabs = false;

    for line in lines {
        let content = line.trim_start();
        if content.is_empty() {
            continue;
        }

        let indent_size = line.len() - content.len();
        let indent_chars = &line[..indent_size];

        has_spaces |= indent_chars.contains(' ');
        has_tabs |= indent_chars.contains('\t');

        min_indent = Some(min_indent.map_or(indent_size, |current| current.min(indent_size)));
    }

    let min_indent = min_indent.unwrap_or(0);

    if (has_spaces && has_tabs) || min_indent == 0 {
        return lines.join("\n");
    }

    lines
        .iter()
        .map(|line| {
            if line.len() >= min_indent {
                &line[min_indent..]
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
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

test!(
    Default::default(),
    |_| as_folder(TransformVisitor::new(
        &Ident::new("template".into(), Default::default()),
        None,
    )),
    strips_leading_trailing_whitespace,
    r#"let x = <template>
  <span>Hello</span>
</template>"#,
    r#"let x = template(`<span>Hello</span>`, { eval() { return eval(arguments[0]) }})"#
);

test!(
    Default::default(),
    |_| as_folder(TransformVisitor::new(
        &Ident::new("template".into(), Default::default()),
        None,
    )),
    strips_common_indentation,
    r#"let x = <template>
    <div>
      <span>Hello</span>
    </div>
  </template>"#,
    r#"let x = template(`<div>
  <span>Hello</span>
</div>`, { eval() { return eval(arguments[0]) }})"#
);

test!(
    Default::default(),
    |_| as_folder(TransformVisitor::new(
        &Ident::new("template".into(), Default::default()),
        None,
    )),
    strips_indentation_multiline,
    r#"<template>
  Hello
  <span>there</span>.
  <p>
    <span>how are you</span>
  </p>
</template>"#,
    r#"export default template(`Hello
<span>there</span>.
<p>
  <span>how are you</span>
</p>`, { eval() { return eval(arguments[0]) }},);"#
);

test!(
    Default::default(),
    |_| as_folder(TransformVisitor::new(
        &Ident::new("template".into(), Default::default()),
        None,
    )),
    class_member_strips_indentation,
    r#"class X {
  <template>
    <span>Hello</span>
  </template>
}"#,
    r#"class X {
  static {
      template(`<span>Hello</span>`, { component: this, eval() { return eval(arguments[0]) }},);
  }
}"#
);

test!(
    Default::default(),
    |_| as_folder(TransformVisitor::new(
        &Ident::new("template".into(), Default::default()),
        None,
    )),
    deeply_nested_indentation,
    r#"class Component {
  method() {
    return <template>
      <div>
        <span>Nested</span>
      </div>
    </template>;
  }
}"#,
    r#"class Component {
  method() {
    return template(`<div>
  <span>Nested</span>
</div>`, { eval() { return eval(arguments[0]) }});
  }
}"#
);

test!(
    Default::default(),
    |_| as_folder(TransformVisitor::new(
        &Ident::new("template".into(), Default::default()),
        None,
    )),
    preserves_internal_indentation,
    r#"let x = <template>
  <div>
    <pre>
      some code
        with indentation
    </pre>
  </div>
</template>"#,
    r#"let x = template(`<div>
  <pre>
    some code
      with indentation
  </pre>
</div>`, { eval() { return eval(arguments[0]) }})"#
);

test!(
    Default::default(),
    |_| as_folder(TransformVisitor::new(
        &Ident::new("template".into(), Default::default()),
        None,
    )),
    opt_out_with_comment,
    r#"let x = <template>
{{!-- prevent automatic de-indent --}}
    <pre>
      content here
    </pre>
  </template>"#,
    r#"let x = template(`{{!-- prevent automatic de-indent --}}
    <pre>
      content here
    </pre>`, { eval() { return eval(arguments[0]) }})"#
);
