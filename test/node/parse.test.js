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
        range: { startByte: 0, endByte: 27, startChar: 0, endChar: 27 },
        contentRange: {
          startByte: 10,
          endByte: 16,
          startChar: 10,
          endChar: 16,
        },
        startRange: { startByte: 0, endByte: 10, startChar: 0, endChar: 10 },
        endRange: { startByte: 16, endByte: 27, startChar: 16, endChar: 27 },
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
        range: { startByte: 12, endByte: 39, startChar: 12, endChar: 39 },
        contentRange: {
          startByte: 22,
          endByte: 28,
          startChar: 22,
          endChar: 28,
        },
        startRange: { startByte: 12, endByte: 22, startChar: 12, endChar: 22 },
        endRange: { startByte: 28, endByte: 39, startChar: 28, endChar: 39 },
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
        range: { startByte: 25, endByte: 52, startChar: 25, endChar: 52 },
        contentRange: {
          startByte: 35,
          endByte: 41,
          startChar: 35,
          endChar: 41,
        },
        startRange: { startByte: 25, endByte: 35, startChar: 25, endChar: 35 },
        endRange: { startByte: 41, endByte: 52, startChar: 41, endChar: 52 },
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
        range: { startByte: 41, endByte: 68, startChar: 41, endChar: 68 },
        contentRange: {
          startByte: 51,
          endByte: 57,
          startChar: 51,
          endChar: 57,
        },
        startRange: { startByte: 41, endByte: 51, startChar: 41, endChar: 51 },
        endRange: { startByte: 57, endByte: 68, startChar: 57, endChar: 68 },
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
        range: { startByte: 43, endByte: 70, startChar: 43, endChar: 70 },
        contentRange: {
          startByte: 53,
          endByte: 59,
          startChar: 53,
          endChar: 59,
        },
        startRange: { startByte: 43, endByte: 53, startChar: 43, endChar: 53 },
        endRange: { startByte: 59, endByte: 70, startChar: 59, endChar: 70 },
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
        { filename: "path/to/my/component.gjs" }
      );
    }).to.throw(`Parse Error at path/to/my/component.gjs:2:15: 2:15`);
  });

  it("handles multibyte characters", function () {
    let output = p.parse(
      "const prefix = '熊';\nconst tpl = <template>Hello!</template>"
    );

    expect(output).to.eql([
      {
        type: "expression",
        tagName: "template",
        contents: "Hello!",
        range: { startByte: 34, endByte: 61, startChar: 32, endChar: 59 },
        contentRange: {
          startByte: 44,
          endByte: 50,
          startChar: 42,
          endChar: 48,
        },
        startRange: { startByte: 34, endByte: 44, startChar: 32, endChar: 42 },
        endRange: { startByte: 50, endByte: 61, startChar: 48, endChar: 59 },
      },
    ]);
  });
});
