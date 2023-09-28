use serde::Serialize;
use swc_common::{self, Span};
use swc_ecma_ast::{
    ClassMember, ContentTagContent, ContentTagEnd, ContentTagExpression, ContentTagMember,
    ContentTagStart,
};
use swc_ecma_visit::{Visit, VisitWith};


#[derive(Default, Debug)]
pub struct LocateContentTagVisitor {
    pub occurrences: Vec<Occurrence>,
}

#[derive(Eq, PartialEq, Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
enum ContentTagKind {
    Expression,
    ClassMember,
}

impl LocateContentTagVisitor {
    fn add_occurrence(
        &mut self,
        kind: ContentTagKind,
        span: &Span,
        opening: &ContentTagStart,
        contents: &ContentTagContent,
        closing: &ContentTagEnd,
    ) {
        let occurrence = Occurrence {
            kind,
            tag_name: "template".to_owned(),
            contents: contents.value.to_string(),
            range: span.into(),
            start_range: opening.span.into(),
            content_range: contents.span.into(),
            end_range: closing.span.into(),
        };

        self.occurrences.push(occurrence);
    }
}

impl Visit for LocateContentTagVisitor {
    fn visit_expr(&mut self, n: &swc_ecma_ast::Expr) {
        match n {
            swc_ecma_ast::Expr::ContentTagExpression(ContentTagExpression {
                span,
                opening,
                contents,
                closing,
            }) => {
                self.add_occurrence(ContentTagKind::Expression, span, opening, contents, closing);
            }
            _ => {}
        }

        n.visit_children_with(self);
    }

    fn visit_class_member(&mut self, n: &ClassMember) {
        match n {
            ClassMember::ContentTagMember(ContentTagMember {
                span,
                opening,
                contents,
                closing,
            }) => {
                self.add_occurrence(
                    ContentTagKind::ClassMember,
                    span,
                    opening,
                    contents,
                    closing,
                );
            }
            _ => {}
        }

        n.visit_children_with(self);
    }
}

#[derive(Serialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Occurrence {
    #[serde(rename = "type")]
    kind: ContentTagKind,
    tag_name: String,
    contents: String,
    range: Range,
    // the span of the opening "<template>" tag
    start_range: Range,
    content_range: Range,
    // the span of the closing "</template>" tag
    end_range: Range,
}

#[derive(Serialize, Debug, Eq, PartialEq)]
pub struct Range {
    start: usize,
    end: usize,
}

impl From<&Span> for Range {
    fn from(value: &Span) -> Self {
        Range {
            start: value.lo.0 as usize - 1,
            end: value.hi.0 as usize - 1,
        }
    }
}

impl From<Span> for Range {
    fn from(value: Span) -> Self {
        (&value).into()
    }
}

#[cfg(test)]
use crate::Preprocessor;

#[test]
fn test_basic_example() {
    let p = Preprocessor::new();
    let output = p
        .parse("<template>Hello!</template>", Default::default())
        .unwrap();
    let expected = Occurrence {
        kind: ContentTagKind::Expression,
        tag_name: "template".into(),
        contents: "Hello!".into(),
        range: Range { start: 0, end: 27 },
        start_range: Range { start: 0, end: 10 },
        content_range: Range { start: 10, end: 16 },
        end_range: Range { start: 16, end: 27 },
    };
    assert_eq!(output, vec![expected]);
}

#[test]
fn test_expression_position() {
    let p = Preprocessor::new();
    let output = p
        .parse(
            "const tpl = <template>Hello!</template>",
            Default::default(),
        )
        .unwrap();

    let expected = vec![Occurrence {
        kind: ContentTagKind::Expression,
        tag_name: "template".into(),
        contents: "Hello!".into(),
        range: Range { start: 12, end: 39 },
        start_range: Range { start: 12, end: 22 },
        content_range: Range { start: 22, end: 28 },
        end_range: Range { start: 28, end: 39 },
    }];

    assert_eq!(output, expected);
}

#[test]
fn test_inside_class_body() {
    let p = Preprocessor::new();
    let output = p
        .parse(
            r#"
                  class A {
                    <template>Hello!</template>
                  }
                "#,
            Default::default(),
        )
        .unwrap();

    let expected = vec![Occurrence {
        kind: ContentTagKind::ClassMember,
        tag_name: "template".into(),
        contents: "Hello!".into(),
        range: Range { start: 49, end: 76 },
        start_range: Range { start: 49, end: 59 },
        content_range: Range { start: 59, end: 65 },
        end_range: Range { start: 65, end: 76 },
    }];

    assert_eq!(output, expected);
}

#[test]
fn test_preceded_by_a_slash_character() {
    let p = Preprocessor::new();
    // What is this testing?
    // Would a better test be:
    // `const divide = 1 / <template>Hello!</template>;`
    let output = p
        .parse(
            r#"
                  const divide = () => 4 / 2;
                  <template>Hello!</template>
                "#,
            Default::default(),
        )
        .unwrap();

    let expected = vec![Occurrence {
        kind: ContentTagKind::Expression,
        tag_name: "template".into(),
        contents: "Hello!".into(),
        range: Range { start: 65, end: 92 },
        start_range: Range { start: 65, end: 75 },
        content_range: Range { start: 75, end: 81 },
        end_range: Range { start: 81, end: 92 },
    }];

    assert_eq!(output, expected);
}

#[test]
fn test_template_inside_a_regexp() {
    let p = Preprocessor::new();
    let output = p
        .parse(
            r#"
                  const myregex = /<template>/;
                  <template>Hello!</template>
                "#,
            Default::default(),
        )
        .unwrap();

    let expected = vec![Occurrence {
        kind: ContentTagKind::Expression,
        tag_name: "template".into(),
        contents: "Hello!".into(),
        range: Range { start: 67, end: 94 },
        start_range: Range { start: 67, end: 77 },
        content_range: Range { start: 77, end: 83 },
        end_range: Range { start: 83, end: 94 },
    }];

    assert_eq!(output, expected);
}

#[test]
fn test_no_match() {
    let p = Preprocessor::new();
    let output = p
        .parse("console.log('Hello world');", Default::default())
        .unwrap();

    assert_eq!(output, vec![]);
}

#[test]
fn test_inner_expression() {
    let p = Preprocessor::new();
    let src = r#"let x = doIt(<template>Hello</template>)"#;
    let output = p.parse(src, Default::default()).unwrap();

    assert_eq!(
        output,
        vec![Occurrence {
            range: Range { start: 13, end: 39 },
            content_range: Range { start: 23, end: 28 },
            contents: "Hello".into(),
            end_range: Range { start: 28, end: 39 },
            start_range: Range { start: 13, end: 23 },
            tag_name: "template".into(),
            kind: ContentTagKind::Expression
        }]
    );
}
