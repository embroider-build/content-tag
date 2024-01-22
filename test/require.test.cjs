const chai = require("chai");
const { codeEquality } = require("code-equality-assertions/chai");

chai.use(codeEquality);

const { expect } = chai;

const { Preprocessor } = require("content-tag");

const p = new Preprocessor();

describe("cjs/require", function () {
  it("can call process", function () {
    let output = p.process("<template>Hi</template>");

    expect(output).to
      .equalCode(`import { template } from "@ember/template-compiler";
  export default template(\`Hi\`, {
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
