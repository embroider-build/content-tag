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
});
