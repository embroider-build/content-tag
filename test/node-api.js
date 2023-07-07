const { Preprocessor } = require('../pkg');
const chai = require('chai');
const { codeEquality } = require("code-equality-assertions/chai");

chai.use(codeEquality)

const { expect } = chai;

const p = new Preprocessor();

describe("something", function() {
  it("works for a basic example", function() {
    let output = p.process('<template>Hi</template>');

    expect(output).to.equalCode(`import { template } from "@ember/template-compiler";
    template("Hi", {
        eval () {
            return eval(arguments[0]);
        }
    });`);
  });

  it("provides a useful error when there is a syntax error", function() {
    expect(function() {
      p.process(`const thing = "face";
  <template>Hi`);
    }).to.throw(`× Unexpected eof
   ╭─[:1:1]
 1 │ const thing = "face";
 2 │   <template>Hi
   ╰────`);
  })

  it("shows a graphical error info that points to the problem", function() {
    expect(function() {
      p.process(`class {`)
    }).to.throw(`× Expected ident
   ╭─[:1:1]
 1 │ class {
   ·       ─
   ╰────`);
  });
})
