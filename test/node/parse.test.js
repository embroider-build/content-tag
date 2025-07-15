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
        range: { startByte: 0, endByte: 27, startUtf16Codepoint: 0, endUtf16Codepoint: 27 },
        contentRange: {
          startByte: 10,
          endByte: 16,
          startUtf16Codepoint: 10,
          endUtf16Codepoint: 16,
        },
        startRange: { startByte: 0, endByte: 10, startUtf16Codepoint: 0, endUtf16Codepoint: 10 },
        endRange: { startByte: 16, endByte: 27, startUtf16Codepoint: 16, endUtf16Codepoint: 27 },
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
        range: { startByte: 12, endByte: 39, startUtf16Codepoint: 12, endUtf16Codepoint: 39 },
        contentRange: {
          startByte: 22,
          endByte: 28,
          startUtf16Codepoint: 22,
          endUtf16Codepoint: 28,
        },
        startRange: { startByte: 12, endByte: 22, startUtf16Codepoint: 12, endUtf16Codepoint: 22 },
        endRange: { startByte: 28, endByte: 39, startUtf16Codepoint: 28, endUtf16Codepoint: 39 },
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
        range: { startByte: 25, endByte: 52, startUtf16Codepoint: 25, endUtf16Codepoint: 52 },
        contentRange: {
          startByte: 35,
          endByte: 41,
          startUtf16Codepoint: 35,
          endUtf16Codepoint: 41,
        },
        startRange: { startByte: 25, endByte: 35, startUtf16Codepoint: 25, endUtf16Codepoint: 35 },
        endRange: { startByte: 41, endByte: 52, startUtf16Codepoint: 41, endUtf16Codepoint: 52 },
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
        range: { startByte: 41, endByte: 68, startUtf16Codepoint: 41, endUtf16Codepoint: 68 },
        contentRange: {
          startByte: 51,
          endByte: 57,
          startUtf16Codepoint: 51,
          endUtf16Codepoint: 57,
        },
        startRange: { startByte: 41, endByte: 51, startUtf16Codepoint: 41, endUtf16Codepoint: 51 },
        endRange: { startByte: 57, endByte: 68, startUtf16Codepoint: 57, endUtf16Codepoint: 68 },
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
        range: { startByte: 43, endByte: 70, startUtf16Codepoint: 43, endUtf16Codepoint: 70 },
        contentRange: {
          startByte: 53,
          endByte: 59,
          startUtf16Codepoint: 53,
          endUtf16Codepoint: 59,
        },
        startRange: { startByte: 43, endByte: 53, startUtf16Codepoint: 43, endUtf16Codepoint: 53 },
        endRange: { startByte: 59, endByte: 70, startUtf16Codepoint: 59, endUtf16Codepoint: 70 },
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
        { filename: "path/to/my/component.gjs" },
      );
    }).to.throw(`Parse Error at path/to/my/component.gjs:2:15: 2:15`);
  });

  it("handles multibyte characters", function () {
    let output = p.parse(
      "const prefix = '熊';\nconst tpl = <template>Hello!</template>",
    );

    expect(output).to.eql([
      {
        type: "expression",
        tagName: "template",
        contents: "Hello!",
        range: { startByte: 34, endByte: 61, startUtf16Codepoint: 32, endUtf16Codepoint: 59 },
        contentRange: {
          startByte: 44,
          endByte: 50,
          startUtf16Codepoint: 42,
          endUtf16Codepoint: 48,
        },
        startRange: { startByte: 34, endByte: 44, startUtf16Codepoint: 32, endUtf16Codepoint: 42 },
        endRange: { startByte: 50, endByte: 61, startUtf16Codepoint: 48, endUtf16Codepoint: 59 },
      },
    ]);
  });

  it("has correct character ranges", function () {
    let file = [
      "const one = <template>💩💩💩💩💩💩💩</template>;" +
        "" +
        "const two = <template>💩</template>;",
    ].join("\n");

    let output = p.parse(file);

    let one = output[0];
    let two = output[1];
    let arr = Array.from(file);

    const slice = (start, end) => arr.slice(start, end).join("");

    {
      let { range, startRange, endRange, contentRange } = one;

      expect(slice(range.startChar, range.endChar)).to.eql(`<template>💩💩💩💩💩💩💩</template>`);
      expect(slice(startRange.startChar, startRange.endChar)).to.eql(`<template>`);
      expect(slice(endRange.startChar, endRange.endChar)).to.eql(`</template>`);
      expect(slice(contentRange.startChar, contentRange.endChar)).to.eql(`💩💩💩💩💩💩💩`);
    }

    {
      let { range, startRange, endRange, contentRange } = two;

      expect(slice(range.startChar, range.endChar)).to.eql(`<template>💩</template>`);
      expect(slice(startRange.startChar, startRange.endChar)).to.eql(`<template>`);
      expect(slice(endRange.startChar, endRange.endChar)).to.eql(`</template>`);
      expect(slice(contentRange.startChar, contentRange.endChar)).to.eql(`💩`);
    }
  });
});
