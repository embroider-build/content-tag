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
    pub src: String,
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
            range: Range::new(&self.src, span),
            start_range: Range::new(&self.src, &opening.span),
            content_range: Range::new(&self.src, &contents.span),
            end_range: Range::new(&self.src, &closing.span),
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
#[serde(rename_all = "camelCase")]
pub struct Range {
    start_byte: usize,
    end_byte: usize,
    start_char: usize,
    end_char: usize,
}
impl Range {
    pub fn new(src: &str, span: &Span) -> Range {
        Range {
            start_byte: span.lo.0 as usize - 1,
            end_byte: span.hi.0 as usize - 1,
            start_char: src[..span.lo.0 as usize - 1].chars().count(),
            end_char: src[..span.hi.0 as usize - 1].chars().count() + 1,
        }
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
        range: Range {
            start_byte: 0,
            end_byte: 27,
            start_char: 0,
            end_char: 27,
        },
        start_range: Range {
            start_byte: 0,
            end_byte: 10,
            start_char: 0,
            end_char: 10,
        },
        content_range: Range {
            start_byte: 10,
            end_byte: 16,
            start_char: 10,
            end_char: 16,
        },
        end_range: Range {
            start_byte: 16,
            end_byte: 27,
            start_char: 16,
            end_char: 27,
        },
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
        range: Range {
            start_byte: 12,
            end_byte: 39,
            start_char: 12,
            end_char: 39,
        },
        start_range: Range {
            start_byte: 12,
            end_byte: 22,
            start_char: 12,
            end_char: 22,
        },
        content_range: Range {
            start_byte: 22,
            end_byte: 28,
            start_char: 22,
            end_char: 28,
        },
        end_range: Range {
            start_byte: 28,
            end_byte: 39,
            start_char: 28,
            end_char: 39,
        },
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
        range: Range {
            start_byte: 49,
            end_byte: 76,
            start_char: 49,
            end_char: 76,
        },
        start_range: Range {
            start_byte: 49,
            end_byte: 59,
            start_char: 49,
            end_char: 59,
        },
        content_range: Range {
            start_byte: 59,
            end_byte: 65,
            start_char: 59,
            end_char: 65,
        },
        end_range: Range {
            start_byte: 65,
            end_byte: 76,
            start_char: 65,
            end_char: 76,
        },
    }];

    assert_eq!(output, expected);
}

#[test]
fn test_multibyte_character_inside_template() {
    let p = Preprocessor::new();
    let output = p
        .parse(
            r#"
                  class A {
                    <template>Hell😀!</template>
                  }
                "#,
            Default::default(),
        )
        .unwrap();

    let expected = vec![Occurrence {
        kind: ContentTagKind::ClassMember,
        tag_name: "template".into(),
        contents: "Hell😀!".into(),
        range: Range {
            start_byte: 49,
            end_byte: 79,
            start_char: 49,
            end_char: 76,
        },
        start_range: Range {
            start_byte: 49,
            end_byte: 59,
            start_char: 49,
            end_char: 59,
        },
        content_range: Range {
            start_byte: 59,
            end_byte: 68,
            start_char: 59,
            end_char: 65,
        },
        end_range: Range {
            start_byte: 68,
            end_byte: 79,
            start_char: 65,
            end_char: 76,
        },
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
        range: Range {
            start_byte: 65,
            end_byte: 92,
            start_char: 65,
            end_char: 92,
        },
        start_range: Range {
            start_byte: 65,
            end_byte: 75,
            start_char: 65,
            end_char: 75,
        },
        content_range: Range {
            start_byte: 75,
            end_byte: 81,
            start_char: 75,
            end_char: 81,
        },
        end_range: Range {
            start_byte: 81,
            end_byte: 92,
            start_char: 81,
            end_char: 92,
        },
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
        range: Range {
            start_byte: 67,
            end_byte: 94,
            start_char: 67,
            end_char: 94,
        },
        start_range: Range {
            start_byte: 67,
            end_byte: 77,
            start_char: 67,
            end_char: 77,
        },
        content_range: Range {
            start_byte: 77,
            end_byte: 83,
            start_char: 77,
            end_char: 83,
        },
        end_range: Range {
            start_byte: 83,
            end_byte: 94,
            start_char: 83,
            end_char: 94,
        },
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
            range: Range {
                start_byte: 13,
                end_byte: 39,
                start_char: 13,
                end_char: 39
            },
            content_range: Range {
                start_byte: 23,
                end_byte: 28,
                start_char: 23,
                end_char: 28
            },
            contents: "Hello".into(),
            end_range: Range {
                start_byte: 28,
                end_byte: 39,
                start_char: 28,
                end_char: 39
            },
            start_range: Range {
                start_byte: 13,
                end_byte: 23,
                start_char: 13,
                end_char: 23
            },
            tag_name: "template".into(),
            kind: ContentTagKind::Expression
        }]
    );
}
