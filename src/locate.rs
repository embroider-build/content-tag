use swc_core::ecma::{
    ast::{ClassMember, ContentTagMember, Expr},
    visit::VisitMut,
    visit::VisitMutWith,
};

use crate::ContentTagInfo;

pub struct LocateVisitor<'a> {
    pub infos: &'a mut Vec<ContentTagInfo>,
}

impl<'a> LocateVisitor<'a> {
    fn found_tag(&mut self, start: u32) {
        self.infos.push(ContentTagInfo {
            start: start - 1,
            content_start: 1,
            content_end: 1,
            end: 1,
            tag_name: "template".to_string(),
            tag_type: "expression".to_string(),
            content: "x".to_string(),
        })
    }
}

impl<'a> VisitMut for LocateVisitor<'a> {
    fn visit_mut_expr(&mut self, n: &mut Expr) {
        n.visit_mut_children_with(self);
        if let Expr::ContentTagExpression(expr) = n {
            self.found_tag(expr.span.lo.0);
        }
    }

    fn visit_mut_class_member(&mut self, n: &mut ClassMember) {
        n.visit_mut_children_with(self);
        if let ClassMember::ContentTagMember(ContentTagMember { span, contents }) = n {
            self.found_tag(span.lo.0);
        }
    }
}
