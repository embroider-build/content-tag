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
    }).to.throw('error at: anon-file:2:39: 2:39 - Unexpected eof');
  })
})
