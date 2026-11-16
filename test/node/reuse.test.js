import chai from "chai";
import { Preprocessor } from "content-tag";

const { expect } = chai;

describe("reusing a Preprocessor", function () {
  const ascii = "let x = <template>Hello</template>";
  const unicode = "let ü = 1;\nlet x = <template>💩</template>";
  const longer = "// padding\n".repeat(50) + "<template>Hi</template>";

  it("gives the same parse results each time", function () {
    const p = new Preprocessor();
    const firstAscii = p.parse(ascii);
    const firstUnicode = p.parse(unicode);
    for (let i = 0; i < 3; i++) {
      p.parse(longer);
      expect(p.parse(ascii)).to.eql(firstAscii);
      expect(p.parse(unicode)).to.eql(firstUnicode);
    }
    expect(firstUnicode[0].range.startUtf16Codepoint).to.eql(19);
  });

  it("gives the same process results each time", function () {
    const p = new Preprocessor();
    const first = p.process(unicode);
    for (let i = 0; i < 3; i++) {
      p.process(longer);
      const again = p.process(unicode);
      expect(again.code).to.eql(first.code);
      expect(again.map).to.eql(first.map);
    }
  });

  it("returns a plain object from process", function () {
    const output = new Preprocessor().process(ascii);
    expect(Object.getPrototypeOf(output)).to.equal(Object.prototype);
    expect(Object.keys(output)).to.eql(["code", "map"]);
  });

  it("reports errors at the right place after earlier files", function () {
    const p = new Preprocessor();
    p.process(longer);
    expect(() => p.process("let x = ;")).to.throw(/Parse Error at <anon>:1:9/);
  });
});
