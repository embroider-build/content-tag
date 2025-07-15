const chai = require("chai");
const { codeEquality } = require("code-equality-assertions/chai");

chai.use(codeEquality);

const { expect } = chai;

const { Preprocessor } = require("content-tag");

const p = new Preprocessor();

function normalizeOutput(output) {
  return output.replace(/template_[0-9a-f]{32}/g, "template_UUID");
}

describe("cjs/require", function () {
  it("can call process", function () {
    let output = p.process("<template>Hi</template>");

    expect(normalizeOutput(output.code)).to
      .equalCode(`import { template as template_UUID } from "@ember/template-compiler";
  export default template_UUID(\`Hi\`, {
      eval () {
          return eval(arguments[0]);
      }
  });`);
  });

  it("can call parse", function () {
    let output = p.parse("<template>Hello!</template>");

    expect(output).to.eql([
      {
        type: "expression",
        tagName: "template",
        contents: "Hello!",
        range: {
          startByte: 0,
          endByte: 27,
          startChar: 0,
          endChar: 27,
          startUtf16Codepoint: 0,
          endUtf16Codepoint: 27,
        },
        contentRange: {
          startByte: 10,
          endByte: 16,
          startChar: 10,
          endChar: 16,
          startUtf16Codepoint: 10,
          endUtf16Codepoint: 16,
        },
        startRange: {
          startByte: 0,
          endByte: 10,
          startChar: 0,
          endChar: 10,
          startUtf16Codepoint: 0,
          endUtf16Codepoint: 10,
        },
        endRange: {
          startByte: 16,
          endByte: 27,
          startChar: 16,
          endChar: 27,
          startUtf16Codepoint: 16,
          endUtf16Codepoint: 27,
        },
      },
    ]);
  });
});
