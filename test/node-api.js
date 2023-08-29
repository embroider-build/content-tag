const { Preprocessor } = require('../pkg');
const chai = require('chai');
const { codeEquality } = require("code-equality-assertions/chai");

chai.use(codeEquality)

const { expect } = chai;

const p = new Preprocessor();

describe("process", function() {
  it("works for a basic example", function() {
    let output = p.process('<template>Hi</template>');

    expect(output).to.equalCode(`import { template } from "@ember/template-compiler";
    export default template("Hi", {
        eval () {
            return eval(arguments[0]);
        }
    });`);
  });

  it("Emits parse errors with anonymous file", function() {
    expect(function() {
      p.process(`const thing = "face";
  <template>Hi`);
    }).to.throw(`Parse Error at <anon>:2:15: 2:15`);
  });

  it("Emits parse errors with real file", function() {
    expect(function() {
      p.process(`const thing = "face";
  <template>Hi`, "path/to/my/component.gjs");
    }).to.throw(`Parse Error at path/to/my/component.gjs:2:15: 2:15`);
  });

  it("Offers source_code snippet on parse errors", function() {
    let parseError;
    try {
      p.process(`class {`)
    } catch (err) {
      parseError = err;
    }
    expect(parseError).to.have.property("source_code").matches(/Expected ident.*class \{/s);
  });

  it("Offers source_code_color snippet on parse errors", function() {
    let parseError;
    try {
      p.process(`class {`)
    } catch (err) {
      parseError = err;
    }
    // eslint-disable-next-line no-control-regex
    expect(parseError).to.have.property("source_code_color").matches(/Expected ident.*[\u001b].*class \{/s);
  });
})

describe("locate", function () {
  it("Can emit location data for template tags", function () {
    const input = `let x = <template>Hello!</template>`;

    const templates = p.locate(input, "foo.gjs", {
      templateTag: "template",
    });

    expect(templates).to.deep.equal(
      [
        {
          start: 8,
          contentStart: 18,
          contentEnd: 24,
          end: 34,
          tagName: 'template',
          type: 'expression',
          content: 'Hello!',
        },
      ]
    );
  });
});
