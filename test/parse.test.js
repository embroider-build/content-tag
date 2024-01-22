import chai from "chai";
import { codeEquality } from "code-equality-assertions/chai";
import { Preprocessor } from "content-tag";

chai.use(codeEquality);

const { expect } = chai;

const p = new Preprocessor();

describe(`parse`, function () {
  it("basic example", function () {
    let output = p.parse("<template>Hello!</template>");

    expect(output).to.eql([
      {
        type: "expression",
        tagName: "template",
        contents: "Hello!",
        range: {
          start: 0,
          end: 27,
        },
        contentRange: {
          start: 10,
          end: 16,
        },
        startRange: {
          end: 10,
          start: 0,
        },
        endRange: {
          start: 16,
          end: 27,
        },
      },
    ]);
  });

  it("expression position", function () {
    let output = p.parse("const tpl = <template>Hello!</template>");

    expect(output).to.eql([
      {
        type: "expression",
        tagName: "template",
        contents: "Hello!",
        range: {
          start: 12,
          end: 39,
        },
        contentRange: {
          start: 22,
          end: 28,
        },
        startRange: {
          start: 12,
          end: 22,
        },
        endRange: {
          start: 28,
          end: 39,
        },
      },
    ]);
  });

  it("inside class body", function () {
    let output = p.parse(`
      class A {
        <template>Hello!</template>
      }
    `);

    expect(output).to.eql([
      {
        type: "class-member",
        tagName: "template",
        contents: "Hello!",
        range: {
          start: 25,
          end: 52,
        },
        contentRange: {
          start: 35,
          end: 41,
        },
        startRange: {
          start: 25,
          end: 35,
        },
        endRange: {
          start: 41,
          end: 52,
        },
      },
    ]);
  });

  it("preceded by a slash character", function () {
    // What is this testing?
    // Would a better test be:
    // `const divide = 1 / <template>Hello!</template>;`
    let output = p.parse(`
      const divide = () => 4 / 2;
      <template>Hello!</template>
    `);

    expect(output).to.eql([
      {
        type: "expression",
        tagName: "template",
        contents: "Hello!",
        range: {
          start: 41,
          end: 68,
        },
        contentRange: {
          start: 51,
          end: 57,
        },
        startRange: {
          start: 41,
          end: 51,
        },
        endRange: {
          start: 57,
          end: 68,
        },
      },
    ]);
  });

  it("/<template>/ inside a regexp", function () {
    let output = p.parse(`
      const myregex = /<template>/;
      <template>Hello!</template>
    `);

    expect(output).to.eql([
      {
        type: "expression",
        tagName: "template",
        contents: "Hello!",
        range: {
          start: 43,
          end: 70,
        },
        contentRange: {
          start: 53,
          end: 59,
        },
        startRange: {
          start: 43,
          end: 53,
        },
        endRange: {
          start: 59,
          end: 70,
        },
      },
    ]);
  });

  it("no match", function () {
    let output = p.parse("console.log('Hello world');");

    expect(output).to.eql([]);
  });

  it("Emits parse errors", function () {
    expect(function () {
      p.process(
        `const thing = "face";
  <template>Hi`,
        "path/to/my/component.gjs",
      );
    }).to.throw(`Parse Error at path/to/my/component.gjs:2:15: 2:15`);
  });
});
