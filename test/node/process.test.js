import chai from "chai";
import { codeEquality } from "code-equality-assertions/chai";
import { Preprocessor } from "content-tag";

chai.use(codeEquality);

const { expect } = chai;

const p = new Preprocessor();

function normalizeOutput(output) {
  return output.replace(/template_[0-9a-f]{32}/g, "template_UUID");
}

describe(`process`, function () {
  it("works for a basic example", function () {
    let output = p.process("<template>Hi</template>");

    expect(normalizeOutput(output.code)).to
      .equalCode(`import { template as template_UUID } from "@ember/template-compiler";
  export default template_UUID(\`Hi\`, {
      eval () {
          return eval(arguments[0]);
      }
  });`);
  });

  it("escapes backticks", function () {
    let input = `
    class Foo extends Component {
      greeting = 'Hello';

      <template>{{this.greeting}}, \`lifeform\`!</template>
    }
  `;

    let output = p.process(input);

    expect(normalizeOutput(output.code)).to.equalCode(
      `import { template as template_UUID } from "@ember/template-compiler";
     class Foo extends Component {
         greeting = 'Hello';
         static{
             template_UUID(\`{{this.greeting}}, \\\`lifeform\\\`!\`, {
                 component: this,
                 eval () {
                     return eval(arguments[0]);
                 }
             });
         }
     };`
    );
  });

  it("Emits parse errors with anonymous file", function () {
    expect(function () {
      p.process(`const thing = "face";
  <template>Hi`);
    }).to.throw(`Parse Error at <anon>:2:15: 2:15`);
  });

  it("Emits parse errors with real file", function () {
    expect(function () {
      p.process(
        `const thing = "face";
  <template>Hi`,
        { filename: "path/to/my/component.gjs" }
      );
    }).to.throw(`Parse Error at path/to/my/component.gjs:2:15: 2:15`);
  });

  it("Offers source_code snippet on parse errors", function () {
    let parseError;
    try {
      p.process(`class {`);
    } catch (err) {
      parseError = err;
    }
    expect(parseError)
      .to.have.property("source_code")
      .matches(/Expected ident.*class \{/s);
  });

  it("Offers source_code_color snippet on parse errors", function () {
    let parseError;
    try {
      p.process(`class {`);
    } catch (err) {
      parseError = err;
    }
    // eslint-disable-next-line no-control-regex
    expect(parseError)
      .to.have.property("source_code_color")
      .matches(/Expected ident.*[\u001b].*class \{/s);
  });

  it("Provides inline source maps if inline_source_map option is set to true", function () {
    let output = p.process(`<template>Hi</template>`, { inline_source_map: true });

    expect(output.code).to.match(
      /sourceMappingURL=data:application\/json;base64,/
    );
  });

  it("Preserves typescript declare", function () {
    let output = p.process(`class X { declare a: string; }`);
    expect(output.code).to.match(/declare a: string/);
  });
});
